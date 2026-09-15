# 07. Engineering principles and rules

**Status:** decided
**Date:** 2026-09-14
**Decides:** the non-negotiable standards this codebase holds itself to
**Feeds:** `CONTRIBUTING.md`, the `pr-review` and `ai-code-review` skills, subplans 12, 17, 18

These are rules an engineer can be held to, not aspirations. Each one is stated so that a reviewer
can say "this violates rule N" and be objectively right. Where a rule has an escape hatch, the
escape hatch is written down, because an absolute rule with an undocumented exception is a rule
people learn to ignore.

---

## Tier 0: the two that are never negotiable

Everything else on this page can be argued about in a PR. These two cannot. They are the project's
differentiators expressed as code (subplan 06), and weakening either one removes the reason the
project exists.

### R-0.1: No API returns a bare value for a predicted quantity

Any function producing a modelled, estimated, or inferred quantity returns `Prediction<T>`, never
`T`. This includes convenience wrappers, test helpers that leak into public surface, and Python and
gRPC bindings.

**Escape hatch:** none. A reviewer who sees `fn predict_strength(...) -> f64` requests changes
without further discussion.

**Why it is absolute:** the moment one convenient accessor returns a bare number, callers use it,
and the uncertainty becomes optional. Optional uncertainty is the state every other tool in this
field is already in.

### R-0.2: Export paths accept only `Cleared<T>`

Any function that writes a design toward physical synthesis, or serialises it into a format
intended for a synthesis vendor, accepts `Cleared<T>` and nothing else. `Cleared` is constructible
only inside `integrase-screen`.

**Escape hatch:** none in library code. Test code may construct `Cleared` through a
`#[cfg(test)]`-gated constructor that is compiled out of release builds, and any PR adding or
widening that constructor requires explicit human sign-off per the `merge-decision` skill.

**Why it is absolute:** this is the difference between a screening gateway and a screening
suggestion.

---

## Error handling

### Rust

**R-1.1 Libraries return typed errors. Binaries may use `anyhow`.**
Every crate under `crates/` except `integrase-cli` uses `thiserror` enums in its public API.
`anyhow::Error` never appears in a public library signature. The CLI, tests, benchmarks, and build
scripts may use `anyhow` freely.

Rationale: a caller who cannot match on the failure cannot handle it, and `anyhow` in a library
signature forces every downstream consumer into string matching.

**R-1.2 No `unwrap()` or `expect()` in library code paths.**
Permitted in tests, benchmarks, build scripts, and in `const`-context initialisation that cannot
fail. Everywhere else, propagate.

Exception, and it must be written down at the call site: `expect()` is allowed where the invariant
is genuinely unbreakable and the message explains *why*, not *what*. `expect("index is in bounds
because we just checked len()")` is acceptable. `expect("should work")` is not.

**R-1.3 No swallowed errors.**
`let _ = fallible();` and `if let Ok(x) = ...` that silently discards the error are review
failures. Either handle it, propagate it, or log it at `warn` with the error attached. This is
called out specifically because it is the single most common tell of AI-generated code, and the
`ai-code-review` skill checks for it.

**R-1.4 `panic!` is for programmer error only.**
Invalid user input, malformed files, and network failures are `Result`. A panic means a bug in
Integrase. This matters especially in `integrase-screen`, where a panic across the gRPC boundary
must fail closed rather than fail open.

### Go (Phase 3)

**R-1.5 Errors are wrapped with context at every boundary crossing**, using `fmt.Errorf("...: %w",
err)`. A bare `return err` through five layers produces an error message that identifies nothing.

**R-1.6 No `panic` in request paths.** Recover at the handler boundary, log, return a gRPC status.

### Python

**R-1.7 Typed exception hierarchy rooted at `IntegraseError`.** No bare `raise Exception`, no
`except:` without a type, no `except Exception: pass`.

**R-1.8 Type hints on every public function**, checked by `mypy --strict` in CI. The SDK's users are
scientists in notebooks, and type hints are the documentation they actually see.

---

## Testing

Subplan 17 owns the full strategy. These are the rules that bind.

**R-2.1 Coverage floors, by subsystem, enforced in CI:**

| Subsystem | Line coverage floor | Why this number |
|---|---:|---|
| `integrase-screen` | **95%** | Highest consequence of a false negative in the system. |
| `integrase-core` | 85% | Sequence manipulation bugs corrupt everything downstream. |
| `integrase-sim` | 85% | Numerical code fails quietly. |
| `integrase-sbol` | 80% | Thin adapter; `sbol-rs` carries the hard correctness. |
| `integrase-viz` | 60% | Rendering is hard to unit test honestly. Visual regression carries the rest. |
| Everything else | 75% | Default. |

