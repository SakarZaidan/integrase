//! Verifiable design records.
//!
//! A design record binds a design, the predictions made about it, and the
//! screening verdict that cleared it, into a signed, independently checkable
//! artifact (subplan 29).
//!
//! Depends only on `integrase-core` and cryptographic primitives, so a verifier
//! can validate a record without any other part of Integrase present. That
//! independence is the point: a record nobody but us can check is not a
//! verifiable record.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Errors from record construction and verification.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum VdrError {
    /// The record's signature did not validate.
    #[error("record signature is invalid")]
    BadSignature,
    /// The record's canonicalisation identifier is not one we implement.
    #[error("unsupported canonicalisation {name}")]
    UnsupportedCanonicalisation {
        /// The identifier found in the record.
        name: String,
    },
}
