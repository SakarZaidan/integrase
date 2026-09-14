# FINAL-DECISION

**Date:** 2026-09-14
**Status:** the governing decision for the project
**Supersedes:** any subplan it contradicts. Where this document and a subplan disagree, this wins,
and the subplan gets amended.

Twenty-four subplans each answered one question well. Several of their answers conflict, and some
questions only become visible once the whole picture is in view. This document resolves those and
sets the build order.

It is a decision, not a summary. If you want the reasoning behind any individual call, the subplan
is cited.

---

## The decision in one paragraph

Build **Integrase**, an open-source platform for designing, simulating, and screening genetic
constructs, where every prediction carries calibrated uncertainty and the path toward physical
synthesis passes through a screening gate enforced by the type system. Build it much smaller than
the brief describes: a Rust core, a Python SDK, a CLI, and one honestly-calibrated biophysical
model, interoperating with the existing ecosystem through `sbol-rs` rather than replacing it. Ship
that, then publish the calibration result, because a citable artifact is worth more to this
project's survival than any additional feature.

---

## The nine tensions, resolved

### 1. The brief wants three languages. The scope cut leaves nothing for Go to do.

**Resolution: Integrase is a polyglot system by design and a Rust-plus-Python system by delivery.**

Subplan 05 deferred Go to Phase 3 because a single-user library has no jobs to orchestrate. That is
correct, and it does not compromise the project's identity, because the identity was never "uses
three languages." It was "a unified, professional-grade, honestly-uncertain, ethically-architected
design and simulation engine."

The `proto/` contracts are written in Phase 1 and validated against a stub server. When Phase 3
arrives, Go implements an existing contract rather than discovering one. **If those contracts need
changing in Phase 3, Phase 1's design was wrong, and that is worth finding out.**

*Rejected: building the Go layer in Phase 1 anyway to honour the brief's letter.* It would add a
third toolchain, CI matrix, and supply-chain surface to serve zero users, and the brief explicitly
grants latitude to restructure when research surfaces a better approach.

### 2. "Unification platform" is both the real opportunity and the thing that killed TinkerCell.

**Resolution: keep the word, change the meaning. Integrase is connective tissue with two original
organs.**

Bad unification reimplements CAD, simulation, logic synthesis, ML, and visualization in one
platform. That is TinkerCell, and it is dead. Good unification makes existing tools interoperate
through SBOL with a consistent API, and adds the two things nobody has: calibrated uncertainty and a
structural screening gate.

Concretely: import Cello's output rather than reimplementing Cello. Export SBML for iBioSim rather
than competing with it. Round-trip SynBioHub designs. Depend on `sbol-rs` rather than writing an
SBOL implementation.

### 3. `sbol-rs` exists, so "native SBOL support" is no longer a differentiator.

**Resolution: say so, in the README, plainly.**

Integrase depends on `sbol-rs` and does not implement SBOL. That is a strength to state rather than
a weakness to hide: we inherit 348 machine-checkable validation rules and a cross-implementation
test corpus we could never have justified building. Subplan 23's verification (33 of 33 fixtures
round-tripping losslessly) is the kind of evidence that belongs in the docs.

*Rejected: a thin reimplementation so we can claim native support.* It would be worse, slower to
build, and dishonest.

### 4. Adoption scores 3 out of 10. Should this be built at all?

**Resolution: yes, and Phase 1 is scoped specifically to attack this rather than to build
infrastructure.**

Subplan 22 established that planning moved adoption by exactly zero and that quality alone does not
produce adoption in this field, with `sbol-rs` at 5 stars as the evidence. That is a reason to
change what Phase 1 optimises for, not a reason to stop.

So **Phase 1's deliverable is a publishable result, not a platform foundation.** A preprint
demonstrating calibrated uncertainty for RBS prediction, with public methodology, converts adoption
from a marketing problem into an academic one, which is the game this field actually plays. The
software is what makes the result reproducible.

This is the single most consequential call in this document. It reorders everything downstream.

### 5. If calibration fails on real data, the central differentiator evaporates.

**Resolution: the differentiator degrades, it does not vanish, and the degraded version still ships
honestly.**

This is the project's largest technical risk (subplans 08, 21). The contingency, decided now so it
is not improvised under pressure:

