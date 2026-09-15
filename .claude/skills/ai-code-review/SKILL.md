---
name: ai-code-review
description: Review code for the specific failure modes of low-effort AI-generated code. Use on any diff, including your own and the project owner's. Complements pr-review, which checks correctness and project rules; this one checks whether the code is doing work it does not need to do.
---

<!-- voice-check: teaches-by-example -->

This document quotes the patterns it forbids, as negative examples. The voice
checker skips it for that reason; the prose outside the examples still follows
the rules.
# AI code review

## Why this exists

The project owner will sometimes generate code with AI assistance, and so will
contributors, and so will I. Generated code has characteristic failure modes that
are different from the ones human code has, and it needs the same scrutiny as any
other contribution rather than less because of who or what wrote it.

**Apply this honestly and symmetrically.** Being lenient on the owner's code or
on your own defeats the point. Equally, do not flag code as "AI-looking" when it
is simply correct and unremarkable. The findings below are defects on their own
terms; the origin of the code is irrelevant to whether they are real.

## What to look for

### 1. Complexity for a problem that does not exist

The most common and most expensive tell.

```rust
// Flagged
pub struct SequenceCache {
    entries: HashMap<DesignId, Arc<RwLock<CachedSequence>>>,
    eviction: LruPolicy,
    max_bytes: usize,
}
```

Ask: is there a profile showing sequence lookup was a bottleneck? R-5.4 requires
a before-and-after measurement for complexity added in the name of performance.
If there is none, this is machinery solving a problem nobody measured, and it
will need maintaining forever.

The same applies to trait hierarchies with one implementor, generics over a type
parameter only ever instantiated once, and configuration for a value nobody has
wanted to change.

### 2. Defensive code for cases that cannot occur

```rust
// Flagged
pub fn feature_count(design: &Design) -> usize {
    if design.features.is_empty() {
        return 0;
    }
    design.features.len()
}
```

The branch cannot change the result. More seriously:

```rust
// Flagged
let seq = match design.sequence() {
    Some(s) => s,
    None => return Err(Error::NoSequence),
};
```

If `sequence()` returns `Sequence` rather than `Option<Sequence>`, this is
handling an impossible state, and it makes a reader believe the state is
possible.

**The distinction that matters:** defending against invalid *input* is correct.
Defending against states the type system already rules out is noise that
obscures the real invariants.

### 3. Comments that restate the code

```rust
// Flagged: increment the counter
counter += 1;

// Flagged: loop over the features
for feature in &design.features {
```

Every comment should answer **why**, or state an invariant, a unit, or a
convention. Per R-4.2. Good:

```rust
// 0-based half-open, matching the rest of the crate. GenBank's 1-based
// inclusive coordinates are converted in the adapter, not here.
```

### 4. Swallowed errors

Explicitly R-1.3, and the single most reliable tell.

```rust
// Flagged
let _ = writer.flush();

// Flagged
if let Ok(parsed) = parse(input) {
    use_it(parsed);
}
// ... and the error is gone
```

Either handle it, propagate it, or log it at `warn` with the error attached.
Silence is not handling.

In Python, the same thing wears different clothes:

```python
try:
    result = screen(design)
except Exception:
    pass
```

In screening code this is not a style issue, it is a fail-open bug.

### 5. Tests that assert trivial properties

```rust
// Flagged
#[test]
fn test_new() {
    let s = Sequence::new(b"ACGT".to_vec(), Alphabet::Dna);
    assert!(s.is_ok());
}
```

What bug does this catch? If `Sequence::new` were completely broken it would
fail, but so would every other test. Per R-2.2, a test that cannot fail is not a
test.

Compare with one that earns its place:

```rust
#[test]
fn rejects_invalid_residue_with_position() {
    let err = Sequence::new(b"ACGTX".to_vec(), Alphabet::Dna).unwrap_err();
    match err {
        CoreError::InvalidResidue { byte, offset, .. } => {
            assert_eq!(byte, b'X');
            assert_eq!(offset, 4, "offset must point at the offending byte");
        }
        other => panic!("wrong variant: {other:?}"),
    }
}
```

That one catches an off-by-one in error reporting, which is a real bug class.

**Related tell:** a checker or validator tested only against input it should
accept. R-2.5 requires both directions. A validator that has never failed is
indistinguishable from one that cannot.

### 6. Inconsistency with the surrounding code

Generated code often looks reasonable in isolation and foreign in place. Check:

- Error type: does it use the crate's `thiserror` enum or invent a new pattern?
- Naming: `as_`, `to_`, `into_` used per convention?
- Coordinates: 0-based half-open like everything else?
- Roles: ontology IRIs, or did someone reintroduce an enum?
- Module placement: does `integrase-core` now depend on something internal?

### 7. Speculative API

Public functions nothing calls, parameters nothing passes, trait methods nothing
implements, `pub` on items only used within the crate.

This project already removed one: a test-only constructor for `Cleared<T>` that
the plan called for and nothing needed yet. It was added "for later," which is
the exact pattern.

### 8. Documentation that describes a different system

Doc comments mentioning parameters that do not exist, or referencing behaviour
from an earlier draft. Usually a sign the code changed and the comment did not.

## What is not a finding

Do not flag:

- Correct code that happens to be simple. Simple is the goal.
- Verbose but genuinely necessary error handling.
- Comments explaining genuinely non-obvious biology or numerics. Those are the
  most valuable comments in this repository.
- Code that follows a project convention you personally would have written
  differently.
- Consistent formatting. That is `rustfmt`'s job, not yours.

## Output

Use `pr-review`'s format. Each finding needs a location, what is wrong, and what
to do instead. Add the category so patterns are visible across a review:

> `crates/integrase-sim/src/rbs.rs:142` — **defensive code for an impossible
> state.** `model.card()` returns `&ModelCard`, not `Option`, so the `None` arm
> cannot execute. Remove the match and use the value directly. As written it
> suggests to a reader that a card can be absent, which is exactly the
> invariant this type was designed to guarantee.
