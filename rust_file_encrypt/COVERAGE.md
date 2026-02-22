# Test Coverage Report

Coverage for `secure-file-crypto` is measured with `cargo-llvm-cov`.

## Current Snapshot

Latest local run (`cargo llvm-cov --all-features --workspace --summary-only`):

- **Line coverage:** 97.27% (534/549)
- **Region coverage:** 96.28% (1034/1074)
- **Function coverage:** 83.78% (31/37)

### File Breakdown

| File | Region Coverage | Function Coverage | Line Coverage |
|------|-----------------|-------------------|---------------|
| `src/crypto.rs` | 97.56% (799/819) | 84.00% (21/25) | 99.05% (415/419) |
| `src/main.rs` | 92.16% (235/255) | 83.33% (10/12) | **91.54%** (119/130) |
| **TOTAL** | **96.28% (1034/1074)** | **83.78% (31/37)** | **97.27% (534/549)** |

## Test Inventory

The project currently includes **51 automated tests**:

- Unit tests in `src/crypto.rs`: 17
- Unit tests in `src/main.rs` (`#[cfg(test)]`): 7
- Integration tests in `tests/integration_tests.rs`: 13
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
- `src/main.rs` includes interactive CLI flows (password prompts via `/dev/tty`) that cannot be fully exercised in automated tests without a controlling terminal. The `run_operation()` helper was extracted specifically to enable unit-testing the core encrypt/decrypt dispatch logic independently of the CLI layer, bringing `main.rs` line coverage from 72.50% to **91.54%**.
- If coverage changes, regenerate this document using the command above.

## Educational Context

The test suite — including unit, integration, and robustness tests — was generated with **GitHub Copilot** as part of a learning exercise in AI-assisted development. The high coverage (97.27% overall line coverage, 91.54% for `main.rs`) demonstrates what Copilot-driven test generation can achieve for a focused Rust project. The `run_operation()` refactor — extracting testable logic from the `main()` function — illustrates a Copilot-guided design pattern for improving coverage in CLI applications where stdin is not easily controllable in tests.
