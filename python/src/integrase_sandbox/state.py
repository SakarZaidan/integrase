"""The shared state every process reads and writes.

One state object, many independent processes that modify it, and a scheduler
that steps them. That is the Karr whole-cell pattern, generalised: 28 submodels
sharing a cell, at far lower fidelity.

The state is a blackboard rather than an owner. No process owns transcription;
transcription is a thing that happens to the state. That separation is what
lets a student add a process without touching any existing one.
"""

from __future__ import annotations

from collections.abc import Iterator
from dataclasses import dataclass, field

from .genome import Genome


class Counts:
    """Molecule species to copy number.

    Counts, not individual molecules. A real cell holds billions, and one entry
    with a number is how every real simulator works. It is the difference
    between this executing and not.

    A missing species reads as zero rather than raising. A cell that has not
    made a protein yet has none of it, which is not an error, and forcing every
    process to check first would put a `.get(name, 0)` on every line of the
    interesting code.
    """

    __slots__ = ("_counts",)

    def __init__(self, initial: dict[str, float] | None = None) -> None:
        self._counts: dict[str, float] = dict(initial or {})

    def __getitem__(self, species: str) -> float:
        return self._counts.get(species, 0.0)

    def __setitem__(self, species: str, value: float) -> None:
        # Clamped at zero rather than allowed negative. A negative molecule
        # count is not a small numerical artefact to tolerate; it is a sign that
        # a process removed more than existed, and every downstream number
        # computed from it would be meaningless.
        self._counts[species] = max(0.0, value)

    def __contains__(self, species: str) -> bool:
        return species in self._counts

    def __iter__(self) -> Iterator[str]:
        return iter(self._counts)

    def __len__(self) -> int:
        return len(self._counts)

    def __repr__(self) -> str:
        shown = ", ".join(f"{k}={v:g}" for k, v in sorted(self._counts.items())[:6])
        more = "" if len(self._counts) <= 6 else f", ... {len(self._counts) - 6} more"
        return f"Counts({shown}{more})"

    def add(self, species: str, amount: float) -> None:
        """Adds to a count, clamping at zero."""
        self[species] = self[species] + amount

    def items(self) -> Iterator[tuple[str, float]]:
        """Species and counts, in insertion order."""
        return iter(self._counts.items())

    def snapshot(self) -> dict[str, float]:
        """A plain dict copy, for recording a trajectory."""
        return dict(self._counts)


@dataclass(slots=True)
class Environment:
    """Conditions outside the cell.

    Deliberately small. Every field here is one a lesson actually changes, and
    an environment with forty knobs is one a student cannot reason about.
    """

    temperature_c: float = 37.0
    """Degrees Celsius."""

    nutrients: float = 1.0
    """Nutrient availability, 0 to 1, where 1 is unlimited.

    A single number standing in for every carbon and nitrogen source. Crude, and
    crude in a way a student should be told about rather than left to infer.
    """

    inducers: Counts = field(default_factory=Counts)
    """Small molecules a student can add, by name and concentration."""

    def __post_init__(self) -> None:
        if not 0.0 <= self.nutrients <= 1.0:
            msg = f"nutrients must be between 0 and 1, got {self.nutrients}"
            raise ValueError(msg)


@dataclass(slots=True)
class CellState:
    """Everything about one cell at one instant.

    Mutable, because processes modify it in place and copying the whole state
    once per process per step would dominate the runtime of a simulation whose
    entire point is to run interactively.
    """

    genome: Genome
    rna: Counts = field(default_factory=Counts)
    protein: Counts = field(default_factory=Counts)
    metabolites: Counts = field(default_factory=Counts)
    environment: Environment = field(default_factory=Environment)
    time: float = 0.0
    """Simulated seconds since the run began."""

    alive: bool = True
    """Whether this cell is still viable.

    A process may set this false. Nothing resurrects a cell, and the simulator
    stops stepping one that has died rather than continuing to compute chemistry
    for it.
    """

    death_reason: str = ""
    """Why the cell died, in plain words, for the student who is about to ask."""

    def die(self, reason: str) -> None:
        """Marks the cell dead, keeping the first reason given.

        The first cause is the interesting one. A cell that has run out of
        energy will then fail several other checks, and reporting the last of
        those would point a student at a symptom.
        """
        if self.alive:
            self.alive = False
            self.death_reason = reason
