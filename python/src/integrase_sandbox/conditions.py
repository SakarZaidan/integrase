"""Worked examples, chosen to teach opposite lessons.

Sickle cell anaemia is the clean case: one gene, one base, a mechanism you can
follow from the change to the protein to the shape of a cell. It is the reason
Mendelian genetics is taught, and it is worth studying carefully.

Eye colour is the correction. The model taught in most schools is wrong, and
this module exists partly to say so with a working example rather than a
footnote.

Between them they make the point the sandbox is for: biology sometimes has
simple rules, and assuming it usually does is the most common mistake a student
brings with them.

# On accuracy

The sickle cell variant here is real and checkable. The sixth codon of the
mature HBB protein is GAG, coding for glutamate, and an A to T change at its
middle position makes it GTG, coding for valine. That substitution is the cause
of sickle cell anaemia and is one of the most thoroughly documented facts in
human genetics.

The first thirteen codons in this module are the real HBB coding sequence, with
a stop appended so the fragment translates cleanly. The real gene continues for
another 134 residues across three exons. Truncating is a simplification; moving
the variant would have been an error, so the position is kept exact and the
length is not.

The eye colour effect sizes are illustrative. The direction and the relative
importance of rs12913832 are well supported; the specific log-odds numbers here
are chosen so the example behaves usefully, and they say so.
"""

from __future__ import annotations

from .diploid import DiploidGenome, Inheritance, heterozygous, homozygous
from .genome import Chromosome, Gene, Genome, Sequence
from .parameters import Parameter, Provenance, illustrative
from .phenotype import Contribution, Locus, MendelianTrait, MonogenicTrait, PolygenicTrait

# --- sickle cell -------------------------------------------------------------

# ATG GTG CAT CTG ACT CCT GAG GAG AAG TCT GCC GTT ACT
# Met Val His Leu Thr Pro Glu Glu Lys Ser Ala Val Thr
#
# Those thirteen codons are the real start of the human HBB coding sequence. A
# stop is appended so the stand-in translates cleanly; the real gene continues
# for another 134 residues across three exons, and reproducing it would add
# nothing a student can read while implying a fidelity the rest of this model
# does not have.
_HBB_REFERENCE = "ATGGTGCATCTGACTCCTGAGGAGAAGTCTGCCGTTACTTAA"

HBB_CODON_6_START = 18
"""Where the sixth codon of the mature protein begins, zero-based.

Mature haemoglobin beta is numbered from the valine that follows the initiator
methionine, so mature residue 6 is the seventh codon of the coding sequence and
begins at base 18. Off-by-one errors between these two numbering conventions are
a real and recurring source of confusion in genetics, which is why the
arithmetic is spelled out here instead of a bare constant.
"""

HBB_VARIANT_POSITION = HBB_CODON_6_START + 1
"""The A that becomes T. GAG codes for glutamate, GTG for valine."""


def hbb_gene(variant: bool = False) -> Gene:
    """The stand-in haemoglobin beta gene.

    `variant=True` returns the sickle allele, with A changed to T at the middle
    position of codon 6.
    """
    sequence = Sequence(_HBB_REFERENCE)
    if variant:
        sequence = sequence.mutate(HBB_VARIANT_POSITION, "T")
    return Gene("HBB", sequence, promoter_strength=1.0, product="haemoglobin-beta")


def hbb_genome(variant: bool = False) -> Genome:
    """A genome holding only the haemoglobin gene, for the sickle cell lesson."""
    return Genome(chromosomes=[Chromosome("chromosome-11", [hbb_gene(variant)])])


SICKLE_CELL = MonogenicTrait(
    name="sickle cell anaemia",
    locus=Locus(gene="HBB", position=HBB_VARIANT_POSITION, reference_base="A"),
    variant_base="T",
    present="affected",
    absent="unaffected",
    penetrance=Parameter(
        value=1.0,
        unit="fraction",
        provenance=Provenance.MEASURED,
        source=(
            "homozygous HbSS is fully penetrant; severity varies widely with "
            "fetal haemoglobin level and with care"
        ),
    ),
    why=(
        "The codon changes from GAG to GTG, so glutamate becomes valine. "
        "Glutamate is negatively charged and hydrophilic; valine is uncharged "
        "and strongly hydrophobic. That puts a sticky patch on the surface of "
        "haemoglobin, and when oxygen is low the molecules link into long fibres "
        "that pull the red cell out of shape."
    ),
    caveat=(
        "This models one copy of the gene. Real sickle cell anaemia is "
        "recessive: one variant copy gives sickle cell trait, which is mostly "
        "without symptoms and confers some resistance to malaria, and two give "
        "the disease. This sandbox's genome holds a single copy of each gene, "
        "so the diploid half of the lesson is missing rather than simplified."
    ),
)
"""Sickle cell anaemia, modelled as a single locus.

**This models one copy, not two.** Real sickle cell anaemia is recessive: one
copy of the variant gives sickle cell trait, which is mostly asymptomatic and
confers some malaria resistance, and two copies give the disease. The sandbox's
`Genome` holds one copy of each gene, so the diploid half of the lesson is not
modelled and is stated here rather than quietly dropped.
"""


