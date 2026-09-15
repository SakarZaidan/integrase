//! Motif and restriction-site search over sequences.
//!
//! # Scope of this module
//!
//! Search over a **single design**, which is kilobases. Genome-scale indexing is
//! subplan 28's job and lives behind a compressed index; the two are separate
//! because their constraints are different. At construct scale a linear scan
//! beats an index: building one costs more than the scan it saves, and R-5.4
//! forbids adding machinery for a performance problem nobody measured.
//!
//! # Both strands, always
//!
//! Every search reports hits on the forward and reverse strands. This is not a
//! convenience. A restriction site or a screening motif present on the reverse
//! strand is physically present in the molecule, and a search that missed it
//! would be wrong in the direction that matters. Subplan 17 names
//! "screening a sequence and its reverse complement agree" as a property test
//! that doubles as a security test, and this module is what makes that hold.

use crate::{
    error::CoreError,
    seq::{Alphabet, SequenceView, Strand},
};

/// A match found in a sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Match {
    /// Start offset, 0-based, in forward-strand coordinates.
    pub start: usize,
    /// Exclusive end offset, in forward-strand coordinates.
    pub end: usize,
    /// Which strand the pattern was found on.
    ///
    /// Coordinates are always forward-strand, whichever strand matched, so a
    /// caller never has to convert. Getting that wrong is the classic
    /// off-by-one in this kind of code.
    pub strand: Strand,
}

impl Match {
    /// Length of the match.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// Whether the match covers no residues.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Expands an IUPAC ambiguity code to the concrete bases it stands for.
///
/// Returns `None` for anything that is not a nucleotide or ambiguity code.
const fn iupac_bases(code: u8) -> Option<&'static [u8]> {
    Some(match code.to_ascii_uppercase() {
        b'A' => b"A",
        b'C' => b"C",
        b'G' => b"G",
        b'T' | b'U' => b"T",
        b'R' => b"AG",
        b'Y' => b"CT",
        b'S' => b"CG",
        b'W' => b"AT",
        b'K' => b"GT",
        b'M' => b"AC",
        b'B' => b"CGT",
        b'D' => b"AGT",
        b'H' => b"ACT",
        b'V' => b"ACG",
        b'N' => b"ACGT",
        _ => return None,
    })
}

/// Whether an IUPAC pattern code matches a concrete base.
///
/// The pattern side may be ambiguous. **The subject side may also be
/// ambiguous**, and that case is handled deliberately: a subject `N` matches a
/// pattern `A`, because an `N` in the sequence *could* be an `A`. Treating
/// ambiguity as a non-match would silently miss real sites in real data, which
/// routinely contains `N`.
const fn code_matches(pattern: u8, subject: u8) -> bool {
    let (Some(p), Some(s)) = (iupac_bases(pattern), iupac_bases(subject)) else {
        return false;
    };
    // Overlap means the two could denote the same base.
    let mut i = 0;
    while i < p.len() {
        let mut j = 0;
        while j < s.len() {
            if p[i] == s[j] {
                return true;
            }
            j += 1;
        }
        i += 1;
    }
    false
}

/// Reverse-complements an IUPAC pattern.
fn revcomp_pattern(pattern: &[u8]) -> Option<Vec<u8>> {
    pattern
        .iter()
        .rev()
        .map(|&b| {
            Alphabet::DnaIupac
                .complement(b)
                .map(|c| c.to_ascii_uppercase())
        })
        .collect()
}

