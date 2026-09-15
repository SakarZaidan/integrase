"""Tests for the Rust-backed bindings.

The float-coercion tests are the ones that matter. They guard the project's
central principle at the boundary where the Rust type system stops protecting
it, which is the second of three documented weakenings of rule R-0.1.
"""

from __future__ import annotations

import pytest

import integrase as ig


def _strong() -> ig.Prediction:
    return ig.predict_translation_rate("TTTTTTAGGAGGTTTTT")


# --- The guarantee this boundary has to carry --------------------------------


def test_float_coercion_is_refused_with_an_actionable_message() -> None:
    with pytest.raises(TypeError) as e:
        float(_strong())
    assert ".value" in str(e.value), "the error must name the correct accessor"


def test_int_coercion_is_refused_too() -> None:
    with pytest.raises(TypeError):
        int(_strong())


def test_sum_over_predictions_fails_rather_than_silently_working() -> None:
    # The subtler path: aggregating is another way uncertainty gets dropped
    # without anyone noticing.
    with pytest.raises(TypeError):
        sum([_strong(), _strong()])  # type: ignore[call-overload]


def test_there_is_no_index_protocol_escape_hatch() -> None:
    # __index__ would let a Prediction be used as a list index, which is another
    # silent coercion route.
    with pytest.raises(TypeError):
        [1, 2, 3][_strong()]  # type: ignore[index]


# --- Honest reporting --------------------------------------------------------


def test_an_uncalibrated_model_reports_no_interval() -> None:
    pred = _strong()
    assert pred.interval is None, "no demonstrated coverage means no honest interval"
    assert pred.domain.coverage_is_unreliable


def test_the_repr_says_why_there_is_no_interval() -> None:
    # A user reading a repr in a notebook should not have to guess.
    assert "no calibration evidence" in repr(_strong())


def test_a_different_host_is_reported_even_though_the_model_is_uncalibrated() -> None:
    # The regression: host mismatch used to be masked by "uncalibrated".
    pred = ig.predict_translation_rate("TTTTTTAGGAGGTTTTT", host="Paracoccus denitrificans")
    assert pred.domain.is_out_of_distribution
    assert pred.domain.reason is not None
    assert "Paracoccus" in pred.domain.reason


def test_the_fitted_host_is_not_flagged_as_out_of_distribution() -> None:
    assert not _strong().domain.is_out_of_distribution


# --- Behaviour ---------------------------------------------------------------


def test_a_canonical_rbs_outperforms_a_poly_t_tract() -> None:
    strong = ig.predict_translation_rate("TTTTTTAGGAGGTTTTT")
    weak = ig.predict_translation_rate("TTTTTTTTTTTTTTTTT")
    assert strong.value > weak.value


def test_batch_returns_one_prediction_per_input_in_order() -> None:
    utrs = ["TTTTTTAGGAGGTTTTT", "TTTTTTTTTTTTTTTTT", "TTTTTTAGGAGGTTTTT"]
    preds = ig.predict_translation_rate_batch(utrs)
    assert len(preds) == len(utrs)
    assert preds[0].value == pytest.approx(preds[2].value)
    assert preds[0].value > preds[1].value


def test_a_malformed_start_codon_is_refused_with_its_length() -> None:
    with pytest.raises(ValueError, match="3 bases"):
        ig.predict_translation_rate("TTTTTTAGGAGGTTTTT", start_codon="AT")


def test_too_short_a_utr_is_refused_and_says_the_minimum() -> None:
    with pytest.raises(ValueError, match="at least"):
        ig.predict_translation_rate("AGGAGG")


# --- Scope -------------------------------------------------------------------


def test_scope_states_the_limits() -> None:
    s = ig.scope()
    for phrase in ["does not edit genomes", "not measurements", "experimental validation"]:
        assert phrase in s, f"scope statement must mention: {phrase}"


# --- The stub must match the extension ---------------------------------------


def test_the_type_stub_covers_every_exported_name() -> None:
    """A binding added in Rust and forgotten in the stub fails here.

    The stub is hand-written because the extension is compiled, so nothing else
    keeps the two in step.
    """
    from pathlib import Path

    stub = (Path(__file__).parent.parent / "src/integrase/_integrase.pyi").read_text()
    import integrase._integrase as ext

    exported = [n for n in dir(ext) if not n.startswith("_") or n == "__version__"]
    missing = [n for n in exported if n not in stub]
    assert not missing, f"names exported by the extension but absent from the stub: {missing}"
