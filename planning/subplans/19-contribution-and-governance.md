# 19. Contribution and governance model

**Status:** decided
**Date:** 2026-09-14
**Decides:** the contributor journey, review standards, and how decisions get made
**Depends on:** subplans 03 (sustainability score 3/10), 07, 10
**Feeds:** `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `CODEOWNERS`, the `merge-decision` skill

---

## The problem this has to solve

Subplan 03 scored sustainability **3 out of 10**, and the evidence behind that score is specific
rather than pessimistic. Every dormant tool in subplan 02's tables followed the same arc: a small
group built something good, the grant ended or the students graduated, and the repository went
quiet. Cello, SBOLDesigner, paraSBOLv, TinkerCell. That is the field's norm.

A solo project has the same failure mode with fewer people. **So governance here is a bus-factor
problem wearing a paperwork costume**, and this subplan treats it as engineering rather than
process theatre.

### The three mechanisms that actually address it

1. **The planning folder is the succession plan.** Twenty-four subplans recording not just what was
   decided but why, and what was rejected, mean a future maintainer can understand the system
   without the original author. This is why the brief's insistence on keeping `planning/` alive is
   load-bearing rather than decorative, and why the `planning-sync` skill exists.
2. **Decisions are written down at the time.** `planning/decisions/` holds ADRs for everything
   decided after `FINAL-DECISION.md`. A decision that lives only in someone's head leaves with them.
3. **Low-friction onboarding.** `just` targets mirroring CI, a documented dev environment, and a
   `good-first-issue` pipeline. A contributor who spends two hours getting a build working does not
   come back.

---

## The honest state of governance today

**There is one contributor.** Pretending otherwise with an elaborate governance structure would be
the kind of process theatre the brief warns against. What exists now:

| Aspect | Now | When there are 2-5 | When there are more |
|---|---|---|---|
| Decisions | Owner decides, records an ADR | Owner decides, contributors consulted on ADRs | Lazy consensus, owner breaks ties |
| Review | Self-review against `pr-review`, findings recorded in the PR | One approval required | CODEOWNERS per subsystem |
| Required approvals | **0** (see below) | 1 | 1, 2 for screening |
| Release authority | Owner | Owner | Owner plus one maintainer |

### The required-approvals deviation, stated plainly

The brief asks for at least one review approval before merge. Branch protection currently sets
**0**, and this is a deliberate deviation with a mechanical cause: **GitHub forbids a PR author from
approving their own PR.** With exactly one contributor, requiring one approval makes every PR
permanently unmergeable.

What is enforced instead:

- **PRs are mechanically required.** No direct pushes to `main`, `enforce_admins` is true, and the
  owner is not exempt.
- **The review itself is enforced by process.** The `pr-review` checklist is run against the diff
  and the findings are recorded as a PR comment before merge. PRs #1 through #8 all carry one, and
  the review on #1 found two real defects (a dangling file reference and a case-sensitivity bug in
  a checker), which is the evidence that the step is not ceremonial.
- **It goes to 1 the moment a second contributor exists**, and to 2 for `integrase-screen`.

Recording this rather than quietly leaving protection weaker than the brief specifies is the point.

---

## The contributor journey

**Before a first PR, read three things:** `README.md`, `CONTRIBUTING.md`, and
`planning/subplans/07-engineering-principles.md`. The third is unusual to require, and it is
required because the two tier-0 rules are not obvious from reading the code and a PR violating them
will be rejected regardless of quality.

1. **Find work.** `good-first-issue` for onboarding, `help-wanted` for real gaps. Every issue states
   which subsystem it touches and links the relevant subplan.
2. **Set up.** `just setup` installs toolchains. `just check` runs what CI runs. Documented for all
   three languages, since a contributor may only care about one.
3. **Branch.** `type/short-description`, matching Conventional Commit types: `feat/`, `fix/`,
   `docs/`, `refactor/`, `perf/`, `test/`, `chore/`.
4. **Commit.** Conventional Commits, enforced in CI. The body explains **why**, because the diff
   already shows what.
5. **PR.** The template asks: what does this change, why, which subplan does it trace to, what tests
   cover it, and does it touch screening.
6. **Review.** Against the `pr-review` skill. Screening changes get additional scrutiny by rule
   (R-7.2), detectable by path because screening is its own crate.
7. **Merge.** Squash only, linear history. Merge commits and rebase are disabled.

### Why squash-only, since the brief asks for the reasoning

A squashed commit is one logical change with one message, which makes `git bisect` meaningful and
makes changelog generation from Conventional Commits mechanical. The cost is losing intermediate
commits, which for a project where PRs are one coherent unit of work is not a real loss. The
alternative, merge commits, produces a history where bisect lands on broken intermediate states.

---

## Review standards

The `pr-review` skill implements this. The bar, in order of what gets checked:

1. **Correctness against the stated intent.** Does it do what the description claims?
2. **Tier-0 compliance.** Does anything return a bare predicted value, or accept an unscreened
   design on an export path? Automatic request-changes.
3. **Test coverage for the actual behaviour**, including the edge cases from subplan 11. Per R-2.2,
   tests that cannot fail are removed rather than counted.
4. **Screening scrutiny**, if the diff touches `crates/integrase-screen/` or `proto/` screening
   messages.
5. **Performance**, if it touches a path subplan 09 gates. R-5.4 requires a before-and-after
   measurement for complexity added in the name of speed.
6. **Documentation**, if public API or user-facing behaviour changed. Same PR, not a follow-up.
7. **Voice**, per the brief's section 11 and `tools/check-voice.sh`.

### What always routes to human sign-off

Per R-7.3, the `merge-decision` skill never auto-merges these regardless of how clean the diff is:

- Anything in `crates/integrase-screen/`
- Any change to the `Cleared` construction surface, including the `#[cfg(test)]` constructor
- License changes
- Breaking API changes
- Changes to `ModelCard` accuracy claims

