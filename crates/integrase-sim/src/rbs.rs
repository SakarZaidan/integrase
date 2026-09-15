//! Thermodynamic model of translation initiation at a ribosome binding site.
//!
//! # What this implements
//!
//! Translation initiation rate is modelled as a function of the free energy
//! released when the 30S ribosomal complex binds the mRNA, following the
//! approach of the RBS Calculator (Salis et al.):
//!
//! ```text
//! ΔG_total = ΔG_mRNA:rRNA + ΔG_start + ΔG_spacing − ΔG_standby − ΔG_mRNA
//! rate     ∝ exp(−β · ΔG_total)
//! ```
//!
//! # What this omits, and why
//!
//! **`ΔG_standby` and `ΔG_mRNA` are not computed.** Both require RNA secondary
//! structure prediction. ViennaRNA is the standard implementation and its
//! licence is not OSI-approved: it forbids redistribution for a fee, which is
//! incompatible with bundling into an Apache-2.0 project. See
//! `planning/decisions/0001-viennarna-and-the-structure-terms.md`.
//!
//! The consequence is reported rather than hidden. Intervals are widened to
//! reflect the missing physics, and a sequence that looks likely to be
//! structure-dominated returns [`DomainVerdict::Marginal`] naming that reason,
//! because that is exactly the regime where the omission matters most.
//!
//! # What this is not
//!
//! This is a simplified implementation of the published approach, not a
//! reimplementation of the RBS Calculator, and it is not calibrated yet. Until
//! the Phase 1a coverage gate is met it reports
//! [`DomainVerdict::Uncalibrated`] and offers no interval, because a model that
//! has not demonstrated coverage has no honest interval to give.

use crate::confidence::{DomainReason, DomainVerdict, ModelCard, ModelId, Prediction};

/// The 3' end of *E. coli* 16S ribosomal RNA, written 5' to 3'.
///
/// This is the anti-Shine-Dalgarno sequence that the mRNA's Shine-Dalgarno
/// region pairs with. Hybridisation between the two is the dominant term in the
/// model.
const ANTI_SD: &[u8] = b"ACCUCCUUA";

/// Boltzmann-like scaling from free energy to rate, in mol/kcal.
///
/// Fitted in the source literature rather than derived. Using a fitted constant
/// from a model whose own accuracy varies by a factor of four across datasets is
/// a reason for wide intervals, not a reason for confidence.
const BETA: f64 = 0.45;

/// Free energy of a single Watson-Crick or wobble pair, kcal/mol.
///
/// A deliberate simplification. Real nearest-neighbour models score stacked
/// pairs, not individual ones, because stacking is where most of the energy
/// lives. This model scores pairs independently, which overestimates the energy
/// of isolated pairs and underestimates long helices.
const PAIR_ENERGY: f64 = -2.0;

/// Optimal spacing between the Shine-Dalgarno region and the start codon, in
/// nucleotides. Deviation is penalised.
const OPTIMAL_SPACING: usize = 5;

/// Penalty per nucleotide of deviation from optimal spacing, kcal/mol.
const SPACING_PENALTY: f64 = 0.7;

/// Input to the model.
#[derive(Debug, Clone)]
pub struct RbsInput {
    /// The 5' untranslated region, 5' to 3', immediately upstream of the start
    /// codon. DNA or RNA alphabet; T and U are treated as equivalent.
    pub utr: Vec<u8>,
    /// The start codon, exactly three bases.
    pub start_codon: [u8; 3],
    /// Host organism, as a free-text identifier.
    ///
    /// Checked against the organism the model was fitted on. A mismatch is not
    /// a silent extrapolation; it returns [`DomainVerdict::OutOfDistribution`].
    pub host: String,
}

/// Errors from model input validation.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum RbsError {
    /// The 5' UTR is too short for the model to score.
    #[error(
        "5' UTR is {len} nt; the model needs at least {min} nt to locate a Shine-Dalgarno region"
    )]
    UtrTooShort {
        /// Length supplied.
        len: usize,
        /// Minimum the model can work with.
        min: usize,
    },
    /// A base in the input is not a nucleotide.
    #[error("byte {byte:#04x} at offset {offset} in the 5' UTR is not a nucleotide")]
    InvalidBase {
        /// The offending byte.
        byte: u8,
        /// Where it appeared.
        offset: usize,
    },
}

