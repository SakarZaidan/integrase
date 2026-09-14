"""Tests for the confidence types.

The float-coercion test is the one that matters: it guards the product's central
principle at the boundary where the Rust type system stops protecting it.
"""

from __future__ import annotations

import pytest

from integrase import DomainVerdict, Interval, Prediction
from integrase.confidence import DomainKind


def _prediction(value: float = 4200.0) -> Prediction[float]:
    return Prediction(
        value=value,
        interval=Interval(1900.0, 9100.0),
        coverage=0.80,
        model="rbs-thermo-v1",
        domain=DomainVerdict(kind=DomainKind.IN_DISTRIBUTION, similarity=0.91),
    )


def test_float_coercion_is_refused_with_an_actionable_message() -> None:
    pred = _prediction()
    with pytest.raises(TypeError) as excinfo:
        float(pred)
    assert ".value" in str(excinfo.value), "the error must name the correct accessor"


def test_sum_over_predictions_also_fails_rather_than_silently_working() -> None:
    # Guards the subtler path: aggregating predictions is another way the
    # uncertainty gets dropped without anyone noticing.
    preds = [_prediction(1.0), _prediction(2.0)]
    with pytest.raises(TypeError):
        sum(preds)  # type: ignore[call-overload]


def test_inverted_interval_is_rejected_at_construction() -> None:
    with pytest.raises(ValueError, match="exceeds upper bound"):
        Interval(9100.0, 1900.0)


def test_interval_contains_its_point_estimate() -> None:
    assert Interval(1900.0, 9100.0).contains(4200.0)
    assert not Interval(1900.0, 9100.0).contains(42.0)


def test_uncalibrated_and_ood_both_report_unreliable_coverage() -> None:
    for kind in (DomainKind.UNCALIBRATED, DomainKind.OUT_OF_DISTRIBUTION):
        assert DomainVerdict(kind=kind).coverage_is_unreliable


def test_in_distribution_does_not_report_unreliable_coverage() -> None:
    verdict = DomainVerdict(kind=DomainKind.IN_DISTRIBUTION, similarity=0.9)
    assert not verdict.coverage_is_unreliable
    assert not verdict.is_out_of_distribution
