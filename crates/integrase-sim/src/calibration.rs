//! Split conformal prediction, and the evaluation that proves it works.
//!
//! # Why conformal
//!
//! The project's central claim is that an 80% interval contains the truth about
//! 80% of the time. That needs a method that can actually deliver it, and the
//! alternatives cannot:
//!
//! | Method | Problem |
//! |---|---|
//! | Gaussian error bars from model variance | Assumes a noise model expression data violates, usually badly |
//! | Ensemble spread | Measures disagreement between models, not error. Confidently wrong ensembles are common. |
//! | MC dropout, Bayesian NN | Approximate posteriors with no coverage guarantee, and hard to test |
//! | **Split conformal** | Distribution-free, finite-sample guarantee, directly testable |
//!
//! Split conformal makes no assumption about the model being correct or errors
//! being Gaussian. It asks only that the calibration data and the test point are
//! **exchangeable**.
//!
//! # The assumption that breaks, stated plainly
//!
//! Exchangeability is exactly what fails on an out-of-distribution input. The
//! guarantee does not weaken gracefully there; it stops applying. That is why
//! [`crate::DomainVerdict`] is a separate field rather than a widened interval:
//! the interval answers "given this input is like my calibration data, how wrong
//! am I likely to be", and the verdict answers "is it".

use crate::confidence::{Interval, MeasuredCoverage};

/// Errors from calibration.
// PartialEq but not Eq: two variants carry f64, and NaN is not reflexive.
#[derive(Debug, thiserror::Error, PartialEq)]
#[non_exhaustive]
pub enum CalibrationError {
    /// Too few calibration points to support the requested coverage level.
    ///
    /// Split conformal needs `n >= 1/alpha - 1` points for the finite-sample
    /// quantile to exist at all. Below that the required quantile index exceeds
    /// `n` and there is no honest interval to return.
    #[error(
        "{n} calibration points cannot support {coverage:.0}% coverage; \
         at least {needed} are required"
    )]
    TooFewPoints {
        /// Points supplied.
        n: usize,
        /// Requested coverage, 0 to 1.
        coverage: f64,
        /// Minimum needed.
        needed: usize,
    },
    /// Coverage must be strictly between 0 and 1.
    #[error("coverage must be in (0, 1); got {coverage}")]
    InvalidCoverage {
        /// The value supplied.
        coverage: f64,
    },
    /// A residual was not a finite number.
    #[error("calibration residual at index {index} is not finite")]
    NonFiniteResidual {
        /// Which point.
        index: usize,
    },
}

/// A fitted split-conformal calibrator.
///
/// Holds the quantile of conformity scores that turns a point estimate into an
/// interval with the stated coverage.
#[derive(Debug, Clone, PartialEq)]
pub struct ConformalCalibrator {
    half_width: f64,
    nominal_coverage: f64,
    n_calibration: usize,
}

impl ConformalCalibrator {
    /// Fits a calibrator from residuals measured on a held-out calibration set.
    ///
    /// `residuals` are `|observed - predicted|`, one per calibration point. They
    /// must come from data the model was **not** trained on. Using training
    /// residuals produces intervals that are far too narrow, and the resulting
    /// coverage failure is the single most likely way this machinery gets
    /// quietly broken.
    ///
    /// # The finite-sample correction
    ///
    /// The quantile index is `ceil((n + 1) * coverage)`, not `ceil(n *
    /// coverage)`. That `+1` is what makes the guarantee hold at finite `n`
    /// rather than only asymptotically. Dropping it produces intervals that are
    /// slightly too narrow in a way that is invisible on large samples and
    /// obvious on small ones.
    ///
    /// # Errors
    ///
    /// [`CalibrationError`] if coverage is out of range, residuals are
    /// non-finite, or there are too few points for the requested level.
    pub fn fit(residuals: &[f64], nominal_coverage: f64) -> Result<Self, CalibrationError> {
        if !(nominal_coverage > 0.0 && nominal_coverage < 1.0) {
            return Err(CalibrationError::InvalidCoverage {
                coverage: nominal_coverage,
            });
        }
        if let Some(index) = residuals.iter().position(|r| !r.is_finite()) {
            return Err(CalibrationError::NonFiniteResidual { index });
        }

        let n = residuals.len();
        // The index we need must exist within the sample.
        #[allow(clippy::cast_precision_loss)]
        let rank = ((n + 1) as f64 * nominal_coverage).ceil();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let rank = rank as usize;
        if rank > n {
            // Smallest n with ceil((n+1) * c) <= n.
            let needed = {
                let mut k = n.max(1);
                loop {
                    #[allow(clippy::cast_precision_loss)]
                    let r = ((k + 1) as f64 * nominal_coverage).ceil();
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let r = r as usize;
                    if r <= k {
                        break k;
                    }
                    k += 1;
                }
            };
            return Err(CalibrationError::TooFewPoints {
                n,
                coverage: nominal_coverage,
                needed,
            });
        }

        let mut sorted = residuals.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        Ok(Self {
            // rank is 1-based; the slice is 0-based.
            half_width: sorted[rank - 1],
            nominal_coverage,
            n_calibration: n,
        })
    }

