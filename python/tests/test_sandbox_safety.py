"""Tests that hold the sandbox's boundaries.

These are the tests most worth having and least likely to be run by anyone
thinking about them. They assert structural properties: what the sandbox
contains, what it cannot reach, and that nothing in it claims to be a
prediction.

A filter guesses at intent and fails. Curation decides what exists, and these
check that the curation is real rather than stated.
"""

from __future__ import annotations

import ast
import pathlib

import pytest

from integrase_sandbox.conditions import eye_colour_genome, hbb_genome
from integrase_sandbox.molecules import (
    COMPLEMENTS,
    Chemistry,
    Group,
    Molecule,
    Pocket,
    Verdict,
    fit,
)
from integrase_sandbox.organism import reference_genome
from integrase_sandbox.safety import CONTENT, describe_content
from integrase_sandbox.trials import PASS_RATE, Phase, develop, overall_success_rate

PACKAGE = pathlib.Path(__file__).resolve().parents[1] / "src" / "integrase_sandbox"

# --- the boundary ------------------------------------------------------------


def test_the_sandbox_never_imports_the_design_tool() -> None:
    """The structural claim the whole safety argument rests on.

    If the sandbox could import `integrase`, it could reach the export path, and
    the screening gate would need to reason about sandbox content. It cannot, so
    it does not.

    Checked by parsing every module rather than by trying an import, because an
    import inside a function would not show up in a runtime check and would be
    exactly the way this property gets lost.
    """
    offenders: list[str] = []
    for path in sorted(PACKAGE.glob("*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                names = [a.name for a in node.names]
            elif isinstance(node, ast.ImportFrom):
                names = [node.module or ""]
            else:
                continue
            for name in names:
                root = name.split(".")[0]
                if root == "integrase":
                    offenders.append(f"{path.name}:{node.lineno} imports {name}")

    assert not offenders, (
        "the sandbox must not reach the design tool, or the screening gate has "
        "to reason about teaching content:\n" + "\n".join(offenders)
    )


def test_no_module_writes_a_file_or_opens_a_socket() -> None:
    """The sandbox computes; it does not reach out.

    A teaching tool that wrote files or opened connections would be one a school
    network administrator has to audit, and there is no reason for it to do
    either.
    """
    banned = {"socket", "urllib", "requests", "http", "subprocess", "shutil"}
    offenders: list[str] = []
    for path in sorted(PACKAGE.glob("*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                names = [a.name for a in node.names]
            elif isinstance(node, ast.ImportFrom):
                names = [node.module or ""]
            else:
                continue
            for name in names:
                if name.split(".")[0] in banned:
                    offenders.append(f"{path.name}:{node.lineno} imports {name}")
    assert not offenders, "\n".join(offenders)


def test_every_shipped_gene_is_accounted_for() -> None:
    """Content in the sandbox that nobody listed is content nobody decided on."""
    listed = {item.name for item in CONTENT}
    shipped = set()
    for genome in (reference_genome(), hbb_genome(), eye_colour_genome()):
        shipped.update(g.name for g in genome.genes)

    unlisted = shipped - listed
    assert not unlisted, f"genes present but not declared in safety.CONTENT: {unlisted}"


def test_only_one_piece_of_content_is_not_invented() -> None:
    """And it is named, with the reason it is an exception.

    If this count ever rises, someone added real sequence without the decision
    being visible, which is the failure this test exists to make loud.
    """
    real = [item for item in CONTENT if not item.invented]
    assert len(real) == 1, f"expected one real fragment, got {[r.name for r in real]}"
    assert real[0].name == "HBB"
    assert "no capability" in real[0].note


def test_the_curation_statement_says_there_is_no_export_path() -> None:
    text = describe_content()
    assert "cannot become physical DNA" in text or "no export path" in text
    assert "no pathogen sequence" in text


# --- the puzzle refuses to be a prediction -----------------------------------


def test_every_fit_carries_the_uncalibrated_verdict() -> None:
    pocket = Pocket("p", (Group(0, Chemistry.HYDROPHOBIC),))
    molecule = Molecule("m", (Group(0, Chemistry.HYDROPHOBIC),))
    assert fit(molecule, pocket).verdict is Verdict.UNCALIBRATED


def test_the_verdict_enum_has_exactly_one_member() -> None:
    """A second member would be somewhere to put "calibrated".

    Keeping it at one makes "this is never a prediction" a fact about the type
    rather than a convention someone can quietly stop following.
    """
    assert len(Verdict) == 1


def test_the_verdict_reads_as_a_warning_not_a_label() -> None:
    text = str(Verdict.UNCALIBRATED)
    assert "teaching puzzle" in text
    assert "not a prediction" in text


def test_no_result_uses_the_language_of_efficacy() -> None:
    """No field or output may read as a physical or clinical quantity.

    A number called "affinity" or "potency" would be mistaken for one, whatever
    the docstring next to it said.
    """
    banned = ("affinity", "potency", "ic50", "efficacy_score", "binding_energy")
    pocket = Pocket("p", (Group(0, Chemistry.HYDROPHOBIC),))
    molecule = Molecule("m", (Group(0, Chemistry.HYDROPHOBIC),))
    result = fit(molecule, pocket)

    surface = set(dir(result)) | set(result.describe().lower().split())
    for word in banned:
        assert not any(word in str(s).lower() for s in surface), f"{word} appears in a result"


def test_a_developed_candidate_is_never_congratulated() -> None:
    """A student who "cures" something has solved a puzzle, not made a discovery.

    Presenting it as discovery teaches a false idea of what medical research
    requires, so the approval text has to say what it is.
    """
    pocket = Pocket("p", tuple(Group(i, Chemistry.HYDROPHOBIC) for i in range(4)))
    molecule = Molecule("m", tuple(Group(i, Chemistry.HYDROPHOBIC) for i in range(4)))
    perfect = fit(molecule, pocket)

    for seed in range(400):
        result = develop(perfect, seed=seed)
        if result.approved:
            text = result.describe()
            assert "not a discovery" in text
            assert "would help anyone" in text
            return
    pytest.fail("no seed in 400 reached approval, which is itself suspicious")


# --- the attrition model -----------------------------------------------------


def test_the_published_phase_rates_are_transcribed_correctly() -> None:
    """These are the only measured numbers in the sandbox, so they get checked."""
    assert float(PASS_RATE[Phase.PHASE_I]) == 0.52
    assert float(PASS_RATE[Phase.PHASE_II]) == 0.289
    assert float(PASS_RATE[Phase.PHASE_III]) == 0.58


def test_the_phase_rates_multiply_to_roughly_the_published_overall_rate() -> None:
    """8.7 percent against a published 8.1, which is a transcription check.

    If someone edits one rate, this drifts, and a drifted number here would
    quietly change the lesson from "most drugs fail" to something else.
    """
    assert overall_success_rate() == pytest.approx(0.087, abs=0.005)


def test_a_perfect_fit_does_not_rescue_the_clinical_odds() -> None:
    """The lesson. Chemistry is not the hard part.

    A perfect puzzle solution still faces the 28.9 percent Phase II rate
    unchanged, because whether a target is the right target is not a question
    about the molecule.
    """
    pocket = Pocket("p", tuple(Group(i, Chemistry.HYDROPHOBIC) for i in range(4)))
    molecule = Molecule("m", tuple(Group(i, Chemistry.HYDROPHOBIC) for i in range(4)))
    perfect = fit(molecule, pocket)
    assert perfect.complementarity == 1.0

    approvals = sum(1 for seed in range(500) if develop(perfect, seed=seed).approved)
    rate = approvals / 500
    assert rate < 0.20, f"a perfect fit approved {rate:.1%} of the time, which is too kind"
    assert rate > 0.02, f"a perfect fit approved {rate:.1%} of the time, which is hopeless"


def test_a_poor_fit_fails_earlier_than_a_good_one() -> None:
    """The puzzle has to matter, or solving it teaches nothing."""
    pocket = Pocket("p", tuple(Group(i, Chemistry.HYDROPHOBIC) for i in range(4)))
    good = fit(Molecule("g", tuple(Group(i, Chemistry.HYDROPHOBIC) for i in range(4))), pocket)
    bad = fit(Molecule("b", tuple(Group(i, Chemistry.POSITIVE) for i in range(4))), pocket)

    good_passes = sum(
        1 for s in range(300) if develop(good, seed=s).reached is not Phase.PRECLINICAL
    )
    bad_passes = sum(1 for s in range(300) if develop(bad, seed=s).reached is not Phase.PRECLINICAL)
    assert good_passes > bad_passes


def test_a_run_is_reproducible_from_its_seed() -> None:
    """A teacher needs to be able to hand a class the same story."""
    pocket = Pocket("p", (Group(0, Chemistry.HYDROPHOBIC),))
    candidate = fit(Molecule("m", (Group(0, Chemistry.HYDROPHOBIC),)), pocket)
    assert develop(candidate, seed=99).describe() == develop(candidate, seed=99).describe()


def test_every_failure_explains_itself() -> None:
    """A failure with no explanation teaches that drug development is arbitrary."""
    pocket = Pocket("p", (Group(0, Chemistry.HYDROPHOBIC),))
    candidate = fit(Molecule("m", (Group(0, Chemistry.NEGATIVE),)), pocket)
    for seed in range(50):
        result = develop(candidate, seed=seed)
        if result.failure is not None:
            assert len(result.failure.explanation) > 40
            assert result.failure.reason


# --- the chemistry puzzle ----------------------------------------------------


def test_complementarity_is_symmetric() -> None:
    """If A pairs with B then B pairs with A, or the puzzle is unsolvable."""
    for chemistry, partner in COMPLEMENTS.items():
        assert COMPLEMENTS[partner] is chemistry


def test_hydrophobic_pairs_with_itself() -> None:
    """The one that surprises students, and the one worth getting right.

    Two greasy surfaces stick together because water pushes them together, not
    because they attract each other.
    """
    assert COMPLEMENTS[Chemistry.HYDROPHOBIC] is Chemistry.HYDROPHOBIC


def test_a_group_extending_past_the_pocket_is_counted() -> None:
    """Adding groups until everything matches is the obvious wrong strategy."""
    pocket = Pocket("p", (Group(0, Chemistry.HYDROPHOBIC),))
    oversized = Molecule(
        "m",
        (
            Group(0, Chemistry.HYDROPHOBIC),
            Group(1, Chemistry.HYDROPHOBIC),
            Group(2, Chemistry.HYDROPHOBIC),
        ),
    )
    result = fit(oversized, pocket)
    assert result.overhang == 2
    assert "collide" in result.describe()


def test_editing_a_molecule_leaves_the_original_alone() -> None:
    original = Molecule("m", (Group(0, Chemistry.HYDROPHOBIC),))
    edited = original.with_group(Group(0, Chemistry.POSITIVE))
    assert original.at(0) is not None
    assert original.at(0).chemistry is Chemistry.HYDROPHOBIC  # type: ignore[union-attr]
    assert edited.at(0).chemistry is Chemistry.POSITIVE  # type: ignore[union-attr]
