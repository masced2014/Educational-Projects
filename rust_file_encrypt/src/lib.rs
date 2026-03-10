//! # secure-file-crypto — library entry point
//!
//! This crate exposes its cryptographic core as a library so it can be consumed
//! by integration tests, fuzz targets, and any future tooling without going
//! through the CLI binary.
//!
//! The only public surface needed by external callers is the [`crypto`] module
//! containing [`crypto::FileCrypto`].
//!
//! ## Feature flags
//!
//! | Feature | Purpose |
//! |---------|---------|
//! | `fast-kdf` | Replaces the production Argon2id parameters with minimal cost parameters. **Never use in production.** Intended exclusively for fuzz targets and CI jobs where fast iteration matters more than KDF hardness. |

pub mod crypto;