| Outcome | Response |
|---|---|
| Calibration works in-distribution and out | Ship as planned. The full claim is supportable. |
| Works in-distribution only | Ship with `OutOfDistribution` for anything unlike the training set, and a `ModelCard` saying so. **Still better than every competitor.** |
| Does not work at all | Ship reporting `Uncalibrated`, with `DomainVerdict` and an honest `ModelCard`. |

Even the worst case leaves Integrase the only tool in the field that tells a user when it is
guessing. **Under no circumstances do we weaken the calibration test to make it pass.** That is the
one failure this project cannot survive, because it would convert honest ignorance into false
assurance, which is worse than the silence everyone else offers.

### 6. Leptos gives type safety and shrinks an already-small contributor pool.

**Resolution: defer the decision. Ship SVG in Phase 1, decide the web framework at the start of
Phase 2 with a bundle measurement in hand.**

Subplan 13 chose Leptos so `DomainVerdict` cannot exist twice with two definitions that disagree.
Subplan 19 identified bus factor as the primary sustainability risk, and a contributor who knows web
frontend probably knows React. Both are right, and this is only resolvable with the whole picture.

What the whole picture shows: **the SVG backend delivers most of the near-term value with no web
framework at all.** It produces publication figures with the confidence grammar intact, it enables
snapshot testing, and it is the Firefox fallback. It is Rust, in `integrase-viz`, and it is cheap.

So `integrase-viz` SVG moves **into Phase 1** (subplan 13 listed it as an open question; this
resolves it), and the interactive-UI framework choice waits until Phase 2, by which point we will
know whether contributors exist and what the WASM bundle actually weighs.

### 7. The screening gate is either too strict to use or too loose to mean anything.

**Resolution: the interchange/synthesis-bound split from subplan 21, now the settled position.**

Interchange export (SBOL round-trip, working files, figures) is not gated. Synthesis-bound export
(the explicit ordering flow, vendor formats) requires `Cleared<T>`. The distinction is the flow, not
the file format, so the gate lives on the function rather than the serializer.

This is a deliberate, partial loophole, and subplan 10 already records the honest version: someone
copying a sequence by hand defeats any gate. The gate's value is that **the convenient path to
ordering passes through screening**, not that every byte is imprisoned.

### 8. Phase 1 as written in subplan 21 is still too big.

**Resolution: split it. Phase 1a is the minimum publishable result. Phase 1b is what makes it a
usable tool.**

Subplan 21's Phase 1 contains nine deliverables. Under decision 4, the thing that matters is a
publishable calibration result, and several of those deliverables are not on that path.

**Phase 1a, the minimum that produces a citable result:** `integrase-core` sequence and design
types, `integrase-sbol` import, `integrase-sim` with `Prediction<T>` and the thermodynamic RBS
model, conformal calibration on Kosuri, the Python SDK, and the evaluation harness.

**Phase 1b, what makes it a tool people use:** CLI, `integrase-screen` with the type-level gate and
the SecureDNA backend, SVG output, SBOL export, docs.

Both ship before Phase 2. The split exists so that if something slips, what slips is 1b, and the
publishable result is not held hostage to the CLI being finished.

### 9. ViennaRNA licensing may block the headline Phase 1 feature.

**Resolution: decided in advance, because discovering it mid-Phase-1 would be disruptive.**

The thermodynamic RBS model needs RNA secondary structure prediction for its `ΔG_mRNA` and
`ΔG_standby` terms (subplan 16). ViennaRNA is the standard implementation and its licensing has not
been verified.

| If | Then |
|---|---|
| Licensing is compatible | Depend on it. Preferred. |
| Not compatible, another implementation exists | Evaluate it against the same criteria. |
| Neither | **Ship the RBS model without the structure-dependent terms, and widen the intervals to reflect the missing physics.** |

The third option is worth stating because it is on-thesis rather than a fallback. A model missing a
term is a model with more uncertainty, and this system is built to say so. It would be the first
tool in the field whose stated uncertainty reflects its own incompleteness.

**Action: resolve this before Phase 1a starts in earnest.** It is a licensing question, not an
engineering one, and it should not be discovered halfway through.

