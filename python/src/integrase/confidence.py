"""Predicted values, inseparable from their uncertainty."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Generic, TypeVar

T = TypeVar("T")


class DomainKind(Enum):
    """Whether an input fell inside a model's validated range."""

    IN_DISTRIBUTION = "in_distribution"
    MARGINAL = "marginal"
    OUT_OF_DISTRIBUTION = "out_of_distribution"
    UNCALIBRATED = "uncalibrated"


@dataclass(frozen=True, slots=True)
class DomainVerdict:
    """A verdict on whether a model has seen anything like this input.

    Separate from the interval, deliberately. Conformal coverage guarantees hold
    under exchangeability between the calibration set and the test point, and
    out-of-distribution inputs are exactly where exchangeability fails. The
    interval answers "given this input is like my calibration data, how wrong am
    I likely to be?". This answers "is it like my calibration data at all?".
    """

    kind: DomainKind
    similarity: float | None = None
    reason: str | None = None

    @property
    def is_out_of_distribution(self) -> bool:
        """True if the model was not validated on inputs like this one."""
        return self.kind is DomainKind.OUT_OF_DISTRIBUTION

    @property
    def coverage_is_unreliable(self) -> bool:
        """True if the stated coverage does not hold for this input."""
        return self.kind in (DomainKind.OUT_OF_DISTRIBUTION, DomainKind.UNCALIBRATED)


@dataclass(frozen=True, slots=True)
class Interval:
    """An inclusive interval around a predicted value."""

    lower: float
    upper: float

    def __post_init__(self) -> None:
        if self.lower > self.upper:
            msg = f"interval lower bound {self.lower} exceeds upper bound {self.upper}"
            raise ValueError(msg)

    @property
    def width(self) -> float:
        """Distance between the bounds."""
        return self.upper - self.lower

    def contains(self, value: float) -> bool:
        """True if ``value`` falls within the interval."""
        return self.lower <= value <= self.upper


@dataclass(frozen=True, slots=True)
class Prediction(Generic[T]):
    """A predicted quantity together with everything needed to judge it."""

    value: T
    interval: Interval | None
    coverage: float
    model: str
    domain: DomainVerdict

    def __float__(self) -> float:
        """Refuse coercion to a bare number.

        This is the single most important line in the Python SDK.

        ``rate = float(predict(design))`` is the one-liner that silently discards
        uncertainty, and in a notebook it would be written within a minute of
        installation. Making it a ``TypeError`` that points at ``.value`` costs a
        user three seconds once and preserves the entire reason this package
        exists.
        """
        msg = (
            "Prediction cannot be coerced to float, deliberately. Use .value for the point "
            "estimate, and .interval and .domain for what it is worth. See "
            "https://github.com/SakarZaidan/integrase docs on uncertainty."
        )
        raise TypeError(msg)
