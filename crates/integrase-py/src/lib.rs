//! Python bindings for Integrase.
//!
//! # The guarantee this module has to carry across
//!
//! In Rust, [`integrase_sim::Prediction`] cannot be reduced to a bare number:
//! there is no `Deref`, no `From`, no `into_value`. The compiler enforces it.
//!
//! **Python has no such enforcement**, so the guarantee degrades from a
//! compile-time property to a runtime one. This is the second of three
//! documented weakenings of tier-0 rule R-0.1, recorded in `ARCHITECTURE.md`,
//! and the one most likely to matter in practice because a notebook is exactly
//! where `float(predict(design))` gets written.
//!
//! So `__float__` raises, and so does `__int__`, and there is no `__index__`.
//! Getting the number out means naming `.value`, which costs a user three
//! seconds once.

#![allow(clippy::needless_pass_by_value)] // PyO3 signatures take owned values.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use integrase_sim::{
    DomainReason, DomainVerdict, Interval, Prediction, RbsInput, RbsThermodynamicModel,
};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

/// An inclusive interval around a predicted value.
#[pyclass(name = "Interval", frozen, get_all, skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PyInterval {
    /// Lower bound.
    pub lower: f64,
    /// Upper bound.
    pub upper: f64,
}

#[pymethods]
impl PyInterval {
    /// Width of the interval.
    #[getter]
    const fn width(&self) -> f64 {
        self.upper - self.lower
    }

    /// Whether `value` falls inside.
    fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }

    fn __repr__(&self) -> String {
        format!("Interval(lower={}, upper={})", self.lower, self.upper)
    }
}

impl From<Interval> for PyInterval {
    fn from(i: Interval) -> Self {
        Self {
            lower: i.lower,
            upper: i.upper,
        }
    }
}

/// Whether an input fell inside a model's validated range.
///
/// Separate from the interval, deliberately. Conformal coverage holds under
/// exchangeability between calibration data and the test point, and an
/// out-of-distribution input is exactly where that fails. The interval answers
/// "given this input is like my calibration data, how wrong am I likely to be".
/// This answers "is it".
#[pyclass(name = "DomainVerdict", frozen, skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PyDomainVerdict {
    kind: String,
    similarity: Option<f64>,
    reason: Option<String>,
}

#[pymethods]
impl PyDomainVerdict {
    /// One of `in_distribution`, `marginal`, `out_of_distribution`, `uncalibrated`.
    #[getter]
    fn kind(&self) -> &str {
        &self.kind
    }

    /// Similarity to the validated range, where the verdict computes one.
    #[getter]
    const fn similarity(&self) -> Option<f64> {
        self.similarity
    }

    /// What triggered a marginal or out-of-distribution verdict.
    #[getter]
    fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    /// True if the model was not validated on inputs like this one.
    #[getter]
    fn is_out_of_distribution(&self) -> bool {
        self.kind == "out_of_distribution"
    }

    /// True if the stated coverage does not hold for this input.
    ///
    /// Covers both the out-of-distribution and uncalibrated cases, because in
    /// both the number attached to the interval means nothing.
    #[getter]
    fn coverage_is_unreliable(&self) -> bool {
        self.kind == "out_of_distribution" || self.kind == "uncalibrated"
    }

    fn __repr__(&self) -> String {
        match (&self.similarity, &self.reason) {
            (Some(s), Some(r)) => format!(
                "DomainVerdict({}, similarity={s:.3}, reason={r})",
                self.kind
            ),
            (Some(s), None) => format!("DomainVerdict({}, similarity={s:.3})", self.kind),
            (None, Some(r)) => format!("DomainVerdict({}, reason={r})", self.kind),
            (None, None) => format!("DomainVerdict({})", self.kind),
        }
    }
}

fn describe_reason(r: &DomainReason) -> String {
    match r {
        DomainReason::HostMismatch { got, expected } => {
            format!("host {got} differs from the fitted host {expected}")
        }
        DomainReason::NonCanonicalStartCodon { codon } => {
            format!("non-canonical start codon {codon}")
        }
        DomainReason::SpacerOutOfRange { length } => {
            format!("spacer length {length} outside range")
        }
        DomainReason::LengthOutOfRange { length } => format!("length {length} outside range"),
        DomainReason::StrongSecondaryStructure {
            free_energy_kcal_per_mol,
        } => {
            format!("strong secondary structure, {free_energy_kcal_per_mol:.2} kcal/mol")
        }
        DomainReason::PossibleSecondaryStructure { stem_fraction } => {
            format!(
                "possible secondary structure, {:.0}% of the sequence in a stem",
                stem_fraction * 100.0
            )
        }
        DomainReason::EmbeddingDistance { distance } => {
            format!("embedding distance {distance:.3} from the calibration set")
        }
        _ => "unspecified".to_owned(),
    }
}

