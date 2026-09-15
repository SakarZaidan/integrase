---
name: pr-review
description: Review a pull request end to end against Integrase's engineering standards. Use when asked to review a PR, before merging any PR including your own, or when a diff needs checking against the project's rules. Produces a specific review with a verdict, not a summary.
---

<!-- voice-check: teaches-by-example -->

This document quotes the patterns it forbids, as negative examples. The voice
checker skips it for that reason; the prose outside the examples still follows
the rules.
# PR review

## The bar

A review that approves something it has not checked is worse than no review,
because it launders an unchecked change through a process that looks rigorous.

Work through every section below. If you cannot check one, say so in the review
rather than staying silent about it. "I did not verify the performance claim
because there is no benchmark in the diff" is a useful review line.

## Self-review counts

This project currently has one contributor, so the author and the reviewer are
usually the same person. That is a reason to run the checklist explicitly and
record the findings, not a reason to skip it. Branch protection requires 0
approvals only because GitHub forbids self-approval, and the review is enforced
by process instead.

Evidence this is not ceremonial: the self-review on PR #1 found two real defects,
a dangling file reference and a case-sensitivity bug in a checker written in that
same PR.

## 1. Correctness against stated intent

Read the PR description first, then the diff. Does the change do what it says?

Watch for the common failure: a PR that does what it says **plus** something
else unmentioned. Scope creep in a diff is a review finding, not a bonus.

## 2. The two tier-0 rules

**Check these before anything else. A violation is an automatic request-changes
with no further discussion needed.**

**R-0.1. No bare predicted values.** Search the diff for any function returning
`f64`, `float`, or `double` for a modelled or estimated quantity. It must be
`Prediction<T>`. This applies to convenience wrappers, Python bindings, and
proto messages.

**R-0.2. Export paths take only `Cleared<T>`.** Any new function that writes a
design toward synthesis must accept `Cleared<T>`. Also check nobody added a
public constructor for `Cleared` outside `#[cfg(test)]`, though CI guards this
too.

Remember the distinction: interchange export (SBOL round-trip, working files,
figures) is **not** gated. Only synthesis-bound paths are. A reviewer who
demands screening on `write_sbol` has misread the rule.

## 3. Does it touch screening?

If the diff includes `crates/integrase-screen/`, the `Cleared` surface, or
screening messages in `proto/`, this PR gets additional scrutiny and **cannot be
auto-merged**. Route to `merge-decision`, which sends it to human sign-off.

For screening changes specifically:

- Does every failure path still refuse? Unreachable, timeout, malformed,
  partial, panic. There must be no code path where "could not determine" becomes
  "allowed."
- Does `NullBackend` still refuse everything?
- Is the fault-injection test updated if a new failure mode was added?
- Did the policy version still get recorded in the verdict?

## 4. Tests

Coverage floors are in `CONTRIBUTING.md`. The floors are a proxy. This is the
real check:

**Does each new test have a plausible answer to "what bug does this catch?"**

Tests asserting that a constructor constructs, a getter gets, or arithmetic works
inside a wrapper get flagged for removal, not counted. Per R-2.2, a test that
cannot fail is not a test.

Also check:

- Sequence manipulation and numerical code: is there a property test? Example
  tests miss whole bug classes there.
- Format round-trips: is the comparison **semantic**? A triple-equality assertion
  on SBOL 2 to 3 conversion fails on correct behaviour and will get weakened
  until it passes.
- A new checker, linter, or validator: was it tested against input it should
  **reject**, not only input it should accept? R-2.5.
- Edge cases from subplan 11 where relevant.

## 5. Performance

Only if the diff touches a gated path.

- A performance claim in the description with no benchmark in the diff is a
  finding. R-5.1.
- Complexity added in the name of speed with no before-and-after measurement is a
  finding. R-5.4, and it is one of the strongest tells of generated code.
- A changed regression threshold with no written justification is a
  request-changes.

## 6. Dependencies

Any new dependency needs justification in the description: what it does, why it
beats alternatives, why not write it ourselves.

In correctness-critical paths (`core`, `sim`, `screen`) also check it is
maintained within 12 months, adopted enough that bugs surface elsewhere first,
permissively licensed, and not a second implementation of something already
depended on.

**Check data licences too.** This is the one most reviewers miss. Registry data
is CC BY-SA and cannot be baked into an Apache-2.0 crate.

## 7. Documentation

- Public items documented. `missing_docs` is denied, so CI catches absence, but
  CI cannot catch a doc comment that restates the signature.
- Doc comments explain **why**. `/// Returns the reverse complement.` on
  `reverse_complement()` is noise.
- User-facing behaviour changed: docs updated **in this PR**, not a follow-up.
- Coordinate conventions stated where relevant.

## 8. Voice

Apply `writing-voice` to the PR description, the commit messages, and any prose
in the diff. `tools/check-voice.sh` catches the mechanical subset; you are
checking tone and specificity.

Commit messages: Conventional Commits, and a body that explains why rather than
narrating the diff.

## Output format

```markdown
## Review

**Verdict: approve | request changes | comment**

### Findings
1. `path/to/file.rs:42` — what is wrong, why it matters, what to do instead.

### Checked and clean
- Brief list, so the author knows what was actually examined.

### Not checked
- What you could not verify and why.
```

Be specific about location. "Some tests are weak" is not actionable. "The test at
`seq.rs:214` asserts only that the constructor returns Ok, which cannot fail
given the input" is.

## Two worked examples

**Good finding**

> `crates/integrase-sim/src/rbs.rs:88` — `free_energy()` returns `f64` for a
> modelled quantity. This violates R-0.1. It needs to return
> `Prediction<f64>`, even if the model is uncalibrated, in which case
> `Prediction::uncalibrated` is the right constructor and the domain verdict
> reports honestly that there is no calibration evidence.

Names the location, names the rule, and says what to write instead.

**Bad finding**

> The error handling could be more robust here.

Vague, unactionable, and uses a banned word. If the concern is real, it is
something like: "`import.rs:130` discards the parse error with `let _ =`. R-1.3
forbids swallowing errors. Either propagate it or log at warn with the error
attached."

## When to approve

Approve when every section above is checked and the findings are either fixed or
genuinely minor. Do not approve to be agreeable, and do not request changes over
style preferences the project has not codified. If something bothers you but no
rule covers it, say so as a comment and propose the rule.
