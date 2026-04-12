#!/usr/bin/env python3
"""add_to_corpus.py — Add documents to the Multi-Agent ChatBot RAG corpus.

Supported sources
-----------------
  --pdf   <file_or_dir>   One PDF file, or a directory of PDFs (recursively scanned)
  --txt   <file_or_dir>   One plain-text file, or a directory of .txt files
  --url   <url>           A web page URL (repeat the flag for multiple URLs)
  --wiki  <topic>         A Wikipedia article title (repeat for multiple topics)

Examples
--------
  # Add a single PDF to the health domain
  python add_to_corpus.py --domain health --pdf /path/to/nhs_diabetes_guide.pdf

  # Add a folder of PDFs to the software-quality domain
  python add_to_corpus.py --domain sw_quality --pdf /path/to/iso_docs/

  # Add a web page + a Wikipedia article to health
  python add_to_corpus.py --domain health \\
      --url https://www.nhs.uk/conditions/diabetes/ \\
      --wiki "Type 2 diabetes"

  # Add multiple Wikipedia topics to sw_quality
  python add_to_corpus.py --domain sw_quality \\
      --wiki "Mutation testing" --wiki "Fuzz testing"

Requirements
------------
  All packages are listed in requirements.txt. Install with:
    pip install -r requirements.txt   (after activating the project venv)
  PDF support is included (pypdf>=4.0.0 is in requirements.txt).
"""

import argparse
import sys
from pathlib import Path

# ── Dependency check ─────────────────────────────────────────────────────────
# All packages below are installed by:  pip install -r requirements.txt
# If you see an ImportError here, activate the project venv first and retry.
# Always run this script with:  python add_to_corpus.py ...

_MISSING = []
for _pkg, _import in [
    ("langchain-community",   "langchain_community"),
    ("langchain-huggingface", "langchain_huggingface"),
    ("langchain-chroma",      "langchain_chroma"),
    ("sentence-transformers", "sentence_transformers"),
    ("langchain-text-splitters", "langchain_text_splitters"),
    ("wikipedia", "wikipedia"),
]:
    try:
        __import__(_import)
    except ImportError:
        _MISSING.append(_pkg)

if _MISSING:
    print(
        "\nERROR: The following packages are not importable with the current Python interpreter:\n"
        + "".join(f"  • {p}\n" for p in _MISSING)
        + "\nMake sure you run the script with the project's virtual environment activated:\n"
        "  python add_to_corpus.py --help\n"
        "\nTo install missing packages:\n"
        "  pip install -r requirements.txt\n",
        file=sys.stderr,
    )
    sys.exit(1)

from langchain_community.document_loaders import (
    WikipediaLoader,
    WebBaseLoader,
    TextLoader,
)
from langchain_huggingface import HuggingFaceEmbeddings
from langchain_chroma import Chroma
from langchain_text_splitters import RecursiveCharacterTextSplitter

# ── Config (must match values in cell 2 of the notebook) ─────────────────────
EMBEDDING_MODEL    = "all-MiniLM-L6-v2"
CHROMA_PERSIST_DIR = "./rag_corpus"
CHUNK_SIZE         = 800
CHUNK_OVERLAP      = 100
VALID_DOMAINS      = ("health", "sw_quality")


def _build_arg_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description="Add documents to the chatbot RAG corpus.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    p.add_argument(
        "--domain", required=True, choices=VALID_DOMAINS,
        help="Target domain collection: health | sw_quality",
    )
    p.add_argument(
        "--pdf", metavar="PATH", action="append", default=[],
        help="PDF file or directory (may be repeated)",
    )
    p.add_argument(
        "--txt", metavar="PATH", action="append", default=[],
        help="Plain-text file or directory (may be repeated)",
    )
    p.add_argument(
        "--url", metavar="URL", action="append", default=[],
        help="Web page URL to scrape (may be repeated)",
    )
    p.add_argument(
        "--wiki", metavar="TOPIC", action="append", default=[],
        help="Wikipedia article title (may be repeated)",
    )
    p.add_argument(
        "--dry-run", action="store_true",
        help="Load and chunk documents but do NOT write to ChromaDB",
    )
    return p