/// Shortest 5' UTR the model will score.
const MIN_UTR: usize = 12;

/// Normalises a base to uppercase RNA. T becomes U.
const fn to_rna(b: u8) -> u8 {
    match b.to_ascii_uppercase() {
        b'T' => b'U',
        other => other,
    }
}

/// Returns true if `a` and `b` form a Watson-Crick pair.
const fn watson_crick(a: u8, b: u8) -> bool {
    matches!(
        (to_rna(a), to_rna(b)),
        (b'A', b'U') | (b'U', b'A') | (b'G', b'C') | (b'C', b'G')
    )
}

/// Returns true if `a` and `b` can pair at all, counting G:U wobble.
///
/// Wobble pairs are real in RNA duplexes and excluding them would systematically
/// underestimate SD:aSD binding. They are weaker than Watson-Crick pairs, which
/// is why [`stem_fraction`] counts the two separately.
const fn pairs(a: u8, b: u8) -> bool {
    watson_crick(a, b) || matches!((to_rna(a), to_rna(b)), (b'G', b'U') | (b'U', b'G'))
}

/// Result of locating the best Shine-Dalgarno alignment.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SdAlignment {
    /// Free energy of the duplex, kcal/mol. More negative binds more tightly.
    energy: f64,
    /// Offset into the UTR where the aligned region starts.
    utr_offset: usize,
    /// Number of consecutive paired bases.
    paired: usize,
}

/// Finds the strongest complementary alignment between the UTR and the
/// anti-Shine-Dalgarno sequence.
///
/// Scans every offset and every contiguous run, keeping the most negative
/// energy. This is a brute-force search rather than a dynamic program, which is
/// tractable because the anti-SD is nine bases and UTRs are short.
fn best_sd_alignment(utr: &[u8]) -> SdAlignment {
    let mut best = SdAlignment {
        energy: 0.0,
        utr_offset: 0,
        paired: 0,
    };

    for start in 0..utr.len() {
        for asd_start in 0..ANTI_SD.len() {
            let mut run = 0usize;
            let mut i = start;
            let mut j = asd_start;
            // The anti-SD pairs antiparallel to the mRNA, so walk it backwards.
            loop {
                if i >= utr.len() || j >= ANTI_SD.len() {
                    break;
                }
                if !pairs(utr[i], ANTI_SD[ANTI_SD.len() - 1 - j]) {
                    break;
                }
                run += 1;
                i += 1;
                j += 1;
            }
            if run > best.paired {
                best = SdAlignment {
                    energy: PAIR_ENERGY * run as f64,
                    utr_offset: start,
                    paired: run,
                };
            }
        }
    }
    best
}

/// Free energy contribution of the start codon, kcal/mol.
///
/// AUG is canonical. GUG and UUG initiate less efficiently, and their
/// parameters rest on far fewer observations, which is why a non-AUG start
/// triggers a [`DomainVerdict::Marginal`] verdict rather than being scored
/// silently.
fn start_codon_energy(codon: [u8; 3]) -> Option<f64> {
    let c = [to_rna(codon[0]), to_rna(codon[1]), to_rna(codon[2])];
    match &c {
        b"AUG" => Some(-1.194),
        b"GUG" => Some(-0.075),
        b"UUG" => Some(-0.0),
        b"CUG" => Some(0.671),
        _ => None,
    }
}

