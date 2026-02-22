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
