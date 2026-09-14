"""Integrase: genetic design with the error bars attached.

Scope, stated up front. Integrase operates on sequence data. It does not edit
genomes, does not interact with living organisms, and does not order DNA.
Predictions are model outputs, not measurements, and their intervals describe
uncertainty about the model's prediction rather than about what will happen in a
cell.

This package is currently a surface definition. The Rust core it binds to is
under construction; see ``planning/FINAL-DECISION.md`` for the phased plan.
"""

from integrase.confidence import DomainVerdict, Interval, Prediction
from integrase.errors import IntegraseError, ScreeningRefused

__all__ = [
    "DomainVerdict",
    "IntegraseError",
    "Interval",
    "Prediction",
    "ScreeningRefused",
]
__version__ = "0.0.0"
