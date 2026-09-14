//! Nucleotide and residue sequences.
//!
//! Sequences are stored as bytes rather than `String`. A genome is not text,
//! and treating it as UTF-8 buys nothing while costing validation on every
//! index.

use crate::error::CoreError;

/// Which residues a sequence may contain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum Alphabet {
    /// A, C, G, T only.
    Dna,
    /// A, C, G, T plus the IUPAC ambiguity codes.
    ///
    /// Ambiguity is preserved, never silently resolved to a concrete base
    /// (subplan 11, edge case E7).
    DnaIupac,
    /// A, C, G, U only.
    Rna,
}

impl Alphabet {
    /// Returns true if `b` is a valid residue in this alphabet.
    ///
    /// Case-insensitive: real files contain both, and rejecting lowercase would
    /// fail on soft-masked reference sequence for no benefit.
    #[must_use]
    pub const fn accepts(self, b: u8) -> bool {
        let u = b.to_ascii_uppercase();
        match self {
            Self::Dna => matches!(u, b'A' | b'C' | b'G' | b'T'),
            Self::Rna => matches!(u, b'A' | b'C' | b'G' | b'U'),
            Self::DnaIupac => matches!(
                u,
                b'A' | b'C'
                    | b'G'
                    | b'T'
                    | b'R'
                    | b'Y'
                    | b'S'
                    | b'W'
                    | b'K'
                    | b'M'
                    | b'B'
                    | b'D'
                    | b'H'
                    | b'V'
                    | b'N'
            ),
        }
    }

    /// Complements a single residue, preserving case.
    ///
    /// Returns `None` for residues this alphabet does not accept, so callers
    /// cannot silently complement garbage into plausible-looking sequence.
    #[must_use]
    pub const fn complement(self, b: u8) -> Option<u8> {
        if !self.accepts(b) {
            return None;
        }
        let lower = b.is_ascii_lowercase();
        let c = match b.to_ascii_uppercase() {
            b'A' => {
                if matches!(self, Self::Rna) {
                    b'U'
                } else {
                    b'T'
                }
            }
            b'C' => b'G',
            b'G' => b'C',
            b'T' | b'U' => b'A',
            // IUPAC ambiguity codes complement to their mirror code.
            b'R' => b'Y',
            b'Y' => b'R',
            b'S' => b'S',
            b'W' => b'W',
            b'K' => b'M',
            b'M' => b'K',
            b'B' => b'V',
            b'V' => b'B',
            b'D' => b'H',
            b'H' => b'D',
            b'N' => b'N',
            _ => return None,
        };
        Some(if lower { c.to_ascii_lowercase() } else { c })
    }
}

/// Which strand a view reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Strand {
    /// The sequence as stored.
    Forward,
    /// The reverse complement of the sequence as stored.
    Reverse,
}

impl Strand {
    /// Returns the opposite strand.
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

/// An owned residue sequence with a declared alphabet.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Sequence {
    data: Vec<u8>,
    alphabet: Alphabet,
}

