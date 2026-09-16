"""A shape-and-chemistry matching puzzle, clearly labelled as a puzzle.

Real computational drug design uses molecular physics, large compute, and is
still weak. Nothing here approximates it. What this builds is the intuition that
a molecule binds because its shape and its chemistry complement a pocket, and
that getting both right is necessary and nowhere near sufficient.

# The marker is not removable

Every result this module produces carries `DomainVerdict.UNCALIBRATED` and the
word "puzzle". There is no flag to turn that off, no "expert mode", and no
scoring in units that resemble a real affinity. A number here is a count of
matched features in an invented scheme, and calling it anything else would make
this the exact thing subplan 33 forbids: something that could be mistaken for
real drug design.

# What is deliberately absent

No real drug structures. No real protein targets. No toxin or pathogen content.
That is curation rather than filtering, and it is the stronger control: a filter
guesses at intent and fails, while curation decides what exists. A student
cannot misuse content that is not here.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum


class Verdict(Enum):
    """How much a result from this module can be trusted.

    One member, on purpose. Every result is uncalibrated, and an enum with a
    single value makes that a type-level fact rather than a convention someone
    can quietly stop following.

    This mirrors `DomainVerdict::Uncalibrated` in the Rust core, which is what
    the design tool reports when a model has no calibration evidence. The
    sandbox never has any.
    """

    UNCALIBRATED = "uncalibrated"

    def __str__(self) -> str:
        return (
            "UNCALIBRATED: this is a teaching puzzle, not a prediction. No number "
            "here says anything about whether a molecule would work in a person."
        )


class Chemistry(Enum):
    """The chemical character of one part of a molecule or pocket.

    Four categories where real chemistry has a continuum. Enough to make
    complementarity a real constraint a student has to solve, and far too few to
    model anything.
    """

    HYDROPHOBIC = "hydrophobic"
    """Avoids water. Pairs with hydrophobic."""

    POSITIVE = "positive"
    """Positively charged. Pairs with negative."""

    NEGATIVE = "negative"
    """Negatively charged. Pairs with positive."""

    DONOR = "hydrogen bond donor"
    """Offers a hydrogen. Pairs with an acceptor."""

    ACCEPTOR = "hydrogen bond acceptor"
    """Accepts a hydrogen. Pairs with a donor."""


COMPLEMENTS: dict[Chemistry, Chemistry] = {
    Chemistry.HYDROPHOBIC: Chemistry.HYDROPHOBIC,
    Chemistry.POSITIVE: Chemistry.NEGATIVE,
    Chemistry.NEGATIVE: Chemistry.POSITIVE,
    Chemistry.DONOR: Chemistry.ACCEPTOR,
    Chemistry.ACCEPTOR: Chemistry.DONOR,
}
"""What pairs with what.