/// Fraction of the UTR sitting inside a plausible hairpin stem.
///
/// **This is not secondary structure prediction**, and it deliberately does not
/// pretend to be. It asks a much narrower question: is there a run of
/// consecutive complementary bases long enough to form a stem, separated by a
/// loop of plausible size?
///
/// The narrowness matters. An earlier version counted any base that could pair
/// with any base nearby, which reported a poly-U tract as 71% structured. Poly-U
/// is about as unstructured as RNA gets; the bug was that uracil wobble-pairs
/// with both A and G, so isolated pairings are everywhere and mean nothing.
/// **Structure needs consecutive pairs, not scattered ones.**
fn stem_fraction(utr: &[u8]) -> f64 {
    /// Shortest run of consecutive pairs that could hold a stem together.
    const MIN_STEM: usize = 4;
    /// Smallest sterically plausible hairpin loop.
    const MIN_LOOP: usize = 3;
    /// Largest loop this heuristic looks for.
    const MAX_LOOP: usize = 8;
    /// Watson-Crick pairs a stem needs before it counts.
    ///
    /// Without this, a U-rich region pairs with a purine-rich one entirely
    /// through G:U wobble and reports as folded. A canonical Shine-Dalgarno
    /// region flanked by poly-U scored 0.88 that way, which would have made the
    /// model cry wolf on ordinary ribosome binding sites. Wobble pairs are real
    /// but weak, and a stem held together mostly by wobble is not a stem.
    const MIN_WATSON_CRICK: usize = 3;

    if utr.len() < 2 * MIN_STEM + MIN_LOOP {
        return 0.0;
    }

    let mut in_stem = vec![false; utr.len()];

    // Anchor on the loop and grow the stem outward from both sides of it. This
    // is the natural formulation: a hairpin is a loop with arms, and growing
    // outward avoids the circular definition you get from trying to fix the
    // stem length up front.
    for loop_start in 1..utr.len() {
        for loop_len in MIN_LOOP..=MAX_LOOP {
            let loop_end = loop_start + loop_len;
            if loop_end >= utr.len() {
                break;
            }
            let mut run = 0usize;
            let mut wc = 0usize;
            // 5' arm walks left from just before the loop, 3' arm walks right
            // from just after it, until the pairing breaks or an arm runs out.
            while let Some(five) = loop_start.checked_sub(run + 1) {
                let three = loop_end + run;
                if three >= utr.len() || !pairs(utr[five], utr[three]) {
                    break;
                }
                if watson_crick(utr[five], utr[three]) {
                    wc += 1;
                }
                run += 1;
            }
            if run >= MIN_STEM && wc >= MIN_WATSON_CRICK {
                for k in 0..run {
                    in_stem[loop_start - k - 1] = true;
                    in_stem[loop_end + k] = true;
                }
            }
        }
    }

    in_stem.iter().filter(|b| **b).count() as f64 / utr.len() as f64
}

/// The organism this model was fitted on.
const FITTED_HOST: &str = "Escherichia coli";

/// Threshold above which a sequence is flagged as likely structure-dominated.
///
/// Chosen by inspection rather than fitted, which is itself a reason the model
/// reports uncalibrated. Recorded here so it is visible rather than buried.
const STRUCTURE_FLAG_THRESHOLD: f64 = 0.55;

/// A thermodynamic ribosome binding site model.
#[derive(Debug, Clone)]
pub struct RbsThermodynamicModel {
    card: ModelCard,
}

impl Default for RbsThermodynamicModel {
    fn default() -> Self {
        Self::new()
    }
}

impl RbsThermodynamicModel {
    /// Builds the model with its card.
    #[must_use]
    pub fn new() -> Self {
        Self {
            card: ModelCard {
                id: ModelId("rbs-thermo-v1".into()),
                summary: "Thermodynamic translation initiation rate for E. coli ribosome binding \
                          sites, computed from Shine-Dalgarno hybridisation, start codon identity, \
                          and spacing."
                    .into(),
                training_data: vec![
                    "Parameters from the published RBS Calculator approach (Salis et al.), not \
                     refitted here"
                        .into(),
                ],
                calibration_data: Vec::new(),
                known_failure_modes: vec![
                    "OMITTED TERMS: ΔG_standby and ΔG_mRNA are not computed. Both need RNA \
                     secondary structure prediction, and ViennaRNA's licence is not OSI-approved. \
                     See planning/decisions/0001. Predictions on structured 5' UTRs are \
                     correspondingly less reliable."
                        .into(),
                    "Fitted on E. coli. Other hosts differ in anti-Shine-Dalgarno sequence and \
                     ribosome, and are reported out of distribution."
                        .into(),
                    "Non-AUG start codons rest on far fewer observations.".into(),
                    "Pairs are scored independently rather than as stacked nearest neighbours, \
                     which overestimates isolated pairs and underestimates long helices."
                        .into(),
                    "NOT CALIBRATED. No coverage has been demonstrated on held-out data, so no \
                     interval is reported."
                        .into(),
                ],
                measured_coverage: None,
            },
        }
    }