Coverage is a floor, not a target, and it is a proxy. R-2.2 is the rule that actually matters.

**R-2.2 A test that cannot fail is not a test.**
Tests asserting that a constructor constructs, that a getter gets, or that `2 + 2 == 4` inside a
wrapper are removed in review, not counted toward coverage. Every new test must be accompanied by a
plausible answer to "what bug does this catch?" This rule exists because coverage floors create
pressure to write exactly this kind of test.

**R-2.3 Property tests for sequence manipulation and numerical code.**
`proptest` for anything transforming sequences or computing biophysical values. Round-tripping,
involution (reverse-complement twice is identity), coordinate remapping under edits, and interval
arithmetic are all places where example-based tests miss whole bug classes.

**R-2.4 Format round-trip tests compare semantically, never syntactically.**
Earned by measurement in subplan 23. SBOL 2 to 3 conversion canonicalises ontology URIs and cannot
preserve `persistentIdentity`, so a triple-equality assertion fails on correct behaviour and gets
weakened until it passes.

**R-2.5 Every tool that checks something is verified in both directions.**
A linter, validator, or checker must be tested against input it should reject, not only input it
should accept. A checker that has never failed is indistinguishable from one that cannot fail.
`tools/check-voice.sh` and `tools/check-mermaid.sh` both have negative fixtures for this reason.

**R-2.6 Calibration is tested, not asserted.**
A model claiming 80% coverage must demonstrate on held-out data that its 80% intervals contain the
true value roughly 80% of the time. Subplan 08 owns methodology. A model without this evidence
reports `DomainVerdict::Uncalibrated` and is not permitted to claim a coverage figure.

---

## Dependencies

**R-3.1 Every new dependency is justified in the PR description.** What it does, why it beats the
alternatives, and why writing it ourselves is worse. A dependency added without this is rejected.

**R-3.2 Vetting criteria.** Every dependency, before adoption:

- **Checked against the advisory database first, not after CI says so.** Amended 2026-09-15:
  `pyo3` was added at 0.27 carrying two known vulnerabilities, including an out-of-bounds read,
  which is a poor look in a workspace that denies `unsafe_code`. `cargo deny check advisories` now
  runs as part of `just check`, and `just advisories` exists as the fast pre-adoption check.

A dependency in a correctness-critical path (`integrase-core`, `integrase-sim`,
`integrase-screen`) must additionally clear all of:

- Maintained: a release or meaningful commit within 12 months.
- Adopted: enough usage that bugs surface elsewhere first. As a rule of thumb, four-figure recent
  downloads on crates.io. `bio-seq` at 955 failed this and was rejected in subplan 05.
- Licensed compatibly: permissive, and recorded in `NOTICE`.
- Auditable: we could read it if we had to. A 50k-line transitive tree for a small convenience is
  not a good trade.
- Not duplicative: does not add a second implementation of something we already depend on.
  `sophia` was rejected for exactly this, since `sbol-rs` brings the Oxigraph stack.

Outside correctness-critical paths, the adoption and auditability bars relax. The licensing and
duplication bars never do.

**R-3.3 Data licensing is vetted like code licensing.**
This is the rule most dependency policies miss. Bundled or redistributed *data* carries its own
terms. The iGEM Registry is CC BY-SA, so derivatives inherit ShareAlike and must not be baked into
an Apache-2.0 crate as if they were ours. BioModels is CC0 and is safe. Every file under
`testdata/` carries a provenance note recording source and license.

**R-3.4 Exact version pinning for `sbol-rs`.** It is a four-month-old v1.0.0 carrying 348
validation rules we depend on. Upgrades are a deliberate PR that re-runs the conformance corpus,
not a Dependabot automerge.

**R-3.5 `cargo deny` and `pip-audit` in CI**, failing on known vulnerabilities and license
violations across all three ecosystems.

---

## Documentation

**R-4.1 Public items are documented.** `#![deny(missing_docs)]` on every library crate. Not
negotiable per-item; if something is public it is documented, and if it should not be documented it
should not be public.

**R-4.2 Doc comments explain why, not what.**
`/// Returns the reverse complement.` on `fn reverse_complement()` is noise. Document invariants,
failure modes, units, coordinate conventions (0-based or 1-based, half-open or closed: get this
wrong and everything downstream is off by one), and anything surprising.

