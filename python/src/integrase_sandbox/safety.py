"""What exists in this sandbox, and why that is the control.

The requested constraint was to restrict the sandbox to beneficial uses. That is
not implementable, and the reason is worth writing down rather than attempting.

**Intent is not in the data.** The edit that disables a harmful gene is the same
edit, reversed, that enables it. The software sees identical letters.

**The same knowledge cures and harms.** Understanding how a pathogen causes
disease is exactly what is required to counter it. This is the dual-use problem.
It is permanent, and no filter resolves it.

Any intent filter would be either trivially bypassable or so restrictive that it
blocks legitimate learning. Both failures are worse than useless, because both
leave someone believing a control is in place.

# What works instead

Curation. The sandbox contains the genes, organisms, and molecules that were put
in it, and nothing else. There is no pathogen sequence, no toxin gene, and no
real orderable construct, because none was written. A student cannot misuse
content that is absent.

**This is stronger than a filter, not weaker.** A filter guesses at intent and
fails. Curation decides what exists.

# The boundary this module states, and the tests that hold it

Four claims, each checkable rather than asserted:

1. Every sequence in the sandbox is invented, except the thirteen-codon HBB
   fragment used for the sickle cell lesson, which is a documented human
   sequence carrying no capability of any kind.
2. The sandbox has no path to synthesis. It imports nothing from `integrase`,
   so the screening gate in `integrase-screen` is not weakened by its existence;
   it is simply unreachable from here.
3. Real conditions appear as read-only worked examples on invented or fragment
   sequence, never as editable pathogen material.
4. Nothing in the sandbox produces a result that claims to be a prediction.
"""

from __future__ import annotations

from dataclasses import dataclass

CURATION_STATEMENT = """The sandbox contains only what was put in it.

There is no pathogen sequence here, no toxin gene, and no real orderable
construct. That is a content decision, not a detection problem, and it is the
reason this sandbox does not try to guess what anyone intends.

Nothing here can become physical DNA. The sandbox has no export path at all, so
the biosecurity screening the design tool routes through is not weakened by it.
"""


@dataclass(frozen=True, slots=True)
class Provenance:
    """Where one piece of sandbox content came from.

    Every gene the sandbox ships has an entry. An entry is not paperwork: it is
    what makes claim 1 above checkable by reading rather than by trusting.
    """

    name: str
    invented: bool
    """Whether the sequence was made up for this sandbox."""

    note: str


CONTENT: tuple[Provenance, ...] = (
    Provenance(
        "reporter",
        invented=True,
        note="made up; a short coding sequence for the simulation lessons",
    ),
    Provenance("regulator", invented=True, note="made up"),
    Provenance("enzyme", invented=True, note="made up"),
    Provenance("essential", invented=True, note="made up"),
    Provenance(
        "HBB",
        invented=False,
        note=(
            "the first thirteen codons of the human haemoglobin beta coding "
            "sequence, with a stop appended. A fragment of a human structural "
            "protein, carrying no capability, used because the sickle cell "
            "lesson depends on the variant sitting at its real position"
        ),
    ),
    Provenance(
        "HERC2",
        invented=True,
        note="made up; a stand-in for the eye colour lesson, not the real gene",
    ),
    Provenance("OCA2", invented=True, note="made up; a stand-in, not the real gene"),
    Provenance("TYR", invented=True, note="made up; a stand-in, not the real gene"),
)
"""Every gene the sandbox ships, and where its sequence came from.

A test asserts this list covers every gene actually reachable through the
shipped genomes. Content that is in the sandbox and not in this list would be
content nobody decided to put there.
"""


def describe_content() -> str:
    """The curation statement and the content list, as text.

    Meant to be readable by a teacher deciding whether to put this in front of a
    class, without reading the source.
    """
    lines = [CURATION_STATEMENT, "Content in this sandbox:"]
    for item in CONTENT:
        origin = "invented" if item.invented else "real fragment"
        lines.append(f"  {item.name:12s} {origin:15s} {item.note}")
    return "\n".join(lines)