    /// The model card.
    #[must_use]
    pub const fn card(&self) -> &ModelCard {
        &self.card
    }

    /// Computes the total free energy of initiation, in kcal/mol.
    ///
    /// Exposed because the components are interpretable and a user checking
    /// whether the model is behaving sensibly should be able to see them. This
    /// returns a raw energy rather than a prediction, which is permitted under
    /// R-0.1 because free energy here is a *computed* quantity from the stated
    /// equation, not a modelled estimate of an observable.
    ///
    /// # Errors
    ///
    /// [`RbsError`] if the UTR is too short or contains a non-nucleotide.
    pub fn free_energy(&self, input: &RbsInput) -> Result<f64, RbsError> {
        if input.utr.len() < MIN_UTR {
            return Err(RbsError::UtrTooShort {
                len: input.utr.len(),
                min: MIN_UTR,
            });
        }
        if let Some((offset, &byte)) = input
            .utr
            .iter()
            .enumerate()
            .find(|&(_, &b)| !matches!(to_rna(b), b'A' | b'C' | b'G' | b'U'))
        {
            return Err(RbsError::InvalidBase { byte, offset });
        }

        let sd = best_sd_alignment(&input.utr);

        // Spacing runs from the 3' end of the SD alignment to the start codon.
        let sd_end = sd.utr_offset + sd.paired;
        let spacing = input.utr.len().saturating_sub(sd_end);
        let deviation = spacing.abs_diff(OPTIMAL_SPACING);
        let dg_spacing = SPACING_PENALTY * deviation as f64;

        let dg_start = start_codon_energy(input.start_codon).unwrap_or(1.0);

        Ok(sd.energy + dg_start + dg_spacing)
    }

    /// Decides whether this input falls inside the model's validated domain.
    ///
    /// Returns [`DomainVerdict::Uncalibrated`] unconditionally at present,
    /// because no calibration has been demonstrated. The other branches are
    /// computed anyway and are what this returns once the Phase 1a coverage
    /// gate is met.
    #[must_use]
    pub fn domain_check(&self, input: &RbsInput) -> DomainVerdict {
        // Honest default. This model has no measured coverage, so it may not
        // claim one, whatever the other signals say.
        if self.card.measured_coverage.is_none() {
            return DomainVerdict::Uncalibrated;
        }
        self.domain_signals(input)
    }

    /// The domain signals, independent of whether calibration exists.
    ///
    /// Separated so the logic is testable now and becomes live the moment a
    /// measured coverage figure is added to the card.
    #[must_use]
    pub fn domain_signals(&self, input: &RbsInput) -> DomainVerdict {
        if input.host != FITTED_HOST {
            return DomainVerdict::OutOfDistribution {
                reason: DomainReason::HostMismatch {
                    got: input.host.clone(),
                    expected: FITTED_HOST.into(),
                },
            };
        }

        if start_codon_energy(input.start_codon).is_none() {
            let codon = String::from_utf8_lossy(&input.start_codon).to_string();
            return DomainVerdict::OutOfDistribution {
                reason: DomainReason::NonCanonicalStartCodon { codon },
            };
        }

        let codon_is_aug = input.start_codon.map(to_rna) == *b"AUG";
        if !codon_is_aug {
            let codon = String::from_utf8_lossy(&input.start_codon).to_string();
            return DomainVerdict::Marginal {
                similarity: 0.6,
                reason: DomainReason::NonCanonicalStartCodon { codon },
            };
        }

        // The omitted structure terms bite hardest here, so flag it.
        let structure = stem_fraction(&input.utr);
        if structure > STRUCTURE_FLAG_THRESHOLD {
            return DomainVerdict::Marginal {
                similarity: 1.0 - structure,
                reason: DomainReason::PossibleSecondaryStructure {
                    stem_fraction: structure,
                },
            };
        }

        let sd = best_sd_alignment(&input.utr);
        if sd.paired < 3 {
            return DomainVerdict::Marginal {
                similarity: 0.5,
                reason: DomainReason::LengthOutOfRange { length: sd.paired },
            };
        }

        DomainVerdict::InDistribution { similarity: 0.9 }
    }