impl Sequence {
    /// Builds a sequence, validating every residue against `alphabet`.
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidResidue`] naming the first offending byte and its
    /// offset.
    pub fn new(data: impl Into<Vec<u8>>, alphabet: Alphabet) -> Result<Self, CoreError> {
        let data = data.into();
        if let Some((offset, &byte)) = data
            .iter()
            .enumerate()
            .find(|&(_, &b)| !alphabet.accepts(b))
        {
            return Err(CoreError::InvalidResidue {
                byte,
                offset,
                alphabet,
            });
        }
        Ok(Self { data, alphabet })
    }

    /// Length in residues.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the sequence has no residues.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The declared alphabet.
    #[must_use]
    pub const fn alphabet(&self) -> Alphabet {
        self.alphabet
    }

    /// The raw residues, as stored.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// A forward-strand view over the whole sequence.
    ///
    /// Allocates nothing. See [`SequenceView`] for why that matters.
    pub fn view(&self) -> SequenceView<'_> {
        SequenceView {
            data: &self.data,
            strand: Strand::Forward,
            alphabet: self.alphabet,
        }
    }
}

/// A borrowed, strand-aware window over a sequence.
///
/// Reverse-complementing is a *view*, not a copy. This is the whole reason the
/// type exists: subplan 09 measures memory as a first-class budget, and a design
/// that clones a chromosome to read its other strand is not one that fits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceView<'a> {
    data: &'a [u8],
    strand: Strand,
    alphabet: Alphabet,
}

impl<'a> SequenceView<'a> {
    /// Length in residues.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the view is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The strand this view reads.
    #[must_use]
    pub const fn strand(&self) -> Strand {
        self.strand
    }

    /// The alphabet of the underlying sequence.
    #[must_use]
    pub const fn alphabet(&self) -> Alphabet {
        self.alphabet
    }

    /// Flips the strand. Allocates nothing.
    #[must_use]
    pub const fn reverse_complement(self) -> Self {
        Self {
            data: self.data,
            strand: self.strand.flip(),
            alphabet: self.alphabet,
        }
    }

    /// A sub-window, in the view's own coordinate space.
    ///
    /// Coordinates are 0-based half-open, as everywhere in this crate.
    ///
    /// # Errors
    ///
    /// [`CoreError::InvertedLocation`] if `start > end`, or
    /// [`CoreError::LocationOutOfBounds`] if `end` exceeds the view.
    pub fn slice(&self, start: usize, end: usize) -> Result<Self, CoreError> {
        if start > end {
            return Err(CoreError::InvertedLocation { start, end });
        }
        if end > self.data.len() {
            return Err(CoreError::LocationOutOfBounds {
                start,
                end,
                length: self.data.len(),
            });
        }
        // On the reverse strand, view coordinates run opposite to storage.
        let (lo, hi) = match self.strand {
            Strand::Forward => (start, end),
            Strand::Reverse => (self.data.len() - end, self.data.len() - start),
        };
        Ok(Self {
            data: &self.data[lo..hi],
            strand: self.strand,
            alphabet: self.alphabet,
        })
    }

    /// Materialises the residues this view reads, applying strand.
    ///
    /// This is the one operation here that allocates, which is why it is named
    /// `to_` rather than `as_` (subplan 12 naming rules).
    #[must_use]
    pub fn to_vec(&self) -> Vec<u8> {
        match self.strand {
            Strand::Forward => self.data.to_vec(),
            Strand::Reverse => self
                .data
                .iter()
                .rev()
                .filter_map(|&b| self.alphabet.complement(b))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn rejects_invalid_residue_with_position() {
        let err = Sequence::new(b"ACGTX".to_vec(), Alphabet::Dna).unwrap_err();
        match err {
            CoreError::InvalidResidue { byte, offset, .. } => {
                assert_eq!(byte, b'X');
                assert_eq!(offset, 4, "offset must point at the offending byte");
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn iupac_alphabet_accepts_ambiguity_dna_does_not() {
        assert!(Sequence::new(b"ACGTN".to_vec(), Alphabet::DnaIupac).is_ok());
        assert!(Sequence::new(b"ACGTN".to_vec(), Alphabet::Dna).is_err());
    }

    #[test]
    fn reverse_complement_allocates_nothing() {
        let s = Sequence::new(b"ACGT".to_vec(), Alphabet::Dna).unwrap();
        let v = s.view();
        let rc = v.reverse_complement();
        assert_eq!(rc.strand(), Strand::Reverse);
        assert_eq!(
            rc.to_vec(),
            b"ACGT".to_vec(),
            "ACGT is its own reverse complement"
        );
    }

    #[test]
    fn reverse_complement_is_not_identity_in_general() {
        let s = Sequence::new(b"AAAC".to_vec(), Alphabet::Dna).unwrap();
        assert_eq!(s.view().reverse_complement().to_vec(), b"GTTT".to_vec());
    }

    fn dna_strategy() -> impl Strategy<Value = Vec<u8>> {
        proptest::collection::vec(
            prop_oneof![Just(b'A'), Just(b'C'), Just(b'G'), Just(b'T')],
            0..200,
        )
    }

    proptest! {
        /// Subplan 17's headline property: reverse-complementing twice is the
        /// identity. Catches strand handling, alphabet asymmetry, and ambiguity
        /// mishandling in one assertion.
        #[test]
        fn revcomp_is_an_involution(data in dna_strategy()) {
            let s = Sequence::new(data.clone(), Alphabet::Dna).unwrap();
            let twice = s.view().reverse_complement().reverse_complement().to_vec();
            prop_assert_eq!(twice, data);
        }

        /// Slicing then materialising agrees with materialising then slicing.
        /// This is where off-by-one errors under strand flips would surface.
        #[test]
        fn slice_commutes_with_materialisation(data in dna_strategy(), a in 0usize..50, b in 0usize..50) {
            let s = Sequence::new(data, Alphabet::Dna).unwrap();
            let v = s.view().reverse_complement();
            let (lo, hi) = (a.min(b), a.max(b));
            prop_assume!(hi <= v.len());
            let via_view = v.slice(lo, hi).unwrap().to_vec();
            let via_full = v.to_vec()[lo..hi].to_vec();
            prop_assert_eq!(via_view, via_full);
        }
    }
}
