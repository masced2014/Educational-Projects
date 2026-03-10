# Secure File Crypto

> **🤖 Built with GitHub Copilot** — This project was developed entirely with [GitHub Copilot](https://github.com/features/copilot) as a hands-on exploration of AI-assisted software development. It is part of the [Educational-Projects](../README.md) repository and demonstrates how Copilot can support writing production-grade Rust code, an extensive test suite, and comprehensive documentation.

A Rust command-line tool for learning and practicing secure file encryption with:

- AES-256-GCM authenticated encryption
- Argon2id password-based key derivation
- Random salt and nonce per encryption
- Test coverage across unit, integration, and robustness suites

This project is part of the educational repository at the workspace root.

## Features

- Encrypts and decrypts files via a simple CLI
- Uses authenticated encryption (confidentiality + integrity)
- Supports interactive password prompt (recommended)
- Supports `-p/--password` argument for automation
- Works across Linux, macOS, and Windows

## Requirements

- Rust 1.82+

## Build

```bash
cargo build --release
```

Binary path:

```bash
./target/release/secure-file-crypto
```

## Quick Start

From the `rust_file_encrypt` directory:

```bash
# Encrypt (interactive prompt with confirmation)
./target/release/secure-file-crypto encrypt -i myfile.txt -o myfile.txt.enc

# Decrypt (interactive prompt)
./target/release/secure-file-crypto decrypt -i myfile.txt.enc -o myfile.txt.dec
```

With password argument (less secure because it can appear in shell history):

```bash
./target/release/secure-file-crypto encrypt -i myfile.txt -o myfile.txt.enc -p "StrongPassword"
./target/release/secure-file-crypto decrypt -i myfile.txt.enc -o myfile.txt.dec -p "StrongPassword"
```

## CLI Help

```bash
secure-file-crypto --help
secure-file-crypto encrypt --help
secure-file-crypto decrypt --help
```

## Encrypted File Format

The output file layout is:

```text
[SALT (16 bytes)][NONCE (12 bytes)][CIPHERTEXT || AUTH_TAG]
```

- Salt: used by Argon2id key derivation
- Nonce: used by AES-GCM
- Auth tag: integrity verification (included in ciphertext output by `aes-gcm`)

## Security Notes

- Wrong password or tampered ciphertext causes decryption failure.
- Passwords are wrapped with `Zeroizing` in key flow and CLI handling.
- There is no password recovery mechanism.
- Use strong passwords and keep secure backups.

## Performance Notes

- Current implementation reads full input into memory before encryption/decryption.
- Tested robustness includes files up to 5MB in automated tests.
- For very large files, memory usage scales with input size.

## Testing

Current test suite: **51 tests total**

- Unit tests: `src/crypto.rs` (17)
- Unit tests: `src/main.rs` — `#[cfg(test)]` module (7)
- Integration tests: `tests/integration_tests.rs` (13)
- Robustness tests: `tests/robustness_tests.rs` (14)

Run all tests:

```bash
cargo test
```

Run with visible output:

```bash
cargo test -- --nocapture
```

Run by suite:

```bash
cargo test --lib
cargo test --test integration_tests
cargo test --test robustness_tests
```

## Fuzzing

Fuzz targets live in `fuzz/fuzz_targets/` and use [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (LibFuzzer).

### Why fuzz this project?

The three targets each attack a distinct threat model:

| Target | What it probes | Security value |
|---|---|---|
| `fuzz_decrypt_arbitrary` | Feed any byte sequence to `decrypt_file` | Guarantees the decryption surface never panics on attacker-controlled input; exercises the minimum-size guard, auth-tag rejection, and all error-handling branches |
| `fuzz_encrypt_plaintext` | Encrypt arbitrary bytes, decrypt, verify roundtrip | Proves encrypt+decrypt is an identity function for **every** possible plaintext; surfaces any data-loss or corruption bug |
| `fuzz_roundtrip` | Structured: arbitrary (plaintext, `password_bytes`), roundtrip + wrong-password check | Covers the entire Argon2 key-derivation path with arbitrary passwords (including empty, non-UTF-8, very long) |

### Prerequisites

```bash
# Nightly Rust (required by libfuzzer-sys)
rustup toolchain install nightly --profile minimal

# cargo-fuzz (installed once per machine)
cargo install cargo-fuzz
```

### `fast-kdf` feature

Argon2id is intentionally slow (~32 ms per call in production).  The `fast-kdf`
feature flag swaps in the minimum legal parameters (m=8 KiB, t=1, p=1) so fuzz
targets can run at **thousands of iterations per second**.

`fast-kdf` is enabled **automatically** by `fuzz/Cargo.toml`; it must **never**
be used in a production build.

### Build all fuzz targets

```bash
cargo +nightly fuzz build
```

### Run a specific target

```bash
# Run until a crash is found (Ctrl-C to stop)
cargo +nightly fuzz run fuzz_decrypt_arbitrary
cargo +nightly fuzz run fuzz_encrypt_plaintext
cargo +nightly fuzz run fuzz_roundtrip

# Time-limited runs (useful in CI)
cargo +nightly fuzz run fuzz_decrypt_arbitrary -- -max_total_time=60
```

### Reproduce a crash

If a crash (or assertion failure) is found, the input is saved to
`fuzz/artifacts/<target>/crash-*`.  Replay it with:

```bash
cargo +nightly fuzz run fuzz_decrypt_arbitrary fuzz/artifacts/fuzz_decrypt_arbitrary/crash-<hash>
```

### Fuzz corpus

libfuzzer automatically maintains a corpus in `fuzz/corpus/<target>/`.  Seed the
corpus with representative encrypted files to accelerate coverage discovery:

```bash
mkdir -p fuzz/corpus/fuzz_decrypt_arbitrary
cp myfile.txt.enc fuzz/corpus/fuzz_decrypt_arbitrary/
```

## Coverage

Coverage is tracked with `cargo-llvm-cov`.

Generate summary:

```bash
cargo llvm-cov --all-features --workspace --summary-only
```

Generate HTML report:

```bash
cargo llvm-cov --all-features --workspace --html --output-dir coverage-report
```

Detailed coverage notes are in `COVERAGE.md`.

## Rust Docs

Generate API docs:

```bash
cargo doc --no-deps
cargo doc --no-deps --open
```

## Supply Chain Security (SBOM)

A [Software Bill of Materials (SBOM)](https://www.cisa.gov/sbom) is generated and scanned for known CVEs on every pull request and push to `main`.

### Workflow summary

The workflow is defined in [`rust-sbom.yml`](../.github/workflows/rust-sbom.yml) and runs two jobs:

| Job | Tool | Output |
|---|---|---|
| `generate-sbom` | [Syft](https://github.com/anchore/syft) | `sbom.cdx.json` — CycloneDX JSON, retained 90 days |
| `scan-vulnerabilities` | [Grype](https://github.com/anchore/grype) | SARIF uploaded to the GitHub Security tab |

**Policy**: the build is blocked if any **Critical** CVE is found. High and Medium findings are reported but do not block merges (adjust the threshold in the workflow to match your security requirements).

All findings are visible under **Security → Code scanning** in the GitHub repository.

### Run locally

Install the tools once:

```bash
# Syft — SBOM generator
curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sudo sh -s -- -b /usr/local/bin

# Grype — vulnerability scanner
curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sudo sh -s -- -b /usr/local/bin
```

Generate the SBOM:

```bash
cd rust_file_encrypt
syft . -o cyclonedx-json=sbom.cdx.json
```

Scan for vulnerabilities (human-readable table):

```bash
grype sbom:./sbom.cdx.json
```

Enforce the same Critical threshold as CI:

```bash
grype sbom:./sbom.cdx.json --fail-on critical
```

Produce a SARIF report (same format as the workflow):

```bash
grype sbom:./sbom.cdx.json -o sarif > grype-results.sarif
```

### Vulnerability policy (`.grype.yaml`)

The [`.grype.yaml`](.grype.yaml) file is picked up automatically by Grype (locally and in CI) and contains:

- The `fail-on-severity: critical` threshold
- Documented ignore rules for confirmed false positives, each with a justification and the advisory reference

Add new ignore entries there whenever Grype flags a false positive, and include the reason so the suppression can be audited later.

## CI / GitHub Actions

Five workflows run automatically on every PR and push to `main`. The first three are scoped to `rust_file_encrypt/` changes via path filters:

| Workflow file | Purpose | Trigger |
|---|---|---|
| [`rust-coverage.yml`](../.github/workflows/rust-coverage.yml) | Run test suite, measure line coverage with `cargo-llvm-cov`, enforce ≥ 95% threshold, upload to Codecov | Push / PR (path-filtered) |
| [`rust-docs.yml`](../.github/workflows/rust-docs.yml) | Build `cargo doc`, fail on any missing-docs warning | Push / PR (path-filtered) |
| [`rust-sbom.yml`](../.github/workflows/rust-sbom.yml) | Generate CycloneDX SBOM with Syft, scan with Grype, upload SARIF, post PR comment | Push / PR (path-filtered) |
| [`security.yml`](../.github/workflows/security.yml) | Run `cargo audit` against known advisory database | Push / PR / Weekly schedule |
| [`codeql.yml`](../.github/workflows/codeql.yml) | CodeQL static analysis for Rust | Push / PR / Weekly schedule |

## Troubleshooting

- **"Failed to open input file"**: check file path and read permissions.
- **"Failed to create output file"**: check output path and write permissions.
- **"Decryption failed"**: verify password and ensure ciphertext was not modified.

## Educational Scope

This project was built entirely with **GitHub Copilot** and serves the following learning goals:

- Explore how Copilot assists in writing idiomatic, safe Rust
- Practice secure coding patterns (authenticated encryption, memory zeroization)
- Build a complete project structure including CLI, library module, unit tests, integration tests, and robustness tests
- Experience AI-assisted documentation and test generation

The entire development workflow — code, tests, and documentation — was driven through Copilot interactions to understand both the capabilities and the limits of AI pair programming.

It uses production-grade crypto crates, but you should perform your own validation before any production use.

## License

MIT — see `LICENSE`.
