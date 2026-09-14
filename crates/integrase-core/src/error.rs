//! Error types for this crate.
//!
//! Per subplan 12 (R-12.1), every variant carries the values involved. An error
//! that says "invalid residue" tells a user nothing; one that names the byte and
//! the offset tells them where to look.

use crate::seq::Alphabet;

/// Anything that can go wrong in sequence or design manipulation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CoreError {
    /// A byte in the input is not valid for the declared alphabet.
    #[error("byte {byte:#04x} at offset {offset} is not valid for {alphabet:?}")]
    InvalidResidue {
        /// The offending byte, as it appeared in the input.
        byte: u8,
        /// Zero-based offset into the sequence.
        offset: usize,
        /// The alphabet the sequence was declared to use.
        alphabet: Alphabet,
    },

    /// A location refers to coordinates outside the sequence it belongs to.
    #[error("location {start}..{end} exceeds sequence length {length}")]
    LocationOutOfBounds {
        /// Inclusive start, 0-based.
        start: usize,
        /// Exclusive end.
        end: usize,
        /// Length of the sequence the location was applied to.
        length: usize,
    },

    /// A location's end precedes its start.
    #[error("location start {start} is greater than end {end}")]
    InvertedLocation {
        /// Inclusive start, 0-based.
        start: usize,
        /// Exclusive end.
        end: usize,
    },

    /// An index or analysis was requested on input larger than is supported.
    ///
    /// Returned *before* allocation, per subplan 11 edge case E3. An
    /// out-of-memory kill is not an error message.
    #[error(
        "cannot index {length} bp: projected {projected_gb:.1} GB at {bytes_per_base} bytes/base, \
         {available_gb:.1} GB available"
    )]
    TooLargeToIndex {
        /// Length of the input in bases.
        length: usize,
        /// Projected memory requirement in gigabytes.
        projected_gb: f64,
        /// Memory currently available, in gigabytes.
        available_gb: f64,
        /// The per-base cost used for the projection.
        bytes_per_base: f64,
    },
}
