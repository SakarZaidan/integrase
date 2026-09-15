//! SBOL interop.
//!
//! **This crate does not implement SBOL.** It adapts between [`sbol-rs`] types
//! and Integrase types, and carries the metadata SBOL has no place for.
//!
//! That decision was made by running the dependency rather than reading its
//! README (subplan 23). `sbol-rs` ships 348 machine-checkable validation rules
//! across SBOL 3.1.0 and 2.3.0, and a probe confirmed 33 of 33 pySBOL3
//! cross-implementation fixtures round-trip losslessly. Reimplementing that
//! would mean owning those rules incompletely with no cross-implementation
//! oracle.
//!
//! # Round-trip comparison is semantic, never syntactic
//!
//! Measured: SBOL 2 to 3 and back is **not** triple-identical, and correctly so.
//! Ontology URIs are canonicalised, and `persistentIdentity` and `version`
//! cannot survive because SBOL 3 removed them. A test asserting triple equality
//! fails on correct behaviour, and the usual outcome is that somebody weakens it
//! until it passes.
//!
//! [`sbol-rs`]: https://github.com/SynBioDex/sbol-rs

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(clippy::module_name_repetitions)]

pub mod export;
pub mod import;

pub use export::{VendorFormat, export_for_synthesis, write_sbol};
pub use import::{ImportResult, from_document, from_turtle};

/// Errors from SBOL import and export.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SbolError {
    /// The document could not be read.
    #[error("could not read SBOL document: {detail}")]
    Read {
        /// What the underlying library reported.
        detail: String,
    },
    /// A feature referenced a role IRI we could not resolve.
    #[error("unresolved ontology term {iri}")]
    UnresolvedTerm {
        /// The IRI in question.
        iri: String,
    },
    /// The document exceeds the size this importer will parse.
    ///
    /// Refused before parsing rather than after allocation.
    #[error("document is {len} bytes; this importer refuses anything over {max}")]
    TooLarge {
        /// Size supplied.
        len: usize,
        /// The limit.
        max: usize,
    },
}

/// What a version conversion changed.
///
/// `sbol-rs` returns an upgrade or downgrade report alongside a converted
/// document. Discarding that would be inconsistent with a project whose thesis
/// is not hiding what it knows, so conversion reports are user-facing data
/// rather than debug output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConversionReport {
    /// Human-readable notes about what the conversion altered or dropped.
    pub notes: Vec<String>,
}

impl ConversionReport {
    /// Returns true if the conversion altered nothing worth reporting.
    #[must_use]
    pub fn is_silent(&self) -> bool {
        self.notes.is_empty()
    }
}
