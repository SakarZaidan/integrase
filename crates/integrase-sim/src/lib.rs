//! Biophysical models and the confidence layer.
//!
//! # The rule this crate exists to enforce
//!
//! **No function here returns a bare value for a predicted quantity.** Every
//! prediction is a [`Prediction<T>`], which carries the value, its interval, the
//! model that produced it, and a verdict on whether the input fell inside the
//! range where that model was validated.
//!
//! This is subplan 07's rule R-0.1 and it has no escape hatch. A reviewer who
//! sees `fn predict_something(..) -> f64` requests changes without further
//! discussion. The moment one convenient accessor returns a bare number,
//! callers use it, and uncertainty becomes optional. Optional uncertainty is
//! the state every other tool in this field is already in.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod confidence;
pub mod rbs;

pub use confidence::{DomainReason, DomainVerdict, Interval, ModelCard, ModelId, Prediction};
pub use rbs::{RbsError, RbsInput, RbsThermodynamicModel};

use integrase_core::Design;

/// A model that predicts something about a design.
///
/// Implementors return [`Prediction`], never a bare value. The trait signature
/// is the enforcement point.
pub trait Model {
    /// What this model consumes.
    type Input;
    /// The quantity it predicts.
    type Output;

    /// Identifier for this model, recorded in every prediction it makes.
    fn id(&self) -> ModelId;

    /// Published accuracy, validation datasets, and known failure modes.
    ///
    /// A model whose card carries no calibration evidence must report
    /// [`DomainVerdict::Uncalibrated`] rather than claim a coverage figure.
    fn card(&self) -> &ModelCard;

    /// Decides whether `input` falls within this model's validated domain.
    fn domain_check(&self, input: &Self::Input) -> DomainVerdict;

    /// Produces a prediction with its uncertainty attached.
    fn predict(&self, input: &Self::Input) -> Prediction<Self::Output>;
}

/// Errors arising from simulation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SimError {
    /// A prediction needs an annotated feature the design does not have.
    ///
    /// We do not infer the feature and predict on our own guess, because that
    /// would attribute our inference error to the model (subplan 11, E5).
    #[error("design has no feature with role {role}; cannot predict without one")]
    MissingFeature {
        /// The ontology IRI that was required.
        role: String,
    },
}

/// Convenience alias for the common case of predicting over a whole design.
pub type DesignModel<T> = dyn Model<Input = Design, Output = T>;