    /// Turns a point estimate into an interval at the calibrated level.
    #[must_use]
    pub fn interval(&self, point_estimate: f64) -> Interval {
        Interval::new(
            point_estimate - self.half_width,
            point_estimate + self.half_width,
        )
    }

    /// The coverage level this calibrator was fitted for.
    #[must_use]
    pub const fn nominal_coverage(&self) -> f64 {
        self.nominal_coverage
    }

    /// Half-width of the interval this calibrator produces.
    #[must_use]
    pub const fn half_width(&self) -> f64 {
        self.half_width
    }

    /// How many points the calibration was fitted on.
    #[must_use]
    pub const fn n_calibration(&self) -> usize {
        self.n_calibration
    }
}

/// One evaluation point: what the model said, and what was measured.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EvalPoint {
    /// The model's point estimate.
    pub predicted: f64,
    /// The measured value.
    pub observed: f64,
}

/// Result of measuring whether intervals actually cover.
#[derive(Debug, Clone, PartialEq)]
pub struct CoverageReport {
    /// The level the intervals claimed.
    pub nominal: f64,
    /// The fraction that actually contained the observed value.
    pub observed: f64,
    /// How many points were evaluated.
    pub n: usize,
    /// Coverage within strata, lowest stratum first.
    ///
    /// Empty if stratified evaluation was not requested.
    pub by_stratum: Vec<StratumCoverage>,
}

/// Coverage within one slice of the data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StratumCoverage {
    /// Which stratum, 0-indexed from the lowest observed values.
    pub index: usize,
    /// Observed coverage in this stratum.
    pub observed: f64,
    /// Points in this stratum.
    pub n: usize,
}

impl CoverageReport {
    /// Whether marginal coverage falls within `tolerance` of nominal.
    #[must_use]
    pub fn marginal_within(&self, tolerance: f64) -> bool {
        (self.observed - self.nominal).abs() <= tolerance
    }

    /// Whether **every** stratum falls within `tolerance` of nominal.
    ///
    /// This is the check that matters, and the one most likely to fail on a
    /// model that looks fine. Marginal coverage is easy to achieve while being
    /// useless: a model overconfident on weak signals and underconfident on
    /// strong ones averages to the right number while being systematically
    /// wrong in a way users actually experience.
    #[must_use]
    pub fn stratified_within(&self, tolerance: f64) -> bool {
        !self.by_stratum.is_empty()
            && self
                .by_stratum
                .iter()
                .all(|s| (s.observed - self.nominal).abs() <= tolerance)
    }

