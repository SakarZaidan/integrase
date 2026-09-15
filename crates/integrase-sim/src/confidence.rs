//! The confidence layer: values that cannot be separated from their uncertainty.

use std::fmt;

/// Identifier for a specific model version.
///
/// Records architecture, training data version, and calibration set version, so
/// any past prediction is reproducible and any past claim is auditable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ModelId(pub String);

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An inclusive interval around a predicted value.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Interval {
    /// Lower bound.
    pub lower: f64,
    /// Upper bound.
    pub upper: f64,
}

impl Interval {
    /// Builds an interval, ordering the bounds if they arrive reversed.
    #[must_use]
    pub fn new(a: f64, b: f64) -> Self {
        if a <= b {
            Self { lower: a, upper: b }
        } else {
            Self { lower: b, upper: a }
        }
    }

    /// Returns true if `value` falls within the interval.
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }

    /// Width of the interval.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }
}

/// Why an input was judged inside or outside a model's validated domain.
// PartialEq but not Eq: two variants carry f64, and f64 is not Eq because NaN
// is not reflexive. Deriving Eq here would not compile, and asserting it via a
// wrapper would be lying about float semantics.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum DomainReason {
    /// The host organism differs from the one the model was fitted on.
    HostMismatch {
        /// The host the input specifies.
        got: String,
        /// The host the model was validated on.
        expected: String,
    },
    /// A non-canonical start codon, which the model's parameters barely cover.
    NonCanonicalStartCodon {
        /// The codon found.
        codon: String,
    },
    /// Spacer length outside the well-sampled range of the fitted penalty curve.
    SpacerOutOfRange {
        /// The observed spacer length.
        length: usize,
    },
    /// Sequence length outside the range the model was fitted on.
    LengthOutOfRange {
        /// The observed length.
        length: usize,
    },
    /// Predicted secondary structure is strong enough to dominate the estimate.
    StrongSecondaryStructure {
        /// Folding free energy, kcal/mol.
        free_energy_kcal_per_mol: f64,
    },
    /// A heuristic suggests the sequence may fold, without computing how much.
    ///
    /// Distinct from [`DomainReason::StrongSecondaryStructure`] deliberately.
    /// That variant carries a real folding free energy. This one carries only
    /// the fraction of the sequence sitting in a possible stem, because no
    /// structure prediction was performed. Reporting a NaN energy would claim a
    /// computation that did not happen.
    PossibleSecondaryStructure {
        /// Fraction of the sequence within a candidate stem, 0 to 1.
        stem_fraction: f64,
    },
    /// The input is far from the calibration set in embedding space.
    EmbeddingDistance {
        /// Distance, in the units the model documents.
        distance: f64,
    },
}

impl fmt::Display for DomainReason {
    /// Human-readable, and deliberately the single definition.
    ///
    /// The CLI and the Python bindings both render these. Two separate
    /// descriptions would drift, and a user comparing output from the two
    /// surfaces would see the same verdict worded differently, which invites
    /// the reasonable conclusion that they mean different things.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No wildcard arm, deliberately. `non_exhaustive` binds other crates,
        // not this one, so this match is exhaustive here and the compiler will
        // reject a new variant that has no rendering. A domain reason a user
        // cannot read is a bug, and it should fail the build rather than print
        // a shrug.
        match self {
            Self::HostMismatch { got, expected } => {
                write!(f, "host {got} differs from the fitted host {expected}")
            }
            Self::NonCanonicalStartCodon { codon } => {
                write!(f, "non-canonical start codon {codon}")
            }
            Self::SpacerOutOfRange { length } => {
                write!(f, "spacer length {length} is outside the fitted range")
            }
            Self::LengthOutOfRange { length } => {
                write!(f, "length {length} is outside the fitted range")
            }
            Self::StrongSecondaryStructure {
                free_energy_kcal_per_mol,
            } => {
                write!(
                    f,
                    "strong secondary structure, {free_energy_kcal_per_mol:.2} kcal/mol"
                )
            }
            Self::PossibleSecondaryStructure { stem_fraction } => write!(
                f,
                "possible secondary structure, {:.0}% of the sequence could sit in a stem",
                stem_fraction * 100.0
            ),
            Self::EmbeddingDistance { distance } => {
                write!(
                    f,
                    "embedding distance {distance:.3} from the calibration set"
                )
            }
        }
    }
}