The first two are obvious. The last is less so: a `ModelCard` states what a model's accuracy is, and
a PR quietly improving a stated number is a claim about the world, not a code change.

---

## Issue triage

| Label | Meaning |
|---|---|
| `subsystem:core`, `:sim`, `:screen`, `:sbol`, `:viz`, `:sdk` | Which part. Maps to CODEOWNERS. |
| `type:bug`, `:feature`, `:docs`, `:perf` | What kind. |
| `needs:triage` | Not yet assessed. |
| `needs:decision` | Blocked on an owner call. Becomes an ADR. |
| `security` | **Private handling.** See below. |
| `good-first-issue` | Scoped, documented, with a pointer to the relevant subplan. |

**Security issues are never filed as public issues.** `SECURITY.md` directs them to GitHub Security
Advisories. For screening-bypass reports specifically, subplan 24's disclosure policy departs from
the usual timeline, and that departure is stated in `SECURITY.md` up front so a reporter knows
before reporting rather than after.

---

## Code of Conduct

Contributor Covenant as the base, with three project-specific additions that make it enforceable
rather than boilerplate:

1. **Named enforcement.** The project owner, with contact details. A CoC with no named enforcer is a
   document, not a policy.
2. **A stated escalation path**: warning, temporary ban, permanent ban, with who decides each.
3. **A dual-use clause.** Requests for help using Integrase to design something harmful, or attempts
   to use the issue tracker to solicit that help, are grounds for immediate removal. This is
   specific to this project and belongs in the CoC rather than only in a threat model, because it
   governs conduct rather than architecture.

---

## CODEOWNERS

Established now even with one contributor, so the pattern is right from the start.

```
*                               @SakarZaidan
/crates/integrase-screen/       @SakarZaidan
/proto/                         @SakarZaidan
/planning/                      @SakarZaidan
/.github/workflows/             @SakarZaidan
```

The redundancy is intentional: when a second maintainer arrives, screening ownership is a one-line
change rather than a restructure. Real team-based ownership waits on the organisation (R-001).

---

## Open items

- [ ] Write `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md`, post-checkpoint
- [ ] Raise required approvals to 1 when a second contributor exists
- [ ] Seed `good-first-issue` items before any public announcement. An empty tracker on a new repo
      converts interested visitors into nothing.