---

## Build order

| Phase | Question it answers | Gate to the next phase |
|---|---|---|
| **0. Foundation** | Can someone contribute to this? | Fresh clone: `just setup && just check` passes. CI green on a trivial PR. |
| **1a. The result** | Does it tell the truth about what it knows? | The calibration criterion below. |
| **1b. The tool** | Can someone actually use it? | Scenario S3 works end to end, no GPU, no network. |
| **2. Seeing it** | Can a user read the uncertainty unaided? | A stranger correctly identifies the less trustworthy of two predictions, without explanation. |
| **3. The service** | Can more than one person use it at once? | Phase 1 `proto/` contracts served **unchanged**. |
| **4. Candidates** | Nothing committed. | Each item needs a scenario first. |

### The one gate that can stop the project

**Phase 1a is complete when an 80% interval contains the true value 80% ± 5% of the time on
held-out data, stratified by expression decile, with component-wise splits.**

Not marginal coverage. Not a random split. Both of those are easy to pass and mean nothing:
component-wise splitting prevents near-duplicates landing on both sides of a combinatorial library,
and stratification catches a model that is overconfident on weak signals and underconfident on
strong ones while averaging to 80%.

If this fails, decision 5 applies. It does not fail silently and it does not get relaxed.

---

## What we are not building, and will keep not building

| Item | Why, with the evidence |
|---|---|
| Circuit logic synthesis | Cello did it well. Import its output. Reimplementing is multi-year. (02) |
| Genome-scale indexing | Measured: 39.4 bytes/base, 113.8 GB for a human genome. Supported range is bacterial scale. (09) |
| 3D construct rendering | Constructs are linear; SBOL Visual is 2D because that is what journals accept. (13) |
| Our own screening matcher | A false negative is the worst failure available here. Delegate to a peer-reviewed implementation. (24) |
| Our own SBOL implementation | 348 validation rules already exist upstream, verified working. (23) |
| Generative sequence design | Evo 2 is open-weight and better than anything we would train. (02, 08) |
| A hosted service we operate | We publish images. Operating a service is a different business. (18) |
| A "hide uncertainty" setting | It will be requested. It is declined. The person who most wants it most needs the information. (13) |

---

## The three things that decide whether this matters

None of them is code, and all three are Phase 1 concerns rather than later ones.

1. **Publish the calibration result.** Critical path for adoption. A preprint with public
   methodology converts adoption into an academic problem, which this field knows how to solve.
2. **Open the SynBioDex conversation.** We have something concrete to bring: a serious downstream
   user of `sbol-rs`, a round-trip observation worth asking about (`urn_design.ttl`), and a request
   for `sbol-py` on PyPI that blocks our Python packaging. A real conversation, not a cold
   introduction.
3. **Find the second calibration dataset.** Without it the central claim is in-distribution only,
   which is honest but much less compelling.

Subplan 22 estimates that all three landing takes the project from 5.6 to roughly 7.2. The gap
between 7.2 and anything higher is occupied by citations, users, and a second maintainer, which take
years and cannot be planned into existence.

---

## Standing commitments

Carried forward from subplan 07's tier 0, restated here because they outrank everything else in this
document.

1. **No API returns a bare value for a predicted quantity.** No exceptions, in any language, at any
   boundary.
2. **No design reaches a synthesis-bound export without passing screening.** Enforced by the
   compiler in Rust, by explicit tested checks at the Python and gRPC boundaries where the compiler
   cannot reach.

Both have now degraded at three boundaries (Rust compile-time, Python runtime, gRPC server-side).
Each degradation is documented and carries its own required test. **That list is not allowed to grow
silently.** A fourth boundary means a fourth test, decided deliberately.

---

## Honest closing statement

This plan scores 5.6 out of 10 on its own rubric (subplan 22). The engineering is sound and the
field is small. Planning removed roughly a year of unnecessary work, killed a briefed feature with a
measurement, resolved the largest external dependency, and caught a usability gap and an
accessibility failure before either shipped. It did nothing for adoption, which remains the binding
constraint and is not an engineering problem.

That is the honest position, and recording it here rather than in a footnote is consistent with the
only principle this project refuses to compromise: report the uncertainty, including about itself.
