# 03. Novelty, uniqueness, and feasibility rating

**Status:** complete
**Date:** 2026-09-14
**Purpose:** score this project honestly against the bar the brief set for it
**Depends on:** `00-research-log.md`, subplan 02
**Feeds:** subplans 21, 22, and `FINAL-DECISION.md`

> **Scores here are SUPERSEDED, and kept as a historical record.** This document scored the project
> *before* planning, which is its purpose. Two of its conclusions were later shown to rest on bad
> evidence: technical feasibility at full scope (the build-versus-adopt analysis was not done) and
> the adoption path (no adoption strategy existed to score). Current scores are in
> `22-final-rating-and-verdict.md`, revised 2026-09-15: plan quality ≈9.3, prospects 7.7.

The brief asks that this be the most self-critical document in the folder. Taking that seriously
means the scores below are not calibrated to be encouraging. Two of the six are bad, and the bad
ones are the ones that decide whether the project matters.

---

## The rubric

Six dimensions, each 1 to 10, with anchored definitions so the scores are checkable rather than
vibes. Anchors are stated before the scoring so they cannot be bent to fit a desired answer.

| Score | Meaning |
|---:|---|
| 1-2 | Actively bad. Evidence points the wrong way. |
| 3-4 | Weak. Real obstacles, no credible plan to clear them yet. |
| 5-6 | Adequate. Works, but nothing here creates advantage. |
| 7-8 | Strong. Clear advantage, defensible, evidence-backed. |
| 9-10 | Exceptional. Best-in-class, hard to replicate, would be cited as an example. |

Scored against the **recommended scope from subplan 02**, not the brief's original scope. The
original scope is also scored where it differs, because the gap between those two numbers is the
single most useful output of this document.

---

## Scores

### 1. Novelty: **6 / 10**

Decomposed, because the aggregate hides everything interesting:

| Component | Score | Reasoning |
|---|---:|---|
| Calibrated uncertainty as a first-class product principle | **9** | Genuinely new in this domain. Cello returns a circuit, the RBS Calculator returns a number, iBioSim returns a trajectory. None reports how much to trust the answer, despite literature showing thermodynamic RBS prediction swings from R² > 0.8 to R² < 0.2 across datasets. Nobody has built the honest version. |
| Screening as a structural gate inside a design environment | **8** | Screening exists (SecureDNA, Common Mechanism). Ordering exists (GP-write CAD). A design tool where the gate sits architecturally between design and export does not. The placement is the novel part. |
| Unified CAD + simulation platform | **2** | Attempted repeatedly. TinkerCell did exactly this and is dead. Not novel, and the prior attempts failed. |
| Rust/Go/Python stack | **1** | Implementation detail. No user cares. |
| SBOL interop | **1** | pySBOL3 is active, `sbol-rs` shipped v1.0.0 in July 2026 from the standards body. Table stakes. |

**6/10.** The novel parts are real and the unoriginal parts are numerous. Weighted toward the two
components that carry the project's identity rather than averaged flat, because a project is
judged on its distinguishing features, not on the mean of its feature list.

### 2. Uniqueness and defensibility: **5 / 10**

Nobody else does the two things Integrase would do. That is uniqueness. Defensibility is a
different question and the answer is worse.

Neither differentiator is technically hard to copy. Calibrated uncertainty is a modelling and
presentation discipline, not an algorithmic breakthrough; pySBOL3 or a motivated grad student
could add interval reporting to an existing tool. The screening gate is an architectural choice
that any tool could adopt in a release.

What is actually defensible is trust, and trust is slow. A tool that has demonstrated calibration
on held-out data, published the methodology, and been used in papers has a moat. A tool that
merely claims calibrated intervals has nothing, and is one skeptical reviewer away from being
worse off than the tools that stayed silent.

**5/10.** Unique today, replicable tomorrow, defensible only through execution and evidence.

### 3. Technical feasibility: **7 / 10** recommended scope, **3 / 10** as briefed

This is the dimension where the gap between scopes is largest and most decision-relevant.