Hydrophobic pairs with itself, which surprises students and is the interesting
one: two greasy surfaces stick together because water pushes them together, not
because they attract each other.
"""


@dataclass(frozen=True, slots=True)
class Group:
    """One chemical group, at one position along a molecule.

    `position` is an integer slot, not a coordinate. Real molecules are
    three-dimensional and the geometry is most of the difficulty; reducing it to
    a line is the largest simplification in this module and it is why nothing
    here transfers to real work.
    """

    position: int
    chemistry: Chemistry
    label: str = ""


@dataclass(frozen=True, slots=True)
class Molecule:
    """A candidate, as a set of groups along a line."""

    name: str
    groups: tuple[Group, ...] = ()

    @property
    def size(self) -> int:
        """How many slots the molecule spans.

        Size carries a cost in the fit, because a large molecule that happens to
        match is not the same achievement as a small one that does. Real
        medicinal chemistry cares about this a great deal.
        """
        if not self.groups:
            return 0
        return max(g.position for g in self.groups) + 1

    def at(self, position: int) -> Group | None:
        return next((g for g in self.groups if g.position == position), None)

    def with_group(self, group: Group) -> Molecule:
        """A new molecule with `group` added or replacing the one at its slot.

        Returns a new molecule rather than mutating, so a student can compare a
        candidate with what it was before the change.
        """
        kept = tuple(g for g in self.groups if g.position != group.position)
        return Molecule(name=self.name, groups=(*kept, group))


@dataclass(frozen=True, slots=True)
class Pocket:
    """A binding site, as the groups lining it.

    Invented. No pocket here is taken from a real protein structure, and there
    is no route from a sandbox pocket to a real target.
    """

    name: str
    groups: tuple[Group, ...] = ()
    description: str = ""

    @property
    def size(self) -> int:
        if not self.groups:
            return 0
        return max(g.position for g in self.groups) + 1

    def at(self, position: int) -> Group | None:
        return next((g for g in self.groups if g.position == position), None)


@dataclass(frozen=True, slots=True)
class Fit:
    """How well a molecule complements a pocket, as a puzzle result.

    `matched`, `mismatched` and `empty` are counts of slots, not energies. There
    is deliberately no field called score, affinity, potency, or anything else
    that reads as a physical quantity.
    """

    molecule: str
    pocket: str
    matched: list[str] = field(default_factory=list)
    mismatched: list[str] = field(default_factory=list)
    empty: list[str] = field(default_factory=list)
    overhang: int = 0
    """Slots the molecule occupies that the pocket does not have.

    A molecule that sticks out of its pocket is a real failure mode, and one of
    the easier ones for a student to cause by adding groups until everything
    matches.
    """

    verdict: Verdict = Verdict.UNCALIBRATED

    @property
    def complementarity(self) -> float:
        """Matched slots over pocket slots, from 0 to 1.

        A puzzle completion fraction. Not a binding affinity, not a probability
        of anything, and not comparable to any published number.
        """
        total = len(self.matched) + len(self.mismatched) + len(self.empty)
        if total == 0:
            return 0.0
        return len(self.matched) / total

    def describe(self) -> str:
        lines = [
            f"{self.molecule} against {self.pocket}",
            f"  {len(self.matched)} of "
            f"{len(self.matched) + len(self.mismatched) + len(self.empty)} "
            f"pocket positions complemented",
        ]
        for line in self.matched:
            lines.append(f"    match     {line}")
        for line in self.mismatched:
            lines.append(f"    clash     {line}")
        for line in self.empty:
            lines.append(f"    unfilled  {line}")
        if self.overhang:
            lines.append(
                f"    {self.overhang} group(s) extend past the pocket, which in a "
                "real site would collide with the protein"
            )
        lines.append("")
        lines.append(str(self.verdict))
        return "\n".join(lines)


def fit(molecule: Molecule, pocket: Pocket) -> Fit:
    """Compares a molecule against a pocket, slot by slot.

    Deterministic, because it is a puzzle and a puzzle whose answer moves is not
    solvable. The uncertainty in this module lives in what happens *after* a
    good fit, which is where the real uncertainty lives too.
    """
    matched: list[str] = []
    mismatched: list[str] = []
    empty: list[str] = []

    for position in range(pocket.size):
        site = pocket.at(position)
        if site is None:
            continue
        group = molecule.at(position)
        where = f"position {position} ({site.chemistry.value} in the pocket)"

        if group is None:
            empty.append(f"{where}: nothing there")
        elif COMPLEMENTS[site.chemistry] is group.chemistry:
            matched.append(f"{where}: {group.chemistry.value} complements it")
        else:
            mismatched.append(
                f"{where}: {group.chemistry.value} does not complement {site.chemistry.value}"
            )

    overhang = sum(1 for g in molecule.groups if g.position >= pocket.size)
    return Fit(
        molecule=molecule.name,
        pocket=pocket.name,
        matched=matched,
        mismatched=mismatched,
        empty=empty,
        overhang=overhang,
    )
