# 02. Idea validation and competitive landscape

**Status:** **revised 2026-09-15.** The original scope cut is withdrawn; see the revision banner and the corrected differentiation grades below.
**Date:** 2026-09-14
**Decides:** whether the project as briefed is worth building, and if not, what to build instead
**Depends on:** `00-research-log.md` sections 1, 2, 3
**Feeds:** subplans 03, 21, 23, and `FINAL-DECISION.md`

---

> **REVISED 2026-09-14.** The first version of this subplan recommended cutting circuit-logic
> compilation, generative design, 3D visualization, the desktop app, and genome-scale search from
> v1.0. **That recommendation is withdrawn.** It rested on two errors: a performance conclusion
> drawn from one library (corrected in subplan 09), and an assumption that delivering these
> features meant reimplementing them. Licence research showed otherwise. The revised position is
> below; the competitive analysis that preceded it was sound and is unchanged.

## The verdict, stated first

**The project is worth building, and the brief's full feature set is deliverable, because almost
none of it needs to be built from scratch.**

The historical failure mode is real: TinkerCell attempted this unification and did not survive,
and the reason is that each subsystem was a multi-year build. The escape from that trap is not to
cut the subsystems. It is to notice that **the field's tooling is dormant, permissively licensed,
and therefore available.**

| Asset | Licence | What it means for us |
|---|---|---|
| Cello (875 stars, dormant since 2022) | **BSD-2-Clause** | Logic synthesis can be ported, not reinvented. 142 forks exist already. |
| `sbol-rs` (v1.0.0, active) | MIT OR Apache-2.0 | 348 validation rules, verified working. |
| iBioSim (67 stars, maintained) | **Apache-2.0** | Simulation interop, and its algorithms are readable. |
| Evo 2 (4,213 stars) | **Apache-2.0** | Generative design integrated, not trained. |
| SecureDNA | MIT OR Apache-2.0 | Screening delegated to a peer-reviewed implementation. |
| Commec / Common Mechanism | open source | A second screening backend. |
| dnaplotlib (304 stars) | **MIT** | SBOL Visual glyph geometry, readable reference. |
| pySBOL3 | MIT | Cross-implementation oracle. |
| `fm-index`, `sucds` | permissive | Compressed indexing at ~3.9 bytes/base, measured. |

**Every significant component of the brief's scope has a permissively licensed, working
implementation that can be ported, wrapped, or depended upon.** The multi-year estimate assumed
greenfield construction of things that already exist and whose authors have stopped maintaining
them.

That changes the strategic question from "which features do we cut?" to "what do we add that
nobody has?" The answer to the second question is in the revised differentiation section below,
and it is a larger idea than the original plan contained.

---

## The competitive landscape, honestly

All activity figures verified via GitHub API on 2026-09-14. Star counts are a crude proxy for
adoption, but in a field this small they are the most honest signal available, and their absolute
magnitude is itself a finding.

### Design and CAD

| Tool | Stars | Last push | Language | What it does well | Where it falls short |
|---|---:|---|---|---|---|
| **Cello** (CIDARLAB) | 875 | 2022-09-19 | Java | The field's landmark result. Compiles Verilog to working genetic circuits with characterized gate libraries, experimentally validated in E. coli. Nothing else does logic synthesis this well. | Dormant four years. Java, hard to install, hard to extend. Gate libraries are narrow. No uncertainty reporting: it hands you a circuit, not a confidence. |
| **Cello 2.0** | 74 | 2022-05-16 | Java | Extended Cello beyond E. coli plasmids to genomic contexts, added Verilog 2005, added SynBioHub integration and a GUI. | Also dormant. The 2.0 rewrite never displaced the original in usage, judging by stars. |
| **SBOLDesigner** | 31 | 2021-06-04 | Java | Sequence-level design editor with SBOL round-tripping. | Dead five years. |
| **SBOLCanvas** | 17 | 2026-09-10 | TypeScript | Actively maintained. Web-based visual SBOL editor, which is the right delivery model. | Small. Canvas editing only, no simulation, no analysis. |
| **GCAD** | n/a | 2026-03 | n/a | New (ACS SynBio, March 2026). Mammalian genetic program CAD. Shows the space is not frozen. | Mammalian focus, so adjacent rather than overlapping with a prokaryote-first system. Not yet assessed for adoption. |
| **GP-write CAD** | n/a | n/a | n/a | Whole-genome design with **direct ordering of synthetic DNA from industrial partners**. | Relevant chiefly as prior art for the design-to-synthesis boundary, which is where our screening gate sits. |

