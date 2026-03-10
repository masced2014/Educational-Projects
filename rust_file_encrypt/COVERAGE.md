# Test Coverage Report

Coverage for `secure-file-crypto` is measured with `cargo-llvm-cov`.

## Current Snapshot

Latest local run (`cargo llvm-cov --all-features --workspace --summary-only`):

- **Line coverage:** 97.11% (554/570)
- **Region coverage:** 96.11% (1039/1081)
- **Function coverage:** 81.58% (31/38)

The `--all-features` flag enables `fast-kdf`, causing the alternative Argon2
parameter branch in `derive_key` to be compiled and executed by the unit tests,
which is why the totals differ slightly from a plain `cargo llvm-cov` run.

### File Breakdown

| File | Region Coverage | Function Coverage | Line Coverage |
|------|-----------------|-------------------|---------------|
| `src/crypto.rs` | 97.34% (804/826) | 80.77% (21/26) | 98.82% (419/424) |
| `src/main.rs` | 92.16% (235/255) | 83.33% (10/12) | **91.54%** (119/130) |
| **TOTAL** | **96.11% (1039/1081)** | **81.58% (31/38)** | **97.11% (554/570)** |

## Test Inventory

The project currently includes **51 automated tests** plus **3 fuzz targets**:

- Unit tests in `src/crypto.rs`: 17
- Unit tests in `src/main.rs` (`#[cfg(test)]`): 7
- Integration tests in `tests/integration_tests.rs`: 13
- Robustness tests in `tests/robustness_tests.rs`: 14

Fuzz targets (compiled with `cargo +nightly fuzz`, not counted in the 51 above):

- `fuzz/fuzz_targets/fuzz_decrypt_arbitrary.rs` — arbitrary bytes → `decrypt_file`
- `fuzz/fuzz_targets/fuzz_encrypt_plaintext.rs` — arbitrary plaintext, roundtrip assert
- `fuzz/fuzz_targets/fuzz_roundtrip.rs` — structured (plaintext + password), roundtrip + wrong-password check

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

## Fuzzing and coverage

Three fuzz targets complement the standard test suite (see `fuzz/fuzz_targets/`):

| Target | Primary coverage benefit |
|---|---|
| `fuzz_decrypt_arbitrary` | Exercises the exact-minimum-size boundary, auth-tag rejection, and every byte-offset mutation in the salt/nonce/ciphertext extraction at ~12 000–17 000 exec/s (with `fast-kdf`) |
| `fuzz_encrypt_plaintext` | Confirms the encrypt→decrypt roundtrip holds for all plaintext lengths (0 bytes, 1 byte, multi-megabyte) at ~2 000 exec/s |
| `fuzz_roundtrip` | Drives Argon2 key derivation with arbitrary passwords including empty, non-UTF-8, and very long strings at ~2 000–3 000 exec/s |

Fuzz targets are compiled with the `fast-kdf` feature (m=8 KiB, t=1, p=1
Argon2id parameters).  They use `cargo +nightly fuzz` and report coverage via
libfuzzer's built-in `cov:` counter, not `cargo-llvm-cov`.

To measure coverage *including* the fast-kdf code path in `cargo-llvm-cov`, run:

```bash
cargo llvm-cov --all-features --workspace --summary-only
```

(`--all-features` enables `fast-kdf`; the extra `#[cfg(feature = "fast-kdf")]`
branch in `derive_key` is then reachable by the existing unit tests.)

## Educational Context

The test suite — including unit, integration, and robustness tests — was generated with **GitHub Copilot** as part of a learning exercise in AI-assisted development. The high coverage (97.27% overall line coverage, 91.54% for `main.rs`) demonstrates what Copilot-driven test generation can achieve for a focused Rust project. The `run_operation()` refactor — extracting testable logic from the `main()` function — illustrates a Copilot-guided design pattern for improving coverage in CLI applications where stdin is not easily controllable in tests.
