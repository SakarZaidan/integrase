## What and why

<!-- What does this change? Why is it needed? The diff shows what; explain why. -->

## Traceability

<!-- Which subplan or ADR does this trace to? If none, say so: not everything
     needs one, but a change with no home in the plan is worth noticing. -->

Relates to: `planning/subplans/...`

## Tests

<!-- What covers this, including the edge cases? Per R-2.2, a test that cannot
     fail is not a test. What bug would these catch? -->

## Checklist

- [ ] `just check` passes locally
- [ ] Commit messages follow Conventional Commits
- [ ] Public items are documented, and doc comments explain *why*
- [ ] Docs updated in this PR if user-facing behaviour changed
- [ ] New dependencies justified above (what, why it beats alternatives, why not
      write it ourselves)

## Does this touch screening?

- [ ] **This PR modifies `crates/integrase-screen/`, the `Cleared` construction
      surface, or screening messages in `proto/`.**

If checked, this requires explicit human sign-off and will not be auto-merged.
Describe what changed about the gate and what you did to confirm it still fails
closed.

## Does this touch the confidence layer?

- [ ] **This PR modifies `Prediction`, `DomainVerdict`, or a `ModelCard`'s
      accuracy claims.**

A `ModelCard` states what a model's accuracy is. Changing a stated number is a
claim about the world, not a code change, and needs the evidence attached.
