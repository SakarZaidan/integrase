"""The processes that make a cell do anything.

Four to start with: transcription, translation, degradation, and metabolism.
Each is independent. None imports another, none knows the others exist, and
removing any one leaves the rest running.

That independence is the point. A student adding a fifth process writes a class
with a `step` method and appends it to the simulator's list. Nothing else in
this file changes.

# The rate model

Mass-action kinetics with first-order rates: the amount of a thing produced or
consumed in `dt` is proportional to how much of its input is present. Real
transcription saturates, real ribosomes compete for messages, and neither is
modelled here. The simplification is stated in each process rather than left to
be discovered by a student wondering why their numbers do not match a textbook.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from .parameters import Parameter, illustrative
from .state import CellState

# --- shared defaults ---------------------------------------------------------
#
# Every number here is illustrative. They are chosen so that a small circuit
# reaches steady state in a few simulated minutes, which is the timescale a
# student will sit and watch. They are not measurements of any organism.

DEFAULT_TRANSCRIPTION_RATE = illustrative(
    0.05, "transcripts/s per unit promoter strength", "gives a visible mRNA pool in ~1 min"
)
DEFAULT_TRANSLATION_RATE = illustrative(
    0.10, "proteins/s per transcript", "roughly two proteins per transcript per 20 s"
)
DEFAULT_RNA_DEGRADATION_RATE = illustrative(
    0.02, "1/s", "mRNA half-life around 35 s, the right order for a bacterium"
)
DEFAULT_PROTEIN_DEGRADATION_RATE = illustrative(
    0.001, "1/s", "protein half-life around 12 min, much longer than mRNA"
)
DEFAULT_ENERGY_YIELD = illustrative(
    5.0, "ATP/s at full nutrients", "enough to support the default circuit with headroom"
)
DEFAULT_ENERGY_PER_PROTEIN = illustrative(
    0.1, "ATP per protein", "makes translation the dominant cost, as it is in a real cell"
)

DEFAULT_ENERGY_CAPACITY = illustrative(
    50.0, "ATP", "a few seconds of production, matching how fast a real ATP pool turns over"
)

ENERGY = "ATP"
"""The single energy currency this sandbox models.

One species standing in for ATP, GTP, NADH and the rest. A student who asks why
there is only one is asking a good question, and the answer is in this docstring
rather than hidden.
"""


@dataclass(slots=True)
class Transcription:
    """Genes produce messenger RNA, at a rate set by promoter strength.

    Does not model RNA polymerase availability, so two genes with the same
    promoter strength transcribe at the same rate no matter how many other genes
    are competing. In a real cell they would compete for a shared pool.
    """

    rate: Parameter = field(default=DEFAULT_TRANSCRIPTION_RATE)

    @property
    def name(self) -> str:
        return "transcription"

    def step(self, state: CellState, dt: float) -> None:
        for gene in state.genome.genes:
            if gene.promoter_strength <= 0:
                continue
            made = float(self.rate) * gene.promoter_strength * dt
            state.rna.add(gene.name, made)


@dataclass(slots=True)
class Translation:
    """Messenger RNA produces protein, and spends energy doing it.

    Energy is checked before the protein is made, not after. A cell that cannot
    pay produces less protein rather than going into energy debt, which is both
    what happens biologically and what keeps the numbers interpretable.
    """

    rate: Parameter = field(default=DEFAULT_TRANSLATION_RATE)
    energy_cost: Parameter = field(default=DEFAULT_ENERGY_PER_PROTEIN)

    @property
    def name(self) -> str:
        return "translation"

    def step(self, state: CellState, dt: float) -> None:
        cost = float(self.energy_cost)
        for gene in state.genome.genes:
            transcripts = state.rna[gene.name]
            if transcripts <= 0:
                continue

            wanted = float(self.rate) * transcripts * dt
            if cost > 0:
                affordable = state.metabolites[ENERGY] / cost
                made = min(wanted, affordable)
            else:
                made = wanted
            if made <= 0:
                continue

            state.protein.add(gene.product, made)
            state.metabolites.add(ENERGY, -made * cost)


@dataclass(slots=True)
class Degradation:
    """RNA and protein decay, each at its own first-order rate.

    Two rates rather than one because the difference between them is the lesson:
    mRNA turns over in under a minute while protein persists for many, which is
    why switching a gene off does not switch its product off.
    """

    rna_rate: Parameter = field(default=DEFAULT_RNA_DEGRADATION_RATE)
    protein_rate: Parameter = field(default=DEFAULT_PROTEIN_DEGRADATION_RATE)

    @property
    def name(self) -> str:
        return "degradation"

    def step(self, state: CellState, dt: float) -> None:
        for species, count in list(state.rna.items()):
            state.rna.add(species, -count * float(self.rna_rate) * dt)
        for species, count in list(state.protein.items()):
            state.protein.add(species, -count * float(self.protein_rate) * dt)


@dataclass(slots=True)
class Metabolism:
    """Nutrients become energy, and a cell with none of it dies.

    The death rule is the only place in this package where a process ends a
    simulation, and it is deliberate: a student who silences an essential gene
    or starves a cell should see it stop, with a reason, rather than watch the
    numbers drift toward zero without comment.
    """

    yield_rate: Parameter = field(default=DEFAULT_ENERGY_YIELD)
    maintenance: Parameter = field(
        default=illustrative(1.0, "ATP/s", "the cost of staying alive doing nothing")
    )
    capacity: Parameter = field(default=DEFAULT_ENERGY_CAPACITY)

    @property
    def name(self) -> str:
        return "metabolism"

    def step(self, state: CellState, dt: float) -> None:
        produced = float(self.yield_rate) * state.environment.nutrients * dt
        spent = float(self.maintenance) * dt
        state.metabolites.add(ENERGY, produced - spent)

        # A cell does not bank energy without limit. Real ATP pools are small
        # and turn over in seconds; what a cell stores for later is glycogen or
        # fat, which this sandbox does not model. Without a cap the pool grows
        # forever and every later lesson about energy limits becomes untestable,
        # because nothing is ever short of it.
        ceiling = float(self.capacity)
        if state.metabolites[ENERGY] > ceiling:
            state.metabolites[ENERGY] = ceiling

        # Starvation is checked against the balance rather than the stock. A
        # cell holding energy while spending more than it makes is dying and
        # should be told so, not left to look healthy until the moment it is not.
        if state.metabolites[ENERGY] <= 0 and produced < spent:
            state.die(
                "out of energy: the cell spent more than it produced. "
                f"Nutrients are at {state.environment.nutrients:g}."
            )
