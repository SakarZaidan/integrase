//! Sequence and design types for Integrase.
//!
//! This crate is the substrate. It deliberately depends on no other Integrase
//! crate (subplan 12, R-12.5), which keeps it reusable and keeps compile and
//! test times low. If something here needs simulation or screening, the design
//! is wrong and the thing belongs in that crate instead.
//!
//! # Coordinate convention
//!
//! **All coordinates are 0-based and half-open**, everywhere, without exception.
//!
//! This is stated once here and enforced at every adapter boundary. GenBank is
//! 1-based inclusive, BED is 0-based half-open, and SBOL has its own rules.
//! Mixed conventions are the endemic off-by-one source in bioinformatics, and
//! the only defence is converting at the edges and never in the middle.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod design;
pub mod error;
pub mod index;
pub mod provenance;
pub mod seq;

pub use design::{Design, DesignId, Feature, FeatureId, Location, OntologyTerm, Strand};
pub use error::CoreError;
pub use index::{
    COMMON_ENZYMES, Match, RestrictionEnzyme, SiteHit, find_pattern, find_restriction_sites,
};
pub use provenance::Provenance;
pub use seq::{Alphabet, Sequence, SequenceView};
