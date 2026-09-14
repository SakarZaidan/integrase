# Architecture Decision Records

`FINAL-DECISION.md` is a point-in-time document. It records what was decided on 2026-09-14, with the
whole planning folder in view. Decisions made **after** that date go here.

Without this directory the planning folder rots in a specific, predictable way: `FINAL-DECISION.md`
stays frozen, real decisions accumulate in commit messages and PR threads, and six months later
nobody can reconstruct why something is the way it is. That is the failure the brief warns about, and
the `planning-sync` skill needs somewhere to write.

## When to write one

Write an ADR when a decision meets any of these:

- It changes something `FINAL-DECISION.md` or a subplan states.
- It resolves an open item listed at the bottom of a subplan.
- It would be expensive to reverse: a dependency in a correctness-critical path, a public API shape,
  a data format, a boundary mechanism.
- Someone might reasonably ask "why is it like this?" in six months.

Do **not** write one for routine work. A bug fix, a refactor that preserves behaviour, or a
dependency bump inside an existing policy is a commit message, not an ADR. An ADR directory full of
trivia is as useless as an empty one.

## Numbering and lifecycle

Files are `NNNN-short-kebab-title.md`, numbered sequentially from `0001`. Numbers are never reused
and files are never deleted.

Status moves in one direction: `proposed` → `accepted` → `superseded`. A superseded ADR stays in
place with a link to the one that replaced it, because the fact that we once decided otherwise, and
why we changed, is the useful part.

## Relationship to the subplans

An ADR that changes a subplan's conclusion **amends that subplan** and links both ways. This
happened once already during planning: subplan 21 exposed a gap in subplan 06's export gating, and
subplan 06 was amended in place rather than left contradicting its successor. That pattern is the
rule, not an exception.
