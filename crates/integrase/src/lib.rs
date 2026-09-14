//! Integrase: genetic design with the error bars attached.
//!
//! An open-source platform for designing, simulating, and screening genetic
//! constructs, built so that every prediction arrives with the uncertainty that
//! belongs to it.
//!
//! # Scope, stated up front
//!
//! Integrase does not edit genomes, does not interact with living organisms, and
//! does not order DNA. It operates on sequence data. Predictions are model
//! outputs, not measurements, and their intervals describe uncertainty about the
//! model's prediction rather than about what will happen in a cell.
//!
//! # The two guarantees
//!
//! 1. **No API returns a bare value for a predicted quantity.** Everything comes
//!    back as [`Prediction`], which carries the interval, the model, and a
//!    verdict on whether the input was inside that model's validated range.
//! 2. **No design reaches a synthesis-bound export without screening.**
//!    [`Cleared`] is constructible only inside `integrase-screen`, and
//!    synthesis-bound export functions accept nothing else.
//!
//! Both are enforced by the type system rather than by convention.
//!
//! # Stability
//!
//! What is re-exported here is the stable surface. Semver applies to it. Items
//! reachable only through the [`unstable`] module may change in any release, and
//! depending on them is opting out of the versioning policy.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use integrase_core::{
    Alphabet, CoreError, Design, DesignId, Feature, FeatureId, Location, OntologyTerm, Provenance,
    Sequence, SequenceView, Strand,
};
pub use integrase_sbol::{ConversionReport, SbolError};
pub use integrase_screen::{
    BackendError, Cleared, Refusal, RefusalCause, Screenable, ScreeningBackend, ScreeningGateway,
    ScreeningVerdict,
};
pub use integrase_sim::{
    DomainReason, DomainVerdict, Interval, Model, ModelCard, ModelId, Prediction, SimError,
};

/// Unstable surface. May change in any release, including a patch.
///
/// This module exists as a deliberate escape valve. Without one, users depend on
/// internals anyway and then get broken, and we get blamed. With one, they opt
/// in visibly and we owe them nothing.
pub mod unstable {
    pub use integrase_logic::LogicError;
    pub use integrase_vdr::VdrError;
}
