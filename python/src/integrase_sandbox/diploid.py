"""Two copies of every chromosome, and what follows from that.

The sandbox held one copy of each gene, which made recessive inheritance
impossible to teach and forced the sickle cell lesson to state its own limit in
prose. One variant copy of HBB gives sickle cell trait, which is mostly without
symptoms and confers some resistance to malaria. Two give the disease. A model
with one copy cannot express the difference, and the difference is the lesson.

# Composition, again

A `DiploidGenome` *has* two `Genome` objects. It is not a subclass of `Genome`
and `Genome` is not a subclass of it. Inheritance here would mean claiming a
diploid genome is a kind of haploid one, which is backwards, or that a haploid
one is a diploid with a piece missing, which is worse.

The two copies are named maternal and paternal because that is what a student
needs to see. Nothing in the model treats them differently: there is no
imprinting, no sex chromosome, and no mitochondrial inheritance. All three are
real and all three are absent.

# Gametes, and why they are random

`inherit` picks one copy of each chromosome at random from each parent. That is
meiosis reduced to a coin flip per chromosome. It leaves out recombination
entirely, which means genes on the same chromosome here are inherited together
always, where in a real genome they are separated in proportion to the distance
between them. Linkage is a real and teachable phenomenon and it is not modelled.
"""

from __future__ import annotations

import random
from dataclasses import dataclass
from enum import Enum

from .genome import Chromosome, Gene, Genome


class Zygosity(Enum):
    """What a genome carries at one position, across its two copies."""

    HOMOZYGOUS_REFERENCE = "two reference copies"
    HETEROZYGOUS = "one reference copy and one variant copy"
    HOMOZYGOUS_VARIANT = "two variant copies"
    UNKNOWN = "cannot be determined"

    @property
    def variant_copies(self) -> int:
        """How many copies carry the variant. `UNKNOWN` counts as zero.

        Callers that care about the difference between "none" and "unknown"
        check the member, not this number. It exists for dose-response models,
        where the count is what matters.
        """
        return {
            Zygosity.HOMOZYGOUS_REFERENCE: 0,
            Zygosity.HETEROZYGOUS: 1,
            Zygosity.HOMOZYGOUS_VARIANT: 2,
            Zygosity.UNKNOWN: 0,
        }[self]


class Inheritance(Enum):
    """How copy number maps to whether a trait appears.

    Four patterns, which covers the classic teaching cases and not much else.
    Real inheritance includes imprinting, mitochondrial transmission,
    X-linkage, anticipation and modifier genes, none of which are here.
    """

    RECESSIVE = "recessive"
    """Two variant copies needed. One copy makes a carrier."""

    DOMINANT = "dominant"
    """One variant copy is enough."""

    CODOMINANT = "codominant"
    """Both copies show. One copy gives an intermediate, visibly distinct state."""

    ADDITIVE = "additive"
    """Each copy contributes, so two copies give roughly twice the effect."""


@dataclass(slots=True)
class DiploidGenome:
    """Two sets of chromosomes, one from each parent."""

    maternal: Genome
    paternal: Genome

    def alleles(self, gene_name: str) -> tuple[Gene | None, Gene | None]:
        """The maternal and paternal copies of a gene, either possibly absent."""
        return (self.maternal.find(gene_name), self.paternal.find(gene_name))

    def zygosity(self, gene_name: str, position: int, reference_base: str) -> Zygosity:
        """What this genome carries at one position, across both copies.

        Returns `UNKNOWN` rather than guessing when either copy is missing or
        the position is outside the gene. A missing copy is not a reference
        copy, and treating it as one would report a carrier as unaffected.
        """
        maternal, paternal = self.alleles(gene_name)
        if maternal is None or paternal is None:
            return Zygosity.UNKNOWN

        bases = []
        for gene in (maternal, paternal):
            if not 0 <= position < len(gene.sequence):
                return Zygosity.UNKNOWN
            bases.append(gene.sequence[position])

        variants = sum(1 for b in bases if b != reference_base)
        return {
            0: Zygosity.HOMOZYGOUS_REFERENCE,
            1: Zygosity.HETEROZYGOUS,
            2: Zygosity.HOMOZYGOUS_VARIANT,
        }[variants]

    def copy(self) -> DiploidGenome:
        """A deep copy, so an edit can be compared with what it replaced."""
        return DiploidGenome(maternal=self.maternal.copy(), paternal=self.paternal.copy())


def homozygous(genome: Genome) -> DiploidGenome:
    """Two identical copies of one genome.

    The starting point for most lessons, and a reminder that "homozygous" is a
    statement about a position rather than about a whole genome. A person is
    homozygous *at a locus*, and homozygous everywhere is not a thing that
    happens.
    """
    return DiploidGenome(maternal=genome.copy(), paternal=genome.copy())


def heterozygous(reference: Genome, variant: Genome) -> DiploidGenome:
    """One reference copy and one variant copy.

    The carrier state, which is where recessive inheritance gets interesting and
    which a one-copy model cannot express at all.
    """
    return DiploidGenome(maternal=reference.copy(), paternal=variant.copy())


def gamete(parent: DiploidGenome, rng: random.Random) -> Genome:
    """One haploid set, drawn from a parent.

    A coin flip per chromosome. Real meiosis also recombines within each
    chromosome, so two genes on the same chromosome are separated in proportion
    to the distance between them. Here they never separate, which makes linkage
    absolute rather than partial.
    """
    chromosomes: list[Chromosome] = []
    pairs = zip(parent.maternal.chromosomes, parent.paternal.chromosomes, strict=False)
    for maternal, paternal in pairs:
        chosen = maternal if rng.random() < 0.5 else paternal
        chromosomes.append(
            Chromosome(
                name=chosen.name,
                genes=[
                    Gene(
                        name=g.name,
                        sequence=g.sequence,
                        promoter_strength=g.promoter_strength,
                        product=g.product,
                    )
                    for g in chosen.genes
                ],
            )
        )
    return Genome(chromosomes=chromosomes)


def inherit(mother: DiploidGenome, father: DiploidGenome, seed: int | None = None) -> DiploidGenome:
    """One offspring, from one gamete of each parent.

    Random, and deliberately so. The classic Punnett square is a statement about
    proportions across many offspring, not a prediction about one, and a student
    who draws a single child from two carriers and gets an unaffected one has
    seen why a 1 in 4 risk is not a guarantee either way.

    `seed` makes a family reproducible, which a lesson needs.
    """
    rng = random.Random(seed)
    return DiploidGenome(maternal=gamete(mother, rng), paternal=gamete(father, rng))


def cross(
    mother: DiploidGenome,
    father: DiploidGenome,
    gene_name: str,
    position: int,
    reference_base: str,
    offspring: int = 1000,
    seed: int = 0,
) -> dict[Zygosity, float]:
    """The proportions a cross produces, over many offspring.

    This is the Punnett square, arrived at by sampling rather than by being
    written down. A student who has seen the square as a rule can watch it
    emerge from coin flips, which is a different and better understanding of
    where the numbers come from.
    """
    if offspring < 1:
        msg = f"offspring must be at least 1, got {offspring}"
        raise ValueError(msg)

    counts: dict[Zygosity, int] = dict.fromkeys(Zygosity, 0)
    rng = random.Random(seed)
    for _ in range(offspring):
        child = DiploidGenome(maternal=gamete(mother, rng), paternal=gamete(father, rng))
        counts[child.zygosity(gene_name, position, reference_base)] += 1

    return {z: n / offspring for z, n in counts.items() if n}
