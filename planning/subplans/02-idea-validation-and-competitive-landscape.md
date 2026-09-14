# 02. Idea validation and competitive landscape

**Status:** decided, with a scope change recommended
**Date:** 2026-09-14
**Decides:** whether the project as briefed is worth building, and if not, what to build instead
**Depends on:** `00-research-log.md` sections 1, 2, 3
**Feeds:** subplans 03, 21, 23, and `FINAL-DECISION.md`

---

## The verdict, stated first

**The project is worth building. The project as scoped in the brief is not.**

The brief describes a platform that unifies genetic circuit CAD, biophysical simulation, ML-guided
part design, visualization, circuit-logic compilation, and a biosafety gateway, across three
languages, with a desktop app and a web target. That is six substantial subsystems, and the
historical record of the field is that unification platforms attempting this scope die before
reaching usefulness. TinkerCell is the cautionary example: it attempted exactly this unification,
and it is no longer maintained.

Two of the brief's claimed differentiators are real, defensible, and not served by any existing
tool. Four are weak, and two of those four were quietly commoditized while this brief was being
written. The recommendation is to keep the identity and cut the scope hard around the two real
ones.

This is the most important finding in the planning folder, and subplan 21's phasing and
`FINAL-DECISION.md` both follow from it.

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

### WEAK: Genome-scale sequence search performance

The brief specifies FM-index-backed motif and restriction-site search at genome scale. `bio`
already ships an FM-index. More importantly, it is unclear this is a felt pain point: the designs
these users work with are plasmids and constructs measured in kilobases, not genomes. Fast search
over a 3 Gb genome is a capability in search of a user.

**Verdict: build it when a scenario demands it, not before. Subplan 11 should be honest about
whether any real scenario needs it.**

### CONTESTED: Unification itself

Unification is simultaneously the real problem (finding 1) and the historical graveyard
(finding 3). The resolution is not to abandon it but to redefine what unification means here.

**Bad unification:** reimplement CAD, simulation, logic synthesis, ML, and visualization in one
platform. This is TinkerCell. It fails.

**Good unification:** be the layer that makes the existing tools interoperate, via SBOL, with a
consistent API and honest uncertainty attached. Import Cello's output. Round-trip SynBioHub
designs. Export SBML for iBioSim. Add the two things nobody has (calibrated uncertainty, screening
gate). Do not rebuild what works.

**Verdict: keep the word, change the meaning. Integrase is connective tissue with two original
organs, not a replacement skeleton.**

---

## Recommended scope change

The brief invites restructuring during planning as long as the core identity survives. This is
that restructuring.

### Cut or defer

| Brief item | Recommendation | Reason |
|---|---|---|
| Circuit-logic compilation (Cello-equivalent) | **Defer past v1.0.** Import Cello output instead. | Logic synthesis with characterized gate libraries is a multi-year research project and Cello already did it well. Reimplementing it is the fastest way to spend two years and ship nothing. |
| Generative sequence design | **Defer.** | Evo 2 exists, is open-weight, and is better than anything this project would train. Integrate later; do not compete. |
| 3D construct visualization | **Defer.** | The brief asks for 3D rendering of constructs. Genetic constructs are linear sequence diagrams. SBOL Visual 2D glyph rendering is what users actually need and what the standard specifies. 3D is a demo feature, not a user need. Subplan 13 should push back on this explicitly. |
| Desktop app | **Defer to after the web/SDK path.** | SBOLCanvas being web-based and maintained while the Java desktop tools died is evidence about delivery model. |
| Genome-scale FM-index search | **Defer until a scenario needs it.** | See above. |

### Keep and centre

| Item | Why |
|---|---|
| Calibrated uncertainty throughout | The real differentiator. Everything else supports it. |
| Biosafety screening gateway | The second real differentiator, and the ethically load-bearing one. |
| SBOL interop via `sbol-rs` | Table stakes, cheap now that `sbol-rs` exists, and the thing that makes interop-as-unification possible at all. |
| Deterministic biophysical models (RBS, promoter) with honest error reporting | The substrate the uncertainty story runs on. Without a prediction there is nothing to attach an interval to. |
| Python SDK | Where the users actually are. pySBOL3's activity proves it. |

### The one-line version of the scope change

Build the tool that answers "here is my construct, what will it do, how much should I trust that,
and is it safe to order," and interoperate with everything else rather than replacing it.

---

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