    /// The stratum furthest from nominal, if any strata were computed.
    #[must_use]
    pub fn worst_stratum(&self) -> Option<StratumCoverage> {
        self.by_stratum.iter().copied().max_by(|a, b| {
            let da = (a.observed - self.nominal).abs();
            let db = (b.observed - self.nominal).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// Measures whether a calibrator's intervals cover at the claimed rate.
///
/// `strata` slices the evaluation set by observed value, so that coverage can be
/// checked within each slice rather than only on average. Pass 1 for marginal
/// coverage only; the Phase 1a gate uses 10.
#[must_use]
pub fn evaluate_coverage(
    calibrator: &ConformalCalibrator,
    points: &[EvalPoint],
    strata: usize,
) -> CoverageReport {
    let covered = |p: &EvalPoint| calibrator.interval(p.predicted).contains(p.observed);

    #[allow(clippy::cast_precision_loss)]
    let observed = if points.is_empty() {
        0.0
    } else {
        points.iter().filter(|p| covered(p)).count() as f64 / points.len() as f64
    };

    let mut by_stratum = Vec::new();
    if strata > 1 && !points.is_empty() {
        // Slice by observed value, so a stratum is a band of real expression
        // levels rather than an arbitrary partition.
        let mut sorted = points.to_vec();
        sorted.sort_by(|a, b| {
            a.observed
                .partial_cmp(&b.observed)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let per = sorted.len().div_ceil(strata);
        for (index, chunk) in sorted.chunks(per).enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let obs = chunk.iter().filter(|p| covered(p)).count() as f64 / chunk.len() as f64;
            by_stratum.push(StratumCoverage {
                index,
                observed: obs,
                n: chunk.len(),
            });
        }
    }

    CoverageReport {
        nominal: calibrator.nominal_coverage(),
        observed,
        n: points.len(),
        by_stratum,
    }
}

impl CoverageReport {
    /// Converts into the evidence a [`crate::ModelCard`] carries.
    ///
    /// A model may only claim a coverage figure if it has one of these, which is
    /// what stops an accuracy claim drifting away from the measurement behind
    /// it.
    #[must_use]
    pub fn as_measured_coverage(&self) -> MeasuredCoverage {
        MeasuredCoverage {
            nominal: self.nominal,
            observed: self.observed,
            n: self.n,
            stratified: !self.by_stratum.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic pseudo-random generator.
    ///
    /// Seeded xorshift rather than a dependency: these tests must produce the
    /// same numbers on every machine and every run, because a flaky coverage
    /// test is one people learn to re-run until it passes.
    struct Rng(u64);

    impl Rng {
        const fn new(seed: u64) -> Self {
            Self(seed)
        }
        fn next_u64(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        /// Uniform in [0, 1).
        fn uniform(&mut self) -> f64 {
            #[allow(clippy::cast_precision_loss)]
            {
                (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
            }
        }
        /// Approximately standard normal, by summing uniforms.
        ///
        /// Irwin-Hall with n = 12 has mean 6 and variance 1, so subtracting 6
        /// gives a good enough normal for testing that coverage holds under a
        /// noise model conformal is not told about.
        fn normal(&mut self) -> f64 {
            (0..12).map(|_| self.uniform()).sum::<f64>() - 6.0
        }
    }

    /// Generates points whose error is normal with the given spread.
    fn gaussian_points(rng: &mut Rng, n: usize, sigma: f64) -> Vec<EvalPoint> {
        (0..n)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let predicted = i as f64;
                EvalPoint {
                    predicted,
                    observed: predicted + rng.normal() * sigma,
                }
            })
            .collect()
    }

    fn residuals(points: &[EvalPoint]) -> Vec<f64> {
        points
            .iter()
            .map(|p| (p.observed - p.predicted).abs())
            .collect()
    }

    // --- The property the whole project rests on -------------------------

    /// **The test that matters.** Fit on one sample, evaluate on a disjoint one,
    /// and check the intervals actually cover at the rate they claim.
    ///
    /// Conformal is never told the noise is Gaussian. If the implementation is
    /// correct, coverage holds anyway, because the guarantee is
    /// distribution-free.
    #[test]
    fn calibrated_intervals_cover_at_the_claimed_rate() {
        let mut rng = Rng::new(0x5EED_1234);
        for &nominal in &[0.50, 0.80, 0.90, 0.95] {
            let cal = gaussian_points(&mut rng, 2000, 3.0);
            let test = gaussian_points(&mut rng, 4000, 3.0);

            let calibrator = ConformalCalibrator::fit(&residuals(&cal), nominal).unwrap();
            let report = evaluate_coverage(&calibrator, &test, 1);

            assert!(
                report.marginal_within(0.03),
                "nominal {nominal:.2} produced observed {:.3}, outside tolerance. \
                 Conformal coverage is a finite-sample guarantee; missing it means the \
                 implementation is wrong, not that the data was unlucky.",
                report.observed
            );
        }
    }

    /// Coverage must hold under a noise distribution nothing was told about.
    ///
    /// Heavy-tailed errors break Gaussian error bars badly. Conformal should be
    /// unmoved, and if it is not, the distribution-free claim is false.
    #[test]
    fn coverage_holds_under_heavy_tailed_noise() {
        let mut rng = Rng::new(0xC0FFEE);
        let heavy = |rng: &mut Rng, n: usize| -> Vec<EvalPoint> {
            (0..n)
                .map(|i| {
                    #[allow(clippy::cast_precision_loss)]
                    let predicted = i as f64;
                    // Ratio of normals is Cauchy-like: very heavy tails.
                    let noise = rng.normal() / (rng.normal().abs() + 0.05);
                    EvalPoint {
                        predicted,
                        observed: predicted + noise,
                    }
                })
                .collect()
        };
        let cal = heavy(&mut rng, 3000);
        let test = heavy(&mut rng, 5000);

        let calibrator = ConformalCalibrator::fit(&residuals(&cal), 0.80).unwrap();
        let report = evaluate_coverage(&calibrator, &test, 1);
        assert!(
            report.marginal_within(0.04),
            "conformal claims to be distribution-free; heavy tails gave {:.3}",
            report.observed
        );
    }

    /// Fitting on training residuals produces intervals that are too narrow.
    ///
    /// This is the single most likely way this machinery gets quietly broken, so
    /// it is worth a test that demonstrates the failure rather than a comment
    /// warning about it.
    #[test]
    fn calibrating_on_optimistic_residuals_undercovers() {
        let mut rng = Rng::new(0xBADC0DE);
        // Stand-in for training residuals: systematically smaller than reality.
        let optimistic: Vec<f64> = gaussian_points(&mut rng, 2000, 0.5)
            .iter()
            .map(|p| (p.observed - p.predicted).abs())
            .collect();
        let test = gaussian_points(&mut rng, 4000, 3.0);

        let calibrator = ConformalCalibrator::fit(&optimistic, 0.80).unwrap();
        let report = evaluate_coverage(&calibrator, &test, 1);
        assert!(
            report.observed < 0.5,
            "residuals from an easier distribution must undercover badly; got {:.3}",
            report.observed
        );
    }

    // --- Why stratified coverage is checked ------------------------------

    /// **Marginal coverage can be right while the model is systematically
    /// wrong.** This constructs exactly that case and shows the marginal check
    /// passing while the stratified check fails.
    ///
    /// This is the argument for the Phase 1a gate being stratified.
    #[test]
    fn marginal_coverage_hides_systematic_failure_that_strata_catch() {
        let mut rng = Rng::new(0xFACE);
        // Error grows with the value: tight at the low end, wide at the high
        // end. A single half-width over-covers the bottom and under-covers the
        // top while averaging out.
        //
        // Both sets must span the SAME range of predicted values. The first
        // version of this fixture used the loop index as the predicted value,
        // so a 2000-point calibration set and a 4000-point test set covered
        // different ranges and therefore different noise levels. That is
        // distribution shift, not heteroscedasticity, and it broke marginal
        // coverage (0.584) for reasons unrelated to what the test is about.
        let heteroscedastic = |rng: &mut Rng, n: usize| -> Vec<EvalPoint> {
            (0..n)
                .map(|_| {
                    let predicted = rng.uniform() * 100.0;
                    let sigma = 0.05 + predicted * 0.02;
                    EvalPoint {
                        predicted,
                        observed: predicted + rng.normal() * sigma,
                    }
                })
                .collect()
        };
        let cal = heteroscedastic(&mut rng, 2000);
        let test = heteroscedastic(&mut rng, 4000);

        let calibrator = ConformalCalibrator::fit(&residuals(&cal), 0.80).unwrap();
        let report = evaluate_coverage(&calibrator, &test, 10);

        assert!(
            report.marginal_within(0.05),
            "marginal coverage should look fine here: {:.3}",
            report.observed
        );
        assert!(
            !report.stratified_within(0.05),
            "the stratified check must catch what the marginal check missed. \
             Worst stratum: {:?}",
            report.worst_stratum()
        );
    }

    #[test]
    fn strata_partition_every_point() {
        let mut rng = Rng::new(7);
        let points = gaussian_points(&mut rng, 1000, 1.0);
        let calibrator = ConformalCalibrator::fit(&residuals(&points), 0.80).unwrap();
        let report = evaluate_coverage(&calibrator, &points, 10);
        let total: usize = report.by_stratum.iter().map(|s| s.n).sum();
        assert_eq!(
            total,
            points.len(),
            "stratification must not lose or duplicate points"
        );
    }

    // --- The finite-sample correction ------------------------------------

    /// The `+1` in `ceil((n + 1) * coverage)` is what makes the guarantee hold
    /// at finite n. Dropping it is invisible on large samples and wrong on small
    /// ones, so this pins the exact quantile chosen.
    #[test]
    fn the_finite_sample_quantile_index_is_correct() {
        // n = 9, coverage = 0.8 -> ceil(10 * 0.8) = 8 -> the 8th smallest.
        let residuals: Vec<f64> = (1..=9).map(f64::from).collect();
        let c = ConformalCalibrator::fit(&residuals, 0.80).unwrap();
        assert_eq!(
            c.half_width(),
            8.0,
            "expected the 8th smallest residual; the naive ceil(n * c) would give the 8th too, \
             so check a case where they differ"
        );

        // n = 4, coverage = 0.8 -> ceil(5 * 0.8) = 4 (the maximum).
        // The naive ceil(4 * 0.8) = 4 as well. Use n = 19, c = 0.9:
        // ceil(20 * 0.9) = 18, whereas ceil(19 * 0.9) = 18. Try n = 9, c = 0.9:
        // ceil(10 * 0.9) = 9 vs naive ceil(9 * 0.9) = 9.
        // n = 14, c = 0.9: ceil(15 * 0.9) = 14 vs naive ceil(14 * 0.9) = 13.
        let residuals: Vec<f64> = (1..=14).map(f64::from).collect();
        let c = ConformalCalibrator::fit(&residuals, 0.90).unwrap();
        assert_eq!(
            c.half_width(),
            14.0,
            "finite-sample correction must select the 14th, not the 13th. \
             Selecting the 13th is the naive ceil(n * coverage) and undercovers."
        );
    }

    #[test]
    fn too_few_points_is_refused_rather_than_approximated() {
        // 95% coverage needs ceil((n+1) * 0.95) <= n, which fails for n = 5.
        let residuals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        match ConformalCalibrator::fit(&residuals, 0.95).unwrap_err() {
            CalibrationError::TooFewPoints { n, needed, .. } => {
                assert_eq!(n, 5);
                assert!(
                    needed > 5,
                    "the error must say how many are needed, got {needed}"
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn invalid_coverage_is_refused() {
        for bad in [0.0, 1.0, -0.5, 1.5] {
            assert!(matches!(
                ConformalCalibrator::fit(&[1.0, 2.0, 3.0], bad),
                Err(CalibrationError::InvalidCoverage { .. })
            ));
        }
    }

    #[test]
    fn a_non_finite_residual_is_refused_with_its_index() {
        match ConformalCalibrator::fit(&[1.0, 2.0, f64::NAN, 4.0], 0.5).unwrap_err() {
            CalibrationError::NonFiniteResidual { index } => assert_eq!(index, 2),
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn higher_coverage_gives_wider_intervals() {
        let residuals: Vec<f64> = (1..=100).map(f64::from).collect();
        let c80 = ConformalCalibrator::fit(&residuals, 0.80).unwrap();
        let c95 = ConformalCalibrator::fit(&residuals, 0.95).unwrap();
        assert!(c95.half_width() > c80.half_width());
    }

    #[test]
    fn an_interval_contains_its_own_point_estimate() {
        let c = ConformalCalibrator::fit(&[1.0, 2.0, 3.0, 4.0, 5.0], 0.5).unwrap();
        assert!(c.interval(42.0).contains(42.0));
    }

    #[test]
    fn coverage_report_converts_to_model_card_evidence() {
        let mut rng = Rng::new(11);
        let points = gaussian_points(&mut rng, 500, 2.0);
        let c = ConformalCalibrator::fit(&residuals(&points), 0.80).unwrap();
        let report = evaluate_coverage(&c, &points, 10);
        let evidence = report.as_measured_coverage();
        assert_eq!(evidence.nominal, 0.80);
        assert_eq!(evidence.n, 500);
        assert!(
            evidence.stratified,
            "strata were computed, so the evidence must say so"
        );
    }
}