HBB_LOCUS = Locus(gene="HBB", position=HBB_VARIANT_POSITION, reference_base="A")


def hbb_diploid(variant_copies: int = 0) -> DiploidGenome:
    """A diploid genome with 0, 1 or 2 copies of the sickle variant.

    One copy is the state most people carrying this variant are in, and it is
    the one a single-copy model could not express at all.
    """
    if variant_copies == 0:
        return homozygous(hbb_genome(variant=False))
    if variant_copies == 1:
        return heterozygous(hbb_genome(variant=False), hbb_genome(variant=True))
    if variant_copies == 2:
        return homozygous(hbb_genome(variant=True))
    msg = f"variant_copies must be 0, 1 or 2, got {variant_copies}"
    raise ValueError(msg)


SICKLE_CELL_INHERITED = MendelianTrait(
    name="sickle cell anaemia",
    locus=HBB_LOCUS,
    mode=Inheritance.RECESSIVE,
    present="affected",
    absent="unaffected",
    carrier="sickle cell trait, a carrier",
    penetrance=Parameter(
        value=1.0,
        unit="fraction",
        provenance=Provenance.MEASURED,
        source=(
            "homozygous HbSS is fully penetrant; severity varies widely with "
            "fetal haemoglobin level and with care"
        ),
    ),
    why=(
        "The codon changes from GAG to GTG, so glutamate becomes valine. "
        "Glutamate is negatively charged and hydrophilic; valine is uncharged "
        "and strongly hydrophobic. That puts a sticky patch on the surface of "
        "haemoglobin, and when oxygen is low the molecules link into long "
        "fibres that pull the red cell out of shape."
    ),
    caveat=(
        "Carrying one copy is not a lesser form of the disease. It is a "
        "different state, mostly without symptoms, and it confers resistance "
        "to malaria. That resistance is why the variant is common where "
        "malaria is common, which is the clearest example in human genetics of "
        "a variant being harmful and useful at once.\n\n"
        "Not modelled here: the wide variation in severity between people with "
        "two copies, which depends heavily on fetal haemoglobin level and on "
        "access to care."
    ),
)
"""Sickle cell anaemia, modelled with both copies.

Replaces the single-copy `SICKLE_CELL` for any lesson about inheritance.
`SICKLE_CELL` is kept because a first lesson about what one base change does to
a protein does not need two copies, and introducing diploidy at that point makes
the simpler point harder to see.
"""


MALARIA_RESISTANCE = MendelianTrait(
    name="resistance to severe malaria",
    locus=HBB_LOCUS,
    mode=Inheritance.CODOMINANT,
    present="strongly protected, at the cost of sickle cell anaemia",
    absent="no protection from this variant",
    carrier="substantially protected, without the disease",
    penetrance=Parameter(
        value=0.9,
        unit="fraction",
        provenance=Provenance.MEASURED,
        source=(
            "heterozygous HbAS is associated with roughly a 90 percent "
            "reduction in the risk of severe malaria; the protection is "
            "consistently large across studies and the exact figure varies"
        ),
        relative_error=0.15,
    ),
    why=(
        "Red cells carrying the variant are cleared faster when infected, and "
        "the parasite grows poorly in them."
    ),
    caveat=(
        "This is the same locus as the sickle cell trait above, evaluated for a "
        "different consequence. One variant, two outcomes, pulling in opposite "
        "directions.\n\n"
        "A variant is not good or bad on its own. Whether it helps depends on "
        "where you live and what you are exposed to, which is why "
        "population-level statements about genetic variants are so often wrong."
    ),
)
"""The other side of the same locus.

Evaluating one variant for two consequences is the point. A student who sees
only the disease learns that this variant is a defect; a student who sees both
learns why it persists at high frequency in parts of the world where malaria is
endemic, which is a fact about selection rather than about defects.
"""


