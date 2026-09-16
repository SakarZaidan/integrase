"""The containment hierarchy, from a base up to an organism.

Biology genuinely is objects inside objects, so modelling it that way is
correct rather than merely convenient. The zoom navigation the sandbox needs is
a walk of this tree, which makes the architecture the lesson: a student learns
the body is layered by moving through the layers.

Composition throughout. A `Cell` *has* a genome; it is not a kind of genome.
Cell type is data on a cell rather than a subclass of it, for the same reason a
feature's role is an ontology IRI rather than an enum in `integrase-core`.
Biology does not sort into a clean tree, an inheritance hierarchy forces a
breaking change every time it fails to, and the escape-hatch variant becomes the
most-used one.
"""

from __future__ import annotations

from dataclasses import dataclass, field

BASES = frozenset("ACGT")
"""The alphabet this sandbox uses.

Unambiguous DNA only. The professional core accepts the full IUPAC set because
real files contain it; a teaching sandbox that let a student write `N` would have
to explain ambiguity codes before anything else worked.
"""

STOP_CODONS = frozenset({"TAA", "TAG", "TGA"})

CODON_TABLE: dict[str, str] = {
    "TTT": "F",
    "TTC": "F",
    "TTA": "L",
    "TTG": "L",
    "CTT": "L",
    "CTC": "L",
    "CTA": "L",
    "CTG": "L",
    "ATT": "I",
    "ATC": "I",
    "ATA": "I",
    "ATG": "M",
    "GTT": "V",
    "GTC": "V",
    "GTA": "V",
    "GTG": "V",
    "TCT": "S",
    "TCC": "S",
    "TCA": "S",
    "TCG": "S",
    "CCT": "P",
    "CCC": "P",
    "CCA": "P",
    "CCG": "P",
    "ACT": "T",
    "ACC": "T",
    "ACA": "T",
    "ACG": "T",
    "GCT": "A",
    "GCC": "A",
    "GCA": "A",
    "GCG": "A",
    "TAT": "Y",
    "TAC": "Y",
    "TAA": "*",
    "TAG": "*",
    "CAT": "H",
    "CAC": "H",
    "CAA": "Q",
    "CAG": "Q",
    "AAT": "N",
    "AAC": "N",
    "AAA": "K",
    "AAG": "K",
    "GAT": "D",
    "GAC": "D",
    "GAA": "E",
    "GAG": "E",
    "TGT": "C",
    "TGC": "C",
    "TGA": "*",
    "TGG": "W",
    "CGT": "R",
    "CGC": "R",
    "CGA": "R",
    "CGG": "R",
    "AGT": "S",
    "AGC": "S",
    "AGA": "R",
    "AGG": "R",
    "GGT": "G",
    "GGC": "G",
    "GGA": "G",
    "GGG": "G",
}
"""The standard genetic code.

Written out rather than generated, because a student will open this file to see
what a codon table is and a comprehension would teach them nothing.
"""


class InvalidSequenceError(ValueError):
    """A sequence contained something that is not a base.

    Its own type so a teaching interface can catch it and explain, rather than
    showing a student a raw `ValueError` from three frames down.
    """


@dataclass(frozen=True, slots=True)
class Sequence:
    """A run of bases.

    Frozen, so an edit produces a new sequence. Mutating in place would let a
    student change a gene and lose the ability to compare it with what it was,
    which is the entire point of the exercise.
    """

    bases: str

    def __post_init__(self) -> None:
        bad = set(self.bases.upper()) - BASES
        if bad:
            msg = (
                f"not DNA: {sorted(bad)}. This sandbox uses A, C, G and T only. "
                "Ambiguity codes like N are real, and they are a later lesson."
            )
            raise InvalidSequenceError(msg)
        object.__setattr__(self, "bases", self.bases.upper())

    def __len__(self) -> int:
        return len(self.bases)

    def __getitem__(self, i: int | slice) -> str:
        return self.bases[i]

    def transcribe(self) -> str:
        """The messenger RNA read off this strand.

        Thymine becomes uracil and nothing else changes, because this is a
        template-strand-to-mRNA view rather than a full transcription model.
        """
        return self.bases.replace("T", "U")

    def translate(self) -> str:
        """Amino acids, one letter each, stopping at the first stop codon.

        Returns the protein as a string. A trailing incomplete codon is ignored
        rather than padded, which is what a ribosome does.
        """
        out: list[str] = []
        for i in range(0, len(self.bases) - 2, 3):
            codon = self.bases[i : i + 3]
            residue = CODON_TABLE[codon]
            if residue == "*":
                break
            out.append(residue)
        return "".join(out)

    def mutate(self, position: int, base: str) -> Sequence:
        """A new sequence with one base changed.

        Zero-based, to match the rest of the system. Out of range raises rather
        than silently doing nothing, because a mutation that did not happen is
        the hardest kind of bug for a student to see.
        """
        if not 0 <= position < len(self.bases):
            msg = f"position {position} is outside a sequence of length {len(self.bases)}"
            raise IndexError(msg)
        return Sequence(self.bases[:position] + base + self.bases[position + 1 :])


@dataclass(slots=True)
class Gene:
    """A named coding region, plus the parts that control it.

    `promoter_strength` is how readily this gene is transcribed, relative to a
    notional average of 1.0. It is a single number standing in for a promoter
    sequence, a sigma factor, and everything else that sets transcription rate.
    That is a large simplification and it is stated here rather than discovered.
    """

    name: str
    sequence: Sequence
    promoter_strength: float = 1.0
    """Relative transcription rate. 0 silences the gene."""

    product: str = ""
    """Name of the protein this gene codes for. Defaults to the gene name."""

    def __post_init__(self) -> None:
        if not self.product:
            self.product = self.name
        if self.promoter_strength < 0:
            msg = f"promoter_strength must not be negative, got {self.promoter_strength}"
            raise ValueError(msg)

    @property
    def protein_sequence(self) -> str:
        """What this gene codes for, as amino acids."""
        return self.sequence.translate()


@dataclass(slots=True)
class Chromosome:
    """An ordered set of genes on one molecule."""

    name: str
    genes: list[Gene] = field(default_factory=list)

    def find(self, gene_name: str) -> Gene | None:
        """The gene with this name, or `None`."""
        return next((g for g in self.genes if g.name == gene_name), None)


@dataclass(slots=True)
class Genome:
    """Every chromosome in a cell."""

    chromosomes: list[Chromosome] = field(default_factory=list)

    @property
    def genes(self) -> list[Gene]:
        """Every gene, in chromosome order."""
        return [g for c in self.chromosomes for g in c.genes]

    def find(self, gene_name: str) -> Gene | None:
        """The gene with this name, searching every chromosome."""
        for c in self.chromosomes:
            hit = c.find(gene_name)
            if hit is not None:
                return hit
        return None

    def copy(self) -> Genome:
        """A deep copy, so an edited genome can be compared with the original.

        Sequences are frozen, so only the mutable containers need rebuilding.
        """
        return Genome(
            chromosomes=[
                Chromosome(
                    name=c.name,
                    genes=[
                        Gene(
                            name=g.name,
                            sequence=g.sequence,
                            promoter_strength=g.promoter_strength,
                            product=g.product,
                        )
                        for g in c.genes
                    ],
                )
                for c in self.chromosomes
            ]
        )
