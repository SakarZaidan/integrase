---
name: planning-sync
description: Keep planning/ current when a change affects architecture, phasing, or a recorded decision. Use proactively during development, not only when asked. Defines what counts as significant enough to trigger, so it is not invoked for every trivial change.
---

# Planning sync

## What this prevents

`planning/` is 32 subplans plus `FINAL-DECISION.md` and `ARCHITECTURE.md`. Its
value depends entirely on being true. A planning folder that is 80% accurate is
worse than none, because a reader cannot tell which 80%, and the confident-wrong
parts are the ones they act on.

This project is also a single-maintainer project with a sustainability score of
6.5. **The planning folder is the succession plan.** A future maintainer
reconstructs intent from it. Letting it drift converts the project's main
bus-factor mitigation into a liability.

## When this fires

**Fires:**

| Trigger | Example |
|---|---|
| A recorded decision is reversed or changed | Scope restored after being cut |
| A measurement contradicts a documented number | Index memory measured at 3.9 B/base against a documented 39.4 |
| A subsystem boundary, crate, or interface changes | A new crate, or a dependency edge added |
| Phase ordering changes | Something moves from Phase 2 to Phase 1 |
| An open item is resolved | R-002 resolved by finding SecureDNA |
| A new external dependency is adopted or rejected at the architecture level | Adopting `sbol-rs` |
| A rule in subplan 07 gains an exception | The `#[cfg(test)]` carve-out for `Cleared` |
| A claimed capability turns out to be infeasible, or an excluded one turns out to be feasible | Either direction |

**Does not fire:**

- Bug fixes that restore documented behaviour
- Refactors preserving the public interface
- Test additions that do not reveal a wrong assumption
- Dependency version bumps inside existing policy
- Documentation wording
- Formatting

The test: **would a reader of `planning/` now believe something false?** If yes,
fire. If no, do not.

## What to update

Work outward from the change. Not every trigger touches every file.

1. **The subplan that owns the decision.** Amend **in place** with a dated
   revision note. Do not silently edit and do not delete the old reasoning: the
   fact that we once decided otherwise, and why we changed, is the useful part.

   The established format:

   ```markdown
   > **CORRECTED 2026-09-14.** The original version concluded X. That conclusion
   > was wrong, because Y. The original measurement is kept below because it is
   > still true of Z; the conclusion drawn from it is replaced in the section
   > that follows.
   ```

2. **Subplans that cite it.** `grep -rn` for the claim. A correction that fixes
   one document and leaves three citing the old number has made things worse,
   because now they disagree.

3. **`ARCHITECTURE.md`**, if structure changed. Re-run
   `./tools/check-mermaid.sh` afterwards; diagrams break easily under edits.

4. **`FINAL-DECISION.md`**, if a resolved tension is re-resolved. Use a revision
   banner and show the correction rather than rewriting history.

5. **`PROGRESS.md`.** Dated entry: what changed, why, what was wrong before.

6. **`TODO.md`.** Check off, add what is newly discovered, remove what became
   obsolete **with a note** rather than silently.

7. **`REQUESTS.md`.** If an owner-only dependency appeared or resolved. Mark
   resolved **in place** and move it to the RESOLVED section so placement matches
   status.

8. **An ADR in `planning/decisions/`**, for a decision made after
   `FINAL-DECISION.md` that is not simply an amendment to one subplan.

## Worked example

Subplan 21 exposed a gap in subplan 06: taken literally, `NullBackend` refusing
everything meant an unconfigured user could not save their own design.

What fired:

- **Subplan 06** amended in place with a dated section adding the
  interchange/synthesis-bound split, and the code example updated so
  `export_sbol` became `export_for_synthesis`.
- **Subplan 21** records it as the finding that prompted the amendment.
- **`FINAL-DECISION.md`** tension 7 updated, since it had resolved this question
  with an answer now known to be incomplete.
- **`PROGRESS.md`** dated entry.
- Nothing in `ARCHITECTURE.md` needed changing, because the diagram already
  showed a distinction between the two export paths.

Note what was not done: subplan 06 was not rewritten to look as though it had
always said this.

## Verify after

```bash
./tools/check-voice.sh
./tools/check-mermaid.sh
grep -rn "<the old claim>" planning/   # should return only the correction itself
```

The third matters most. It catches the failure this skill exists to prevent.

## The rule about corrections

**Corrections are recorded, not hidden.** Editing a document so it looks like it
was always right destroys the reader's ability to calibrate how much to trust it,
and this project's entire thesis is about honest calibration. A planning folder
containing five recorded corrections is more trustworthy than one containing
none, and the folder currently records six.
