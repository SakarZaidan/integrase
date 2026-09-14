//! Screening backends.
//!
//! We do not implement screening matching logic. A false negative is the worst
//! failure this system can produce, and two credible peer-reviewed open-source
//! implementations already exist. Writing our own matcher would mean inventing a
//! false-negative source to solve a solved problem (subplan 24).

use crate::verdict::{PolicyVersion, ScreeningVerdict};

/// Which backend produced a verdict.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BackendIdentity(pub String);

/// Why a backend could not return a verdict.
///
/// Every variant results in a refusal. None of them results in a pass.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum BackendError {
    /// No backend is configured.
    ///
    /// The default state, because curated sequence-of-concern data is not
    /// publicly downloadable and an unconfigured system is the common case for a
    /// new user.
    #[error("no screening backend configured")]
    NotConfigured,
    /// The backend could not be contacted.
    #[error("screening backend unreachable")]
    Unreachable,
    /// The backend did not respond in time.
    #[error("screening backend timed out")]
    Timeout,
    /// The response could not be parsed. Unparseable is not clean.
    #[error("screening backend returned a malformed response")]
    MalformedResponse,
    /// The response was truncated. A partial screen is not a screen.
    #[error("screening backend returned a partial response")]
    PartialResponse,
}

/// Something that can screen a sequence.
pub trait ScreeningBackend {
    /// Screens `sequence`, correlating the audit record with `id`.
    ///
    /// # Errors
    ///
    /// [`BackendError`] for any failure. Callers must treat every error as a
    /// refusal; there is no recoverable error here that permits a pass.
    fn screen(&self, sequence: &[u8], id: &str) -> Result<ScreeningVerdict, BackendError>;

    /// Identifies this backend, recorded in every verdict.
    fn identity(&self) -> BackendIdentity;

    /// The policy version this backend is currently applying.
    fn policy_version(&self) -> PolicyVersion;
}

/// The default backend, which refuses everything.
///
/// Three behaviours were possible when no reference data is configured, and only
/// one is honest:
///
/// | Behaviour | Verdict |
/// |---|---|
/// | Pass everything | Silently unsafe. Users would believe they were screened. |
/// | Ship a toy dataset | Worse. Implies real screening exists. |
/// | **Refuse, and say why** | Chosen. |
#[derive(Debug, Clone, Copy, Default)]
pub struct NullBackend;

impl ScreeningBackend for NullBackend {
    fn screen(&self, _sequence: &[u8], _id: &str) -> Result<ScreeningVerdict, BackendError> {
        Err(BackendError::NotConfigured)
    }
    fn identity(&self) -> BackendIdentity {
        BackendIdentity("null".into())
    }
    fn policy_version(&self) -> PolicyVersion {
        PolicyVersion("none".into())
    }
}

#[cfg(test)]
/// A backend that always fails with a given error, for fault injection.
#[derive(Debug)]
pub(crate) struct FailingBackend(pub BackendError);

#[cfg(test)]
impl ScreeningBackend for FailingBackend {
    fn screen(&self, _sequence: &[u8], _id: &str) -> Result<ScreeningVerdict, BackendError> {
        Err(self.0.clone())
    }
    fn identity(&self) -> BackendIdentity {
        BackendIdentity("failing-test-backend".into())
    }
    fn policy_version(&self) -> PolicyVersion {
        PolicyVersion("test".into())
    }
}

#[cfg(test)]
/// A backend that always returns clean, for testing the success path only.
#[derive(Debug)]
pub(crate) struct PassingBackend;

#[cfg(test)]
impl ScreeningBackend for PassingBackend {
    fn screen(&self, _sequence: &[u8], _id: &str) -> Result<ScreeningVerdict, BackendError> {
        Ok(ScreeningVerdict::new(
            Vec::new(),
            self.policy_version(),
            self.identity(),
        ))
    }
    fn identity(&self) -> BackendIdentity {
        BackendIdentity("passing-test-backend".into())
    }
    fn policy_version(&self) -> PolicyVersion {
        PolicyVersion("test".into())
    }
}
