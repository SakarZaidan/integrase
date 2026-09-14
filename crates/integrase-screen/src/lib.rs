//! The biosafety screening gateway.
//!
//! # The rule this crate exists to enforce
//!
//! **No design reaches a synthesis-bound export without passing through here.**
//! [`Cleared`] has a private field and no public constructor, so it can only be
//! produced inside this crate. Synthesis-bound export functions accept
//! `Cleared<T>` and nothing else, which makes the guarantee a compile-time
//! property rather than a review checklist item.
//!
//! The brief asks that it be obvious nothing bypasses the gate. It is obvious
//! because the alternative does not compile.
//!
//! # What is and is not gated
//!
//! Two export classes, and the distinction is the **flow**, not the file format
//! (subplan 21, amending subplan 06):
//!
//! - **Interchange** (SBOL round-trip, working files, figures) is *not* gated.
//!   A design that came from the ecosystem returns to it, and we are not the
//!   synthesis boundary there.
//! - **Synthesis-bound** (the explicit ordering flow, vendor formats) requires
//!   `Cleared<T>`.
//!
//! The same FASTA bytes are interchange when a user saves a working file and
//! synthesis-bound when produced by the ordering path.
//!
//! # Fail closed
//!
//! Every uncertain outcome resolves to refusal. Unreachable backend, timeout,
//! malformed response, partial response: all of them produce a [`Refusal`],
//! never a pass. There is no code path where "could not determine" becomes
//! "allowed".

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod backend;
pub mod verdict;

pub use backend::{BackendError, BackendIdentity, NullBackend, ScreeningBackend};
pub use verdict::{HitClass, PolicyVersion, Refusal, RefusalCause, ScreeningVerdict};

use integrase_core::Design;

/// A design that has passed screening.
///
/// The inner value is private and there is no public constructor. This is the
/// entire mechanism: [`ScreeningGateway::screen`] is the only function anywhere
/// that can produce one.
#[derive(Debug, Clone)]
pub struct Cleared<T> {
    inner: T,
    verdict: ScreeningVerdict,
}

impl<T> Cleared<T> {
    /// The verdict that cleared this item, including which backend and policy
    /// version produced it.
    #[must_use]
    pub const fn verdict(&self) -> &ScreeningVerdict {
        &self.verdict
    }

    /// Borrows the cleared item.
    #[must_use]
    pub const fn get(&self) -> &T {
        &self.inner
    }

    /// Consumes the wrapper, yielding the item.
    ///
    /// Note that this does not defeat the gate: the caller already had to pass
    /// screening to obtain the `Cleared` in the first place, and export
    /// functions take `&Cleared<T>` so the verdict travels with the value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

/// Anything that can be submitted for screening.
///
/// The surface is deliberately minimal (subplan 24). A backend gets the
/// sequence, its length, and an identifier for audit correlation. It does not
/// get design names, user-authored descriptions, annotations, project metadata,
/// or user identity. Those would be a privacy leak with no screening benefit,
/// and the natural drift is toward sending more context "in case it helps".
pub trait Screenable {
    /// The residues to screen.
    fn sequence_bytes(&self) -> &[u8];
    /// An identifier used only to correlate the audit record.
    fn screening_id(&self) -> String;
}

impl Screenable for Design {
    fn sequence_bytes(&self) -> &[u8] {
        self.sequence.as_bytes()
    }
    fn screening_id(&self) -> String {
        self.id.0.clone()
    }
}

/// The gate.
pub trait ScreeningGateway {
    /// Screens an item, returning [`Cleared`] on a pass and [`Refusal`] otherwise.
    ///
    /// This is the only constructor of [`Cleared`] in the system.
    ///
    /// # Errors
    ///
    /// [`Refusal`] on a screening hit, on a backend failure of any kind, and
    /// when no backend is configured. All three block export; the cause
    /// distinguishes them, because "screening said no" and "screening could not
    /// be reached" mean different things to a user.
    fn screen<T: Screenable>(&self, item: T) -> Result<Cleared<T>, Refusal>;
}

/// A gateway backed by a [`ScreeningBackend`].
#[derive(Debug)]
pub struct Gateway<B: ScreeningBackend> {
    backend: B,
}

impl<B: ScreeningBackend> Gateway<B> {
    /// Wraps a backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }
}

impl<B: ScreeningBackend> ScreeningGateway for Gateway<B> {
    fn screen<T: Screenable>(&self, item: T) -> Result<Cleared<T>, Refusal> {
        let verdict = self
            .backend
            .screen(item.sequence_bytes(), &item.screening_id())
            // Any backend failure is a refusal. This mapping is the fail-closed
            // property, and subplan 17 fault-injects every variant of it.
            .map_err(|e| Refusal {
                cause: RefusalCause::BackendFailure(e),
                backend: self.backend.identity(),
                policy_version: self.backend.policy_version(),
            })?;

        if verdict.is_clean() {
            Ok(Cleared {
                inner: item,
                verdict,
            })
        } else {
            Err(Refusal {
                cause: RefusalCause::Hit {
                    hits: verdict.hits().to_vec(),
                },
                backend: self.backend.identity(),
                policy_version: verdict.policy_version.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{FailingBackend, PassingBackend};

    #[derive(Debug)]
    struct Seq(Vec<u8>);
    impl Screenable for Seq {
        fn sequence_bytes(&self) -> &[u8] {
            &self.0
        }
        fn screening_id(&self) -> String {
            "test".into()
        }
    }

    #[test]
    fn null_backend_refuses_everything_including_empty_input() {
        let g = Gateway::new(NullBackend);
        assert!(g.screen(Seq(b"ACGT".to_vec())).is_err());
        assert!(
            g.screen(Seq(Vec::new())).is_err(),
            "empty input is not a free pass"
        );
    }

    #[test]
    fn null_backend_refusal_says_it_is_unconfigured_not_that_it_found_something() {
        let g = Gateway::new(NullBackend);
        let r = g.screen(Seq(b"ACGT".to_vec())).unwrap_err();
        assert!(
            matches!(
                r.cause,
                RefusalCause::BackendFailure(BackendError::NotConfigured)
            ),
            "a user must be able to tell 'no screener' from 'screener said no'"
        );
    }

    /// Fault injection: every backend failure mode must fail closed.
    /// Subplan 17 names this as the property most likely to rot silently.
    #[test]
    fn every_backend_failure_mode_fails_closed() {
        for err in [
            BackendError::Unreachable,
            BackendError::Timeout,
            BackendError::MalformedResponse,
            BackendError::PartialResponse,
            BackendError::NotConfigured,
        ] {
            let g = Gateway::new(FailingBackend(err.clone()));
            let result = g.screen(Seq(b"ACGT".to_vec()));
            assert!(
                result.is_err(),
                "backend failure {err:?} must refuse, never pass"
            );
        }
    }

    #[test]
    fn a_clean_verdict_produces_cleared() {
        let g = Gateway::new(PassingBackend);
        let cleared = g
            .screen(Seq(b"ACGT".to_vec()))
            .expect("clean backend should clear");
        assert!(cleared.verdict().is_clean());
        assert_eq!(cleared.get().0, b"ACGT".to_vec());
    }
}