    /// Predicts the relative translation initiation rate.
    ///
    /// The returned value is in arbitrary units and is meaningful only by
    /// comparison against another prediction from the same model version.
    ///
    /// # Errors
    ///
    /// [`RbsError`] if the input is not scoreable.
    pub fn predict(&self, input: &RbsInput) -> Result<Prediction<f64>, RbsError> {
        let dg = self.free_energy(input)?;
        let rate = (-BETA * dg).exp();
        let verdict = self.domain_check(input);

        match self.card.measured_coverage {
            // No demonstrated coverage means no honest interval. Absence is the
            // truthful rendering; a wide interval would imply a quantified
            // uncertainty this model has not earned.
            None => Ok(Prediction::uncalibrated(rate, self.card.id.clone())),
            Some(cov) => {
                // Placeholder until calibration exists. The Phase 1a gate
                // replaces this with conformal quantiles from held-out data.
                let half = rate * 0.5;
                Ok(Prediction::new(
                    rate,
                    crate::confidence::Interval::new(rate - half, rate + half),
                    cov.nominal,
                    self.card.id.clone(),
                    verdict,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ecoli(utr: &str) -> RbsInput {
        RbsInput {
            utr: utr.as_bytes().to_vec(),
            start_codon: *b"ATG",
            host: FITTED_HOST.into(),
        }
    }

    /// A canonical Shine-Dalgarno core. Pairs with the 16S 3' end.
    const STRONG: &str = "TTTTTTAGGAGGTTTTT";
    /// No complementarity to the anti-SD worth speaking of.
    const WEAK: &str = "TTTTTTTTTTTTTTTTT";

    #[test]
    fn finds_the_shine_dalgarno_in_a_canonical_rbs() {
        let sd = best_sd_alignment(STRONG.as_bytes());
        assert!(
            sd.paired >= 4,
            "a canonical AGGAGG should pair with at least 4 bases of the anti-SD, got {}",
            sd.paired
        );
        assert!(
            sd.energy < -6.0,
            "energy should be strongly negative, got {}",
            sd.energy
        );
    }

    #[test]
    fn a_sequence_with_no_sd_binds_weakly() {
        let strong = best_sd_alignment(STRONG.as_bytes());
        let weak = best_sd_alignment(WEAK.as_bytes());
        assert!(
            strong.energy < weak.energy,
            "canonical SD ({}) must bind more tightly than a poly-U tract ({})",
            strong.energy,
            weak.energy
        );
    }

    #[test]
    fn stronger_binding_predicts_a_higher_rate() {
        let m = RbsThermodynamicModel::new();
        let strong = m.predict(&ecoli(STRONG)).expect("strong RBS should score");
        let weak = m.predict(&ecoli(WEAK)).expect("weak RBS should score");
        assert!(
            strong.value() > weak.value(),
            "a canonical SD should out-initiate a poly-U tract: {} vs {}",
            strong.value(),
            weak.value()
        );
    }

    #[test]
    fn spacing_deviation_is_penalised() {
        let m = RbsThermodynamicModel::new();
        // Same SD, different distance from the start codon.
        let near = m.free_energy(&ecoli("TTTTTTAGGAGGTTTTT")).unwrap();
        let far = m.free_energy(&ecoli("TTTTTTAGGAGGTTTTTTTTTTTTTT")).unwrap();
        assert!(
            far > near,
            "a badly spaced SD should have higher (less favourable) energy: near {near}, far {far}"
        );
    }

    #[test]
    fn start_codon_identity_changes_the_energy() {
        let m = RbsThermodynamicModel::new();
        let mut aug = ecoli(STRONG);
        aug.start_codon = *b"ATG";
        let mut gtg = ecoli(STRONG);
        gtg.start_codon = *b"GTG";
        assert!(
            m.free_energy(&aug).unwrap() < m.free_energy(&gtg).unwrap(),
            "AUG initiates more favourably than GUG"
        );
    }

    // --- The tier-0 rule -------------------------------------------------

    #[test]
    fn an_uncalibrated_model_returns_no_interval() {
        let m = RbsThermodynamicModel::new();
        let p = m.predict(&ecoli(STRONG)).unwrap();
        assert!(
            p.interval().is_none(),
            "a model with no measured coverage must not report an interval"
        );
        assert_eq!(*p.domain(), DomainVerdict::Uncalibrated);
        assert!(p.domain().coverage_is_unreliable());
    }

    #[test]
    fn the_card_records_that_calibration_is_absent() {
        let m = RbsThermodynamicModel::new();
        assert!(
            m.card().measured_coverage.is_none(),
            "this model has not demonstrated coverage and its card must say so"
        );
    }

    /// Verification clause of ADR 0001. If someone adds structure prediction
    /// and forgets to update the card, this fails.
    #[test]
    fn the_card_discloses_the_omitted_structure_terms() {
        let m = RbsThermodynamicModel::new();
        let disclosed = m
            .card()
            .known_failure_modes
            .iter()
            .any(|s| s.contains("ΔG_standby") && s.contains("ΔG_mRNA"));
        assert!(
            disclosed,
            "the model card must name the omitted terms; see planning/decisions/0001"
        );
    }

    // --- Domain signals --------------------------------------------------

    #[test]
    fn a_different_host_is_out_of_distribution_with_the_host_named() {
        let m = RbsThermodynamicModel::new();
        let mut input = ecoli(STRONG);
        input.host = "Paracoccus denitrificans".into();
        match m.domain_signals(&input) {
            DomainVerdict::OutOfDistribution {
                reason: DomainReason::HostMismatch { got, expected },
            } => {
                assert_eq!(got, "Paracoccus denitrificans");
                assert_eq!(expected, FITTED_HOST);
            }
            other => panic!("expected a host mismatch, got {other:?}"),
        }
    }

    #[test]
    fn a_non_canonical_start_codon_is_flagged_rather_than_scored_silently() {
        let m = RbsThermodynamicModel::new();
        let mut input = ecoli(STRONG);
        input.start_codon = *b"GTG";
        assert!(
            matches!(
                m.domain_signals(&input),
                DomainVerdict::Marginal {
                    reason: DomainReason::NonCanonicalStartCodon { .. },
                    ..
                }
            ),
            "GUG rests on few observations and must not pass as in-distribution"
        );
    }

    #[test]
    fn an_unknown_start_codon_is_out_of_distribution() {
        let m = RbsThermodynamicModel::new();
        let mut input = ecoli(STRONG);
        input.start_codon = *b"AAA";
        assert!(matches!(
            m.domain_signals(&input),
            DomainVerdict::OutOfDistribution { .. }
        ));
    }

    #[test]
    fn a_canonical_rbs_in_the_fitted_host_is_in_distribution() {
        let m = RbsThermodynamicModel::new();
        assert!(
            m.domain_signals(&ecoli(STRONG)).is_in_distribution(),
            "a canonical E. coli RBS should not be flagged"
        );
    }

    // --- Input validation ------------------------------------------------

    #[test]
    fn a_short_utr_is_refused_with_both_lengths_named() {
        let m = RbsThermodynamicModel::new();
        match m.free_energy(&ecoli("AGGAGG")).unwrap_err() {
            RbsError::UtrTooShort { len, min } => {
                assert_eq!(len, 6);
                assert_eq!(min, MIN_UTR);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn a_non_nucleotide_is_refused_with_its_offset() {
        let m = RbsThermodynamicModel::new();
        match m.free_energy(&ecoli("TTTTTTAGGAGGXTTTT")).unwrap_err() {
            RbsError::InvalidBase { byte, offset } => {
                assert_eq!(byte, b'X');
                assert_eq!(offset, 12, "offset must point at the offending base");
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn dna_and_rna_spellings_agree() {
        let m = RbsThermodynamicModel::new();
        let dna = m.free_energy(&ecoli("TTTTTTAGGAGGTTTTT")).unwrap();
        let rna = m.free_energy(&ecoli("UUUUUUAGGAGGUUUUU")).unwrap();
        assert!(
            (dna - rna).abs() < 1e-9,
            "T and U must be treated as equivalent: {dna} vs {rna}"
        );
    }

    #[test]
    fn wobble_pairs_are_counted() {
        assert!(
            pairs(b'G', b'U'),
            "G:U wobble is real in RNA and must count"
        );
        assert!(pairs(b'G', b'T'), "the DNA spelling of the same pair");
        assert!(!pairs(b'A', b'G'));
    }
}

#[cfg(test)]
mod stem_tests {
    use super::*;

    /// A textbook hairpin: GGGG pairs with CCCC across a four-base loop.
    const HAIRPIN: &[u8] = b"AAGGGGAAAACCCCAA";
    /// Poly-U. About as unstructured as RNA gets.
    const POLY_U: &[u8] = b"TTTTTTTTTTTTTTTTTT";
    /// The canonical RBS from the model tests. Should not read as folded.
    const CANONICAL_RBS: &[u8] = b"TTTTTTAGGAGGTTTTT";

    #[test]
    fn a_real_hairpin_is_detected() {
        let f = stem_fraction(HAIRPIN);
        assert!(
            f > 0.4,
            "GGGG/CCCC across a loop is a stem; got fraction {f}"
        );
    }

    /// The regression this heuristic was rewritten for.
    ///
    /// The first version counted any base that could pair with any base nearby,
    /// and reported poly-U as 71% structured. Uracil wobble-pairs with both A
    /// and G, so isolated pairings are everywhere and mean nothing.
    #[test]
    fn poly_u_is_not_structured() {
        let f = stem_fraction(POLY_U);
        assert!(
            f < 0.2,
            "poly-U is unstructured; a heuristic reporting {f} is counting isolated pairs"
        );
    }

    #[test]
    fn a_canonical_rbs_is_not_flagged_as_folded() {
        let f = stem_fraction(CANONICAL_RBS);
        assert!(
            f <= STRUCTURE_FLAG_THRESHOLD,
            "a normal RBS must not trip the structure flag; got {f}"
        );
    }

    #[test]
    fn a_hairpin_scores_higher_than_an_unstructured_sequence() {
        assert!(
            stem_fraction(HAIRPIN) > stem_fraction(POLY_U),
            "the ordering is the whole point of the heuristic"
        );
    }

    #[test]
    fn too_short_to_hold_a_hairpin_returns_zero() {
        assert_eq!(stem_fraction(b"AAGGGG"), 0.0);
    }

    #[test]
    fn a_folded_utr_is_reported_as_marginal_with_the_fraction() {
        let m = RbsThermodynamicModel::new();
        let input = RbsInput {
            utr: b"GGGGGGAAAACCCCCCTTTAGGAGGTTTTT".to_vec(),
            start_codon: *b"ATG",
            host: FITTED_HOST.into(),
        };
        match m.domain_signals(&input) {
            DomainVerdict::Marginal {
                reason: DomainReason::PossibleSecondaryStructure { stem_fraction: f },
                ..
            } => {
                assert!(f > STRUCTURE_FLAG_THRESHOLD);
                assert!(f.is_finite(), "the reported fraction must be a real number");
            }
            other => panic!("expected a structure flag, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod wobble_tests {
    use super::*;

    /// The regression that motivated `MIN_WATSON_CRICK`.
    ///
    /// A U-rich region pairs with a purine-rich one almost entirely through G:U
    /// wobble. Counting those equally reported a canonical RBS as 0.88
    /// structured, which would have made the model cry wolf on ordinary
    /// sequences.
    #[test]
    fn a_stem_held_together_by_wobble_alone_does_not_count() {
        // GGGG against TTTT is four G:U wobbles and no Watson-Crick pair at
        // all. Note the first attempt at this test used TTTT against AAAA,
        // which is four *Watson-Crick* A:U pairs: a genuine stem, and a
        // reminder that a badly chosen fixture tests nothing.
        let mostly_wobble = b"CCAAGGGGAAAATTTTAACC";
        let f = stem_fraction(mostly_wobble);
        assert!(
            f < STRUCTURE_FLAG_THRESHOLD,
            "a mostly-wobble stem is not a stem; got {f}"
        );
    }

    #[test]
    fn watson_crick_and_wobble_are_distinguished() {
        assert!(watson_crick(b'G', b'C'));
        assert!(watson_crick(b'A', b'T'), "the DNA spelling of A:U");
        assert!(!watson_crick(b'G', b'T'), "G:U is wobble, not Watson-Crick");
        assert!(pairs(b'G', b'T'), "but it is still a pair");
    }

    #[test]
    fn a_watson_crick_stem_still_counts() {
        // GGGG against CCCC: four Watson-Crick pairs.
        assert!(stem_fraction(b"AAGGGGAAAACCCCAA") > 0.4);
    }
}
