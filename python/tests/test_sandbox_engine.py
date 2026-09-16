"""Tests for the sandbox simulation engine.

The properties worth asserting here are not "does the number match". Every
parameter is illustrative, so a number matching is only evidence that nobody
changed it. What matters is that the model behaves the way the biology it stands
for behaves, and that the architecture holds: processes stay independent, a
student can add one, and nothing silently pretends a guess is a measurement.
"""

from __future__ import annotations

import dataclasses
import math

import pytest

from integrase_sandbox import (
    CellState,
    Counts,
    Environment,
    Gene,
    Genome,
    InvalidSequenceError,
    Parameter,
    Process,
    Provenance,
    Sequence,
    illustrative,
)
from integrase_sandbox.genome import Chromosome
from integrase_sandbox.organism import default_processes, reference_cell, reference_genome
from integrase_sandbox.processes import (
    ENERGY,
    Degradation,
    Metabolism,
    Transcription,
    Translation,
)
from integrase_sandbox.simulator import Simulator

# --- parameters --------------------------------------------------------------


def test_a_measured_parameter_must_name_its_source() -> None:
    """An unsourced number is indistinguishable from a guess.

    The whole point of the provenance field is that a student can ask where a
    number came from. Allowing MEASURED with an empty source would make the
    label meaningless while looking authoritative.
    """
    with pytest.raises(ValueError, match="must name its source"):
        Parameter(value=1.0, unit="1/s", provenance=Provenance.MEASURED)


def test_an_illustrative_parameter_says_so_when_described() -> None:
    p = illustrative(0.5, "1/s", "makes the curve settle in a minute")
    assert "illustrative" in p.describe()
    assert p.is_illustrative


def test_a_measured_parameter_shows_its_error_bar() -> None:
    p = Parameter(
        value=2.0,
        unit="1/s",
        provenance=Provenance.MEASURED,
        source="a paper",
        relative_error=0.25,
    )
    described = p.describe()
    assert "+/-" in described
    assert "0.5" in described
    assert not p.is_illustrative


def test_every_default_parameter_declares_its_provenance() -> None:
    """No number in the shipped processes may be an unlabelled float.

    This is the test that stops the honesty rule decaying. Adding a bare float
    default to a process is easy and looks harmless, and it removes the one
    thing that tells a student the number was invented.
    """
    for process in default_processes():
        values = [getattr(process, f.name) for f in dataclasses.fields(process)]  # type: ignore[arg-type]
        params = [v for v in values if isinstance(v, Parameter)]
        assert params, f"{process.name} has no parameters, which is unexpected"
        for p in params:
            assert p.provenance in Provenance
            if p.provenance is not Provenance.ILLUSTRATIVE:
                assert p.source


# --- sequences ---------------------------------------------------------------


def test_a_sequence_rejects_anything_that_is_not_a_base() -> None:
    with pytest.raises(InvalidSequenceError, match="A, C, G and T only"):
        Sequence("ACGTN")


def test_translation_stops_at_the_first_stop_codon() -> None:
    # ATG GGT TAA GGT: methionine, glycine, stop, and the trailing codon must
    # not appear.
    assert Sequence("ATGGGTTAAGGT").translate() == "MG"


def test_translation_ignores_a_trailing_incomplete_codon() -> None:
    """A ribosome cannot read two bases, so neither does this."""
    assert Sequence("ATGGGTAC").translate() == "MG"


def test_a_mutation_produces_a_new_sequence_and_leaves_the_original() -> None:
    """Editing in place would destroy the comparison the lesson depends on."""
    original = Sequence("ATGGGT")
    edited = original.mutate(4, "C")
    assert original.bases == "ATGGGT"
    assert edited.bases == "ATGGCT"


def test_a_point_mutation_can_truncate_a_protein() -> None:
    """The lesson a single base change is supposed to teach.

    ATG GGT CAA -> ATG GGT TAA turns a glutamine into a stop, and the protein
    loses everything after it.
    """
    before = Sequence("ATGGGTCAAGGTGGT")
    after = before.mutate(6, "T")
    assert before.translate() == "MGQGG"
    assert after.translate() == "MG"


def test_mutating_outside_the_sequence_raises() -> None:
    with pytest.raises(IndexError):
        Sequence("ATG").mutate(10, "C")


# --- state -------------------------------------------------------------------


def test_a_missing_species_reads_as_zero() -> None:
    assert Counts()["nothing"] == 0.0


def test_a_count_cannot_go_negative() -> None:
    """A negative molecule count is a bug indicator, not a rounding artefact."""
    c = Counts({"x": 1.0})
    c.add("x", -5.0)
    assert c["x"] == 0.0


def test_death_keeps_the_first_reason() -> None:
    """The first cause is the interesting one; later failures are symptoms."""
    state = CellState(genome=Genome())
    state.die("out of energy")
    state.die("no ribosomes")
    assert state.death_reason == "out of energy"


def test_nutrients_outside_zero_to_one_are_refused() -> None:
    with pytest.raises(ValueError, match="between 0 and 1"):
        Environment(nutrients=1.5)