/// Finds every occurrence of an IUPAC `pattern`, on both strands.
///
/// Coordinates in the returned matches are forward-strand, regardless of which
/// strand matched.
///
/// A palindromic pattern, which most restriction sites are, matches the same
/// span on both strands. Both are reported, because they are genuinely two
/// occurrences of the recognition sequence and a caller counting cut sites
/// needs to decide what to do about it rather than have the decision made
/// silently here.
///
/// # Errors
///
/// [`CoreError::InvalidResidue`] if the pattern contains a byte that is not an
/// IUPAC nucleotide code.
pub fn find_pattern(view: &SequenceView<'_>, pattern: &[u8]) -> Result<Vec<Match>, CoreError> {
    if let Some((offset, &byte)) = pattern
        .iter()
        .enumerate()
        .find(|&(_, &b)| iupac_bases(b).is_none())
    {
        return Err(CoreError::InvalidResidue {
            byte,
            offset,
            alphabet: Alphabet::DnaIupac,
        });
    }
    if pattern.is_empty() || view.is_empty() {
        return Ok(Vec::new());
    }

    let subject = view.to_vec();
    let mut hits = Vec::new();

    scan_into(&subject, pattern, Strand::Forward, &mut hits);

    // The reverse strand is searched by reverse-complementing the *pattern* and
    // scanning the same forward subject. That keeps every coordinate in
    // forward-strand space, so no conversion is needed afterwards and the
    // off-by-one that conversion invites cannot happen.
    if let Some(rc) = revcomp_pattern(pattern) {
        if rc != pattern {
            scan_into(&subject, &rc, Strand::Reverse, &mut hits);
        } else {
            // Palindromic: the same span is a hit on both strands.
            let forward: Vec<Match> = hits.clone();
            hits.extend(forward.into_iter().map(|m| Match {
                strand: Strand::Reverse,
                ..m
            }));
        }
    }

    hits.sort_by_key(|m| (m.start, m.end, matches!(m.strand, Strand::Reverse)));
    Ok(hits)
}

fn scan_into(subject: &[u8], pattern: &[u8], strand: Strand, out: &mut Vec<Match>) {
    if pattern.len() > subject.len() {
        return;
    }
    for start in 0..=(subject.len() - pattern.len()) {
        if pattern
            .iter()
            .zip(&subject[start..start + pattern.len()])
            .all(|(&p, &s)| code_matches(p, s))
        {
            out.push(Match {
                start,
                end: start + pattern.len(),
                strand,
            });
        }
    }
}

/// A restriction enzyme's recognition sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestrictionEnzyme {
    /// Conventional name, for example `EcoRI`.
    pub name: &'static str,
    /// Recognition sequence in IUPAC codes.
    pub site: &'static [u8],
}

/// A small set of enzymes commonly used in assembly.
///
/// Deliberately small. This is not a REBASE replacement, and pretending
/// otherwise by shipping a partial copy of a curated database would be worse
/// than shipping a handful with their provenance clear. A full enzyme set is a
/// data-licensing question (R-3.3) rather than a coding one.
pub const COMMON_ENZYMES: &[RestrictionEnzyme] = &[
    RestrictionEnzyme {
        name: "EcoRI",
        site: b"GAATTC",
    },
    RestrictionEnzyme {
        name: "BamHI",
        site: b"GGATCC",
    },
    RestrictionEnzyme {
        name: "HindIII",
        site: b"AAGCTT",
    },
    RestrictionEnzyme {
        name: "XbaI",
        site: b"TCTAGA",
    },
    RestrictionEnzyme {
        name: "SpeI",
        site: b"ACTAGT",
    },
    RestrictionEnzyme {
        name: "PstI",
        site: b"CTGCAG",
    },
    RestrictionEnzyme {
        name: "NotI",
        site: b"GCGGCCGC",
    },
    RestrictionEnzyme {
        name: "BsaI",
        site: b"GGTCTC",
    },
];

/// A restriction site found in a sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteHit {
    /// Which enzyme recognises it.
    pub enzyme: &'static str,
    /// Where it sits.
    pub location: Match,
}