# ── Loaders ───────────────────────────────────────────────────────────────────

def _load_pdfs(paths: list[str]) -> list:
    try:
        from langchain_community.document_loaders import PyPDFLoader
    except ImportError:
        print("ERROR: PDF support requires 'pypdf'.\n  Run: pip install pypdf", file=sys.stderr)
        sys.exit(1)

    docs = []
    for raw in paths:
        p = Path(raw)
        files = sorted(p.rglob("*.pdf")) if p.is_dir() else [p]
        for f in files:
            print(f"  Loading PDF: {f}")
            try:
                docs.extend(PyPDFLoader(str(f)).load())
            except Exception as e:
                print(f"    ✗ Failed: {e}", file=sys.stderr)
    return docs


def _load_txts(paths: list[str]) -> list:
    docs = []
    for raw in paths:
        p = Path(raw)
        files = sorted(p.rglob("*.txt")) if p.is_dir() else [p]
        for f in files:
            print(f"  Loading text: {f}")
            try:
                docs.extend(TextLoader(str(f), encoding="utf-8").load())
            except Exception as e:
                print(f"    ✗ Failed: {e}", file=sys.stderr)
    return docs


def _load_urls(urls: list[str]) -> list:
    docs = []
    for url in urls:
        print(f"  Loading URL: {url}")
        try:
            docs.extend(WebBaseLoader([url]).load())
        except Exception as e:
            print(f"    ✗ Failed: {e}", file=sys.stderr)
    return docs


def _load_wiki(topics: list[str]) -> list:
    docs = []
    for topic in topics:
        print(f"  Loading Wikipedia: '{topic}'")
        try:
            loaded = WikipediaLoader(
                query=topic,
                load_max_docs=1,
                doc_content_chars_max=8000,
            ).load()
            docs.extend(loaded)
        except Exception as e:
            print(f"    ✗ Failed: {e}", file=sys.stderr)
    return docs


# ── Metadata normalisation ───────────────────────────────────────────────────

def _normalise_metadata(chunks: list) -> None:
    """Ensure every chunk has consistent ``title``, ``source``, and
    ``source_type`` metadata so the chatbot UI can build accurate citations.

    Fields set (only when the loader has not already provided them):

    * ``source_type`` – one of ``"wikipedia"``, ``"url"``, ``"pdf"``,
      ``"txt"``, or ``"local"``.
    * ``title`` – human-readable name: article title for Wikipedia, URL for
      web pages, filename stem for PDFs/text files.
    * ``source`` – guaranteed non-empty string (falls back to ``title``).
    """
    for chunk in chunks:
        m = chunk.metadata
        source = str(m.get("source") or "").strip()

        # ── source_type ───────────────────────────────────────────────────
        if not m.get("source_type"):
            lower = source.lower()
            if lower.startswith(("http://", "https://")):
                # Identify Wikipedia by its canonical domain (//-anchored to
                # avoid false matches like "evil-en.wikipedia.org/").
                if "//en.wikipedia.org/" in lower:
                    m["source_type"] = "wikipedia"
                else:
                    m["source_type"] = "url"
            elif lower.endswith(".pdf"):
                m["source_type"] = "pdf"
            elif lower.endswith(".txt"):
                m["source_type"] = "txt"
            else:
                m["source_type"] = "local"

        # ── title ─────────────────────────────────────────────────────────
        if not m.get("title"):
            if m["source_type"] in ("pdf", "txt", "local"):
                m["title"] = Path(source).stem if source else "Unknown"
            else:
                # URL / Wikipedia: the URL is a readable citation fallback.
                m["title"] = source or "Unknown"

        # ── source ────────────────────────────────────────────────────────
        # Guarantee source is always a non-empty string.
        if not source:
            m["source"] = m.get("title", "Unknown")


