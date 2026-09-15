# Contributing to Integrase

Thanks for looking. This document is long because a polyglot repository has more
ways to trip you up than a single-language one, and because two of our rules are
unusual enough that finding out about them in review would be irritating.

## Read this first

Before your first pull request, read three things:

1. This file.
2. [`README.md`](README.md), for what the project is and is not.
3. [`planning/subplans/07-engineering-principles.md`](planning/subplans/07-engineering-principles.md).

The third is an unusual ask and it is deliberate. Two rules there are
non-negotiable, they are not obvious from reading the code, and a PR that
violates either will be rejected regardless of how good it otherwise is.

### The two rules

**R-0.1. No API returns a bare value for a predicted quantity.**

Anything modelled, estimated, or inferred comes back as `Prediction<T>`, which
carries the value, its interval, the model that produced it, and a verdict on
whether the input fell inside that model's validated range. This applies to
convenience wrappers, to the Python bindings, and to gRPC.

There is no escape hatch. If you find yourself wanting one, the thing you
actually want is `prediction.value()`, and writing it out is the point.

**R-0.2. Nothing reaches a synthesis-bound export without passing screening.**

`Cleared<T>` is constructible only inside `integrase-screen`. Synthesis-bound
export functions accept `Cleared<T>` and nothing else, so bypassing the gate does
not compile.

Note the distinction: **interchange** export (SBOL round-trip, saving a working
file, figures) is *not* gated, because a design that came from the ecosystem
returns to it. Only the path toward physical synthesis is.

## Setting up

You need Rust (stable), Python 3.11+, [`uv`](https://docs.astral.sh/uv/), and
[`just`](https://github.com/casey/just). You do not need Go or `protoc` yet;
those arrive in Phase 3.

```bash
git clone https://github.com/SakarZaidan/integrase
cd integrase
just setup
just check
```

`just check` runs exactly what CI runs. If it passes locally it should pass in
CI, and if it does not, that is a bug in our setup worth reporting.

You only need the toolchain for the part you are touching. Fixing a typo in the
Python SDK does not require you to care about Rust.

### Useful targets

| Command | What |
|---|---|
| `just check` | Everything CI runs |
| `just fmt` | Format Rust and Python |
| `just test` | Test suites only |
| `just lint` | Clippy, ruff, mypy |
| `just docs-check` | Doctests and doc build |
| `just voice` | The prose rules, see below |
| `just msrv` | Verify the 1.93 floor still builds |
| `just audit` | Dependency vulnerability and licence scan |

## Branches and commits

Branch names are `type/short-description`, where the type matches the
[Conventional Commits](https://www.conventionalcommits.org/) type:
`feat/`, `fix/`, `docs/`, `refactor/`, `perf/`, `test/`, `chore/`, `build/`.

Commit messages follow Conventional Commits, enforced in CI. We chose it because
it makes changelog generation mechanical, which matters when a release has to
coordinate versions across crates.io, PyPI, and a Go module tag.

**The body explains why.** The diff already shows what. A commit message that
restates the diff in prose has told the reader nothing.

## The prose rules

This one surprises people, so it is worth stating plainly: **the writing style
in this repository is enforced in CI**, by `tools/check-voice.sh`, across
planning documents, docs, commit messages, and code comments.

It catches em dashes, a list of filler vocabulary, hedging phrases, and
manufactured enthusiasm. It is not a matter of taste. A project whose entire
thesis is that it will not overstate what it knows cannot afford prose that
overstates by habit.

Run `just voice` before you push. If you think a rule is wrong for a specific
case, say so in the PR: the checker has documented exceptions already (`leverage`
as a noun is fine, `robustness analysis` is fine) and can gain more.

## Pull requests

One coherent unit of work per PR. The template asks:

- What does this change, and why?
- Which subplan does it trace to?
- What tests cover it, including the edge cases?
- Does it touch `integrase-screen`?

That last question exists because screening changes get additional scrutiny by
rule, and because screening is its own crate specifically so the answer is
visible from the diff path.

We squash-merge and keep linear history. A squashed commit is one logical change
with one message, which keeps `git bisect` meaningful and changelog generation
mechanical. Merge commits are disabled.

## Tests

The rule that matters most: **a test that cannot fail is not a test.**

Tests asserting that a constructor constructs, that a getter gets, or that
arithmetic works inside a wrapper get removed in review rather than counted
toward coverage. Every new test should have an answer to "what bug does this
catch?"

Coverage floors, by subsystem:

| Crate | Floor | Why |
|---|---:|---|
| `integrase-screen` | **95%** | Highest consequence of a false negative. Hard gate. |
| `integrase-core` | 85% | Sequence bugs corrupt everything downstream |
| `integrase-sim` | 85% | Numerical code fails quietly |
| `integrase-sbol` | 80% | Thin adapter; `sbol-rs` carries the hard correctness |
| `integrase-viz` | 60% | Rendering is hard to unit test honestly |
| Everything else | 75% | Default |

Use `proptest` for sequence manipulation and numerical code. Example-based tests
miss whole classes of bugs there, and we have already caught two off-by-one
mutations with property tests that a hand-written suite would have passed.

**Format round-trip tests compare semantically, never on triples.** SBOL 2 to 3
conversion canonicalises ontology URIs and cannot preserve `persistentIdentity`.
A triple-equality assertion fails on correct behaviour, and the usual outcome is
someone weakening it until it passes.

## Adding a dependency

Justify it in the PR: what it does, why it beats the alternatives, and why
writing it ourselves is worse. A dependency added without that is rejected.

In correctness-critical paths (`integrase-core`, `integrase-sim`,
`integrase-screen`) it must also be maintained within the last 12 months,
adopted enough that bugs surface elsewhere first, permissively licensed, and
readable if we ever had to read it.

**Data has licences too**, and this is the part most policies miss. The iGEM
Registry is CC BY-SA, so redistributed derivatives inherit ShareAlike and must
not be baked into an Apache-2.0 crate. Every file under `testdata/` carries a
provenance note.

## Documentation

Public items are documented; `missing_docs` is denied at the workspace level.

Doc comments explain **why**, not what. `/// Returns the reverse complement.` on
`fn reverse_complement()` is noise. Document invariants, units, failure modes,
and coordinate conventions.

**Coordinates are 0-based and half-open, everywhere**, converted only at adapter
boundaries. GenBank is 1-based inclusive, BED is 0-based half-open, SBOL has its
own rules, and mixed conventions are the endemic off-by-one source in this field.

Examples in doc comments are doctests, so documentation that stops compiling
fails CI. A PR changing user-facing behaviour updates the docs in the same PR,
not a follow-up. Follow-ups do not happen.

## Issues

| Label | Meaning |
|---|---|
| `good-first-issue` | Scoped, documented, with a pointer to the relevant subplan |
| `subsystem:*` | Which part it touches |
| `type:*` | bug, feature, docs, perf |
| `needs:decision` | Blocked on a maintainer call; becomes an ADR |

**Security issues are never filed as public issues.** See
[`SECURITY.md`](SECURITY.md), and note that we handle one class of vulnerability
differently from standard practice.

## Where decisions live

`planning/` is where decisions were made and why, including what was rejected.
`docs/` is what the system is today. If you want to know why something is shaped
the way it is, the subplan will tell you, and if the answer is not there, that is
a gap worth filing.

Decisions made after `FINAL-DECISION.md` go in `planning/decisions/` as ADRs.

## Getting help

Open a discussion or an issue. "I tried to follow CONTRIBUTING and got stuck at
step N" is a genuinely useful bug report, and onboarding friction is a
sustainability problem for a project this size, not a minor annoyance.