**As briefed: 3/10.** Six substantial subsystems (sequence CAD, biophysical simulation, ML-guided
design, circuit-logic compilation, visualization, biosafety gateway), three languages, a desktop
app, a web target, and native 3D rendering. TinkerCell attempted a subset of this with a team and
institutional backing, and did not survive. The failure mode is structural rather than a matter of
effort: each subsystem is individually a multi-year project, none is useful alone, and the
integration payoff only arrives after all of them exist. The project dies in the trough before
reaching it.

**Recommended scope: 7/10.** Deterministic biophysical models with honest error reporting, SBOL
interop through `sbol-rs`, a screening integration, and a Python SDK is a genuinely achievable
body of work. The hard parts are bounded and known: thermodynamic model implementation is
published and reproducible, `sbol-rs` removes the largest unknown, and screening can delegate to
existing implementations rather than being invented.

**On frontier models as the builder**, since the brief asks specifically. They change the
feasibility calculation asymmetrically, and it is worth being precise about how:

- **Collapsed:** implementation speed, cross-language boilerplate, test generation, documentation,
  the cost of working competently in three languages at once. Work that would have needed a team
  of four is now plausible for one person with good tooling.
- **Not collapsed, at all:** access to screening reference data, experimental validation of model
  calibration, credibility with domain scientists, the years it takes for a tool to be cited,
  maintaining the project in 2031. An agent writes the code; it does not run the wet lab, and it
  does not make a PI choose your tool.

The honest summary is that AI moved the bottleneck off engineering and onto everything the next
three dimensions measure. Engineering was never the binding constraint on this project.

### 4. Data availability: **4 / 10**

| Data need | Status | Score impact |
|---|---|---|
| SBOL designs for interop testing | Freely available from SynBioHub | Fine |
| iGEM Registry of Standard Biological Parts | Available, licensing not yet verified (subplan 04) | Probably fine |
| Published RBS/promoter strength datasets | Exist, quality and comparability vary, curation is real work | Workable |
| **Held-out experimental data for calibration validation** | **Not sourced** | **Severe** |
| **Curated sequence-of-concern reference data** | **Not obtainable through public channels** | **Severe** |

The two severe entries are the two that the project's differentiators depend on, which is an
uncomfortable symmetry.

Calibration cannot be claimed without demonstrating it. Demonstrating it needs sequences with
measured expression values, held out from training, ideally from multiple labs and conditions so
the intervals are tested against real distribution shift rather than a convenient split. Public
datasets exist but assembling a defensible evaluation set from them is a research task with an
uncertain outcome, not a data-loading task.

Screening reference data is worse, because it is not a matter of effort. The curated databases are
deliberately not public, for sound reasons (`REQUESTS.md` R-002). SecureDNA's privacy-preserving
hosted protocol appears to exist precisely to solve this, which may be the way through, but that
is unverified as of this writing.

**4/10.** Not fatal. Both problems have credible paths (integrate rather than replicate for
screening; careful curation plus honest scope limits for calibration). But scoring this higher
would be pretending the paths are walked rather than merely visible.

### 5. Adoption path: **3 / 10**

The lowest score, and the most robustly evidenced.

The field is small. The most-starred genetic CAD tool has 875 stars and has been abandoned since
2022. The most-starred maintained tool in the core ecosystem has 84. The brief's stated goal is to
"become the reference implementation in its field," and being the reference implementation in this
field currently means double-digit to low-triple-digit adoption.

Adoption in this community does not run through GitHub. It runs through papers, PI relationships,
lab teaching, and iGEM. A tool becomes standard because someone influential used it in a
publication and their students learned it. None of those levers are pulled by shipping good
software, and none are available to a solo project without institutional connection.

Meanwhile Evo 2 has 4,213 stars. Attention, funding, and talent in computational biology have
moved to genomic foundation models. Integrase enters a quiet room.

**3/10.** Good engineering does not fix this. Stating otherwise would be the exact dishonesty the
brief warns against.

### 6. Sustainability: **3 / 10**

Every dormant tool in subplan 02's tables followed the same arc: a small academic group built
something good, the grant ended or the students graduated, and the repository went quiet. Cello,
SBOLDesigner, paraSBOLv, TinkerCell. The pattern is the field's norm, not its exception.

A solo open-source project has the identical failure mode with fewer people and no grant to lose.
There is no funding model, no institutional host, and a bus factor of one.

**3/10.** Subplan 19 should treat governance and bus factor as engineering problems with real
mitigations (documented decisions, a planning folder that survives the author, low-friction
contributor onboarding) rather than as paperwork. That raises the ceiling but does not change
today's score.