### Simulation

| Tool | Stars | Last push | What it does well | Where it falls short |
|---|---:|---|---|---|
| **iBioSim** | 67 | 2026-04-07 | Genuinely capable. SBML and SED-ML support, stochastic and ODE simulation, model analysis, still maintained. The most serious simulation tool in the open ecosystem. | Java desktop application. Learning curve. Not designed to be driven programmatically from a notebook. |
| **TinkerCell** | n/a | abandoned | Attempted the exact unification this brief describes: CAD plus simulation plus plugins. | **It is dead.** This is the single most important data point for scoping. Somebody already tried this and it did not survive. |

### Standards, data, and libraries

| Tool | Stars | Last push | Notes |
|---|---:|---|---|
| **SynBioHub** | 84 | 2026-09-06 | Active. The field's design repository. REST API plus a SPARQL endpoint. Real infrastructure. |
| **pySBOL3** | 48 | 2026-09-09 | Active. The most maintained SBOL library in the ecosystem, and it is Python. |
| **sbol-rs** | 5 | 2026-09-08 | New (v1.0.0, July 2026). Official SynBioDex Rust implementation of SBOL 3.1.0 and 2.3.0, with conversion, fuzzing, benchmarks. Apache-2.0. |
| **SBOL-utilities** | 22 | 2025-10-26 | Slower, still alive. |
| **libSBOLj3** | 2 | 2026-04-16 | Java. Low attention. |

### Visualization

| Tool | Stars | Last push | Notes |
|---|---:|---|---|
| **dnaplotlib** | 304 | 2024-03-20 | VoigtLab. The most-used SBOL Visual renderer. Python, matplotlib-based. Slowing but not dead. |
| **paraSBOLv** | 17 | 2022-08-09 | Dormant. Parametric SBOL Visual glyph rendering. Worth reading for glyph geometry. |

### Biosecurity screening

| Tool | Stars | Last push | Language | Notes |
|---|---:|---|---|---|
| **SecureDNA** | 21 | 2026-07-16 | **Rust** | Apache-2.0. Full monorepo with crates, frontend, certificate infrastructure. Privacy-preserving screening protocol. Has an independent security analysis published. |
| **Common Mechanism** (IBBIS) | 47 | 2026-09-03 | Python | Free, open-source, globally available DNA sequence screening. |

### The AI-for-biology side of the same field

| Tool | Stars | Last push |
|---|---:|---|
| **Evo 2** (Arc Institute) | **4,213** | 2026-06-19 |

---

## What the landscape data actually says

### 1. The fragmentation is real. The brief did not overstate it.

Cello dormant since 2022. SBOLDesigner dead since 2021. paraSBOLv dead since 2022. TinkerCell
abandoned. The tools that survive are narrow: SynBioHub stores things, pySBOL3 parses things,
SBOLCanvas draws things, iBioSim simulates things. Nothing connects them. A user who wants to go
from "design a circuit" to "simulate it" to "check it is safe to order" is manually moving files
between four tools written in three languages by four groups, two of which stopped maintaining
their tool during the pandemic.

That is a real, felt problem and the brief correctly identifies it.

### 2. The field is much smaller than the brief implies, and this is the finding that hurts.

The brief's ambition is for Integrase to "become the reference implementation in its field." Here
is what being the reference implementation in this field currently looks like: 875 stars, and
abandoned. The most-starred *maintained* tool in the core ecosystem has 84.

