"""Numbers, and where they came from.

Every rate constant in this simulator is an estimate. Some are measured values
with a published error bar, most are chosen to make a teaching example behave
the way a lesson needs it to. A student who cannot tell those apart learns that
biology is a set of known constants, which is the opposite of the truth.

So a parameter is not a float. It is a value, a uncertainty, and a provenance,
and the provenance is not optional.
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from enum import Enum


class Provenance(Enum):
    """Where a parameter's value came from.

    The distinction that matters is not "accurate" against "inaccurate". It is
    "someone measured this" against "someone chose this so the lesson works".
    """

    MEASURED = "measured"
    """From a published measurement. `source` names it."""

    DERIVED = "derived"
    """Computed from measured values. `source` says how."""

    ILLUSTRATIVE = "illustrative"
    """Chosen to make a teaching example behave a particular way.

    Not a real number about a real organism. Marked so, everywhere it surfaces.
    """


@dataclass(frozen=True, slots=True)
class Parameter:
    """A simulation constant with its uncertainty and its origin.

    Immutable, because a parameter that changed during a run would make the run
    unreproducible and the reason invisible.
    """

    value: float
    """Best estimate, in `unit`."""

    unit: str
    """Physical unit, as plain text. `"1/s"`, `"molecules/cell"`, `"nM"`."""

    provenance: Provenance
    """Where the value came from."""

    source: str = ""
    """Citation, derivation, or the reason an illustrative value was chosen."""

    relative_error: float | None = None
    """Fractional one-sigma uncertainty, if known. `0.2` means 20 percent.

    `None` means the uncertainty is unknown, which is different from zero and is
    displayed differently. An illustrative parameter has no meaningful error bar
    because it is not an estimate of anything.
    """

    def __post_init__(self) -> None:
        if not math.isfinite(self.value):
            msg = f"parameter value must be finite, got {self.value}"
            raise ValueError(msg)
        if self.relative_error is not None and self.relative_error < 0:
            msg = f"relative_error must not be negative, got {self.relative_error}"
            raise ValueError(msg)
        if self.provenance is not Provenance.ILLUSTRATIVE and not self.source:
            msg = (
                "a measured or derived parameter must name its source; "
                "an unsourced number is indistinguishable from a guess"
            )
            raise ValueError(msg)

    @property
    def is_illustrative(self) -> bool:
        """Whether this number was chosen rather than measured."""
        return self.provenance is Provenance.ILLUSTRATIVE

    def describe(self) -> str:
        """One line, for a tooltip or a parameter table.

        Reads as a sentence rather than a row of fields, because this is what a
        student sees when they ask where a number came from.
        """
        if self.relative_error is None:
            head = f"{self.value:g} {self.unit}"
        else:
            head = f"{self.value:g} +/- {self.value * self.relative_error:g} {self.unit}"

        if self.is_illustrative:
            tail = "illustrative: chosen for teaching, not measured"
            if self.source:
                tail = f"{tail} ({self.source})"
        else:
            tail = f"{self.provenance.value}: {self.source}"
        return f"{head} [{tail}]"

    def __float__(self) -> float:
        """The value alone, for arithmetic inside a process.

        Deliberately available here, unlike `Prediction` in the Rust core, which
        refuses it. The reason the two differ: a `Prediction` crossing an API
        boundary without its interval is a claim made without its uncertainty,
        while a parameter is an input to a model that is already labelled as a
        model. The uncertainty travels with the run, not with each multiply.
        """
        return self.value


def illustrative(value: float, unit: str, why: str = "") -> Parameter:
    """Shorthand for a teaching value.

    Used constantly in the reference organism, so it gets a helper. The
    provenance is still recorded; only the typing is shortened.
    """
    return Parameter(value=value, unit=unit, provenance=Provenance.ILLUSTRATIVE, source=why)