# --- processes ---------------------------------------------------------------


def test_transcription_rate_follows_promoter_strength() -> None:
    """Twice the promoter strength, twice the message."""
    genome = Genome(
        chromosomes=[
            Chromosome(
                "c",
                [
                    Gene("weak", Sequence("ATGTAA"), promoter_strength=1.0),
                    Gene("strong", Sequence("ATGTAA"), promoter_strength=2.0),
                ],
            )
        ]
    )
    state = CellState(genome=genome)
    Transcription().step(state, dt=10.0)
    assert state.rna["strong"] == pytest.approx(2 * state.rna["weak"])


def test_a_silenced_gene_produces_no_message() -> None:
    genome = Genome(
        chromosomes=[Chromosome("c", [Gene("off", Sequence("ATGTAA"), promoter_strength=0.0)])]
    )
    state = CellState(genome=genome)
    Transcription().step(state, dt=100.0)
    assert state.rna["off"] == 0.0


def test_translation_is_limited_by_available_energy() -> None:
    """A cell that cannot pay makes less protein rather than going into debt."""
    genome = Genome(chromosomes=[Chromosome("c", [Gene("g", Sequence("ATGTAA"))])])
    rich = CellState(genome=genome, rna=Counts({"g": 100.0}), metabolites=Counts({ENERGY: 100.0}))
    poor = CellState(genome=genome, rna=Counts({"g": 100.0}), metabolites=Counts({ENERGY: 0.1}))

    Translation().step(rich, dt=1.0)
    Translation().step(poor, dt=1.0)

    assert poor.protein["g"] < rich.protein["g"]
    assert poor.metabolites[ENERGY] >= 0.0


def test_messages_decay_faster_than_proteins() -> None:
    """The difference between the two rates is the lesson, not an accident.

    Switching a gene off does not switch its product off, because the protein
    outlives the message by an order of magnitude.
    """
    state = CellState(genome=Genome(), rna=Counts({"x": 100.0}), protein=Counts({"x": 100.0}))
    Degradation().step(state, dt=60.0)
    assert state.rna["x"] < state.protein["x"]


def test_a_starved_cell_dies_with_a_reason() -> None:
    state = CellState(genome=Genome(), environment=Environment(nutrients=0.0))
    for _ in range(10):
        Metabolism().step(state, dt=1.0)
    assert not state.alive
    assert "energy" in state.death_reason
    assert "0" in state.death_reason


def test_energy_does_not_accumulate_without_limit() -> None:
    """A cell does not bank ATP forever.

    Without a cap, every later lesson about energy limits is untestable because
    nothing is ever short of it.
    """
    state = CellState(genome=Genome(), environment=Environment(nutrients=1.0))
    m = Metabolism()
    for _ in range(1000):
        m.step(state, dt=1.0)
    assert state.metabolites[ENERGY] <= float(m.capacity)


# --- the simulator -----------------------------------------------------------


def test_a_student_process_needs_no_base_class() -> None:
    """The single most important property of this design.

    A class with a `step` method and a `name` is a process. If this test ever
    needs a base class, an import, or a registration call to pass, the design
    has regressed into something a beginner cannot extend.
    """

    class CountSteps:
        def __init__(self) -> None:
            self.calls = 0

        @property
        def name(self) -> str:
            return "counter"

        def step(self, _state: CellState, _dt: float) -> None:
            self.calls += 1

    counter = CountSteps()
    assert isinstance(counter, Process)

    sim = Simulator(reference_cell(), [counter])
    sim.run(10, dt=1.0)
    assert counter.calls == 10


def test_something_without_a_step_method_is_refused_clearly() -> None:
    """The error has to name the missing method, not fail three frames down."""

    class NotAProcess:
        pass

    with pytest.raises(TypeError, match="missing a step method"):
        Simulator(reference_cell(), [NotAProcess()])  # type: ignore[list-item]


def test_processes_run_in_the_order_given() -> None:
    """Order is visible and fixed, because a student needs to reason about it."""
    order: list[str] = []

    class Recorder:
        def __init__(self, label: str) -> None:
            self.label = label

        @property
        def name(self) -> str:
            return self.label

        def step(self, _state: CellState, _dt: float) -> None:
            order.append(self.label)

    Simulator(reference_cell(), [Recorder("a"), Recorder("b")]).step(1.0)
    assert order == ["a", "b"]


def test_a_dead_cell_stops_being_stepped() -> None:
    """No chemistry is computed for something that is no longer alive."""
    ran_after: list[int] = []

    class Killer:
        @property
        def name(self) -> str:
            return "killer"

        def step(self, state: CellState, _dt: float) -> None:
            state.die("test")

    class ShouldNotRun:
        @property
        def name(self) -> str:
            return "after"

        def step(self, _state: CellState, _dt: float) -> None:
            ran_after.append(1)

    sim = Simulator(reference_cell(), [Killer(), ShouldNotRun()])
    sim.run(5, dt=1.0)
    assert ran_after == []


