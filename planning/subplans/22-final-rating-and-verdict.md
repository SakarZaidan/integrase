# 22. Final rating and verdict

**Status:** complete, revised 2026-09-15
**Purpose:** audit the planning work against the bar the brief set, and score it honestly
**Depends on:** all other subplans, and subplan 03 in particular

**Revision note (2026-09-15).** The first version of this document scored 5.6 and reported that
planning had moved adoption by exactly zero. That was accurate about the plan as it stood, and it
was a bad piece of work, because **the plan contained no adoption strategy at all.** Scoring a
dimension low and then not working on it is not rigour; it is giving up and calling it rigour. Since
then: scope was restored (subplans 25 through 29, 31), a distribution strategy was built (subplan
30), and a funding path was researched (subplan 32). This is the re-score.

---

## The rubric problem, and the fix

The original rubric conflated three different questions under one number. That is why "adoption: 3"
was simultaneously true and useless.

| Rubric | Question | Can planning move it? |
|---|---|---|
| **A. Plan quality** | Is the plan complete, evidenced, specific, honest? | **Yes. Entirely.** |
| **B. Prospects** | What would this score if the plan were executed? | Yes, by improving the plan |
| **C. Current state** | What is true today? | **No.** Today is day two. |

Scoring C is not interesting: the project has no users, no code, and no funding, so it is near zero
by construction and will be until work happens. Every number below is **A** or **B**, labelled.

Conflating A and C is exactly how the first version produced a number that discouraged the work
needed to change it.

---

## Rubric A: plan quality

Does the plan address this dimension with evidence, specificity, and honesty about its own limits?

| Dimension | Score | Basis |
|---|---:|---|
| Novelty analysis | **9.5** | Every claimed novelty is checked against literature and prior art. Circuit interval propagation is correctly identified as published research that no tool implements, rather than claimed as our invention. |
| Differentiation analysis | **9.5** | Six claimed differentiators graded individually, two kept, four demoted. Evidence-based rather than aspirational. |
| Technical planning | **9.5** | Architecture down to type signatures. Build-versus-adopt resolved by running the dependency, not reading its README. Two benchmarks that changed decisions. |
| Data planning | **9** | Every source mapped with licence and access method. Two blocking gaps identified, one resolved, one still open and named. |
| Adoption planning | **9** | A verified channel of 4,600+ annual participants whose mandatory safety workflow our differentiating subsystem implements. Was absent entirely. |
| Sustainability planning | **9** | Funding landscape researched, specific programmes with amounts and deadlines, and an eligibility blocker found that was invisible before. |
| **Honesty and self-correction** | **9.5** | Three of my own errors corrected in place rather than quietly edited, including one that had removed a feature. |
| **Rubric A weighted** | **≈9.3** | |

Rubric A is where the brief's standard actually applies, and it is close to the bar. The deductions
are real: data planning is a 9 rather than 9.5 because the second calibration dataset is still
unidentified, and sustainability planning is a 9 because the eligibility blocker is found but
unresolved.

---

## Rubric B: prospects if executed

The original six dimensions, re-scored against the restored scope and the new strategy.

| Dimension | Weight | v1 (pre-planning) | v2 (scope cut) | **v3 (now)** | Δ from v2 |
|---|---:|---:|---:|---:|---:|
| Novelty | 15% | 6 | 6 | **8.5** | +2.5 |
| Uniqueness / defensibility | 15% | 5 | 6 | **8** | +2 |
| Technical feasibility | 20% | 7 | 8 | **7.5** | −0.5 |
| Data availability | 20% | 4 | 6 | **7** | +1 |
| Adoption path | 20% | 3 | 3 | **8.5** | **+5.5** |
| Sustainability | 10% | 3 | 4 | **6.5** | +2.5 |
| **Weighted total** | | **4.7** | **5.6** | **7.7** | **+2.1** |

### Novelty: 6 → 8.5

Restoring scope raised this, which is the opposite of what I expected when I cut it. The cut version
had two novel components. The restored version has five, and they compose:

- Calibrated uncertainty as a type-enforced product principle
- Screening as a structural gate inside a design environment
- **Uncertainty propagated compositionally through a genetic circuit** (subplan 25). Interval
  propagation through gate libraries is published work (ACS SynBio, 2021) that **no usable tool
  implements.** Not our invention, and implementing it properly would be a first.
- **Verifiable design records** (subplan 29)
- Generative design constrained by screening rather than bolted to it (subplans 26, 31)

Not 10, because the individual pieces mostly exist in literature. The integration is what is new,
and "nobody has built this" is a weaker claim than "nobody has thought of this."

### Uniqueness and defensibility: 6 → 8

The moat is now multi-layered: type-enforced uncertainty, a screening gate, compositional
propagation, verifiable records, and the accumulating calibration data. Retrofitting that into an
existing tool means changing its type system, its export paths, and its data model at once.

Still 8 rather than 9.5, because a funded team could build this. The durable moat is the data and
the citations, and neither exists yet.

### Technical feasibility: 8 → 7.5, and this is the honest cost of the instruction

Restoring scope lowered feasibility. It could not do otherwise, and pretending it did would be the
kind of dishonesty this project exists to reject.

What keeps it at 7.5 rather than the 3 the full scope originally scored:

| Subsystem | Why it is tractable now |
|---|---|
| SBOL | `sbol-rs` adopted. 348 validation rules inherited. Verified by running it. |
| Screening | SecureDNA delegated to. Test database ships in their repo. |
| Circuit compilation | **Cello is BSD-2-Clause with 142 forks.** Fork and revive, not reimplement. |
| Generative design | Evo 2 is open-weight. Integrate, do not train. |
| Genome-scale indexing | Measured at ~3.9 bytes/base compressed. The original 39.4 figure was one library's design choice. |

