"""Amino acid properties, so a substitution can be explained rather than asserted.

A sandbox that says "this mutation causes sickle cell anaemia" has taught a fact.
One that says "glutamate is negatively charged and strongly hydrophilic, valine
is uncharged and strongly hydrophobic, and putting a hydrophobic residue on the
surface of a protein makes it stick to its neighbours" has taught a mechanism.

The second is the point of having this file.

# Where the numbers come from

Hydropathy is the Kyte and Doolittle scale, which is a real published index with
a real range. Charge is the formal charge of the side chain at pH 7. Neither is
invented, and both are sourced, because these are the values a student might go
and check.
"""

from __future__ import annotations

from dataclasses import dataclass

KYTE_DOOLITTLE_SOURCE = (
    "Kyte J, Doolittle RF (1982), A simple method for displaying the hydropathic "
    "character of a protein, J Mol Biol 157:105-132"
)


@dataclass(frozen=True, slots=True)
class AminoAcid:
    """One residue, and the properties that explain what it does in a protein."""

    letter: str
    """Single-letter code."""

    three: str
    """Three-letter code, which is what most textbooks use."""

    name: str

    hydropathy: float
    """Kyte and Doolittle index. Positive is hydrophobic, negative hydrophilic.

    The scale runs from -4.5 for arginine to 4.5 for isoleucine.
    """

    charge: int
    """Formal charge of the side chain at pH 7. One of -1, 0 or +1."""

    @property
    def is_hydrophobic(self) -> bool:
        """Whether the side chain avoids water.

        The threshold is zero on the Kyte and Doolittle scale, which is the
        conventional split and not a sharp physical boundary. A residue near
        zero is ambivalent, and calling it one or the other is a simplification.
        """
        return self.hydropathy > 0

    @property
    def is_charged(self) -> bool:
        return self.charge != 0

    def describe(self) -> str:
        """One clause, for an explanation of a substitution."""
        charge = {(-1): "negatively charged", 0: "uncharged", 1: "positively charged"}[self.charge]
        water = "hydrophobic" if self.is_hydrophobic else "hydrophilic"
        return f"{self.name} is {charge} and {water} ({self.hydropathy:+g})"


_TABLE: tuple[AminoAcid, ...] = (
    AminoAcid("A", "Ala", "alanine", 1.8, 0),
    AminoAcid("R", "Arg", "arginine", -4.5, 1),
    AminoAcid("N", "Asn", "asparagine", -3.5, 0),
    AminoAcid("D", "Asp", "aspartate", -3.5, -1),
    AminoAcid("C", "Cys", "cysteine", 2.5, 0),
    AminoAcid("E", "Glu", "glutamate", -3.5, -1),
    AminoAcid("Q", "Gln", "glutamine", -3.5, 0),
    AminoAcid("G", "Gly", "glycine", -0.4, 0),
    AminoAcid("H", "His", "histidine", -3.2, 0),
    AminoAcid("I", "Ile", "isoleucine", 4.5, 0),
    AminoAcid("L", "Leu", "leucine", 3.8, 0),
    AminoAcid("K", "Lys", "lysine", -3.9, 1),
    AminoAcid("M", "Met", "methionine", 1.9, 0),
    AminoAcid("F", "Phe", "phenylalanine", 2.8, 0),
    AminoAcid("P", "Pro", "proline", -1.6, 0),
    AminoAcid("S", "Ser", "serine", -0.8, 0),
    AminoAcid("T", "Thr", "threonine", -0.7, 0),
    AminoAcid("W", "Trp", "tryptophan", -0.9, 0),
    AminoAcid("Y", "Tyr", "tyrosine", -1.3, 0),
    AminoAcid("V", "Val", "valine", 4.2, 0),
)

BY_LETTER: dict[str, AminoAcid] = {a.letter: a for a in _TABLE}
"""Every amino acid, keyed by its single-letter code.

Histidine is listed as uncharged. Its side chain has a pKa near 6, so at pH 7 it
is mostly neutral and partly protonated. Calling it uncharged is the usual
simplification and it is worth knowing that it is one.
"""


def substitution_effect(before: str, after: str) -> str:
    """Explains what changing one residue for another does to a protein.

    Returns a sentence, not a severity score. A score would imply this sandbox
    can rank mutations by how bad they are, which requires structural context it
    does not have. What it can honestly say is how the chemistry changed.
    """
    if before == after:
        return "no change: the codon is different but codes for the same amino acid"

    if after == "*":
        return (
            "the codon became a stop, so the protein is cut short here and "
            "everything after this point is missing"
        )
    if before == "*":
        return (
            "a stop codon became an amino acid, so translation runs past where "
            "it used to end and the protein gains an unintended tail"
        )

    old = BY_LETTER.get(before)
    new = BY_LETTER.get(after)
    if old is None or new is None:
        return f"unrecognised residue: {before} to {after}"

    parts = [f"{old.describe()}; {new.describe()}"]

    if old.charge != new.charge:
        parts.append(
            f"the charge at this position changes by {new.charge - old.charge:+d}, "
            "which alters how this part of the protein interacts with water and "
            "with other proteins"
        )
    if old.is_hydrophobic != new.is_hydrophobic:
        direction = (
            "hydrophilic to hydrophobic" if new.is_hydrophobic else "hydrophobic to hydrophilic"
        )
        parts.append(
            f"the residue goes from {direction}. On the surface of a protein "
            "that matters a great deal, because a hydrophobic patch where there "
            "was none gives the protein somewhere to stick"
        )
    if abs(new.hydropathy - old.hydropathy) < 1.0 and old.charge == new.charge:
        parts.append(
            "the two are chemically similar, so this substitution is often "
            "tolerated. Often, not always: position matters more than chemistry"
        )

    sentence = ". ".join(parts) + "."
    return sentence[0].upper() + sentence[1:]
