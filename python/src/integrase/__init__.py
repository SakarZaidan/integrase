"""Integrase: genetic design with the error bars attached.

Scope, stated up front. Integrase operates on sequence data. It does not edit
genomes, does not interact with living organisms, and does not order DNA.
Predictions are model outputs, not measurements, and their intervals describe
uncertainty about the model's prediction rather than about what will happen in a
cell.

The types here are backed by the Rust core. In particular ``Prediction`` refuses
coercion to a number::

    >>> import integrase as ig
    >>> pred = ig.predict_translation_rate("TTTTTTAGGAGGTTTTT")
    >>> pred.value                      # the point estimate
    >>> pred.interval                   # None until calibration is demonstrated
    >>> pred.domain.coverage_is_unreliable
    True
    >>> float(pred)                     # TypeError, deliberately
    Traceback (most recent call last):
    TypeError: Prediction cannot be coerced to float...

That refusal is the second of three documented weakenings of the project's
no-bare-predictions rule: Rust enforces it at compile time, Python cannot, so it
is enforced at runtime instead.
"""

from integrase._integrase import (
    DomainVerdict,
    Interval,
    Prediction,
    __version__,
    predict_translation_rate,
    predict_translation_rate_batch,
    scope,
)
from integrase.errors import (
    CoreError,
    IntegraseError,
    SbolError,
    ScreeningRefused,
    SimulationError,
)

__all__ = [
    "CoreError",
    "DomainVerdict",
    "IntegraseError",
    "Interval",
    "Prediction",
    "SbolError",
    "ScreeningRefused",
    "SimulationError",
    "__version__",
    "predict_translation_rate",
    "predict_translation_rate_batch",
    "scope",
]
