# 22. Final rating and verdict

**Status:** complete
**Date:** 2026-09-14
**Purpose:** audit the planning work against the bar the brief set, and re-score honestly
**Depends on:** all 23 other subplans, and subplan 03 in particular

Subplan 03 scored the project before planning and said: "Subplan 22 re-scores after the full plan
exists. If these numbers have not moved by then, the planning did not do its job." This is that
re-score, using the identical rubric so the comparison is real rather than rhetorical.

---

## The re-score

| Dimension | Weight | Before (03) | After | Δ |
|---|---:|---:|---:|---:|
| Novelty | 15% | 6 | **6** | 0 |
| Uniqueness / defensibility | 15% | 5 | **6** | +1 |
| Technical feasibility | 20% | 7 | **8** | +1 |
| Data availability | 20% | 4 | **6** | +2 |
| Adoption path | 20% | 3 | **3** | **0** |
| Sustainability | 10% | 3 | **4** | +1 |
| **Weighted total** | | **4.7** | **5.6** | **+0.9** |

### Novelty: 6, unchanged

Planning does not create novelty. The two novel components are the same two: calibrated uncertainty
as a first-class product principle, and screening as a structural gate inside a design environment.
Everything else remains unoriginal, and planning correctly did not invent new claims to inflate this.

### Uniqueness and defensibility: 5 → 6

The one-point move is earned by architecture, not by wishful thinking. Subplan 06 turned both
differentiators from features into structural properties: `Prediction<T>` means no API can return a
bare predicted value, and `Cleared<T>` means the export gate is enforced by the compiler.

Why that improves defensibility: a competitor adding "confidence intervals" as a feature adds a
field. Adding it the way Integrase has it requires changing their type system so that omitting
uncertainty is impossible. The first is a sprint; the second is a rewrite. That is a modest moat,
not a large one, which is why this is +1 and not +3.

### Technical feasibility: 7 → 8

The largest concrete improvement, and it comes from three decisions that removed work rather than
adding capability:

- **`sbol-rs` adopted** (subplan 23). 348 validation rules, SBOL 2 and 3, conversion, and a
  cross-implementation corpus, verified by running it. A year of work became a dependency.
- **SecureDNA delegated to** (subplan 24). Screening matching logic, the single most dangerous piece
  of not-invented-here available here, is not ours to write.
- **Scope cut** (subplan 02). Logic synthesis, generative design, 3D rendering, desktop, and
  genome-scale search all removed from v1.0.

Not a 9 or 10, because calibration on real data is still unproven and remains the project's largest
technical unknown.

### Data availability: 4 → 6

The biggest single move, from resolving `REQUESTS.md` R-002. SecureDNA is free, runs locally, uses
an oblivious protocol so nobody holds the hazard database, and **ships a test HDB**, so the
highest-consequence test suite runs in CI with no credentials.

It is 6 and not 8 because the other blocking gap is still open: **calibration validation has one
dataset.** Kosuri 2013 gives 12,653 constructs, and a held-out split of one MPRA library measures
interpolation rather than the distribution shift users actually face. Without a second independent
dataset, the honest claim is in-distribution calibration only.

### Adoption path: 3, unchanged, and this is the finding

**Planning moved this by exactly zero, and that is the most important line in this document.**

Nothing in twenty-four subplans changes that the most-starred genetic CAD tool has 875 stars and was
abandoned in 2022, that the most-starred maintained tool in the core ecosystem has 84, or that Evo 2
has 4,213 while the CAD side stays quiet.

Subplan 23 added the sharpest evidence available: **`sbol-rs` is objectively excellent and has 5
stars.** Fourteen crates, 348 validation rules, fuzzing, benchmarks, Python bindings orders of
magnitude faster than the incumbent, from the standards body itself. Five stars.

If that repository cannot get attention, quality alone will not get Integrase attention either. This
is the binding constraint, it is not an engineering problem, and no amount of further planning
touches it.

### Sustainability: 3 → 4

A modest, real improvement. The planning folder is now a genuine succession artifact: twenty-four
documents recording not just decisions but rejected alternatives and reasoning, plus an ADR
convention for everything decided later. A future maintainer can reconstruct the reasoning without
the original author.

Still a 4, because bus factor is one, there is no funding model, and no institutional host. Writing
things down mitigates the failure mode; it does not remove it.

---

## Did the planning do its job?

**On engineering questions, yes.** Feasibility and data availability moved 3 points combined, driven
by measurement rather than opinion. Concretely, planning produced:

- A benchmark proving the FM-index costs 39.4 bytes per base, which killed a briefed feature with a
  number instead of an argument.
