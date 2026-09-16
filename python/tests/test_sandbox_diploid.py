"""Tests for two copies of every chromosome.

The properties here divide into two. Some check that the classic results of
Mendelian genetics fall out of the model rather than being written into it: a
1:2:1 ratio from two carriers has to emerge from coin flips, or the sandbox is
reciting the answer. The rest check that carrier status is handled as a distinct
state and never collapsed into affected or unaffected, which is the failure a
single-copy model could not avoid.
"""

from __future__ import annotations

import pytest

from integrase_sandbox.conditions import (
    HBB_VARIANT_POSITION,
    MALARIA_RESISTANCE,
    SICKLE_CELL_INHERITED,
    hbb_diploid,
    hbb_genome,
)
from integrase_sandbox.diploid import (
    DiploidGenome,
    Inheritance,
    Zygosity,
    cross,
    gamete,
    heterozygous,
    homozygous,
    inherit,
)
from integrase_sandbox.genome import Genome
from integrase_sandbox.parameters import Parameter, Provenance
from integrase_sandbox.phenotype import Locus, MendelianTrait
from integrase_sandbox.state import Environment

ENV = Environment()
LOCUS = ("HBB", HBB_VARIANT_POSITION, "A")

# --- zygosity ----------------------------------------------------------------


def test_copy_number_is_read_from_both_copies() -> None:
    assert hbb_diploid(0).zygosity(*LOCUS) is Zygosity.HOMOZYGOUS_REFERENCE
    assert hbb_diploid(1).zygosity(*LOCUS) is Zygosity.HETEROZYGOUS
    assert hbb_diploid(2).zygosity(*LOCUS) is Zygosity.HOMOZYGOUS_VARIANT


def test_a_missing_copy_is_unknown_and_not_a_reference_copy() -> None:
    """Treating a missing copy as reference would report a carrier as clear.

    That is a silent wrong answer in exactly the direction that matters: it
    turns "we do not know" into "you are fine".
    """
    half = DiploidGenome(maternal=hbb_genome(variant=True), paternal=Genome())
    assert half.zygosity(*LOCUS) is Zygosity.UNKNOWN


def test_a_position_outside_the_gene_is_unknown() -> None:
    assert hbb_diploid(0).zygosity("HBB", 9999, "A") is Zygosity.UNKNOWN


def test_unknown_reports_zero_copies_without_claiming_zero() -> None:
    """The count exists for dose-response models; the member carries the doubt."""
    assert Zygosity.UNKNOWN.variant_copies == 0
    assert Zygosity.UNKNOWN is not Zygosity.HOMOZYGOUS_REFERENCE


def test_copies_of_a_diploid_genome_are_independent() -> None:
    original = hbb_diploid(0)
    duplicate = original.copy()
    maternal = duplicate.maternal.find("HBB")
    assert maternal is not None
    maternal.promoter_strength = 0.0
    other = original.maternal.find("HBB")
    assert other is not None
    assert other.promoter_strength == 1.0


# --- inheritance -------------------------------------------------------------


def test_two_carriers_produce_the_classic_one_two_one_ratio() -> None:
    """The Punnett square, arrived at by sampling rather than by assertion.

    If this ratio were hardcoded the test would be worthless. It comes out of
    one coin flip per chromosome per parent, which is what makes it evidence
    that the model does meiosis rather than reciting a result.
    """
    carrier = hbb_diploid(1)
    proportions = cross(carrier, carrier, *LOCUS, offspring=4000, seed=1)

    assert proportions[Zygosity.HOMOZYGOUS_REFERENCE] == pytest.approx(0.25, abs=0.03)
    assert proportions[Zygosity.HETEROZYGOUS] == pytest.approx(0.50, abs=0.03)
    assert proportions[Zygosity.HOMOZYGOUS_VARIANT] == pytest.approx(0.25, abs=0.03)


def test_a_carrier_and_an_unaffected_parent_never_produce_an_affected_child() -> None:
    """One parent with no variant copy means no child can have two."""
    proportions = cross(hbb_diploid(1), hbb_diploid(0), *LOCUS, offspring=2000, seed=2)
    assert Zygosity.HOMOZYGOUS_VARIANT not in proportions
    assert proportions[Zygosity.HETEROZYGOUS] == pytest.approx(0.5, abs=0.04)


def test_two_affected_parents_produce_only_affected_children() -> None:
    proportions = cross(hbb_diploid(2), hbb_diploid(2), *LOCUS, offspring=200, seed=3)
    assert proportions == {Zygosity.HOMOZYGOUS_VARIANT: 1.0}


def test_a_gamete_carries_one_copy_of_each_chromosome() -> None:
    import random

    child_set = gamete(hbb_diploid(1), random.Random(0))
    assert len(child_set.chromosomes) == len(hbb_diploid(1).maternal.chromosomes)


def test_a_family_is_reproducible_from_its_seed() -> None:
    """A teacher needs to be able to hand a class the same family."""
    a = inherit(hbb_diploid(1), hbb_diploid(1), seed=42)
    b = inherit(hbb_diploid(1), hbb_diploid(1), seed=42)
    assert a.zygosity(*LOCUS) is b.zygosity(*LOCUS)


def test_different_seeds_give_different_families() -> None:
    """Otherwise the randomness is decorative and the lesson is a fixed story."""
    outcomes = {inherit(hbb_diploid(1), hbb_diploid(1), seed=s).zygosity(*LOCUS) for s in range(40)}
    assert len(outcomes) > 1