**Almost every large subsystem has an existing, licence-compatible component underneath it.** That
is what separates this from TinkerCell, which had to build everything.

The remaining 2.5 points of deduction are honest: this is multi-year work for one person, and the
timeline in subplan 21 has to say so rather than compress.

### Data availability: 6 → 7

Registry parts (84,000+, CC BY-SA), BioModels (CC0), `sbol-rs` fixtures, SecureDNA test HDB, Kosuri
for calibration. **The second calibration dataset is still unidentified**, which caps this. Without
it the central claim is in-distribution only.

### Adoption path: 3 → 8.5, the largest single movement

This is what the first version should have done instead of scoring 3 and stopping.

**iGEM: 4,600+ participants, 400+ teams, 50+ countries** at the 2025 Jamboree, with 2026 reported as
a record. Every step of an iGEM team's workflow maps to a subsystem we are building, and the
mandatory **Team Safety Form and White List** map directly onto the screening gateway. That is not
marketing adjacency; it is the same problem.

Plus: SynBioHub plugin integration, course modules, two distinct papers, and the biosecurity
community as a small but disproportionately influential audience.

**8.5 and not 9.5 because none of it is executed.** The plan is strong; the adoption is zero. Rubric
C exists for exactly this distinction.

### Sustainability: 4 → 6.5

Specific programmes with amounts: PESOSE at up to $1.5M, CZI EOSS for maintenance, biosecurity
funders as the fastest fit. Five bus-factor mitigations, of which the planning folder is already
done.

Held down by two real facts: the **eligibility blocker** (an individual cannot apply to NSF, R-004),
and **no funding before late 2027 at the earliest** since this year's deadline passed two weeks ago.

---

## Rubric C: current state

Recorded so the other numbers cannot be misread.

| | |
|---|---|
| Users | 0 |
| Lines of implementation code | 0 |
| Citations | 0 |
| Funding | none |
| Contributors | 1 |
| Institutional affiliation | none |

**Everything above is a plan.** A good plan, now, and still a plan.

---

## Did it reach 9.5 on all aspects?

Asked directly, answered directly.

**Rubric A, plan quality: ≈9.3, and the remaining gap is specific and closeable.** Data planning
reaches 9.5 when a second calibration dataset is identified. Sustainability planning reaches 9.5
when R-004 is decided. Both are bounded tasks rather than open problems.

**Rubric B, prospects: 7.7, not 9.5.** Getting there requires three things that planning cannot
produce:

| To reach | Requires | Would move |
|---|---|---|
| Data 7 → 9 | A second independent expression dataset | Bounded research task |
| Sustainability 6.5 → 9 | An institutional home and funding | R-004, then ~18 months |
| Adoption 8.5 → 9.5 | Actual users, from actual execution | Phase 1 plus a season |
| Feasibility 7.5 → 9 | A second maintainer, or a longer stated timeline | People, or honesty about time |

With all four, the weighted total is approximately **9.2**. That is the ceiling this plan can
reach, and every step to it is named.

**I am not writing 9.5 into a table that the evidence does not support.** A planning folder whose
central thesis is that predictions must carry honest uncertainty cannot end by reporting a number it
cannot defend. The work to earn a higher score is real and specified; the score follows the work,
not the other way round.

---

## What the planning got wrong, and what it corrected

An audit that finds nothing wrong with its own work is not an audit. Six items, three of them from
this revision.

1. **I scored adoption 3 and did not work on it.** The worst of the six, because it was
   self-fulfilling: I concluded planning could not help, so I did not plan, so planning did not
   help. Corrected by subplan 30, which moved it to 8.5.
2. **I declared genome-scale indexing infeasible on a single library's benchmark.** `rust-bio`'s
   FM-index keeps a full `u64` suffix array. A compressed index measured afterwards costs roughly a
   tenth as much. The measurement was right and the inference was wrong, and it cost a feature. New
   rule in subplan 07: a benchmark licenses a claim about the thing benchmarked and nothing wider.
3. **I nearly published an overclaim about `sbol-rs`.** "0 of 7 triple-identical" reads as a defect
   until you measure what differs: ontology canonicalisation, not data loss. My test was stricter
   than the claim.
4. **Subplan 06 had a gap only sequencing exposed.** `NullBackend` refusing everything meant an
   unconfigured user could not save their own design. Would have shipped as a usability disaster.
5. **The palette failed WCAG at 4.20:1.** It looked fine.
6. **I cut scope partly because I had not done the work to make the full scope tractable.** The
   build-versus-adopt analysis in this revision shows nearly every large subsystem has an existing
   licence-compatible component underneath it. The cut was defensible on the evidence I had, and
   the evidence was thin.

---

## Verdict

**Build it, at full scope, on a multi-year timeline that is stated honestly rather than compressed.**

The instruction to keep the features and compensate with deeper planning was the right call, and the
evidence says so: restoring scope moved novelty from 6 to 8.5 and defensibility from 6 to 8, at a
cost of one point of feasibility that the build-versus-adopt analysis mostly absorbs. The cut
version was a smaller, safer, less interesting project that would have been easier to finish and
less worth finishing.

**7.7 out of 10 on prospects, ≈9.3 on plan quality, and near zero on current state.** The four steps
from 7.7 to roughly 9.2 are named above, three of them depend on execution rather than planning, and
one of them (R-004) is a decision waiting on the owner.
