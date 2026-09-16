"""A small fictional organism, for lessons to start from.

Not a model of anything that exists. The genes are invented, the sequences are
short enough to read on one screen, and the parameters are chosen so a circuit
settles within a few simulated minutes.

# Why fictional rather than a real minimal genome

Two reasons. A real organism invites the question "is this accurate", and the
answer at this fidelity would be no, in ways a student could not see. And a real
genome's smallest member still has several hundred genes, which is unreadable.

`Mycoplasma genitalium` has 525 genes and was the subject of the whole-cell
model this architecture follows. That took a large team years. Four genes is
what an afternoon's lesson can hold.
"""

from __future__ import annotations

from .genome import Chromosome, Gene, Genome, Sequence
from .process import Process
from .processes import ENERGY, Degradation, Metabolism, Transcription, Translation
from .state import CellState, Counts, Environment

# Short coding sequences, each beginning with a start codon and ending with a
# stop. Written out so a student can read one, change a base, and see what the
# protein becomes.
_REPORTER = "ATGGGTAGCAAAGGTGAAGAACTGTTTACCGGTGTTGTTCCGATTCTGGTTGAACTGTAA"
_REGULATOR = "ATGAAACGTACCGCGACCCTGGAAGATGTTGCGAAACGTGCGAACGTTAGCAAAGGTTAA"
_ENZYME = "ATGAGCACCAAAGGTGTTCTGACCGCGGGTGCGACCGGTAGCCTGGGTGCGTATCTGTAA"
_ESSENTIAL = "ATGGATCTGAAAGCGCTGCTGGAAGCGCTGAAAGCGGGTAAACTGGAAGCGAAATTCTAA"


def reference_genome() -> Genome:
    """Four genes on one chromosome.

    `essential` is named that because switching it off is meant to kill the
    cell. Nothing in the engine treats the name specially. A lesson that wants
    it to be essential adds a process that checks for its product, which is the
    honest way to teach what "essential" means: it is a statement about the rest
    of the cell, not a property of the gene.
    """
    return Genome(
        chromosomes=[
            Chromosome(
                name="chromosome-1",
                genes=[
                    Gene("reporter", Sequence(_REPORTER), promoter_strength=1.0),
                    Gene("regulator", Sequence(_REGULATOR), promoter_strength=0.5),
                    Gene("enzyme", Sequence(_ENZYME), promoter_strength=0.8),
                    Gene("essential", Sequence(_ESSENTIAL), promoter_strength=1.2),
                ],
            )
        ]
    )


def reference_cell(nutrients: float = 1.0) -> CellState:
    """A cell holding the reference genome, with a small starting energy pool.

    Starting energy is not zero, because a cell with none of it would die on the
    first step before metabolism had run once, and a student's first run showing
    instant death teaches nothing.
    """
    return CellState(
        genome=reference_genome(),
        metabolites=Counts({ENERGY: 10.0}),
        environment=Environment(nutrients=nutrients),
    )


def default_processes() -> list[Process]:
    """The four processes, in the order they should run.

    Order matters and is worth reading. Metabolism runs first so energy produced
    this step is available to translation in the same step. Transcription then
    translation, so a transcript made now can be translated now rather than
    waiting a step. Degradation last, so nothing is removed before it has had a
    chance to be used.

    A different order gives different numbers. That is not a flaw to hide: it is
    a real property of stepping continuous processes discretely, and a student
    who reorders this list and sees the curves change has learned something
    true about simulation.
    """
    return [Metabolism(), Transcription(), Translation(), Degradation()]
