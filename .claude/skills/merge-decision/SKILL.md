---
name: merge-decision
description: Decide whether a reviewed PR actually merges. Use after pr-review, as the final gate before merging. Distinct from review: this checks that required conditions genuinely hold and routes anything needing human authority to the project owner.
---

# Merge decision

`pr-review` judges whether the change is good. This decides whether it merges
**now**, and by whose authority. They are separate steps because a good change
can still need a human to sign it off.

**The default is caution.** When genuinely uncertain whether something needs
human sign-off, ask rather than merge. The cost of asking is a short delay. The
cost of merging something that needed sign-off is not recoverable in the same
way.

## Never auto-merge these, regardless of how clean the diff is

Route to the project owner with a short note on what needs deciding.

| Category | Why |
|---|---|
| Anything in `crates/integrase-screen/` | Highest-consequence component. A false negative here is the worst failure the system can produce. |
| Any change to the `Cleared` construction surface, including the `#[cfg(test)]` constructor | This is the export guarantee. Widening it is a decision, not a refactor. |
| Licence changes | Legal consequences outside engineering authority. |
| Breaking API changes | Affects every downstream user, and pre-1.0 status does not make it costless. |
| Changes to a `ModelCard`'s accuracy claims | A `ModelCard` states what a model's accuracy **is**. Changing a stated number is a claim about the world, not a code change, and needs evidence attached. |
| Anything touching `proto/` wire compatibility | Breaks users silently and at a distance. |
| Adding a dependency to `integrase-screen` | That crate depends only on `integrase-core` deliberately. Every addition widens the audit surface of the component least able to afford it. |

"A bot opened it" is not an exemption. Dependabot bumping a dependency inside
`integrase-screen` still routes to a human.

## Verify, do not assume

The distinction that makes this skill worth having: check that conditions
**hold**, not that they were **triggered**.

**Checks actually passed.** Not "CI ran." Confirm each required check reports
success. A cancelled, skipped, or still-pending check is not a pass.

```
gh pr checks <N>
```

Pay attention to skipped jobs. A path filter that caused the screening suite to
skip is a reason to stop, because that suite is required and non-skippable by
design.

**The review requirement is genuinely satisfied.** With one contributor, GitHub
reports 0 required approvals. That does not mean no review happened; it means
the review is enforced by process. Confirm a `pr-review` was actually performed
and its findings recorded on the PR. If not, the answer is to review it, not to
merge it.

**The branch is up to date with the base.** Required in settings, but check: a
green CI run against a stale base tested a combination that will not exist after
merge.

**Findings were addressed.** If `pr-review` requested changes, confirm they
landed rather than were argued away in a comment.

## Merge mechanics

Squash only, delete the branch. Merge commits and rebase are disabled. The squash
commit message is the PR title and body, so it inherits the voice rules.

## Output

```markdown
## Merge decision

**Decision: merge | hold for human sign-off | blocked**

**Checks:** <N> passed, <N> skipped (which, and why that is acceptable)
**Review:** performed by <who>, findings <addressed | outstanding>
**Base:** up to date | stale
**Sign-off needed:** no | yes, because <category from the table>

<One or two sentences of reasoning.>
```

## Examples

**Merging**

> Decision: merge.
> Checks: 11 passed, 0 skipped. Review performed, two findings, both fixed in
> 8f4ba0c. Branch up to date. No category requiring sign-off: touches
> `integrase-core` and its tests only.

**Holding**

> Decision: hold for human sign-off.
> The diff adds a `timeout_ms` parameter to `ScreeningBackend::screen`. The
> change looks correct and the fault-injection tests still pass. It touches
> `crates/integrase-screen/`, so it routes to the owner regardless. Specifically
> worth their attention: a configurable timeout is a knob that can be set high
> enough to matter operationally, and the default belongs to them rather than to
> me.

Note what the second one does. It does not just cite the rule; it says what the
human should actually think about.

**Blocked**

> Decision: blocked.
> The screening gateway job shows "skipped" because a `paths` filter was added
> to `ci.yml` in this same PR. A change that makes the required screening check
> skip itself cannot be merged on the strength of that check passing. Remove the
> filter or justify it separately.
