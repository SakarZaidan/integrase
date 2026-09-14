//! Genetic circuit logic compilation.
//!
//! Cello solved technology mapping for genetic logic and is BSD-2-Clause with
//! 142 forks. It is dormant, not wrong. This crate revives and interoperates
//! with that work rather than reimplementing it (subplan 25).
//!
//! # What is new here
//!
//! **Uncertainty propagated compositionally through a circuit.** Interval
//! propagation through characterised gate libraries is published research (ACS
//! Synthetic Biology, 2021) that no usable tool implements. Implementing it
//! properly would be a first, and the distinction matters: we did not invent
//! the method, we are building the tool that was missing.
//!
//! Characterised Cello circuits are documented to contain cryptic promoters,
//! transcription attenuation, alternative start sites, failed parts, off-target
//! interactions, and occasionally a gate that fails completely. Current tools
//! surface none of that. A circuit score that carries its own uncertainty does.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Errors from circuit compilation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LogicError {
    /// The specification could not be parsed.
    #[error("could not parse logic specification: {detail}")]
    Parse {
        /// Parser detail.
        detail: String,
    },
    /// No assignment of gates satisfies the specification with the given library.
    #[error("no satisfying gate assignment found in library {library}")]
    NoAssignment {
        /// Which gate library was used.
        library: String,
    },
}
