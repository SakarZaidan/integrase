"""Tests for the trait layer.

Two kinds of assertion here. The first checks the biology is right, which for
sickle cell means checking a fact anyone can look up. The second checks the
model refuses to overclaim, which is the property most likely to erode as
someone tunes numbers to make a demo look convincing.
"""

from __future__ import annotations

import pytest

from integrase_sandbox.aminoacids import BY_LETTER, substitution_effect
from integrase_sandbox.conditions import (
    EYE_COLOUR,
    HBB_VARIANT_POSITION,
    SICKLE_CELL,
    eye_colour_genome,
    hbb_gene,
    hbb_genome,
)
from integrase_sandbox.genome import Genome
from integrase_sandbox.phenotype import Contribution, Locus, Outcome, PolygenicTrait, Trait
from integrase_sandbox.state import Environment

ENV = Environment()

# --- amino acid chemistry ----------------------------------------------------


def test_the_hydropathy_scale_has_the_range_kyte_and_doolittle_published() -> None:
    """Arginine at -4.5 and isoleucine at +4.5 are the ends of the scale.

    A published index with known endpoints is worth checking, because a typo in
    a table like this is invisible and would quietly change every explanation
    built on it.
    """
    values = [a.hydropathy for a in BY_LETTER.values()]
    assert min(values) == -4.5
    assert max(values) == 4.5
    assert BY_LETTER["R"].hydropathy == -4.5
    assert BY_LETTER["I"].hydropathy == 4.5


def test_exactly_the_charged_residues_are_charged() -> None:
    """Aspartate and glutamate negative; lysine and arginine positive."""
    charged = {a.letter: a.charge for a in BY_LETTER.values() if a.is_charged}
    assert charged == {"D": -1, "E": -1, "K": 1, "R": 1}


def test_a_substitution_to_a_stop_says_the_protein_is_truncated() -> None:
    assert "cut short" in substitution_effect("E", "*")


def test_a_silent_substitution_says_nothing_changed() -> None:
    assert "no change" in substitution_effect("E", "E")


def test_the_sickle_substitution_is_explained_by_chemistry() -> None:
    """Glutamate to valine: charge lost, and hydrophilic becomes hydrophobic.

    This is the mechanism the whole lesson rests on, and it must come out of the
    amino acid table rather than out of a hardcoded sentence about sickle cell.
    """
    text = substitution_effect("E", "V")
    assert "charge" in text
    assert "hydrophilic to hydrophobic" in text
    assert "stick" in text


# --- sickle cell -------------------------------------------------------------


def test_the_reference_gene_carries_glutamate_at_the_sickle_position() -> None:
    """Mature residue 6 is glutamate. If it is not, the lesson is wrong."""
    assert hbb_gene().protein_sequence == "MVHLTPEEKSAVT"
    # Mature numbering skips the initiator methionine, so mature residue 6 is
    # index 6 of this string.
    assert hbb_gene().protein_sequence[6] == "E"


def test_the_variant_changes_glutamate_to_valine_and_nothing_else() -> None:
    """One base, one residue. Any other difference means a coordinate is wrong."""
    reference = hbb_gene().protein_sequence
    variant = hbb_gene(variant=True).protein_sequence

    assert len(reference) == len(variant)
    differences = [
        (i, a, b) for i, (a, b) in enumerate(zip(reference, variant, strict=True)) if a != b
    ]
    assert differences == [(6, "E", "V")], f"expected only Glu6Val, got {differences}"


def test_the_variant_position_sits_inside_the_sixth_codon() -> None:
    """Guards the arithmetic that relates the two numbering conventions."""
    bases = hbb_gene().sequence.bases
    codon = bases[HBB_VARIANT_POSITION - 1 : HBB_VARIANT_POSITION + 2]
    assert codon == "GAG"
    mutated = hbb_gene(variant=True).sequence.bases
    assert mutated[HBB_VARIANT_POSITION - 1 : HBB_VARIANT_POSITION + 2] == "GTG"


