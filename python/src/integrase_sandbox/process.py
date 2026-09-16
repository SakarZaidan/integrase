"""What a biological process is, and how one is added.

A `Protocol` rather than an abstract base class. A student writing a new process
should not have to inherit from anything or call `super().__init__()`. They
write a class with a `step` method and add it to the list. That is the single
best exercise this sandbox offers, and a base class would put a barrier in front
of it.

Inheritance appears in exactly one place in this package, where it is genuinely
an is-a relationship: concrete processes sharing small numerical helpers.
"""

from __future__ import annotations

from typing import Protocol, runtime_checkable

from .state import CellState


@runtime_checkable
class Process(Protocol):
    """One thing that happens to a cell over time.

    Implement `step` and you have a process. There is nothing to inherit and
    nothing to register.

    `runtime_checkable` so the simulator can give a clear error when something
    without a `step` method is added to the process list, rather than failing
    with an `AttributeError` partway through the first step.
    """

    @property
    def name(self) -> str:
        """Short label, for trajectory output and error messages."""
        ...

    def step(self, state: CellState, dt: float) -> None:
        """Advances this process by `dt` seconds, modifying `state` in place.

        Called once per simulator step, in the order the processes were listed.

        A process reads whatever it needs from the state and writes whatever it
        produces. It does not return anything, and it does not know which other
        processes exist. That is what lets a student add one without reading the
        rest.
        """
        ...