/// Whether an input falls inside the range where a model was validated.
///
/// This is deliberately a **separate field** from the interval, and that
/// separation is the technical core of what the project calls confidence
/// cartography.
///
/// Split conformal prediction gives a coverage guarantee under *exchangeability*
/// between the calibration set and the test point. Out-of-distribution inputs
/// are precisely where exchangeability fails, so the guarantee evaporates
/// exactly where a user most needs it. The two fields answer different
/// questions:
///
/// - the interval: given that this input is like my calibration data, how wrong
///   am I likely to be?
/// - this verdict: is this input like my calibration data at all?
///
/// Collapsing them into one widened interval would hide the second question,
/// and the second question is what separates a sequence the model understands
/// from one it has never seen anything like.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum DomainVerdict {
    /// Input resembles the model's validation set.
    InDistribution {
        /// Similarity score, 0 to 1, in the units the model documents.
        similarity: f64,
    },
    /// Input sits at the edge. Intervals are reported but widened.
    Marginal {
        /// Similarity score, 0 to 1.
        similarity: f64,
        /// What triggered the marginal call.
        reason: DomainReason,
    },
    /// Outside anything the model was validated on.
    ///
    /// A value is still produced. Refusing to predict anything unfamiliar makes
    /// the tool useless for novel design, which is what the field is for. But
    /// the output is marked unvalidated and must never be presented as a
    /// measurement.
    OutOfDistribution {
        /// What put it outside.
        reason: DomainReason,
    },
    /// The model has no calibration evidence at all.
    ///
    /// The honest default for any model that has not been through an evaluation.
    /// It is a default rather than an error so that a contributor can add a
    /// model without first producing calibration data, and the system reports
    /// truthfully that it has none rather than implying confidence it has not
    /// earned.
    Uncalibrated,
}

impl DomainVerdict {
    /// Returns true if the model was validated on inputs like this one.
    #[must_use]
    pub const fn is_in_distribution(&self) -> bool {
        matches!(self, Self::InDistribution { .. })
    }

    /// Returns true if this prediction carries no usable coverage guarantee.
    ///
    /// True for both out-of-distribution and uncalibrated, because in both cases
    /// the stated coverage does not hold.
    #[must_use]
    pub const fn coverage_is_unreliable(&self) -> bool {
        matches!(self, Self::OutOfDistribution { .. } | Self::Uncalibrated)
    }
}

/// Published accuracy, validation data, and known failure modes for a model.
///
/// Machine-readable and required, so that documentation is generated from the
/// code that makes the claims rather than hand-written alongside it. A
/// hand-written accuracy number is a number that will eventually be wrong, and
/// in this project a wrong accuracy claim is worse than no documentation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ModelCard {
    /// Which model this describes.
    pub id: ModelId,
    /// One-line description of what it predicts.
    pub summary: String,
    /// Datasets it was fitted on, by citation or accession.
    pub training_data: Vec<String>,
    /// Datasets calibration was measured on, disjoint from training.
    pub calibration_data: Vec<String>,
    /// Known failure modes, in prose a biologist can act on.
    pub known_failure_modes: Vec<String>,
    /// Measured coverage, if calibration has actually been demonstrated.
    ///
    /// `None` means no calibration evidence exists, and any model in that state
    /// must report [`DomainVerdict::Uncalibrated`].
    pub measured_coverage: Option<MeasuredCoverage>,
}

/// Evidence that a model's stated intervals hold on held-out data.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MeasuredCoverage {
    /// The coverage level claimed, for example 0.80.
    pub nominal: f64,
    /// The coverage actually observed on held-out data.
    pub observed: f64,
    /// How many held-out points the observation is based on.
    pub n: usize,
    /// Whether coverage was checked within strata, not only marginally.
    ///
    /// Marginal coverage is easy to achieve while being useless: a model
    /// overconfident on weak signals and underconfident on strong ones can hit
    /// 80% overall while being systematically wrong in a way users experience.
    pub stratified: bool,
}

/// A predicted quantity, inseparable from its uncertainty.
///
/// There is no `From<Prediction<T>> for T`, no `Deref`, and no `into_value()`
/// that discards the rest. Getting the number out requires naming the field,
/// which is a deliberate three-second cost that preserves the product's thesis.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Prediction<T> {
    value: T,
    interval: Option<Interval>,
    nominal_coverage: f64,
    model: ModelId,
    domain: DomainVerdict,
}