# --- eye colour --------------------------------------------------------------

_HERC2_STANDIN = "ATGAAAGGTCTGGATGCGACCCTGGAAGCGAAAGGTCTGTATTAA"
_OCA2_STANDIN = "ATGAGCCTGACCGGTAAAGCGCTGGAAGATGTTCTGGGTAAATAA"
_TYR_STANDIN = "ATGGGTAAACTGGCGACCCTGTATGAAGCGGGTCTGAAAGTTTAA"

RS12913832_POSITION = 12
"""Where the sandbox places the rs12913832 stand-in.

The real variant is an intronic single nucleotide polymorphism in HERC2, not a
coding change, and it works by controlling how much OCA2 is expressed. This
sandbox has no introns and no regulatory model, so it sits at a coding position
instead. That is a real simplification of the most important part of the
mechanism, and it is written down rather than glossed.
"""


def eye_colour_genome(
    herc2_variant: str = "G", oca2_variant: str = "C", tyr_variant: str = "A"
) -> Genome:
    """A three-gene genome for the eye colour lesson.

    Defaults are the brown-associated alleles at all three loci.
    """
    herc2 = Sequence(_HERC2_STANDIN).mutate(RS12913832_POSITION, herc2_variant)
    oca2 = Sequence(_OCA2_STANDIN).mutate(15, oca2_variant)
    tyr = Sequence(_TYR_STANDIN).mutate(21, tyr_variant)
    return Genome(
        chromosomes=[
            Chromosome(
                "chromosome-15",
                [
                    Gene("HERC2", herc2, product="HERC2"),
                    Gene("OCA2", oca2, product="P-protein"),
                ],
            ),
            Chromosome("chromosome-11", [Gene("TYR", tyr, product="tyrosinase")]),
        ]
    )


EYE_COLOUR = PolygenicTrait(
    name="eye colour",
    baseline={"brown": 0.55, "blue": 0.30, "green or hazel": 0.15},
    contributions=[
        Contribution(
            locus=Locus("HERC2", RS12913832_POSITION, "G"),
            variant_base="A",
            log_odds=illustrative(
                2.2,
                "log odds",
                "large, because rs12913832 really does dominate this trait",
            ),
            affects="blue",
            note=(
                "rs12913832 is the single largest known contributor. It is not "
                "in a pigment gene at all: it sits in a regulatory region of "
                "HERC2 and controls how much OCA2 is expressed"
            ),
        ),
        Contribution(
            locus=Locus("OCA2", 15, "C"),
            variant_base="T",
            log_odds=illustrative(0.8, "log odds", "moderate secondary effect"),
            affects="blue",
            note="OCA2 makes the P-protein, which the melanin pathway depends on",
        ),
        Contribution(
            locus=Locus("TYR", 21, "A"),
            variant_base="G",
            log_odds=illustrative(0.5, "log odds", "small effect, shifts toward green"),
            affects="green or hazel",
            note="TYR makes tyrosinase, the enzyme that starts melanin synthesis",
        ),
    ],
    note=(
        "These are shifted odds, not a determined outcome. No combination of "
        "these three loci makes any eye colour certain, and the model will "
        "never report one."
    ),
    missing_genes=(
        "What this leaves out: roughly 13 more genes with measurable effects, "
        "the diploid nature of real genomes, and the fact that rs12913832 acts "
        "through gene regulation rather than by changing a protein.\n\n"
        "The brown-dominant, blue-recessive model taught in most schools is not "
        "a simplification of this. It is wrong. Two blue-eyed parents having a "
        "brown-eyed child is uncommon but entirely possible, which the simple "
        "model says cannot happen."
    ),
)
"""Eye colour, as the lesson in why simple inheritance rules fail.

Roughly 16 genes contribute. The dominant one, rs12913832, is not in a pigment
gene: it sits in a regulatory region of HERC2 and controls OCA2 expression.

Sources:
https://www.nature.com/articles/jhg2010126
https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10454093/

Baseline frequencies are for a broadly European population and are illustrative.
Eye colour frequency varies enormously between populations, and a single
"population baseline" is itself a simplification worth naming.
"""
