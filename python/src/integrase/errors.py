"""Exception hierarchy.

Mirrors the Rust error enums (subplan 12) so a user reading Rust docs and a user
reading Python docs learn the same failure model.
"""

from __future__ import annotations


class IntegraseError(Exception):
    """Base class for every error this package raises."""


class CoreError(IntegraseError):
    """Sequence or design manipulation failed."""


class SbolError(IntegraseError):
    """SBOL import or export failed."""


class SimulationError(IntegraseError):
    """A prediction could not be produced."""


class ScreeningRefused(IntegraseError):  # noqa: N818
    """Screening refused to clear a design for synthesis-bound export.

    Raised rather than returned, deliberately. In Rust a refusal is a typed
    result because callers must handle it. In Python, an exception is what stops
    a notebook cell proceeding as though nothing happened, and that is the
    behaviour that matters here. Same semantics, idiomatic shape in each
    language.

    This is also the boundary where the compile-time guarantee degrades to a
    runtime check: Python cannot express "constructible only by that module", so
    the check is explicit and separately tested.

    The ``N818`` suppression above is deliberate. Ruff wants exception names to
    end in ``Error``, which is a good default and wrong here. Subplan 12 (R-12.3)
    holds that a refusal is not an error, it is a result: the system working
    exactly as designed. Naming it ``ScreeningRefusedError`` would file it
    alongside I/O failures and lose that distinction, which matters in the UI,
    where a refusal should be explained rather than rendered as a fault.
    """