For calibration, a mid-tier JavaScript utility library has more stars than the entire genetic CAD
ecosystem combined. This is not a market. It is a research community, and research communities
adopt tools through papers, collaborations, and teaching, not through GitHub trending.

Meanwhile Evo 2 has 4,213 stars. The attention, the funding, and the talent in computational
biology have moved decisively toward genomic foundation models. A new CAD tool is entering a
quiet room while the party is next door.

**What this changes:** every adoption assumption in the plan. Subplan 03 has to score the adoption
path against this evidence and not against wishful thinking. Subplan 21's phasing should optimise
for producing one genuinely citable, genuinely useful thing rather than for platform completeness,
because in this field a tool that gets cited gets adopted and a tool that is merely comprehensive
gets ignored.

### 3. TinkerCell is the warning label.

TinkerCell tried to be the unified CAD-plus-simulation-plus-plugin platform for synthetic biology.
It had academic backing and a real user base. It is gone.

The failure mode for unification platforms is well understood and the brief walks straight into it:
each subsystem is individually a multi-year project, none of them is good enough on its own to
attract users during the years before the others exist, and the integration value only materialises
once all of them are done. The project dies in the trough.

**This is the argument for cutting scope, and it is not a hedge.** It is the single most reliable
prediction that can be made about this project from the historical record.

---

## Differentiation: which claims survive contact with the research

The brief asserts six differentiators, explicitly or implicitly. Graded honestly.

### REAL: Calibrated uncertainty as a product principle

**No existing tool does this.** Cello hands you a circuit. The RBS Calculator hands you a number.
iBioSim hands you a trajectory. None of them tells you how much to trust the answer, and the
literature is unambiguous that the answer often should not be trusted much: thermodynamic RBS
prediction ranges from R² > 0.8 to R² < 0.2 depending on the dataset (`00-research-log.md` 4.2).
A tool that reports "predicted translation initiation rate 4,200 au" and a tool that reports
"4,200 au, 80% interval 1,900 to 9,100, and this sequence is outside the range where this model
was validated" are giving the user materially different information, and only one of them is
honest.

This is genuinely novel, genuinely valuable to the target user, and cheap to build relative to its
differentiating power. It is also the thing a working scientist will notice in the first five
minutes of using the tool.

**Verdict: real, and it should be the project's centre of gravity rather than a feature.**

### REAL: Screening as an architectural gate in a design tool

Screening exists (SecureDNA, Common Mechanism). Ordering exists (GP-write CAD offers direct
synthesis ordering). What does not exist is a **design environment where the screening gate is
structurally between design and export**, so that the path to physical synthesis cannot bypass it.

The brief's framing is correct and the placement is right: never restrict thinking, designing, or
simulating in the sandbox, only gate the boundary where a design would leave the system toward
physical synthesis. That mirrors how industry screening already works, and it is defensible to
both biosecurity people and to scientists who would reject a tool that lectured them.

Caveat that subplan 24 must resolve: this differentiator is only real if the screening is real,
and real screening needs reference data we do not currently have (`REQUESTS.md` R-002).

**Verdict: real, conditional on solving the reference-data problem, and worth building even if the
answer is "integrate SecureDNA rather than implement our own."**

### WEAK: Native SBOL2/SBOL3 support

This was a differentiator when the brief was written. It is not one now. pySBOL3 is active and
well-maintained, and `sbol-rs` shipped v1.0.0 in July 2026 from the standards body itself.
"We can read SBOL" is table stakes, not a wedge.

**Verdict: necessary, not differentiating. Adopt `sbol-rs`, do not build it. See subplan 23.**

### WEAK: Modern language stack

Rust, Go, and Python instead of Java matters for maintainability, performance, and contributor
appeal. It does not matter to a synthetic biologist choosing a tool. No user has ever adopted
scientific software because of its implementation language, and framing the stack as a
differentiator in user-facing material would read as engineer-centric naivety to the target
audience.

**Verdict: correct engineering choice, not a user-facing differentiator. Keep it out of the
README's pitch.**

### REAL, on revision: genome-scale sequence search

> Originally graded WEAK, on two grounds that both turned out to be wrong. Corrected 2026-09-15.

