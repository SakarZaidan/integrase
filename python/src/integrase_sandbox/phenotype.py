"""From genotype to what you can observe, with the uncertainty kept.

A trait takes a genome and an environment and returns a distribution over
outcomes, never a single answer. That is not a hedge. For most traits a single
answer would be false, and the ones where it is nearly true are the minority.

# Why this module refuses to be certain

The model of inheritance taught in most schools is Mendelian: one gene, two
alleles, dominant beats recessive, outcome determined. It is true for a small
number of conditions and false for almost everything a student cares about.
Height, eye colour, diabetes risk, most cancers: many genes, each contributing
a little, plus environment.

A sandbox that reported "blue eyes" would be teaching the false model with
better graphics. This one reports shifted odds, and shows what shifted them.
That is harder to build and it is the whole reason the project's uncertainty
machinery exists.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import Protocol, runtime_checkable

from .genome import Genome
from .parameters import Parameter
from .state import Environment


@dataclass(frozen=True, slots=True)
class Outcome:
    """A distribution over what a trait might be, and why.

    Probabilities rather than a value. `explanation` is not decoration: a
    student who cannot see which changes moved the numbers has learned that the
    sandbox is an oracle, which is the opposite of the intended lesson.
    """

    trait: str
    """What is being predicted, in plain words."""

    probabilities: dict[str, float]
    """Possible values and their probabilities. Sums to 1."""

    explanation: list[str] = field(default_factory=list)
    """What contributed, one line each, in the order they were applied."""

    certainty: str = ""
    """How much this outcome should be trusted, in words rather than a number.

    Deliberately prose. A number here would be a confidence about a confidence,
    and the sandbox has no basis for one.
    """

    def __post_init__(self) -> None:
        if not self.probabilities:
            msg = "an outcome needs at least one possible value"
            raise ValueError(msg)
        total = sum(self.probabilities.values())
        if abs(total - 1.0) > 1e-6:
            msg = f"probabilities must sum to 1, got {total:g}"
            raise ValueError(msg)
        if any(p < 0 for p in self.probabilities.values()):
            msg = "probabilities must not be negative"
            raise ValueError(msg)

    @property
    def most_likely(self) -> str:
        """The highest-probability value.

        Available, and deliberately not what `describe` leads with. Reducing a
        distribution to its mode is exactly the move this module exists to
        discourage, and a student who wants the single answer should have to ask
        for it by name.
        """
        return max(self.probabilities.items(), key=lambda kv: kv[1])[0]

    def describe(self) -> str:
        """The full distribution and its reasoning, as text.

        Leads with the distribution rather than the mode, because the shape of
        the distribution is the result and the mode is a summary of it.
        """
        lines = [f"{self.trait}:"]
        for value, p in sorted(self.probabilities.items(), key=lambda kv: kv[1], reverse=True):
            lines.append(f"  {p:6.1%}  {value}")
        if self.explanation:
            lines.append("")
            lines.append("What went into this:")
            lines.extend(f"  - {line}" for line in self.explanation)
        if self.certainty:
            lines.append("")
            lines.append(self.certainty)
        return "\n".join(lines)


@runtime_checkable
class Trait(Protocol):
    """Something observable that a genome and an environment influence.

    A `Protocol`, like `Process`, and for the same reason: a student adding a
    trait writes a class with an `evaluate` method and nothing else.
    """

    @property
    def name(self) -> str:
        """What this trait is called."""
        ...

    def evaluate(self, genome: Genome, environment: Environment) -> Outcome:
        """Returns the distribution this genome and environment imply."""
        ...


@dataclass(frozen=True, slots=True)
class Locus:
    """One position in one gene that a trait depends on.

    `position` is an index into the gene's coding sequence, zero-based, so it
    lines up with `Sequence.mutate`. Real variant coordinates are given against
    a reference genome and are one-based; the difference is a genuine source of
    error in real work and is worth a student knowing about.
    """

    gene: str
    position: int
    reference_base: str
    """The base the reference carries here."""

    def base_in(self, genome: Genome) -> str | None:
        """The base this genome carries at this locus, or `None` if absent."""
        gene = genome.find(self.gene)
        if gene is None or not 0 <= self.position < len(gene.sequence):
            return None
        return gene.sequence[self.position]

    def is_reference(self, genome: Genome) -> bool:
        """Whether this genome matches the reference here."""
        return self.base_in(genome) == self.reference_base


@dataclass(frozen=True, slots=True)
class Contribution:
    """One locus's effect on the odds of a trait value.

    `log_odds` rather than a probability, because effects combine by addition on
    the log-odds scale and by nothing simple on the probability scale. Adding
    probabilities would let two moderate effects sum past 1, and multiplying
    them would make every additional gene reduce the odds.
    """

    locus: Locus
    variant_base: str
    """The base that carries this effect."""

    log_odds: Parameter
    """Shift in log odds when `variant_base` is present."""

    affects: str
    """Which outcome value this shifts toward."""

    note: str = ""
    """What is known about the mechanism, if anything."""


class PolygenicTrait:
    """A trait several loci contribute to, reported as shifted odds.

    The baseline is the population distribution before any genotype is
    considered. Each contributing variant shifts it. The result never reaches 0
    or 1, because no combination of the loci modelled here determines an outcome
    and reporting certainty would be the specific falsehood this class exists to
    avoid.
    """

    __slots__ = ("_baseline", "_contributions", "_label", "_missing_genes", "_note")

    def __init__(
        self,
        name: str,
        baseline: dict[str, float],
        contributions: list[Contribution],
        note: str = "",
        missing_genes: str = "",
    ) -> None:
        total = sum(baseline.values())
        if abs(total - 1.0) > 1e-6:
            msg = f"baseline probabilities must sum to 1, got {total:g}"
            raise ValueError(msg)
        self._label = name
        self._baseline = dict(baseline)
        self._contributions = list(contributions)
        self._note = note
        self._missing_genes = missing_genes

    @property
    def name(self) -> str:
        return self._label

    def evaluate(self, genome: Genome, environment: Environment) -> Outcome:
        """Shifts the baseline by whichever variants this genome carries."""
        del environment  # No environmental term modelled for these traits yet.

        weights = {k: _to_log_weight(v) for k, v in self._baseline.items()}
        baseline = ", ".join(f"{v:.0%} {k}" for k, v in sorted(self._baseline.items()))
        explanation: list[str] = [f"Starting from the population baseline: {baseline}"]

        for c in self._contributions:
            carried = c.locus.base_in(genome)
            if carried is None:
                explanation.append(
                    f"{c.locus.gene} is not in this genome, so its contribution "
                    "could not be applied"
                )
                continue
            if carried != c.variant_base:
                continue
            if c.affects not in weights:
                continue

            weights[c.affects] += float(c.log_odds)
            detail = f" ({c.note})" if c.note else ""
            explanation.append(
                f"{c.locus.gene} position {c.locus.position} carries "
                f"{c.variant_base}, shifting {c.affects} by "
                f"{float(c.log_odds):+.2f} log odds{detail}"
            )

        probabilities = _normalise(weights)

        certainty = self._note or ""
        if self._missing_genes:
            certainty = f"{certainty}\n\n{self._missing_genes}".strip()

        return Outcome(
            trait=self._label,
            probabilities=probabilities,
            explanation=explanation,
            certainty=certainty,
        )


class MonogenicTrait:
    """A trait one change in one gene is sufficient to cause.

    The minority case, and the one school biology generalises from. Reported
    with high confidence because that confidence is earned here, and the class
    docstring on `PolygenicTrait` explains why it is not earned elsewhere.

    Even here the outcome is a distribution rather than a certainty. Penetrance
    is rarely 100 percent, and a condition that always follows from a genotype
    is rarer still than the conditions that usually do.
    """

    __slots__ = (
        "_absent",
        "_caveat",
        "_label",
        "_locus",
        "_penetrance",
        "_present",
        "_variant",
        "_why",
    )

    def __init__(
        self,
        name: str,
        locus: Locus,
        variant_base: str,
        present: str,
        absent: str,
        penetrance: Parameter,
        why: str = "",
        caveat: str = "",
    ) -> None:
        self._label = name
        self._locus = locus
        self._variant = variant_base
        self._present = present
        self._absent = absent
        self._penetrance = penetrance
        self._why = why
        self._caveat = caveat

    @property
    def name(self) -> str:
        return self._label

    def evaluate(self, genome: Genome, environment: Environment) -> Outcome:
        del environment

        carried = self._locus.base_in(genome)
        if carried is None:
            return Outcome(
                trait=self._label,
                probabilities={"cannot be assessed": 1.0},
                explanation=[
                    f"{self._locus.gene} is not present in this genome, so this "
                    "trait cannot be evaluated"
                ],
                certainty="No result. This is not the same as a negative result.",
            )

        if carried != self._variant:
            return Outcome(
                trait=self._label,
                probabilities={self._absent: 1.0},
                explanation=[
                    f"{self._locus.gene} position {self._locus.position} carries "
                    f"{carried}, which is the reference base"
                ],
                certainty=(
                    "This locus is unaffected. Other causes of the same condition "
                    "are not modelled here, so this is a statement about one "
                    "position and not a clean bill of health."
                ),
            )

        p = float(self._penetrance)
        explanation = [
            f"{self._locus.gene} position {self._locus.position} carries "
            f"{self._variant} instead of {self._locus.reference_base}"
        ]
        if self._why:
            explanation.append(self._why)
        explanation.append(
            f"Penetrance is {p:.0%}, meaning that fraction of people carrying "
            f"this variant show the trait. {self._penetrance.describe()}"
        )

        return Outcome(
            trait=self._label,
            probabilities={self._present: p, self._absent: 1.0 - p},
            explanation=explanation,
            certainty="\n\n".join(
                part
                for part in (
                    "One gene, one change, and a mechanism that can be traced "
                    "from the base to the protein to the cell. Most traits are "
                    "not like this, which is why this one is worth studying and "
                    "worth not generalising from.",
                    self._caveat,
                )
                if part
            ),
        )


def _to_log_weight(p: float) -> float:
    """Probability to an additive weight, clamped away from zero.

    `log(p)`, not `log(p / (1 - p))`. That distinction is the difference between
    a model that reproduces its own baseline and one that does not.

    With log odds and a softmax, a genome carrying no variants at all came back
    with the baseline distorted: 55/30/15 became 67/23/10. Odds are a two-class
    idea, and converting each of three mutually exclusive classes to odds
    against "everything else" and then renormalising is not the identity.

    Working in log probability is. Adding an effect to one class's weight and
    renormalising multiplies that class's odds against every other class by
    `exp(effect)`, so with two outcomes it is exactly a log-odds shift, which is
    how effect sizes are reported in the literature the numbers come from.

    The clamp is doing real work. A baseline of exactly zero has a log weight of
    negative infinity, which no finite effect can move, so "rare in this
    population" would silently become "impossible" and every rare-disease lesson
    built on it would be wrong.
    """
    return math.log(max(p, 1e-9))


def _normalise(log_weights: dict[str, float]) -> dict[str, float]:
    """Additive weights back to a distribution, by softmax.

    The largest weight is subtracted before exponentiating. Without that, a
    large effect size overflows to infinity and every probability becomes NaN,
    which is invisible on a chart.
    """
    largest = max(log_weights.values())
    weights = {k: math.exp(v - largest) for k, v in log_weights.items()}
    total = sum(weights.values())
    return {k: w / total for k, w in weights.items()}
