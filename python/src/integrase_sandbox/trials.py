"""Why a molecule that fits still does not become a medicine.

This is the part of the drug design lesson that matters, and it is the part
most teaching tools leave out. A student who finishes a matching puzzle and is
told "you cured it" has learned something false. A student who finishes the
puzzle and then watches the candidate fail in Phase II for lack of efficacy has
learned what solving a medical problem actually requires.

# The numbers here are real, and they are the curriculum

About 90 percent of drug candidates entering clinical trials fail. Phase I
passes roughly 52 percent, Phase II drops to 28.9 percent, and 8.1 percent
reach approval. Over half of late-stage failures are lack of efficacy, and
another 17 percent are adverse effects. Twelve or more years and over 2.5
billion dollars per approved drug.

Source: https://www.mdpi.com/1424-8247/19/6/916

Those are the only measured numbers in this sandbox. Everything else is
illustrative, and the contrast is deliberate: the failure rates are real
because the failure rates are the lesson.

# Discovery is not a goal here, and saying so matters

A simulator contains only what was put into it. A student who "cures" something
inside it has found a solution someone programmed, which is a fine puzzle and is
not discovery. Presenting it as discovery would teach a false idea of what
medical research requires, so nothing in this module congratulates anyone for
finding a cure.
"""

from __future__ import annotations

import random
from dataclasses import dataclass
from enum import Enum

from .molecules import Fit, Verdict
from .parameters import Parameter, Provenance

PHASE_SOURCE = (
    "Clinical trial success rates, as reviewed in Pharmaceuticals 2026, 19(6):916, "
    "https://www.mdpi.com/1424-8247/19/6/916"
)


class Phase(Enum):
    """The stages a candidate has to pass."""

    PRECLINICAL = "preclinical"
    PHASE_I = "Phase I"
    PHASE_II = "Phase II"
    PHASE_III = "Phase III"
    APPROVED = "approved"

    @property
    def label(self) -> str:
        """The phase name as it should be written.

        Roman numerals do not survive `str.title()`, which turns "Phase II" into
        "Phase Ii". The values here are already correctly cased, so this exists
        to make that explicit rather than to transform anything.
        """
        return self.value


PASS_RATE: dict[Phase, Parameter] = {
    Phase.PRECLINICAL: Parameter(
        value=0.60,
        unit="fraction",
        provenance=Provenance.DERIVED,
        source=(
            "approximate, derived from the gap between candidates entering "
            "preclinical work and those reaching Phase I; see " + PHASE_SOURCE
        ),
        relative_error=0.25,
    ),
    Phase.PHASE_I: Parameter(
        value=0.52,
        unit="fraction",
        provenance=Provenance.MEASURED,
        source=PHASE_SOURCE,
        relative_error=0.10,
    ),
    Phase.PHASE_II: Parameter(
        value=0.289,
        unit="fraction",
        provenance=Provenance.MEASURED,
        source=PHASE_SOURCE,
        relative_error=0.10,
    ),
    Phase.PHASE_III: Parameter(
        value=0.58,
        unit="fraction",
        provenance=Provenance.MEASURED,
        source=PHASE_SOURCE,
        relative_error=0.10,
    ),
}
"""Probability of passing each phase, given the candidate reached it.

Phase II is the cliff. It is where a drug first has to show it helps patients
rather than merely being tolerated, and it is where most good chemistry dies.
"""


@dataclass(frozen=True, slots=True)
class Failure:
    """Why a candidate stopped, in the words a student needs to hear."""

    phase: Phase
    reason: str
    explanation: str


FAILURE_MODES: dict[Phase, tuple[Failure, ...]] = {
    Phase.PRECLINICAL: (
        Failure(
            Phase.PRECLINICAL,
            "does not reach the target",
            "The molecule works in a dish and never gets into the cell. "
            "Crossing a membrane is a separate problem from binding, and "
            "solving one does nothing for the other.",
        ),
        Failure(
            Phase.PRECLINICAL,
            "cleared too quickly",
            "The liver removes it within minutes. A molecule that binds "
            "perfectly and is gone before it arrives has no effect.",
        ),
        Failure(
            Phase.PRECLINICAL,
            "toxic in animals",
            "Harm appeared at a dose below the one that would help.",
        ),
    ),
    Phase.PHASE_I: (
        Failure(
            Phase.PHASE_I,
            "adverse effects in healthy volunteers",
            "Phase I asks whether people can tolerate it at all. Roughly 17 "
            "percent of all failures are safety, and this is where most of "
            "them surface.",
        ),
        Failure(
            Phase.PHASE_I,
            "unpredictable absorption",
            "Blood levels varied so much between people that no safe dose could be chosen.",
        ),
    ),
    Phase.PHASE_II: (
        Failure(
            Phase.PHASE_II,
            "no measurable benefit to patients",
            "The biggest single reason drugs fail. Over half of late-stage "
            "failures are lack of efficacy. The target was hit and the "
            "patients did not get better, which usually means the target was "
            "the wrong one.",
        ),
        Failure(
            Phase.PHASE_II,
            "off-target effects",
            "The molecule bound something else as well. A pocket that looks "
            "distinctive in isolation often is not, across twenty thousand "
            "human proteins.",
        ),
    ),
    Phase.PHASE_III: (
        Failure(
            Phase.PHASE_III,
            "benefit did not hold in a larger group",
            "An effect seen in a few hundred people did not survive a few "
            "thousand. This failure costs the most, because Phase III is where "
            "most of the money goes.",
        ),
        Failure(
            Phase.PHASE_III,
            "rare harm appeared at scale",
            "A one-in-two-thousand adverse event is invisible in Phase II and "
            "unacceptable in a marketed drug.",
        ),
    ),
}