**Wrong claim 1: "`bio` already ships an FM-index, so this is solved."** It ships one that costs
**39.4 bytes per base** (measured, subplan 09), because it retains a full `u64` suffix array. That
puts a human genome at 113.8 GB. A compressed index measures **~3.9 bytes per base** on the same
data and hardware, putting the same genome near 12 GB, and tuned 2-bit packing should improve on
that again. No existing Rust bioinformatics crate offers the compressed form with the ergonomics we
need, so this is a real capability gap rather than a solved problem. Subplan 28 owns it.

**Wrong claim 2: "users work on plasmids, so nobody needs genome scale."** True of the design
artifact, false of the queries. Once the circuit compiler exists (subplan 25), genome-scale queries
fall directly out of ordinary workflows: finding off-target integration sites across a host genome,
checking a designed part against every sequenced isolate of a species, and screening a construct
against a pangenome. The user appears as soon as the rest of the system does.

**Verdict: real. Subplan 28 owns the index, and subplan 21 places it.**

### REAL, and arguably the centrepiece: uncertainty through circuit composition

Not in the brief's original list of differentiators, and on revision it is the strongest one the
project has.

Cello compiles logic to genetic circuits and returns a deterministic score. It does not model
context effects, and characterisation of real Cello circuits documents cryptic promoters,
transcription attenuation, alternative start sites, off-target interactions, and gates that fail
outright. Prior work on uncertainty-aware circuit scoring and interval propagation through gate
libraries exists (ACS Synthetic Biology, 2021), **and it lives in papers rather than in a maintained
tool.**

What nobody offers is calibrated uncertainty propagated through composition, with failure
attribution telling a designer *which gate* to characterise better. That combination is this
project's clearest reason to exist, and it requires exactly the compiler the original version of
this subplan cut. See subplan 25.

**This is the single strongest argument that cutting scope was the wrong call:** the cut removed the
component that the best differentiator depends on.

### CONTESTED: Unification itself

Unification is simultaneously the real problem (finding 1) and the historical graveyard
(finding 3). The resolution is not to abandon it but to redefine what unification means here.

**Bad unification:** reimplement CAD, simulation, logic synthesis, ML, and visualization in one
platform. This is TinkerCell. It fails.

**Good unification:** be the layer that makes the existing tools interoperate, via SBOL, with a
consistent API and honest uncertainty attached. Import Cello's output. Round-trip SynBioHub
designs. Export SBML for iBioSim. Add the two things nobody has (calibrated uncertainty, screening
gate). Do not rebuild what works.

**Verdict: keep the word and keep the scope. Integrase is connective tissue with original organs,
and the connective tissue is made of other people's permissively licensed code.**

The distinction that matters is not scope, it is **origin**. TinkerCell died building six
subsystems. Integrase ports one (Cello, BSD-2), depends on three (`sbol-rs`, SecureDNA,
`fm-index`), integrates one (Evo 2), and builds two originals: the confidence layer and the
verifiable design record. That is a fundamentally different amount of work for the same user-facing
surface.

---

## Revised scope: everything stays, delivered by integration

The brief invites restructuring during planning. This is that restructuring, and it restores rather
than removes.

### Delivered by porting or wrapping

| Brief item | Original recommendation | Revised | How |
|---|---|---|---|
| Circuit-logic compilation | Defer past v1.0 | **In scope, Phase 3** | Cello is BSD-2-Clause. Port its gate-assignment and UCF handling to Rust incrementally, with its published results as the correctness oracle. Subplan 25. |
| Generative sequence design | Defer | **In scope, Phase 4** | Evo 2 is Apache-2.0 with open weights. Integrate as a `Model` behind the same `Prediction<T>` contract, so generated sequences carry uncertainty and provenance like everything else. Subplan 26. |
| Genome-scale search | Defer until needed | **In scope, Phase 2** | My performance conclusion was wrong. Compressed indexing measures ~3.9 bytes/base. Subplan 28. |
| 3D visualization | Defer | **In scope, Phase 4, redefined** | Not 3D constructs, which are linear and where 3D is actively worse. 3D where the data is genuinely three-dimensional: RNA secondary structure, protein structure, supercoiling. Subplan 27. |
| Desktop app | Defer to after web | **In scope, Phase 5** | Tauri v2 wrapping the same frontend. Unchanged reasoning, later position. |