def _chroma_count(store) -> int:
    """Return the number of documents in *store* without fetching all IDs.

    Tries the native ``_collection.count()`` fast path first (mirrors the
    notebook's RAG-setup cell).  Falls back to ``len(store.get()["ids"])``
    when the private attribute is unavailable so the CLI stays compatible
    across ``langchain-chroma`` / ``chromadb`` version changes.
    """
    collection = getattr(store, "_collection", None)
    if collection is not None and callable(getattr(collection, "count", None)):
        return collection.count()
    return len(store.get()["ids"])


# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    parser = _build_arg_parser()
    args = parser.parse_args()

    if not any([args.pdf, args.txt, args.url, args.wiki]):
        parser.error("Provide at least one source: --pdf, --txt, --url, or --wiki")

    # ── 1. Collect raw documents ──────────────────────────────────────────────
    print("\n── Loading documents ─────────────────────────────────────────────")
    raw_docs = []
    if args.pdf:
        raw_docs.extend(_load_pdfs(args.pdf))
    if args.txt:
        raw_docs.extend(_load_txts(args.txt))
    if args.url:
        raw_docs.extend(_load_urls(args.url))
    if args.wiki:
        raw_docs.extend(_load_wiki(args.wiki))

    if not raw_docs:
        print("No documents loaded — nothing to add.", file=sys.stderr)
        sys.exit(1)

    print(f"\n  → {len(raw_docs)} document(s) loaded.")

    # ── 2. Chunk ──────────────────────────────────────────────────────────────
    print("\n── Chunking ──────────────────────────────────────────────────────")
    splitter = RecursiveCharacterTextSplitter(
        chunk_size=CHUNK_SIZE,
        chunk_overlap=CHUNK_OVERLAP,
    )
    chunks = splitter.split_documents(raw_docs)
    print(f"  → {len(chunks)} chunk(s) produced (size={CHUNK_SIZE}, overlap={CHUNK_OVERLAP})")

    # Normalise metadata so every chunk has consistent title/source/source_type.
    _normalise_metadata(chunks)

    if args.dry_run:
        print("\n── Dry run — skipping ChromaDB write. ────────────────────────────")
        for i, c in enumerate(chunks[:5], 1):
            preview = c.page_content[:120].replace("\n", " ")
            meta = {k: c.metadata.get(k) for k in ("title", "source_type", "source")}
            print(f"  [{i}] {preview}…")
            print(f"       metadata: {meta}")
        if len(chunks) > 5:
            print(f"  … and {len(chunks) - 5} more chunks.")
        return

    # ── 3. Embed & store ──────────────────────────────────────────────────────
    print("\n── Loading embedding model ───────────────────────────────────────")
    embeddings = HuggingFaceEmbeddings(
        model_name=EMBEDDING_MODEL,
        model_kwargs={"device": "cpu"},
        encode_kwargs={"normalize_embeddings": True},
    )
    print(f"  ✓ {EMBEDDING_MODEL} ready")

    collection = f"chatbot_{args.domain}"
    store = Chroma(
        collection_name=collection,
        embedding_function=embeddings,
        persist_directory=CHROMA_PERSIST_DIR,
    )

    # Use a guarded count helper – tries _collection.count() (fast) and falls
    # back to store.get()["ids"] for compatibility across dependency versions.
    before = _chroma_count(store)
    print(f"\n── Writing to ChromaDB ────────────────────────────────────────────")
    print(f"  Collection : {collection}")
    print(f"  Store path : {CHROMA_PERSIST_DIR}")
    print(f"  Chunks before: {before}")

    store.add_documents(chunks)

    after = _chroma_count(store)
    print(f"  Chunks after : {after}  (+{after - before} added)")
    print("\n✅ Done. Re-run cell 5 in the notebook to reload the retrievers.")


if __name__ == "__main__":
    main()