impl From<&DomainVerdict> for PyDomainVerdict {
    fn from(v: &DomainVerdict) -> Self {
        match v {
            DomainVerdict::InDistribution { similarity } => Self {
                kind: "in_distribution".into(),
                similarity: Some(*similarity),
                reason: None,
            },
            DomainVerdict::Marginal { similarity, reason } => Self {
                kind: "marginal".into(),
                similarity: Some(*similarity),
                reason: Some(describe_reason(reason)),
            },
            DomainVerdict::OutOfDistribution { reason } => Self {
                kind: "out_of_distribution".into(),
                similarity: None,
                reason: Some(describe_reason(reason)),
            },
            DomainVerdict::Uncalibrated => Self {
                kind: "uncalibrated".into(),
                similarity: None,
                reason: None,
            },
            _ => Self {
                kind: "uncalibrated".into(),
                similarity: None,
                reason: None,
            },
        }
    }
}

/// A predicted quantity together with everything needed to judge it.
#[pyclass(name = "Prediction", frozen, skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PyPrediction {
    value: f64,
    interval: Option<PyInterval>,
    coverage: f64,
    model: String,
    domain: PyDomainVerdict,
}

#[pymethods]
impl PyPrediction {
    /// The point estimate.
    #[getter]
    const fn value(&self) -> f64 {
        self.value
    }

    /// The interval, if this model has demonstrated calibration.
    ///
    /// `None` means the model has no coverage evidence, in which case there is
    /// no honest interval to report. Absence is the truthful answer; a wide
    /// interval would imply a quantified uncertainty the model has not earned.
    #[getter]
    fn interval(&self) -> Option<PyInterval> {
        self.interval.clone()
    }

    /// The coverage level the interval claims.
    #[getter]
    const fn coverage(&self) -> f64 {
        self.coverage
    }

    /// Which model version produced this.
    #[getter]
    fn model(&self) -> &str {
        &self.model
    }

    /// Whether the input fell inside the model's validated range.
    #[getter]
    fn domain(&self) -> PyDomainVerdict {
        self.domain.clone()
    }

    /// Refuses coercion to a bare number.
    ///
    /// **The single most important method in these bindings.**
    ///
    /// `rate = float(predict(design))` is the one line that silently discards
    /// the interval, the coverage, and the domain verdict, and in a notebook it
    /// would be written within a minute of installation. Rust prevents it at
    /// compile time. Python cannot, so it is refused at runtime with a message
    /// that names the right accessor.
    fn __float__(&self) -> PyResult<f64> {
        Err(PyTypeError::new_err(
            "Prediction cannot be coerced to float, deliberately. Use .value for the point \
             estimate, and .interval and .domain for what it is worth. \
             See docs/concepts/uncertainty.md.",
        ))
    }

    /// Refuses coercion to an integer, for the same reason as `__float__`.
    fn __int__(&self) -> PyResult<i64> {
        Err(PyTypeError::new_err(
            "Prediction cannot be coerced to int, deliberately. Use .value, and see \
             .interval and .domain for how much to trust it.",
        ))
    }

    fn __repr__(&self) -> String {
        match &self.interval {
            Some(i) => format!(
                "Prediction(value={:.4}, interval=[{:.4}, {:.4}], coverage={:.2}, domain={})",
                self.value, i.lower, i.upper, self.coverage, self.domain.kind
            ),
            None => format!(
                "Prediction(value={:.4}, interval=None, domain={}) \
                 # no interval: this model has no calibration evidence",
                self.value, self.domain.kind
            ),
        }
    }
}

impl From<&Prediction<f64>> for PyPrediction {
    fn from(p: &Prediction<f64>) -> Self {
        Self {
            value: *p.value(),
            interval: p.interval().map(PyInterval::from),
            coverage: p.nominal_coverage(),
            model: p.model().0.clone(),
            domain: PyDomainVerdict::from(p.domain()),
        }
    }
}

/// Predicts the relative translation initiation rate for a ribosome binding site.
///
/// `utr` is the 5' untranslated region immediately upstream of the start codon.
/// `start_codon` must be exactly three bases. `host` defaults to the organism
/// the model was fitted on.
///
/// Returns a [`PyPrediction`]. The model currently reports `uncalibrated` and no
/// interval, because no coverage has been demonstrated on held-out data. That is
/// the truthful state, not a placeholder.
#[pyfunction]
#[pyo3(signature = (utr, start_codon="ATG", host="Escherichia coli"))]
fn predict_translation_rate(utr: &str, start_codon: &str, host: &str) -> PyResult<PyPrediction> {
    let codon: [u8; 3] = start_codon.as_bytes().try_into().map_err(|_| {
        PyValueError::new_err(format!(
            "start_codon must be exactly 3 bases; got {:?} ({} bases)",
            start_codon,
            start_codon.len()
        ))
    })?;

    let model = RbsThermodynamicModel::new();
    let input = RbsInput {
        utr: utr.as_bytes().to_vec(),
        start_codon: codon,
        host: host.to_owned(),
    };
    let prediction = model
        .predict(&input)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyPrediction::from(&prediction))
}