def test_carrying_the_variant_reports_the_condition() -> None:
    outcome = SICKLE_CELL.evaluate(hbb_genome(variant=True), ENV)
    assert outcome.most_likely == "affected"
    assert any("GAG to GTG" in line for line in outcome.explanation)


def test_not_carrying_the_variant_is_not_a_clean_bill_of_health() -> None:
    """A negative at one locus is a statement about one locus.

    The wording matters. "Unaffected" with no qualification would teach that a
    single-locus test rules a condition out, which is how real genetic testing
    gets misread.
    """
    outcome = SICKLE_CELL.evaluate(hbb_genome(variant=False), ENV)
    assert outcome.most_likely == "unaffected"
    assert "not a clean bill of health" in outcome.certainty


def test_a_missing_gene_is_not_reported_as_a_negative() -> None:
    """No result and a negative result are different, and must read differently."""
    outcome = SICKLE_CELL.evaluate(Genome(), ENV)
    assert outcome.most_likely == "cannot be assessed"
    assert "not the same as a negative" in outcome.certainty


def test_the_single_locus_model_admits_it_is_not_diploid() -> None:
    """Real sickle cell is recessive, and this sandbox holds one copy of a gene.

    Dropping that silently would teach that one variant copy causes the disease,
    which is the specific thing sickle cell trait disproves.
    """
    certainty = SICKLE_CELL.evaluate(hbb_genome(variant=True), ENV).certainty
    assert "recessive" in certainty
    assert "sickle cell trait" in certainty


# --- eye colour --------------------------------------------------------------


def test_eye_colour_is_never_certain() -> None:
    """The property this whole module exists to protect.

    Every combination of the modelled loci, checked exhaustively. If any reaches
    95 percent, someone has tuned an effect size until the demo looked
    convincing, and the lesson has inverted into the false model it corrects.
    """
    for herc2 in ("G", "A"):
        for oca2 in ("C", "T"):
            for tyr in ("A", "G"):
                outcome = EYE_COLOUR.evaluate(eye_colour_genome(herc2, oca2, tyr), ENV)
                worst = max(outcome.probabilities.values())
                assert worst < 0.95, (
                    f"{herc2}/{oca2}/{tyr} reports {worst:.1%}, which is close "
                    "enough to certainty to teach the wrong thing"
                )
                assert min(outcome.probabilities.values()) > 0.0


def test_a_genome_with_no_variants_reproduces_the_baseline() -> None:
    """The identity case, which an earlier version got wrong.

    Converting each of three mutually exclusive classes to odds against
    "everything else" and renormalising is not the identity: 55/30/15 came back
    as 67/23/10 for a genome carrying nothing. Effects are additive on the log
    probability scale, not the log odds scale, once there are more than two
    outcomes.
    """
    outcome = EYE_COLOUR.evaluate(eye_colour_genome("G", "C", "A"), ENV)
    assert outcome.probabilities["brown"] == pytest.approx(0.55, abs=1e-9)
    assert outcome.probabilities["blue"] == pytest.approx(0.30, abs=1e-9)
    assert outcome.probabilities["green or hazel"] == pytest.approx(0.15, abs=1e-9)


def test_the_blue_variant_shifts_the_odds_toward_blue() -> None:
    brown_allele = EYE_COLOUR.evaluate(eye_colour_genome(herc2_variant="G"), ENV)
    blue_allele = EYE_COLOUR.evaluate(eye_colour_genome(herc2_variant="A"), ENV)
    assert blue_allele.probabilities["blue"] > brown_allele.probabilities["blue"]


def test_the_dominant_locus_dominates() -> None:
    """rs12913832 must outweigh the other two combined.

    That is the real finding, and a model where the minor loci could overturn it
    would be teaching a different and false story about this trait.
    """
    only_major = EYE_COLOUR.evaluate(eye_colour_genome("A", "C", "A"), ENV)
    only_minor = EYE_COLOUR.evaluate(eye_colour_genome("G", "T", "G"), ENV)
    assert only_major.probabilities["blue"] > only_minor.probabilities["blue"]