impl<T> Prediction<T> {
    /// Builds a calibrated prediction.
    ///
    /// Use [`Prediction::uncalibrated`] when no calibration evidence exists;
    /// this constructor is for models that have been evaluated.
    #[must_use]
    pub fn new(
        value: T,
        interval: Interval,
        nominal_coverage: f64,
        model: ModelId,
        domain: DomainVerdict,
    ) -> Self {
        Self {
            value,
            interval: Some(interval),
            nominal_coverage,
            model,
            domain,
        }
    }

    /// Builds a prediction with no interval, preserving the domain verdict.
    ///
    /// No interval is attached, because a model that has not demonstrated
    /// coverage has no honest interval to report. Absence is the truthful
    /// rendering; a wide interval would imply a quantified uncertainty we have
    /// not earned.
    ///
    /// **The verdict is carried through rather than replaced.** An earlier
    /// version hardcoded [`DomainVerdict::Uncalibrated`] here, which silently
    /// discarded stronger signals: a sequence from the wrong organism reported
    /// only "uncalibrated" and the host mismatch vanished between
    /// `domain_check` and the value the user received. The Rust unit tests
    /// missed it because they exercised `domain_check` directly; the Python
    /// binding test caught it because it went through the real path.
    #[must_use]
    pub fn without_interval(value: T, model: ModelId, domain: DomainVerdict) -> Self {
        Self {
            value,
            interval: None,
            nominal_coverage: 0.0,
            model,
            domain,
        }
    }

    /// Convenience for a model with no calibration evidence and nothing else to
    /// report.
    #[must_use]
    pub fn uncalibrated(value: T, model: ModelId) -> Self {
        Self::without_interval(value, model, DomainVerdict::Uncalibrated)
    }

    /// The predicted value.
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// The interval, if this model has demonstrated calibration.
    #[must_use]
    pub const fn interval(&self) -> Option<Interval> {
        self.interval
    }

    /// The coverage level the interval claims, for example 0.80.
    #[must_use]
    pub const fn nominal_coverage(&self) -> f64 {
        self.nominal_coverage
    }

    /// Which model produced this.
    #[must_use]
    pub const fn model(&self) -> &ModelId {
        &self.model
    }

    /// Whether the input fell inside the model's validated domain.
    #[must_use]
    pub const fn domain(&self) -> &DomainVerdict {
        &self.domain
    }

    /// Applies `f` to the value, preserving all uncertainty metadata.
    ///
    /// The only sanctioned way to transform a prediction. It cannot drop the
    /// interval or the domain verdict, which is the point.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Prediction<U> {
        Prediction {
            value: f(self.value),
            interval: self.interval,
            nominal_coverage: self.nominal_coverage,
            model: self.model,
            domain: self.domain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card_id() -> ModelId {
        ModelId("rbs-thermo-v1".into())
    }

    #[test]
    fn uncalibrated_predictions_carry_no_interval() {
        let p = Prediction::uncalibrated(4200.0_f64, card_id());
        assert!(
            p.interval().is_none(),
            "no calibration evidence means no honest interval"
        );
        assert_eq!(*p.domain(), DomainVerdict::Uncalibrated);
        assert!(p.domain().coverage_is_unreliable());
    }

    #[test]
    fn interval_orders_reversed_bounds() {
        let i = Interval::new(9100.0, 1900.0);
        assert_eq!(i.lower, 1900.0);
        assert_eq!(i.upper, 9100.0);
    }

    #[test]
    fn interval_contains_its_point_estimate() {
        let i = Interval::new(1900.0, 9100.0);
        assert!(i.contains(4200.0));
        assert!(!i.contains(42.0));
    }

    #[test]
    fn map_preserves_uncertainty_metadata() {
        let p = Prediction::new(
            4200.0_f64,
            Interval::new(1900.0, 9100.0),
            0.80,
            card_id(),
            DomainVerdict::InDistribution { similarity: 0.91 },
        );
        let scaled = p.clone().map(|v| v / 1000.0);
        assert_eq!(*scaled.value(), 4.2);
        assert_eq!(
            scaled.interval(),
            p.interval(),
            "map must not drop the interval"
        );
        assert_eq!(
            scaled.domain(),
            p.domain(),
            "map must not drop the domain verdict"
        );
        assert_eq!(scaled.nominal_coverage(), 0.80);
    }

    #[test]
    fn out_of_distribution_flags_coverage_as_unreliable() {
        let v = DomainVerdict::OutOfDistribution {
            reason: DomainReason::HostMismatch {
                got: "Paracoccus denitrificans".into(),
                expected: "Escherichia coli".into(),
            },
        };
        assert!(v.coverage_is_unreliable());
        assert!(!v.is_in_distribution());
    }
}