**R-4.3 Examples in doc comments are doctests**, so documentation that stops compiling fails CI.
Documentation rot is prevented mechanically or not at all.

**R-4.4 A PR changing user-facing behaviour updates the docs in the same PR.** Not a follow-up.
Follow-ups do not happen.

**R-4.5 Every model carries a `ModelCard`.** Machine-readable, containing published accuracy,
validation datasets, known failure modes, and calibration evidence. A model without one reports
`Uncalibrated`. This is how accuracy claims are stopped from drifting away from evidence.

---

## Performance

Subplan 09 sets the numbers. These are the rules.

**R-5.1 Performance claims require a benchmark in the repository.** "Fast" in a README without a
`criterion` benchmark behind it is a claim we cannot defend and will not make.

**R-5.2 Regression gates in CI on the numbers subplan 09 defines.** A PR that regresses a gated
benchmark beyond its threshold fails, and the fix is either an optimisation or an explicit,
justified threshold change.

**R-5.3 Memory is a first-class budget, not an afterthought.** The 7 GB development constraint is
treated as a feature. `integrase-core` streams and memory-maps rather than reading whole sequences
into memory.

**R-5.4 No optimisation without a profile.** Complexity added for performance requires a before and
after measurement in the PR. This is the other most common tell of AI-generated code: elaborate
machinery solving a performance problem nobody measured.

---

## API stability and versioning

**R-6.1 Semantic versioning, coordinated across all three ecosystems.** The Rust crates, the Python
package, and the Go module share a version number. A user should never have to reason about whether
`integrase` 0.4 in Python matches `integrase-core` 0.4 in Rust.

**R-6.2 Pre-1.0 means the API moves, and we say so.** Until 1.0, breaking changes ship in minor
versions with changelog entries. Pretending otherwise while actually breaking things is worse than
being clear.

**R-6.3 Post-1.0, breaking changes require a major version and a migration note.** Not just a
changelog line: what broke, why, and what to write instead.

**R-6.4 `proto/` contracts are checked for breaking changes by `buf` in CI.** Wire compatibility is
the one thing that breaks users silently and at a distance.

**R-6.5 What is public is deliberate.** `pub` inside a crate is not the same as public API. The
umbrella `integrase` crate defines the stable surface by re-export; everything else is
`pub(crate)` or documented as unstable. Subplan 12 owns the mechanics.

---

## Code review

**R-7.1 Every change goes through a PR. No direct pushes to `main`, including by the project
owner.** Enforced mechanically: branch protection has `enforce_admins` set.

**R-7.2 A PR touching `integrase-screen` gets additional scrutiny**, per the `pr-review` skill and
subplan 10's threat model. Detectable by path, which is why screening is its own crate.

**R-7.3 Changes requiring explicit human sign-off, never auto-merged:** screening logic, the
`Cleared` construction surface, license changes, and breaking API changes. The `merge-decision`
skill routes these to the owner regardless of how clean the diff looks.

**R-7.4 Self-review is a real step.** With one contributor, the author and reviewer are the same
person. That is a reason to run the checklist explicitly and record the findings in the PR, not a
reason to skip it. Approvals-required is currently 0 only because GitHub forbids self-approval, and
it goes to 1 the moment a second contributor exists.

---

## Style and formatting

Not opinions. Enforced in CI, so nobody argues about them in review.

| Language | Tools |
|---|---|
| Rust | `cargo fmt --check`, `cargo clippy -- -D warnings` |
| Go | `gofmt -l`, `golangci-lint` |
| Python | `ruff format --check`, `ruff check`, `mypy --strict` |
| Protobuf | `buf lint`, `buf breaking` |
| Prose | `tools/check-voice.sh` |
| Diagrams | `tools/check-mermaid.sh` |

**R-8.1 Conventional Commits**, enforced in CI. Chosen because it makes changelog generation
mechanical, which matters for R-6 coordinated releases across three package ecosystems.

**R-8.2 The writing rules in the brief's section 11 apply to code comments, commit messages, PR
descriptions, and docs.** They are checked by `tools/check-voice.sh` where mechanically possible and
by the `writing-voice` skill where not.

---

## Open items

- [ ] Coverage measurement tooling per language -> **17**, **18**
- [ ] Exact benchmark regression thresholds -> **09**
- [ ] `#[cfg(test)]` `Cleared` constructor design, so tests work without weakening R-0.2 -> **24**