---

## Aggregate

| Dimension | Score | Weight | Why this weight |
|---|---:|---:|---|
| Novelty | 6 | 15% | Necessary to matter, insufficient to succeed. |
| Uniqueness / defensibility | 5 | 15% | Determines whether early advantage survives. |
| Technical feasibility | 7 | 20% | Gates whether anything ships at all. |
| Data availability | 4 | 20% | Both differentiators depend on it. Weighted equal to feasibility deliberately. |
| Adoption path | 3 | 20% | Decides whether shipping mattered. |
| Sustainability | 3 | 10% | Decides whether it still matters in five years. |

**Weighted total: 4.7 / 10.**

As briefed rather than as rescoped, technical feasibility drops to 3 and the total falls to
**3.9 / 10**.

---

## Does this reach the bar the brief set?

The brief asks for a project that "becomes the reference implementation in its field" and that a
senior infrastructure engineer or working synthetic biologist "would immediately take seriously."
Those are two different bars and the project's prospects differ sharply between them.

**The engineering bar: reachable.** A well-architected, well-tested, honestly-documented tool with
calibrated uncertainty and a real screening gate would be taken seriously by both audiences on
inspection. Nothing in the scores above prevents the code from being excellent. Scores 1, 2, and 3
support it.

**The field-standard bar: not reachable on the current trajectory.** Scores 5 and 6 are the
binding constraints and they are not engineering problems. A 3/10 adoption path and a 3/10
sustainability score describe a project that will be good and unused, which is the exact fate of
every tool in subplan 02's dormant column.

Saying this plainly, as the brief requires: **the project as planned does not reach the bar set
for it, and the gap is not one that more or better code closes.**

---

## What would need to change to reach it

Ordered by leverage. The first two are worth more than everything else combined.

1. **A citable artifact.** In this field, tools become standard by being cited. A paper or preprint
   demonstrating calibrated uncertainty on held-out data, with the methodology public, converts
   adoption from a marketing problem into an academic one, which is the game this field actually
   plays. This is the highest-leverage single action available and it depends on solving the
   calibration data problem, which makes data availability the true critical path rather than
   engineering. Raises adoption from 3 toward 6.

2. **One institutional relationship.** A single lab or group using Integrase in real work provides
   validation data, a citation path, credibility, and a second maintainer. SynBioDex is the natural
   first approach, since we will be a serious downstream user of `sbol-rs` and contributing
   upstream is a genuine relationship rather than a favour request (subplan 23). Raises
   sustainability from 3 toward 6.

3. **Ship one narrow thing that is unambiguously the best available.** Not a platform. The honest
   candidate: a calibrated RBS and promoter strength predictor with a Python SDK, that reports
   intervals and flags out-of-distribution inputs, and that round-trips SBOL. Small enough to
   finish, useful enough to adopt, and it is the wedge the rest can grow from. Raises feasibility
   and adoption together.

4. **Reach the adjacent, larger audience.** The ML-for-biology community is where the attention is.
   A Python SDK that works well in a notebook next to Evo 2 reaches more people than a desktop CAD
   app reaches in the CAD community. This is a positioning decision more than an engineering one.

5. **Solve or honestly close the screening data question.** Either integrate SecureDNA and inherit
   real screening, or ship a documented integration point that fails closed and say so plainly.
   Both are acceptable. Shipping a toy dataset and implying it is real screening is not, and would
   damage the project more than having no screening at all.

---

## The uncomfortable summary

The brief asks for ambition and for calibrated honesty together. Held to both, this is the honest
reading: **the interesting, novel, valuable part of this project is small, and the ambitious,
comprehensive part of it is the part most likely to kill it.**

The recommendation to subplan 21 and `FINAL-DECISION.md` is to build the small thing extremely
well, connect it to everything else through SBOL rather than absorbing them, and treat the
platform framing as a direction rather than a v1.0 deliverable. That path scores meaningfully
higher than the briefed one on every dimension that decides whether the work matters, and it is
the only version of this project with a credible route from 4.7 to something that deserves the
word world-class.

Subplan 22 re-scores after the full plan exists. If these numbers have not moved by then, the
planning did not do its job.
