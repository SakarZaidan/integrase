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

from .aminoacids import BY_LETTER, AminoAcid, substitution_effect
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
from .molecules import Chemistry, Fit, Group, Molecule, Pocket, Verdict, fit
from .parameters import Parameter, Provenance, illustrative
from .phenotype import Contribution, Locus, MonogenicTrait, Outcome, PolygenicTrait, Trait
from .process import Process
from .safety import CURATION_STATEMENT, describe_content
from .state import CellState, Counts, Environment
from .trials import Phase, Result, develop

__all__ = [
    "BASES",
    "BY_LETTER",
    "CODON_TABLE",
    "CURATION_STATEMENT",
    "STOP_CODONS",
    "AminoAcid",
    "CellState",
    "Chemistry",
    "Chromosome",
    "Contribution",
    "Counts",
    "Environment",
    "Fit",
    "Gene",
    "Genome",
    "Group",
    "InvalidSequenceError",
    "Locus",
    "Molecule",
    "MonogenicTrait",
    "Outcome",
    "Parameter",
    "Phase",
    "Pocket",
    "PolygenicTrait",
    "Process",
    "Provenance",
    "Result",
    "Sequence",
    "Trait",
    "Verdict",
    "describe_content",
    "develop",
    "fit",
    "illustrative",
    "substitution_effect",
]