### The addition that changes the project's ceiling

Restoring features does not by itself make this a better idea. What does is noticing what the
restored scope makes possible that the cut scope did not.

**Verifiable Design Records.** A cryptographically signed, portable record binding a design to
every prediction made about it, the screening verdict that cleared it, and the provenance chain it
came from, including whether a generative model produced it.

Nobody has this. It is not in SBOL, which has provenance vocabulary but no cryptographic binding
and no screening attestation. And three forces converging in 2026 make it urgent rather than
merely clever:

1. **The October 2026 compliance deadline.** The OSTP framework threshold drops to 50 bp and the
   sequence-of-concern definition expands, with IGSC conformance due 2026-10-24. Providers need to
   demonstrate screening. A design-time attestation a provider can verify is directly useful to a
   market with a date on it.
2. **AI-generated sequences.** Evo 2 generates DNA. Commec v2.0's July 2026 release specifically
   added sensitivity to AI-redesigned proteins split into short fragments. The field's emerging
   question is *where did this sequence come from and was it checked*. A signed provenance chain
   answers it.
3. **The calibration gap.** Foundation models produce sequences with no uncertainty attached. A
   record that carries calibrated predictions and their model identity is the honest interchange
   format the generative era lacks.

This is developed fully in subplan 29. It is the difference between "a good CAD tool in a small
field" and "infrastructure a field needs."

### What stays cut

| Item | Why |
|---|---|
| Reimplementing SBOL | 348 validation rules exist upstream, verified. Depending is correct. |
| Our own screening matcher | A false negative is the worst failure available. Delegate. |
| 3D rendering of linear constructs | The data is 1D. SBOL Visual is 2D because that communicates. 3D goes where 3D data is. |
| Operating a hosted service ourselves | We publish images. |


## Honest risks that scoping does not fix

1. **The field may be too small to sustain the project.** No amount of engineering quality fixes an
   addressable community this size. Mitigation is to make the tool useful to the adjacent, larger
   ML-for-biology audience through the Python SDK, so it is not betting solely on the CAD crowd.
2. **Reference data for screening may be unobtainable.** `REQUESTS.md` R-002. If it is, the
   screening differentiator degrades to "a well-designed integration point with nothing behind it,"
   which is honest but much less compelling.
3. **Single-maintainer sustainability.** Every dormant tool in the tables above was maintained by a
   small academic group that moved on. A solo open-source project has the same failure mode with
   fewer people. Subplan 19 has to take governance and bus-factor seriously rather than treating it
   as paperwork.
4. **Uncertainty quantification is harder than it sounds.** Claiming calibrated intervals means
   being able to demonstrate calibration on held-out data, which requires data we have not yet
   sourced (subplan 04) and evaluation methodology that is genuinely hard (subplan 08). If the
   calibration is not real, the central differentiator is a lie, and it would be a worse lie than
   the silence the existing tools offer.

Risk 4 is the one that should worry us most, because it turns the project's greatest strength into
its greatest liability if executed carelessly. Subplan 08 and subplan 17 both have to treat
calibration testing as a first-class requirement, not a nice-to-have.

---

## Open questions for downstream subplans

- [ ] Does any realistic user scenario actually require genome-scale sequence search? -> **11**
- [ ] Can Cello's UCF format and output be imported without reimplementing Cello? -> **06**
- [ ] Is there enough public RBS/promoter data to demonstrate calibration honestly? -> **04**, **08**
- [ ] Does integrating SecureDNA or Common Mechanism preserve the screening differentiator, or does
      it reduce Integrase to a thin wrapper? -> **24**
- [ ] Is the ML-for-biology audience reachable through a Python SDK, or is that wishful? -> **21**