/// Finds every site from `enzymes` present in the sequence.
///
/// # Errors
///
/// [`CoreError::InvalidResidue`] if an enzyme's recognition sequence is not
/// valid IUPAC, which would be a bug in the enzyme table rather than in user
/// input.
pub fn find_restriction_sites(
    view: &SequenceView<'_>,
    enzymes: &[RestrictionEnzyme],
) -> Result<Vec<SiteHit>, CoreError> {
    let mut out = Vec::new();
    for e in enzymes {
        for location in find_pattern(view, e.site)? {
            out.push(SiteHit {
                enzyme: e.name,
                location,
            });
        }
    }
    out.sort_by_key(|h| (h.location.start, h.enzyme));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seq::Sequence;
    use proptest::prelude::*;

    fn seq(s: &str) -> Sequence {
        Sequence::new(s.as_bytes().to_vec(), Alphabet::DnaIupac).expect("valid test sequence")
    }

    fn starts(hits: &[Match]) -> Vec<usize> {
        let mut v: Vec<usize> = hits.iter().map(|m| m.start).collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    #[test]
    fn finds_an_exact_match() {
        let s = seq("AAAGAATTCAAA");
        let hits = find_pattern(&s.view(), b"GAATTC").unwrap();
        assert_eq!(starts(&hits), vec![3]);
    }

    #[test]
    fn finds_nothing_when_absent() {
        let s = seq("AAAAAAAAAAAA");
        assert!(find_pattern(&s.view(), b"GAATTC").unwrap().is_empty());
    }

    #[test]
    fn finds_overlapping_occurrences() {
        // AAAA contains three overlapping AAs at 0, 1, 2.
        let s = seq("AAAA");
        let hits = find_pattern(&s.view(), b"AA").unwrap();
        assert_eq!(
            starts(&hits),
            vec![0, 1, 2],
            "overlapping matches must all be reported; skipping them would miss real sites"
        );
    }

    // --- Both strands ----------------------------------------------------

    /// A site present only on the reverse strand is physically in the molecule.
    /// Missing it would be wrong in the direction that matters.
    #[test]
    fn finds_a_site_present_only_on_the_reverse_strand() {
        // CCCGGG reverse-complements to CCCGGG (palindrome), so use an
        // asymmetric pattern. GGTCTC (BsaI) reverse-complements to GAGACC.
        let s = seq("AAAGAGACCAAA");
        let hits = find_pattern(&s.view(), b"GGTCTC").unwrap();
        assert_eq!(hits.len(), 1, "the reverse-strand occurrence must be found");
        assert_eq!(hits[0].strand, Strand::Reverse);
        assert_eq!(hits[0].start, 3, "coordinates are forward-strand");
        assert_eq!(hits[0].end, 9);
    }

    #[test]
    fn a_palindromic_site_is_reported_on_both_strands() {
        let s = seq("AAAGAATTCAAA");
        let hits = find_pattern(&s.view(), b"GAATTC").unwrap();
        assert_eq!(
            hits.len(),
            2,
            "GAATTC is palindromic, so it is a site on both strands"
        );
        assert!(hits.iter().any(|m| m.strand == Strand::Forward));
        assert!(hits.iter().any(|m| m.strand == Strand::Reverse));
        assert!(hits.iter().all(|m| m.start == 3 && m.end == 9));
    }

    // --- IUPAC ambiguity -------------------------------------------------

    #[test]
    fn an_ambiguous_pattern_matches_each_concrete_base() {
        let s = seq("AAACAAAGAAA");
        // Y stands for C or T; both AC and AT windows should match "AY".
        let hits = find_pattern(&s.view(), b"AY").unwrap();
        assert!(starts(&hits).contains(&2), "AC at offset 2 must match AY");
    }

    /// An `N` in the *subject* must match, because an N could be that base.
    /// Real sequence data contains N, and treating it as a non-match would
    /// silently miss sites.
    #[test]
    fn an_ambiguous_subject_base_matches_a_concrete_pattern() {
        let s = seq("AAAGAANTCAAA");
        let hits = find_pattern(&s.view(), b"GAATTC").unwrap();
        assert!(
            !hits.is_empty(),
            "GAANTC must match GAATTC: the N could be the T, and missing it would hide a site"
        );
    }

    #[test]
    fn a_non_iupac_pattern_byte_is_refused_with_its_offset() {
        let s = seq("AAAA");
        match find_pattern(&s.view(), b"AAZA").unwrap_err() {
            CoreError::InvalidResidue { byte, offset, .. } => {
                assert_eq!(byte, b'Z');
                assert_eq!(offset, 2);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn an_empty_pattern_matches_nothing_rather_than_everything() {
        let s = seq("ACGT");
        assert!(
            find_pattern(&s.view(), b"").unwrap().is_empty(),
            "an empty pattern matching at every offset would be useless and surprising"
        );
    }

    #[test]
    fn a_pattern_longer_than_the_sequence_finds_nothing() {
        let s = seq("ACGT");
        assert!(find_pattern(&s.view(), b"ACGTACGT").unwrap().is_empty());
    }

    // --- Restriction sites -----------------------------------------------

    #[test]
    fn finds_known_enzymes_and_names_them() {
        let s = seq("AAAGAATTCAAAGGATCCAAA");
        let hits = find_restriction_sites(&s.view(), COMMON_ENZYMES).unwrap();
        let names: std::collections::BTreeSet<&str> = hits.iter().map(|h| h.enzyme).collect();
        assert!(names.contains("EcoRI"), "GAATTC is EcoRI");
        assert!(names.contains("BamHI"), "GGATCC is BamHI");
    }

    #[test]
    fn a_clean_sequence_has_no_sites() {
        let s = seq("AAAAAAAAAAAAAAAAAAAA");
        assert!(
            find_restriction_sites(&s.view(), COMMON_ENZYMES)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn every_shipped_enzyme_site_is_valid_iupac() {
        // Guards the table itself. A typo here would surface as a confusing
        // error from user input rather than as a bug in our data.
        let s = seq("ACGT");
        for e in COMMON_ENZYMES {
            assert!(
                find_pattern(&s.view(), e.site).is_ok(),
                "enzyme {} has an invalid recognition sequence",
                e.name
            );
        }
    }

    // --- Properties ------------------------------------------------------

    fn dna() -> impl Strategy<Value = String> {
        proptest::collection::vec(prop_oneof!["A", "C", "G", "T"], 0..120).prop_map(|v| v.concat())
    }

    fn pattern() -> impl Strategy<Value = String> {
        proptest::collection::vec(prop_oneof!["A", "C", "G", "T"], 1..8).prop_map(|v| v.concat())
    }

    proptest! {
        /// **The property that doubles as a security test** (subplan 17).
        ///
        /// Searching a sequence must find the same set of sites as searching its
        /// reverse complement. If it did not, a screening motif could be hidden
        /// simply by submitting the other strand.
        #[test]
        fn search_agrees_on_a_sequence_and_its_reverse_complement(
            s in dna(), p in pattern()
        ) {
            let seq = Sequence::new(s.into_bytes(), Alphabet::DnaIupac).unwrap();
            let forward = find_pattern(&seq.view(), p.as_bytes()).unwrap();
            let reverse = find_pattern(&seq.view().reverse_complement(), p.as_bytes()).unwrap();
            prop_assert_eq!(
                forward.len(), reverse.len(),
                "a pattern hidden by flipping the strand would be an evasion vector"
            );
        }

        /// Every reported match must actually be there.
        #[test]
        fn every_reported_match_is_real(s in dna(), p in pattern()) {
            let seq = Sequence::new(s.into_bytes(), Alphabet::DnaIupac).unwrap();
            let bytes = seq.as_bytes().to_vec();
            let rc = revcomp_pattern(p.as_bytes()).unwrap();
            for m in find_pattern(&seq.view(), p.as_bytes()).unwrap() {
                prop_assert!(m.end <= bytes.len(), "match must be in bounds");
                let window = &bytes[m.start..m.end];
                let expected: &[u8] = match m.strand {
                    Strand::Forward => p.as_bytes(),
                    Strand::Reverse => &rc,
                };
                prop_assert!(
                    window.iter().zip(expected).all(|(&s, &e)| code_matches(e, s)),
                    "reported match does not actually match"
                );
            }
        }

        /// Matches are always the pattern's length.
        #[test]
        fn match_spans_equal_the_pattern_length(s in dna(), p in pattern()) {
            let seq = Sequence::new(s.into_bytes(), Alphabet::DnaIupac).unwrap();
            for m in find_pattern(&seq.view(), p.as_bytes()).unwrap() {
                prop_assert_eq!(m.len(), p.len());
            }
        }
    }
}
