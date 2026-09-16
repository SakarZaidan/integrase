"""A teaching sandbox for cell simulation.

A separate surface from the professional design tool, sharing its sequence core.
Different users, different fidelity expectations, and a different safety
profile: this is not a mode hidden inside the CAD tool.

What it is: a simulator built on the whole-cell pattern of Karr et al. 2012, at
educational fidelity. One shared `CellState`, many independent `Process`
implementations that read and modify it, and a `Simulator` that steps them.

What it is not: a whole-cell model. Karr's took a large team years for the
simplest organism known. This uses the same architecture at far lower fidelity,
and every parameter that was chosen rather than measured says so.
"""

from .genome import (
    BASES,
    CODON_TABLE,
    STOP_CODONS,
    Chromosome,
    Gene,
    Genome,
    InvalidSequenceError,
    Sequence,
)
from .parameters import Parameter, Provenance, illustrative
from .process import Process
from .state import CellState, Counts, Environment

__all__ = [
    "BASES",
    "CODON_TABLE",
    "STOP_CODONS",
    "CellState",
    "Chromosome",
    "Counts",
    "Environment",
    "Gene",
    "Genome",
    "InvalidSequenceError",
    "Parameter",
    "Process",
    "Provenance",
    "Sequence",
    "illustrative",
]
