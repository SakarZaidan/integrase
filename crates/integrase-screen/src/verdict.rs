//! Screening verdicts and refusals.

use crate::backend::{BackendError, BackendIdentity};

/// The version of the screening policy that produced a verdict.
///
/// Thresholds and reference data are versioned configuration, never compiled
/// constants. The OSTP framework's flagging threshold drops to 50 bp and its
/// sequence-of-concern definition expands in October 2026, with IGSC conformance
/// due 2026-10-24. The rules move on a known cadence, so every verdict records
/// the policy that produced it and an audit can reconstruct why a past decision
/// was made.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PolicyVersion(pub String);

/// What class of concern a hit belongs to.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum HitClass {
    /// Matches a sequence associated with a regulated pathogen.
    RegulatedPathogen,
    /// Matches a sequence associated with a toxin.
    Toxin,
    /// Flagged by the backend without a more specific classification.
    Unclassified,
}

/// A single region the backend flagged.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Hit {
    /// Start offset of the matched region, 0-based.
    pub start: usize,
    /// Exclusive end offset of the matched region.
    pub end: usize,
    /// What kind of concern.
    pub class: HitClass,
    /// The backend's own description, passed through verbatim.
    ///
    /// We report the backend's verdict as given and never apply our own
    /// threshold on top. Adding an interpretation layer over a peer-reviewed
    /// screener would mean inventing a false-negative source.
    pub detail: String,
}

/// The result of screening a sequence.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScreeningVerdict {
    hits: Vec<Hit>,
    /// Which policy version produced this verdict.
    pub policy_version: PolicyVersion,
    /// Which backend produced it.
    pub backend: BackendIdentity,
}

impl ScreeningVerdict {
    /// Builds a verdict from a backend's findings.
    #[must_use]
    pub const fn new(
        hits: Vec<Hit>,
        policy_version: PolicyVersion,
        backend: BackendIdentity,
    ) -> Self {
        Self {
            hits,
            policy_version,
            backend,
        }
    }

    /// Returns true if nothing was flagged.
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.hits.is_empty()
    }

    /// The flagged regions, if any.
    #[must_use]
    pub fn hits(&self) -> &[Hit] {
        &self.hits
    }
}

/// Why a design was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RefusalCause {
    /// Screening found something.
    Hit {
        /// The flagged regions.
        hits: Vec<Hit>,
    },
    /// Screening could not be completed.
    ///
    /// Distinct from [`RefusalCause::Hit`] on purpose. Both block export, and
    /// they mean entirely different things to a user.
    BackendFailure(BackendError),
}

/// A refusal to clear a design for synthesis-bound export.
///
/// Modelled as a distinct type rather than lumped in with I/O errors, because a
/// refusal is the system working correctly (subplan 12, R-12.3). The UI should
/// explain it, not render a stack trace.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("screening refused: {cause:?} (backend {backend:?}, policy {policy_version:?})")]
pub struct Refusal {
    /// Why.
    pub cause: RefusalCause,
    /// Which backend was involved.
    pub backend: BackendIdentity,
    /// Which policy version applied.
    pub policy_version: PolicyVersion,
}

impl Refusal {
    /// A short, user-facing explanation and the route forward.
    ///
    /// A hit is overwhelmingly likely to be a researcher who did not know a
    /// component was regulated, so the wording points at institutional review
    /// rather than reading as an accusation.
    #[must_use]
    pub fn explain(&self) -> String {
        match &self.cause {
            RefusalCause::Hit { hits } => format!(
                "{} region(s) matched biosecurity screening criteria. The design remains fully \
                 usable for analysis, simulation, and interchange export; only synthesis-bound \
                 export is blocked. To proceed, consult your institutional biosafety officer about \
                 an exemption certificate.",
                hits.len()
            ),
            RefusalCause::BackendFailure(BackendError::NotConfigured) => {
                "No screening backend is configured, so synthesis-bound export is refused. \
                 Analysis, simulation, and interchange export are unaffected. Configure a backend \
                 to enable ordering."
                    .into()
            }
            RefusalCause::BackendFailure(e) => format!(
                "Screening could not be completed ({e}). This is not a finding against your \
                 design. Synthesis-bound export fails closed until screening succeeds."
            ),
        }
    }
}