@dataclass(frozen=True, slots=True)
class Result:
    """What happened to a candidate, and what it means.

    Carries the same uncalibrated verdict as everything else in this sandbox. A
    candidate that reaches `APPROVED` here has passed a sequence of coin flips
    weighted by real published rates. It has not been shown to work.
    """

    candidate: str
    reached: Phase
    failure: Failure | None
    history: list[str]
    verdict: Verdict = Verdict.UNCALIBRATED

    @property
    def approved(self) -> bool:
        return self.reached is Phase.APPROVED

    def describe(self) -> str:
        lines = [f"{self.candidate}:"]
        lines.extend(f"  {line}" for line in self.history)
        lines.append("")
        if self.failure is not None:
            lines.append(f"Stopped at {self.failure.phase.label}: {self.failure.reason}")
            lines.append(f"  {self.failure.explanation}")
        else:
            lines.append(
                "Reached approval. In the model, and after a sequence of weighted "
                "coin flips. This is not a discovery, and nothing here says the "
                "molecule would help anyone."
            )
        lines.append("")
        lines.append(str(self.verdict))
        return "\n".join(lines)


def develop(candidate: Fit, seed: int | None = None) -> Result:
    """Walks a candidate through development, and usually kills it.

    A good fit improves the odds and does not rescue them, which is the point.
    The complementarity of the puzzle shifts the preclinical pass rate only:
    chemistry is what preclinical work tests, and it is not what Phase II
    tests. A perfect puzzle solution still faces the 28.9 percent Phase II
    rate unchanged, because whether a target is the right target is not a
    question about the molecule.

    `seed` makes a run reproducible, which matters for a lesson: a teacher
    needs to be able to hand a class the same story.
    """
    rng = random.Random(seed)
    history: list[str] = [
        f"Puzzle complementarity {candidate.complementarity:.0%} "
        f"({len(candidate.matched)} matched, {len(candidate.mismatched)} clashing, "
        f"{len(candidate.empty)} unfilled)"
    ]

    for phase in (Phase.PRECLINICAL, Phase.PHASE_I, Phase.PHASE_II, Phase.PHASE_III):
        rate = float(PASS_RATE[phase])

        if phase is Phase.PRECLINICAL:
            # Chemistry is what preclinical work tests, so this is the one place
            # the puzzle result changes anything. A clash or an overhang is a
            # real structural problem and costs more than an unfilled slot.
            penalty = 0.10 * len(candidate.mismatched) + 0.05 * candidate.overhang
            rate = max(0.05, min(0.95, rate * (0.5 + candidate.complementarity) - penalty))
            history.append(f"Preclinical pass chance adjusted to {rate:.0%} by the fit")
        else:
            history.append(f"{phase.label} pass rate {rate:.1%}, unchanged by the fit")

        if rng.random() >= rate:
            failure = rng.choice(FAILURE_MODES[phase])
            history.append(f"Failed {phase.label}")
            return Result(
                candidate=candidate.molecule,
                reached=phase,
                failure=failure,
                history=history,
            )
        history.append(f"Passed {phase.label}")

    return Result(
        candidate=candidate.molecule,
        reached=Phase.APPROVED,
        failure=None,
        history=history,
    )


def overall_success_rate() -> float:
    """The product of the three clinical phase rates.

    8.7 percent, against the 8.1 percent reported for candidates entering Phase
    I. The small gap is that the published figure aggregates across therapeutic
    areas and years while this multiplies three point estimates, and it is a
    reasonable check that the numbers were transcribed correctly.

    Preclinical attrition is deliberately outside this. The published rate
    counts candidates from Phase I onward, and folding preclinical into the same
    number would make it incomparable to the source.
    """
    product = 1.0
    for phase in (Phase.PHASE_I, Phase.PHASE_II, Phase.PHASE_III):
        product *= float(PASS_RATE[phase])
    return product
