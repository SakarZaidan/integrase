"""The scheduler, and the record of what happened.

Steps each process once per tick, in list order, over one shared state. That is
the whole scheduler. Its simplicity is the design: anything cleverer would make
the order of effects harder to reason about, and a student debugging their own
process needs to be able to say exactly when it ran.

# Why fixed steps and not an adaptive solver

An adaptive integrator would be more accurate and would make the timeline
unpredictable. A student watching a simulation needs "one step is one second" to
stay true, and the error from a fixed step at this fidelity is far smaller than
the error already introduced by the parameters.
"""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass, field

from .process import Process
from .state import CellState


@dataclass(slots=True)
class Trajectory:
    """What the state was, at each recorded instant.

    Recorded as plain dicts rather than references to the live state, because
    the state is mutated in place and storing references would give every
    recorded point the same final values.
    """

    times: list[float] = field(default_factory=list)
    rna: list[dict[str, float]] = field(default_factory=list)
    protein: list[dict[str, float]] = field(default_factory=list)
    metabolites: list[dict[str, float]] = field(default_factory=list)

    def record(self, state: CellState) -> None:
        """Appends the current state."""
        self.times.append(state.time)
        self.rna.append(state.rna.snapshot())
        self.protein.append(state.protein.snapshot())
        self.metabolites.append(state.metabolites.snapshot())

    def series(self, pool: str, species: str) -> list[float]:
        """One species over time, from `"rna"`, `"protein"` or `"metabolites"`.

        Missing values read as zero, so a species that appeared partway through
        a run gives a series starting at zero rather than one that is shorter
        than `times` and cannot be plotted against it.
        """
        source = {
            "rna": self.rna,
            "protein": self.protein,
            "metabolites": self.metabolites,
        }.get(pool)
        if source is None:
            msg = f"unknown pool {pool!r}; expected 'rna', 'protein' or 'metabolites'"
            raise KeyError(msg)
        return [frame.get(species, 0.0) for frame in source]

    def __len__(self) -> int:
        return len(self.times)


class Simulator:
    """Runs processes over a cell state.

    Processes run in the order given. That order is visible and fixed rather
    than sorted by some internal priority, because a student asking "why did my
    process see zero protein" needs the answer to be "it ran before
    translation", which they can see in the list they wrote.
    """

    __slots__ = ("processes", "state", "trajectory")

    def __init__(self, state: CellState, processes: Iterable[Process]) -> None:
        # Checked at runtime as well as statically. The people most likely to
        # pass something without a `step` method are the ones writing their
        # first process, and they are not running a type checker. A clear error
        # here beats an AttributeError three frames into the first step.
        checked: list[Process] = []
        for i, candidate in enumerate(processes):
            as_object: object = candidate
            if not isinstance(as_object, Process):
                msg = (
                    f"process {i} ({type(as_object).__name__}) is missing a step method. "
                    "A process needs `def step(self, state, dt) -> None` and a "
                    "`name` property. It does not need to inherit from anything."
                )
                raise TypeError(msg)
            checked.append(candidate)

        self.state = state
        self.processes = checked
        self.trajectory = Trajectory()

    def step(self, dt: float) -> None:
        """Advances every process by `dt` seconds.

        Stops early if a process kills the cell, so that the remaining processes
        do not compute chemistry for something that is no longer alive.
        """
        if dt <= 0:
            msg = f"dt must be positive, got {dt}"
            raise ValueError(msg)
        # Read into a local rather than testing the attribute directly. A type
        # checker narrows `self.state.alive` to True after this guard, then
        # calls the break below unreachable, because a `step` returning None
        # gives it no reason to believe the flag can change. It can: killing the
        # cell is exactly what a process does through the state.
        alive = self.state.alive
        if not alive:
            return

        for p in self.processes:
            p.step(self.state, dt)
            if not self.state.alive:
                break
        self.state.time += dt

    def run(self, duration: float, dt: float = 1.0, record_every: int = 1) -> Trajectory:
        """Steps until `duration` seconds have passed, recording as it goes.

        Returns the trajectory, which is also kept on the simulator. Recording
        every step for a long run produces more points than any plot can use, so
        `record_every` thins it.

        A run that ends because the cell died stops there. The trajectory then
        ends early, which is the honest record: there is nothing after death to
        report, and padding it with the final values would show a flat line that
        looks like a steady state.
        """
        if duration <= 0:
            msg = f"duration must be positive, got {duration}"
            raise ValueError(msg)
        if record_every < 1:
            msg = f"record_every must be at least 1, got {record_every}"
            raise ValueError(msg)

        if not self.trajectory.times:
            self.trajectory.record(self.state)

        steps = int(duration / dt)
        for i in range(steps):
            self.step(dt)
            if (i + 1) % record_every == 0 or not self.state.alive:
                self.trajectory.record(self.state)
            if not self.state.alive:
                break
        return self.trajectory

    def add(self, process: Process) -> None:
        """Appends a process, so it runs last in each step."""
        as_object: object = process
        if not isinstance(as_object, Process):
            msg = f"{type(as_object).__name__} is missing a step method"
            raise TypeError(msg)
        self.processes.append(process)
