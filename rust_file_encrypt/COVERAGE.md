# Test Coverage Report

Coverage for `secure-file-crypto` is measured with `cargo-llvm-cov`.

## Current Snapshot

Latest local run (`cargo llvm-cov --all-features --workspace --summary-only`):

- **Line coverage:** 96.73% (444/459)
- **Region coverage:** 95.38% (846/887)
- **Function coverage:** 78.57% (22/28)

### File Breakdown

| File | Region Coverage | Function Coverage | Line Coverage |
|------|-----------------|-------------------|---------------|
| `src/crypto.rs` | 97.44% (798/819) | 84.00% (21/25) | 99.05% (415/419) |
| `src/main.rs` | 70.59% (48/68) | 33.33% (1/3) | 72.50% (29/40) |
| **TOTAL** | **95.38% (846/887)** | **78.57% (22/28)** | **96.73% (444/459)** |

## Test Inventory

The project currently includes **42 automated tests**:

- Unit tests in `src/crypto.rs`: 17
- Integration tests in `tests/integration_tests.rs`: 11
- Robustness tests in `tests/robustness_tests.rs`: 14

## Generate Coverage

Install tool once:

```bash
cargo install cargo-llvm-cov
```

Coverage summary:

```bash
cargo llvm-cov --all-features --workspace --summary-only
```

Terminal table:

```bash
cargo llvm-cov --all-features --workspace
```

HTML report:

```bash
cargo llvm-cov --all-features --workspace --html --output-dir coverage-report
```

LCOV output:

```bash
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

## Notes

- `src/crypto.rs` has very high coverage and contains the core cryptographic logic.
- `src/main.rs` includes interactive CLI flows that are harder to fully exercise in automated tests.
- If coverage changes, regenerate this document using the command above.

## Educational Context

The test suite — including unit, integration, and robustness tests — was generated with **GitHub Copilot** as part of a learning exercise in AI-assisted development. The high coverage (96.73% line) demonstrates what Copilot-driven test generation can achieve for a focused Rust project.