def test_an_offspring_count_below_one_is_refused() -> None:
    with pytest.raises(ValueError, match="at least 1"):
        cross(hbb_diploid(1), hbb_diploid(1), *LOCUS, offspring=0)


# --- the carrier state -------------------------------------------------------


def test_one_copy_is_a_carrier_and_not_a_mild_case() -> None:
    """The state a single-copy model could not express.

    Reporting a carrier as affected or unaffected are both wrong, and both were
    the only options before diploidy existed.
    """
    outcome = SICKLE_CELL_INHERITED.evaluate(hbb_diploid(1), ENV)
    assert outcome.most_likely == "sickle cell trait, a carrier"
    assert outcome.probabilities.get("affected") is None
    assert "can pass the variant on" in "\n".join(outcome.explanation)


def test_two_copies_give_the_condition() -> None:
    outcome = SICKLE_CELL_INHERITED.evaluate(hbb_diploid(2), ENV)
    assert outcome.most_likely == "affected"


def test_no_copies_is_not_a_clean_bill_of_health() -> None:
    outcome = SICKLE_CELL_INHERITED.evaluate(hbb_diploid(0), ENV)
    assert outcome.most_likely == "unaffected"
    assert "not a clean bill of health" in outcome.certainty


def test_the_carrier_result_explains_the_one_in_four() -> None:
    """A carrier who is not told what it means for their children learns nothing."""
    outcome = SICKLE_CELL_INHERITED.evaluate(hbb_diploid(1), ENV)
    assert "one in four" in outcome.certainty


def test_carrying_one_copy_is_not_described_as_a_lesser_disease() -> None:
    outcome = SICKLE_CELL_INHERITED.evaluate(hbb_diploid(1), ENV)
    assert "not a lesser form" in outcome.certainty


# --- one variant, two consequences -------------------------------------------


def test_the_same_locus_gives_protection_as_well_as_disease() -> None:
    """The point of shipping both traits on one position.

    A student who sees only the disease learns this variant is a defect. One
    who sees both learns why it is common where malaria is common, which is a
    fact about selection rather than about defects.
    """
    carrier = hbb_diploid(1)
    disease = SICKLE_CELL_INHERITED.evaluate(carrier, ENV)
    protection = MALARIA_RESISTANCE.evaluate(carrier, ENV)

    assert "carrier" in disease.most_likely
    assert "protected" in protection.most_likely
    assert "without the disease" in protection.most_likely


def test_the_trade_off_is_stated_rather_than_left_to_be_noticed() -> None:
    outcome = MALARIA_RESISTANCE.evaluate(hbb_diploid(1), ENV)
    assert "opposite directions" in outcome.certainty
    assert "where you live" in outcome.certainty


# --- inheritance modes -------------------------------------------------------


def _trait(mode: Inheritance) -> MendelianTrait:
    return MendelianTrait(
        name="test trait",
        locus=Locus("HBB", HBB_VARIANT_POSITION, "A"),
        mode=mode,
        present="shows",
        absent="does not show",
        carrier="carrier",
        penetrance=Parameter(
            value=1.0, unit="fraction", provenance=Provenance.MEASURED, source="a test"
        ),
    )


def test_a_dominant_trait_shows_with_one_copy() -> None:
    outcome = _trait(Inheritance.DOMINANT).evaluate(hbb_diploid(1), ENV)
    assert outcome.most_likely == "shows"


def test_a_recessive_trait_does_not_show_with_one_copy() -> None:
    outcome = _trait(Inheritance.RECESSIVE).evaluate(hbb_diploid(1), ENV)
    assert outcome.most_likely == "carrier"


def test_an_additive_trait_scales_with_copy_number() -> None:
    """Two copies give roughly twice the chance of one. Roughly, and it says so."""
    one = _trait(Inheritance.ADDITIVE).evaluate(hbb_diploid(1), ENV)
    two = _trait(Inheritance.ADDITIVE).evaluate(hbb_diploid(2), ENV)
    assert one.probabilities["shows"] == pytest.approx(0.5)
    assert two.probabilities["shows"] == pytest.approx(1.0)


def test_a_codominant_trait_shows_an_intermediate_state() -> None:
    outcome = _trait(Inheritance.CODOMINANT).evaluate(hbb_diploid(1), ENV)
    assert outcome.most_likely == "carrier"


def test_a_missing_gene_is_not_assessed() -> None:
    half = DiploidGenome(maternal=Genome(), paternal=Genome())
    outcome = _trait(Inheritance.RECESSIVE).evaluate(half, ENV)
    assert outcome.most_likely == "cannot be assessed"
    assert "not the same as a negative" in outcome.certainty


# --- construction helpers ----------------------------------------------------


def test_homozygous_builds_two_independent_copies() -> None:
    """Sharing one object between both copies would make every edit appear twice."""
    both = homozygous(hbb_genome())
    assert both.maternal is not both.paternal
    maternal = both.maternal.find("HBB")
    assert maternal is not None
    maternal.promoter_strength = 0.0
    paternal = both.paternal.find("HBB")
    assert paternal is not None
    assert paternal.promoter_strength == 1.0


def test_heterozygous_puts_the_variant_on_one_copy() -> None:
    mixed = heterozygous(hbb_genome(variant=False), hbb_genome(variant=True))
    assert mixed.zygosity(*LOCUS) is Zygosity.HETEROZYGOUS


def test_an_invalid_copy_count_is_refused() -> None:
    with pytest.raises(ValueError, match="0, 1 or 2"):
        hbb_diploid(3)