def test_the_explanation_names_every_variant_that_moved_the_odds() -> None:
    """A student who cannot see what moved the number learns the tool is an oracle."""
    outcome = EYE_COLOUR.evaluate(eye_colour_genome("A", "T", "G"), ENV)
    text = "\n".join(outcome.explanation)
    assert "HERC2" in text
    assert "OCA2" in text
    assert "TYR" in text
    assert "baseline" in text


def test_the_school_model_is_contradicted_explicitly() -> None:
    """Correcting it in passing is not enough; it has to be named and refuted."""
    outcome = EYE_COLOUR.evaluate(eye_colour_genome(), ENV)
    assert "wrong" in outcome.certainty
    assert "blue-eyed parents" in outcome.certainty


def test_what_the_model_leaves_out_is_stated() -> None:
    outcome = EYE_COLOUR.evaluate(eye_colour_genome(), ENV)
    assert "13 more genes" in outcome.certainty
    assert "diploid" in outcome.certainty


# --- the trait interface -----------------------------------------------------


def test_probabilities_that_do_not_sum_to_one_are_refused() -> None:
    with pytest.raises(ValueError, match="sum to 1"):
        Outcome(trait="x", probabilities={"a": 0.5, "b": 0.2})


def test_an_empty_outcome_is_refused() -> None:
    with pytest.raises(ValueError, match="at least one possible value"):
        Outcome(trait="x", probabilities={})


def test_a_polygenic_baseline_must_be_a_distribution() -> None:
    with pytest.raises(ValueError, match="sum to 1"):
        PolygenicTrait(name="x", baseline={"a": 0.9, "b": 0.9}, contributions=[])


def test_a_rare_baseline_can_still_be_shifted() -> None:
    """A baseline near zero must not become unreachable.

    Log odds of exactly zero probability is negative infinity, which no finite
    effect can move. Clamping keeps "rare in this population" from silently
    becoming "impossible", which would make every rare-disease lesson wrong.
    """
    trait = PolygenicTrait(
        name="rare thing",
        baseline={"yes": 0.0, "no": 1.0},
        contributions=[
            Contribution(
                locus=Locus("HERC2", 12, "G"),
                variant_base="A",
                log_odds=__import__("integrase_sandbox").illustrative(20.0, "log odds"),
                affects="yes",
            )
        ],
    )
    unshifted = trait.evaluate(eye_colour_genome(herc2_variant="G"), ENV)
    shifted = trait.evaluate(eye_colour_genome(herc2_variant="A"), ENV)

    # The clamp puts a zero baseline at 1e-9, so log weight -20.7. A +20 effect
    # lands at -0.7 against the other class's 0, which is 33 percent. The number
    # matters less than the property: an effect can still move it.
    assert unshifted.probabilities["yes"] < 1e-6
    assert shifted.probabilities["yes"] > 0.3


def test_a_missing_gene_is_reported_rather_than_skipped_silently() -> None:
    trait = PolygenicTrait(
        name="x",
        baseline={"a": 0.5, "b": 0.5},
        contributions=[
            Contribution(
                locus=Locus("ABSENT", 0, "A"),
                variant_base="T",
                log_odds=__import__("integrase_sandbox").illustrative(1.0, "log odds"),
                affects="a",
            )
        ],
    )
    outcome = trait.evaluate(Genome(), ENV)
    assert any("not in this genome" in line for line in outcome.explanation)


def test_a_student_trait_needs_no_base_class() -> None:
    """Same property as `Process`, and it matters for the same reason."""

    class AlwaysBlue:
        @property
        def name(self) -> str:
            return "always blue"

        def evaluate(self, _genome: Genome, _environment: Environment) -> Outcome:
            return Outcome(trait="always blue", probabilities={"blue": 1.0})

    assert isinstance(AlwaysBlue(), Trait)


def test_describe_leads_with_the_distribution_not_the_mode() -> None:
    """Reducing a distribution to its mode is the move this module discourages."""
    text = EYE_COLOUR.evaluate(eye_colour_genome(), ENV).describe()
    header, _, rest = text.partition("What went into this:")
    del rest
    lines = [line for line in header.splitlines() if "%" in line]
    assert len(lines) == 3, f"expected all three values listed, got {lines}"