- A working probe against `sbol-rs` proving 33 of 33 fixtures round-trip losslessly, and correcting
  my own near-overclaim about SBOL 2 conversion.
- A contrast computation that caught an accessibility failure in the palette before it shipped.
- A verified finding that SynBioHub requires authentication, which changed the testing strategy.
- Resolution of the project's largest external dependency.

**On the thing that decides whether the project matters, no.** Adoption is still 3.

That asymmetry is the honest summary of what planning can and cannot do, and it is worth stating
plainly rather than burying under the improved total.

---

## Does it reach the bar the brief set?

The brief set two bars. The answer differs for each, and it has not changed since subplan 03.

**The engineering bar: yes, reachable.** A senior infrastructure engineer reading this repository
would find a coherent architecture, rules stated so they can be cited by number, measured claims,
two differentiators enforced by types rather than by convention, and documented limitations. A
working synthetic biologist would find a tool that tells them what it does not know, which no other
tool in the field does.

**The field-standard bar: no, not on the current trajectory.** Unchanged from subplan 03, for
unchanged reasons. Adoption at 3 and sustainability at 4 describe a project that will be good and
unused, which is precisely the fate of every tool in subplan 02's dormant column.

Saying this again at the end, after all the planning, is the point of this document. The plan got
better. The prognosis did not.

---

## What the planning got wrong, or nearly did

An audit that finds nothing wrong with its own work is not an audit.

1. **I nearly published an overclaim about `sbol-rs`.** The SBOL 2 round-trip test returned "0 of 7
   triple-identical," which reads as a defect. Measuring what actually differed showed symmetric
   lost/added pairs on ontology URIs: canonicalization, not data loss. **My test was stricter than
   the claim being made.** Caught only because I looked at the diff instead of reporting the number.

2. **Subplan 06 had a real gap that only sequencing exposed.** Taken literally, `NullBackend`
   refusing everything meant an unconfigured user could not save their own design. Found while
   writing subplan 21 and fixed by splitting interchange from synthesis-bound export. It would have
   shipped as a usability disaster.

3. **The brief's student scenario cannot be supported in v1.0** and I did not notice until writing
   subplan 11, well after the scope cut in subplan 02 made it true. Scope decisions have downstream
   consequences that are easy to miss for several documents.

4. **The palette failed accessibility on first attempt.** `--caution` measured 4.20:1 against a
   required 4.5:1. It looked fine.

5. **Adoption may be scored generously at 3.** An argument exists for 2. The project has no
   institutional relationship, no publication, no user, and enters a field whose attention has
   moved elsewhere. I kept 3 because the Python SDK genuinely does reach an adjacent, larger
   audience, but that is a hypothesis rather than evidence, and subplan 03's own rubric says 3-4
   means "real obstacles, no credible plan to clear them yet."

---

## What would move the number now

Unchanged in substance from subplan 03, sharpened by what planning established. All three are
outside the code.

1. **Demonstrate calibration on held-out data and publish it.** This is now the critical path for
   both adoption and data availability, and it is a Phase 1 output rather than a later one. A
   preprint showing calibrated uncertainty for RBS prediction, with methodology public, converts
   adoption from a marketing problem into an academic one, which is the game this field plays.
   Plausibly 3 → 6.

2. **One institutional relationship.** SynBioDex is the natural first approach, and subplan 23 gave
   us something concrete to bring: we are a serious downstream user of `sbol-rs`, we found a
   round-trip question worth asking about, and we want `sbol-py` on PyPI. That is a real
   conversation, not a cold introduction. Plausibly sustainability 4 → 6.

3. **Find the second calibration dataset.** Without it the central claim is in-distribution only,
   which is honest but much less compelling. Data 6 → 8.

**If all three land, the weighted total is roughly 7.2.** That is a genuinely good project. It is
still not a 9, and the gap between 7.2 and 9 is occupied entirely by things that take years:
citations, users, and a second maintainer.

---

## Verdict

**Build it. Build the small version. Publish the calibration result.**

The planning was worth doing: it removed roughly a year of unnecessary work, resolved the largest
external dependency, killed one briefed feature with a measurement, caught a usability gap and an
accessibility failure before either shipped, and replaced several assumptions with numbers.

What it could not do is make the field larger or make a new tool get adopted, and those remain the
binding constraints. **5.6 out of 10 is an honest score for a well-planned project in a small field
with no distribution.** Presenting it as higher would be exactly the dishonesty this project's
central principle exists to reject, and it would be a strange way to end a planning folder whose
whole thesis is reporting uncertainty rather than hiding it.
