# Multi-Agent ChatBot — AI-Powered Learning Assistant

An agentic AI system that routes user questions to specialised chatbot agents — each an expert in its own domain. Built with **LangGraph** and **Ollama**, it runs entirely locally — no paid API keys required.

## Agents

| Agent | Domain |
|---|---|
| 🏥 **HealthBot** | Medical conditions, symptoms, treatments, wellness |
| 💻 **QualityBot** | Software quality, testing, QA, standards, best practices |

## Features

- **LLM-based routing** — an LLM classifier automatically selects the right specialist agent for each topic
- **Hybrid RAG retrieval** — each agent first queries a local ChromaDB corpus (Wikipedia articles, indexed with `all-MiniLM-L6-v2` embeddings); DuckDuckGo supplements when coverage is sparse
- **Domain-specific summaries** — the LLM condenses retrieved context into clear, accessible explanations
- **Comprehension quizzes** — generates and grades a short-answer question based solely on the presented material
- **Multi-topic sessions** — loop through as many topics as you like, switching agents as needed
- **Fully local** — runs on your machine via Ollama; no data sent to external AI APIs
- **Single chat UI** — one interactive ipywidgets panel with a scrollable message area; agents are identified by name in the chat

## Architecture

The system is implemented as a [LangGraph](https://github.com/langchain-ai/langgraph) state machine with an LLM-based router:

```
START → get_topic → classify_topic ─┬─► search_health    → summarize_health    ─┐
                                    └─► search_sw_quality → summarize_sw_quality ─┤
                                                                                  ▼
                                                                          present_summary
      → prompt_ready_for_quiz → generate_quiz → present_quiz
      → collect_answer → evaluate_answer → present_result
      → ask_continue ──► (yes) → get_topic
                     └─► (no)  → END
```

Each `search_*` node uses **hybrid retrieval**:

```
search_* ──► ChromaDB (local RAG corpus)  ─── enough chunks? ──► summarize_*
                                          └── too sparse?    ──► DDG supplement ──► summarize_*
```

## Prerequisites

| Requirement | Notes |
|---|---|
| Python 3.10+ | Tested with 3.12 |
| [Ollama](https://ollama.com) | Must be installed and running locally |
| A pulled Ollama model | Default: `llama3.1:8b` |

## Setup

1. **Install Ollama** from [ollama.com](https://ollama.com) and pull a model:
   ```bash
   ollama pull llama3.1:8b
   ```

2. **Create and activate a virtual environment:**
   ```bash
   python3 -m venv .venv
   source .venv/bin/activate   # Windows: .venv\Scripts\activate
   ```

3. **Install Python dependencies:**
   ```bash
   pip install -r requirements.txt
   ```

4. **Start Ollama** (if not already running):
   ```bash
   ollama serve
   ```

## Usage

Open `multi-agent-chatbot.ipynb` in **VS Code** (recommended) or Jupyter and run all cells in order:

```bash
# Jupyter alternative
jupyter notebook multi-agent-chatbot.ipynb
```

Run cells in this order:

| Cell | Action |
|---|---|
| 2 | Configuration (model name, RAG settings) |
| 3 | Scaffolding (UI, imports, LLM instance) |
| 4 | Multi-Agent Graph (build and display) |
| **5** | **RAG Corpus Setup** — downloads Wikipedia articles and indexes them into ChromaDB *(first run takes a few minutes; subsequent runs are instant)* |
| 6 | Runner — starts the interactive chat session |

> **Tip:** Cell 5 only needs to be run once. The corpus is persisted to `./rag_corpus/` and reused on every subsequent run. If you skip cell 5 the chatbot falls back to DuckDuckGo-only retrieval and still works.

> **Note:** The graph runs in a background thread so the kernel stays idle between responses. VS Code dispatches button/Enter events only when the kernel is not busy, which is why the last cell returns immediately after starting the thread.

### Changing the model

Edit `OLLAMA_MODEL` in cell 2 of the notebook:

```python
OLLAMA_MODEL = "mistral"   # or "gemma3", "phi3", "llama3.1:8b", etc.
```

## Extending the RAG Corpus

Use the standalone `add_to_corpus.py` script to add documents without rebuilding:

```bash
# Add a PDF to the health domain
python add_to_corpus.py --domain health --pdf /path/to/document.pdf

# Add a whole folder of PDFs
python add_to_corpus.py --domain sw_quality --pdf /path/to/iso_docs/

# Add a web page
python add_to_corpus.py --domain health --url https://www.nhs.uk/conditions/diabetes/

# Add a Wikipedia article
python add_to_corpus.py --domain sw_quality --wiki "Mutation testing"

# Preview chunks without writing (dry run)
python add_to_corpus.py --domain health --pdf report.pdf --dry-run
```

After adding documents, re-run cell 5 in the notebook to reload the retrievers.

## Project Structure

```
.
├── multi-agent-chatbot.ipynb   # Multi-Agent ChatBot — router + specialist agents
├── add_to_corpus.py            # CLI tool: add PDFs, text, URLs or Wikipedia to ChromaDB
├── rag_corpus/                 # ChromaDB on-disk vector store (auto-created; gitignored)
├── requirements.txt            # Python dependencies
├── .gitignore
└── README.md
```

## Dependencies

| Package | Version | Purpose | Licence |
|---|---|---|---|
| `langchain-ollama` | >=0.2.0 | Ollama integration | MIT |
| `langgraph` | >=1.0.2,<1.1.0 | Agentic state machine | MIT |
| `ddgs` | >=9.0.0 | DuckDuckGo web search fallback (no API key) | MIT |
| `ipywidgets` | >=8.0 | Interactive chat UI | BSD |
| `chromadb` | >=0.5.0 | Local vector store (RAG corpus) | Apache 2.0 |
| `langchain-chroma` | >=0.2.0 | LangChain ↔ ChromaDB integration | MIT |
| `langchain-community` | >=0.3.0 | WikipediaLoader + document loaders | MIT |
| `langchain-huggingface` | >=0.1.0 | HuggingFaceEmbeddings | MIT |
| `sentence-transformers` | >=3.0.0 | Local embedding model runner (`all-MiniLM-L6-v2`) | Apache 2.0 |
| `wikipedia` | >=1.4.0 | Wikipedia API client | MIT |
| `pypdf` | >=4.0.0 | PDF loader backend for `add_to_corpus.py` | BSD |

## Notebooks

### `multi-agent-chatbot.ipynb` — Multi-Agent ChatBot

The primary notebook described above. Routes topics to specialist agents (HealthBot, QualityBot) via an LLM classifier, with hybrid RAG + DuckDuckGo retrieval.

---

## Adding More Agents

To add a new specialist agent:

1. Add an entry to `AGENT_LABELS` (e.g. `"devops": "🔧 DevOpsBot"`) in cell 4
2. Add a Wikipedia topic list constant (e.g. `DEVOPS_WIKI_TOPICS = [...]`) in cell 2
3. Register the topic list in the `_DOMAIN_TOPICS` dict in cell 5
4. Create `search_<domain>()` and `summarize_<domain>()` node functions in cell 4
5. Register them as nodes in `build_chatbot()` and wire the edges
6. Add the new domain to the classifier's system prompt in `classify_topic()`

## Disclaimer

This chatbot is an **educational tool only**. All information is sourced from Wikipedia and/or public websites and summarised by a local AI model. The HealthBot agent is **not a substitute for professional medical advice, diagnosis, or treatment**. Always consult a qualified healthcare provider with any questions about a medical condition.