def test_a_trajectory_records_values_not_references() -> None:
    """Storing references would give every point the final state's numbers."""
    sim = Simulator(reference_cell(), default_processes())
    t = sim.run(120, dt=1.0, record_every=60)
    reporter = t.series("protein", "reporter")
    assert len(set(reporter)) > 1, "every recorded point holds the same value"


def test_a_run_that_ends_in_death_stops_there() -> None:
    """Padding the trajectory would draw a flat line that looks like steady state."""
    sim = Simulator(reference_cell(nutrients=0.0), default_processes())
    t = sim.run(1000, dt=1.0, record_every=1)
    assert not sim.state.alive
    assert sim.state.time < 1000
    assert t.times[-1] == sim.state.time


def test_an_unknown_pool_is_named_in_the_error() -> None:
    sim = Simulator(reference_cell(), default_processes())
    sim.run(2, dt=1.0)
    with pytest.raises(KeyError, match="rna"):
        sim.trajectory.series("proteins", "reporter")


def test_a_zero_or_negative_step_is_refused() -> None:
    sim = Simulator(reference_cell(), default_processes())
    with pytest.raises(ValueError, match="dt must be positive"):
        sim.step(0.0)


# --- the model as a whole ----------------------------------------------------


def test_message_levels_reach_the_steady_state_the_rates_predict() -> None:
    """Production over decay, which is the first analytic result in the subject.

    A student derives 0.05 / 0.02 = 2.5 on paper. The simulation settles at
    2.45, and the 2 percent gap is not an error to hide.

    It is the discretisation. Transcription runs before degradation in the same
    step, so the message made this step is immediately subject to decay, and the
    discrete fixed point works out to `(k/d)(1 - d*dt)` rather than `k/d`. At
    `dt = 1` and `d = 0.02` that is exactly 2 percent low.

    Asserting the continuous value with a loose tolerance would paper over a
    real and teachable property of stepping a continuous model.
    """
    sim = Simulator(reference_cell(), default_processes())
    sim.run(3000, dt=1.0, record_every=1000)

    t = Transcription()
    d = Degradation()
    gene = sim.state.genome.find("reporter")
    assert gene is not None

    k = float(t.rate) * gene.promoter_strength
    decay = float(d.rna_rate)
    continuous = k / decay
    discrete = continuous * (1 - decay * 1.0)

    assert sim.state.rna["reporter"] == pytest.approx(discrete, rel=1e-4)
    assert sim.state.rna["reporter"] < continuous


def test_halving_the_step_halves_the_discretisation_error() -> None:
    """First-order convergence, which is what a fixed Euler step should give.

    This is the test that would catch someone changing the integration without
    saying so. A scheme that is not first order, or one where the order of
    processes within a step changed, moves this number.
    """
    t = Transcription()
    d = Degradation()
    gene = reference_genome().find("reporter")
    assert gene is not None
    continuous = float(t.rate) * gene.promoter_strength / float(d.rna_rate)

    def settled(dt: float) -> float:
        sim = Simulator(reference_cell(), default_processes())
        sim.run(6000, dt=dt, record_every=10_000)
        return sim.state.rna["reporter"]

    coarse = abs(settled(1.0) - continuous)
    fine = abs(settled(0.5) - continuous)
    assert fine == pytest.approx(coarse / 2, rel=0.05), f"coarse {coarse}, fine {fine}"


def test_silencing_a_gene_removes_its_protein_slowly() -> None:
    """The protein outlives the message, which is why regulation has lag."""
    sim = Simulator(reference_cell(), default_processes())
    sim.run(600, dt=1.0)
    before = sim.state.protein["reporter"]
    assert before > 0

    gene = sim.state.genome.find("reporter")
    assert gene is not None
    gene.promoter_strength = 0.0

    before_rna = sim.state.rna["reporter"]
    sim.run(60, dt=1.0)

    # One minute is a little under two mRNA half-lives (ln 2 / 0.02 is 35 s), so
    # roughly 30 percent of the message remains. Over the same minute the
    # protein loses about 6 percent, because its half-life is 20 times longer.
    assert sim.state.rna["reporter"] < 0.35 * before_rna, "the message should be mostly gone"
    assert sim.state.protein["reporter"] > 0.9 * before, "the protein should still be there"


def test_the_reference_genome_translates_without_error() -> None:
    """Every invented gene must be a readable coding sequence.

    A gene whose sequence does not translate would be a broken example in front
    of a student on their first run.
    """
    for gene in reference_genome().genes:
        protein = gene.protein_sequence
        assert protein.startswith("M"), f"{gene.name} does not start with a start codon"
        assert "*" not in protein
        assert len(protein) > 5, f"{gene.name} is implausibly short: {protein}"


def test_nothing_in_the_engine_produces_a_non_finite_number() -> None:
    """A NaN in a count propagates everywhere and is invisible on a plot."""
    sim = Simulator(reference_cell(), default_processes())
    sim.run(2000, dt=1.0, record_every=200)
    for frame in sim.trajectory.rna + sim.trajectory.protein + sim.trajectory.metabolites:
        for species, value in frame.items():
            assert math.isfinite(value), f"{species} became {value}"