/// Predicts for many sequences in one crossing of the language boundary.
///
/// Subplan 11's primary scenario is a researcher screening hundreds of variants.
/// Doing that as N separate calls would make the boundary cost dominate, so the
/// batch path exists from the start rather than being added once someone
/// complains.
#[pyfunction]
#[pyo3(signature = (utrs, start_codon="ATG", host="Escherichia coli"))]
fn predict_translation_rate_batch(
    utrs: Vec<String>,
    start_codon: &str,
    host: &str,
) -> PyResult<Vec<PyPrediction>> {
    utrs.iter()
        .map(|u| predict_translation_rate(u, start_codon, host))
        .collect()
}

/// The scope limits this software operates under.
///
/// Exposed as a function rather than only documented, because a user about to
/// trust an output should be able to ask what it means without leaving their
/// notebook.
#[pyfunction]
const fn scope() -> &'static str {
    "Integrase operates on sequence data. It does not edit genomes, interact with living \
     organisms, or order DNA. Predictions are model outputs, not measurements: their intervals \
     describe uncertainty about the model's prediction, not about what will happen in a cell. \
     Genomic context, host physiology, and growth conditions are invisible to a sequence-only \
     model. No output substitutes for experimental validation."
}

/// The Rust extension module.
///
/// # Errors
///
/// Propagates PyO3 registration failures.
#[pymodule]
fn _integrase(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPrediction>()?;
    m.add_class::<PyInterval>()?;
    m.add_class::<PyDomainVerdict>()?;
    m.add_function(wrap_pyfunction!(predict_translation_rate, m)?)?;
    m.add_function(wrap_pyfunction!(predict_translation_rate_batch, m)?)?;
    m.add_function(wrap_pyfunction!(scope, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// These tests exist partly for their own sake and partly as a **link
    /// canary**.
    ///
    /// This crate previously had no Rust tests, so nothing built a binary that
    /// had to link against libpython, and `pyo3/extension-module` being a
    /// default feature went unnoticed locally. Linux tolerates the undefined
    /// symbols; macOS does not, and CI failed there with a wall of
    /// `_PyExc_AttributeError, referenced from:`.
    ///
    /// Any test in this module forces the link. Deleting them all would let the
    /// same misconfiguration return silently.
    #[test]
    fn this_crate_links_against_libpython_in_test_builds() {
        // The assertion is incidental. Building this at all is the test.
        assert!(!scope().is_empty());
    }

    #[test]
    fn a_host_mismatch_reason_names_both_organisms() {
        let r = DomainReason::HostMismatch {
            got: "Paracoccus denitrificans".into(),
            expected: "Escherichia coli".into(),
        };
        let s = describe_reason(&r);
        assert!(
            s.contains("Paracoccus denitrificans"),
            "must name what was given: {s}"
        );
        assert!(
            s.contains("Escherichia coli"),
            "must name what was expected: {s}"
        );
    }

    #[test]
    fn a_structure_reason_reports_a_percentage_not_a_raw_fraction() {
        let s = describe_reason(&DomainReason::PossibleSecondaryStructure {
            stem_fraction: 0.62,
        });
        assert!(
            s.contains("62%"),
            "a user reads percentages, not fractions: {s}"
        );
    }

    /// The two structure reasons must not be confused: one carries a computed
    /// free energy, the other carries only a heuristic fraction.
    #[test]
    fn the_two_structure_reasons_describe_themselves_differently() {
        let computed = describe_reason(&DomainReason::StrongSecondaryStructure {
            free_energy_kcal_per_mol: -12.5,
        });
        let heuristic = describe_reason(&DomainReason::PossibleSecondaryStructure {
            stem_fraction: 0.62,
        });
        assert!(
            computed.contains("kcal/mol"),
            "the computed one reports energy: {computed}"
        );
        assert!(
            !heuristic.contains("kcal/mol"),
            "the heuristic one must not imply an energy it did not compute: {heuristic}"
        );
    }

    #[test]
    fn the_scope_statement_names_what_the_tool_does_not_do() {
        let s = scope();
        for phrase in [
            "does not edit genomes",
            "not measurements",
            "experimental validation",
        ] {
            assert!(s.contains(phrase), "scope must mention {phrase:?}");
        }
    }

    #[test]
    fn an_uncalibrated_prediction_converts_with_no_interval() {
        use integrase_sim::ModelId;
        let p = Prediction::without_interval(
            42.0_f64,
            ModelId("test-model".into()),
            DomainVerdict::OutOfDistribution {
                reason: DomainReason::HostMismatch {
                    got: "other".into(),
                    expected: "E. coli".into(),
                },
            },
        );
        let py = PyPrediction::from(&p);
        assert!(
            py.interval.is_none(),
            "no interval must survive the conversion"
        );
        assert_eq!(
            py.domain.kind, "out_of_distribution",
            "the verdict must survive it too"
        );
    }
}
