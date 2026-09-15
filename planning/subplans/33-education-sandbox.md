# 33. The education sandbox

**Status:** decided
**Date:** 2026-09-15
**Decides:** what a student-facing simulation sandbox simulates, how it is architected, and
where simulation stops and authored content begins
**Depends on:** subplans 30 (adoption), 31 (ethics), 06 (architecture), 12 (API design)
**Feeds:** subplan 21 (phasing), and the adoption strategy that depends on reaching students

---

## Why this is in scope rather than scope creep

Subplan 30 identified iGEM as the project's single best distribution channel: **4,600+
participants and 400+ teams a year**, overwhelmingly undergraduates, whose first tools become the
tools they specify as PIs. That subplan proposed reaching them with the professional tool. This
subplan proposes reaching them with something built for them.

There is a second, better reason. **School genetics teaches a model that is, in places, flatly
wrong**, and a tool whose entire purpose is refusing to overstate certainty is unusually well
placed to teach it correctly. The education sandbox is not a detour from the honesty thesis. It may
be its clearest demonstration.

---

## What "realistic" can actually mean

The ceiling is higher than intuition suggests, and the shape of the ceiling determines the
architecture.

**The reference point.** Karr et al. 2012 produced the first whole-cell computational model of a
complete organism: *Mycoplasma genitalium*, the simplest known free-living bacterium. It covered
**401 genes, 722 compounds, 1,857 reactions, 1,836 parameters, across 28 separate submodels.**
Whole-cell models of *E. coli* now exist, integrating 788 operons and 1,231 transcription units.

**The architectural finding.** Those models do not use one simulation engine. Their subsystems are
handled by **different methods composed together**: ordinary differential equations for some, flux
balance analysis for others, stochastic simulation for the rest.

Sources: https://www.cell.com/fulltext/S0092-8674(12)00776-3 ,
https://www.nature.com/articles/s41540-022-00242-9 ,
https://pmc.ncbi.nlm.nih.gov/articles/PMC11520977/

This matters because it means **"semi-realistic, composed from pieces" is not a compromise. It is
what the state of the art does.** The architecture below follows that pattern deliberately.

### What is genuinely simulatable

| Layer | Method | Fidelity |
|---|---|---|
| DNA to RNA to protein | Deterministic, real codon table, real reading frames | **Exact.** No approximation. |
| Gene regulatory networks | Gillespie stochastic simulation, or ODEs | **Genuine.** This is what iBioSim and COPASI do. |
| Amino acid substitution consequences | Charge, size, hydrophobicity change | **Real, and explanatory.** Sickle cell is Glu to Val: negative and polar becomes neutral and hydrophobic, which is *why* the protein aggregates. |
| Metabolism | Flux balance analysis | Real technique, simplified parameters |
| Environment | Arrhenius rate dependence, nutrient limitation, pH | Standard physical chemistry |
| A single simple cell | Composition of the above | Achievable at bacterium scale |
| Tissue, many cells | Agent-based, cells as agents | Simplified but real in kind |

### Where simulation stops

**A whole human body is not simulated. It is authored.** Nothing in the field simulates development
from genome to organism, and claiming to would be the exact overstatement this project exists to
refuse.

The honest structure is two distinct things, clearly labelled as different in the interface:

- **Simulated:** molecule through to cell, and small populations of cells. Real methods, real
  parameters, honest uncertainty.
- **Authored:** cell through to tissue, organ, and body. Hand-written explanatory content with
  diagrams. A guided tour, not a computation.

A student must always be able to tell which one they are looking at. Subplan 14's confidence
grammar extends naturally here: simulated results carry their uncertainty encoding; authored
content is visually distinct and never displays a fake confidence interval.

---

## Architecture

### The design that follows from the Karr pattern

One shared state object, many independent processes that read and modify it, and a scheduler that
steps them. This is 28 submodels sharing a cell, generalised.

```python
@dataclass
class CellState:
    """Everything about a cell at one instant. The shared blackboard."""
    genome: Genome
    rna: Counts            # molecule species -> count
    protein: Counts
    metabolites: Counts
    environment: Environment
    time: float

class Process(Protocol):
    """One biological process. An interface, not a base class."""
    def step(self, state: CellState, dt: float) -> None: ...

class Transcription(Process): ...
class Translation(Process): ...
class Degradation(Process): ...
class Metabolism(Process): ...
class Division(Process): ...

class Simulator:
    processes: list[Process]

    def step(self, dt: float) -> None:
        for p in self.processes:
            p.step(self.state, dt)
```

**Why a `Protocol` rather than an abstract base class.** A student writing a new process should not
have to inherit from anything or call `super().__init__()`. They write a class with a `step` method
and add it to the list. That is the single best exercise this sandbox offers, and inheritance would
put a barrier in front of it.

### The containment hierarchy is composition

Biology genuinely is objects inside objects, and modelling it that way is correct:

```
Organism
  └─ Tissue          has many Cells
      └─ Cell        has a Genome, an RNA pool, a Protein pool, an Environment
          └─ Genome  has Chromosomes
              └─ Chromosome has Genes
                  └─ Gene has a Sequence
                      └─ Sequence has Bases
```

The zoom navigation the sandbox needs is a walk of this tree. **The architecture is the lesson**: a
student learns the body is layered by moving through the layers.

### What is deliberately not inherited

```python
class Cell: ...
class LiverCell(Cell): ...          # rejected
class Hepatocyte(LiverCell): ...    # rejected
```

Cell type is **data on a cell**, not a subclass of it, for the same reason a feature's role is an
ontology IRI rather than an enum in `integrase-core` (subplan 06). Biology does not sort into a
clean tree, an inheritance hierarchy forces a breaking change every time it fails to, and the
escape hatch variant becomes the most-used one.

Inheritance is used in exactly one place, where it is genuinely an is-a relationship: concrete
`Process` implementations sharing a small amount of numerical helper code. Even there, composition
is preferred if it is close.

### The constraint that makes it run at all

**Model concentrations, not individual molecules.** A real cell contains billions. One `Protein`
entry with a count is how every real simulator works, and it is the difference between this
executing and not.

A student who wants to see individual molecules gets a visualisation of a sampled subset, clearly
labelled as illustrative rather than as the simulation state.

---

## What students can change

Four levers, in increasing order of depth. The fourth is the one that makes this a programming
education tool and not a toy.

1. **Edit a base.** Change one letter, watch it propagate up through codon, amino acid, protein
   property, and cell behaviour.
2. **Change the environment.** Temperature, nutrients, pH. Real rate effects.
3. **Rewire a network.** Connect a sensor gene to a different output gene.
4. **Write a new `Process`.** Add a behaviour the sandbox did not ship with, in Python, and watch it
   participate in the simulation.

---

## Teaching content, and the eye colour correction

### Diseases that genuinely are one small change

These carry full accuracy and are the right starting content:

| Condition | The change | Why it teaches well |
|---|---|---|
| **Sickle cell anaemia** | One base: A becomes T in *HBB* | Glu to Val. Negative and polar becomes neutral and hydrophobic, so the protein aggregates. The mechanism is visible at every layer. |
| **Cystic fibrosis** (ΔF508) | Three bases deleted from *CFTR* | A protein that misfolds and never reaches the membrane |
| **Huntington's disease** | A CAG sequence repeated too many times | Repeat count maps to age of onset: a dose-response a student can explore |

### Eye colour is a correction, not a lesson in simple inheritance

**The model taught in schools is wrong and must not be reproduced here.**

Eye colour involves roughly **16 genes**. The dominant single factor, rs12913832, is not even inside
a pigment gene: it sits in a regulatory region of the neighbouring *HERC2* gene and controls how
much *OCA2* is expressed. The classic brown-dominant, blue-recessive model is provably false,
demonstrated by the not-uncommon case of two blue-eyed parents having a brown-eyed child.

Sources: https://www.nature.com/articles/jhg2010126 ,
https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10454093/

**So eye colour ships as the lesson in why biology resists simple rules.** A student changes the
variant and the sandbox reports a shift in probability, not a determined outcome, alongside the
other contributing genes. Most tools cannot teach this because they do not model uncertainty. This
one does, and it is the single clearest demonstration of why the uncertainty machinery matters.

The same treatment applies to diabetes, heart disease, and most cancers: many genes plus
environment, reported as shifted odds.

---

## Drug design: a puzzle, clearly labelled

Real computational drug design uses molecular physics and large compute, and it is still weak. The
numbers are the curriculum:

> About **90% of drug candidates entering clinical trials fail.** Phase I passes 52%, Phase II drops
> to 28.9%, and only **8.1%** reach approval. Over half of late-stage failures are lack of efficacy;
> another 17% are adverse effects. Twelve-plus years and more than $2.5 billion per approved drug.

Source: https://www.mdpi.com/1424-8247/19/6/916

**What ships:** a simplified shape-and-property matching puzzle that builds intuition about why
molecular structure matters.

**What is required alongside it:** a permanent, non-dismissable marker that this is a toy model and
not a prediction about reality. The project already has the machinery for this in `DomainVerdict`;
a student-designed molecule is permanently `Uncalibrated`.

**What is explicitly not built:** anything that could be mistaken for real drug design, and any
scoring that implies a molecule would work in a person.

### Discovery is not a goal, and saying so is important

A simulator contains only what was put in it. A student who "cures" something inside it has found a
solution someone programmed, which is a fine puzzle and is not discovery. **Presenting it as
discovery would be a genuine harm**, because it teaches a false idea of what solving a medical
problem requires.

The better design, and the honest one: **let students experience why treatments fail.** The gene is
fixed but the delivery does not reach the tissue. It works in the model and causes an off-target
effect. A student who finishes understanding why 90% fail is far better educated than one who won a
game.

---

## Safety: curation, not intent detection

The requested constraint was to restrict the sandbox to beneficial uses. **That is not
implementable, and the reason is worth recording rather than attempting.**

**Intent is not in the data.** The edit that disables a harmful gene is the same edit, reversed,
that enables it. The software sees identical letters.

**The same knowledge cures and harms.** Understanding how a pathogen causes disease is exactly what
is required to counter it. This is the dual-use problem, it is permanent, and subplan 31 covers it
for the project as a whole.

Any intent filter would be either trivially bypassable or so restrictive it blocks legitimate
learning.

### What works instead

| Control | Why it is reliable |
|---|---|
| **Curated content only** | The sandbox contains the genes, organisms, and molecules we ship. No pathogen data, no toxin genes. A student cannot misuse content that is absent. This is a content decision, not a detection problem. |
| **Invented sequences** | The organism is fictional. Nothing in the sandbox is a real orderable construct. |
| **No path to synthesis** | The sandbox has no export toward physical DNA at all, so the screening gate in `integrase-screen` is not weakened; it is simply not reachable from here. |
| **Real diseases are read-only case studies** | Sickle cell and the rest are presented as worked examples on fictional sequence, not as editable pathogen material. |

**This is stronger than a filter, not weaker.** A filter guesses at intent and fails. Curation
decides what exists.

---

## Implementation notes

**Language: Python first.** This is the one part of the project where Python leads rather than
follows. Students read and write the model, `Process` implementations are the core exercise, and at
educational scale (hundreds of genes, not millions of molecules) Python with numpy is fast enough.
Per R-5.4, no optimisation without a profile. The Rust core is available through the existing
bindings for sequence operations if they become the bottleneck, which is a measurement to take
rather than assume.

**Uncertainty is not optional here either.** Simulation parameters are estimates, and the sandbox
reports them as such. A student should see that a rate constant is a measured value with error, not
a fact.

**Delivery:** a separate surface from the professional tool, sharing the sequence core. Different
users, different fidelity expectations, different safety profile. It is not a mode hidden inside the
CAD tool.

---

## Honest limitations

1. **Content authoring is the real cost.** The engine is the smaller half. Someone has to write
   accurate, pedagogically sound lessons, and getting the biology wrong in a teaching tool is worse
   than getting it wrong in a research tool.
2. **It is not a whole-cell model.** Karr's took a large team years for the simplest organism known.
   This is an educational simulator using the same architectural pattern at far lower fidelity.
3. **Parameters are largely illustrative.** Real kinetic constants exist for a small fraction of
   reactions. Where a number is invented for teaching, it is marked.
4. **Multicellular fidelity is low**, and body-level content is authored rather than simulated.
5. **This is a significant body of work** and competes with Phase 1a for attention. Subplan 21 has
   to place it honestly rather than assume it is free.

---

## Open items

- [ ] Decide where this sits in the phase order. It serves adoption, which subplan 22 scored as the
      binding constraint, so it may deserve to come earlier than its size suggests.
- [ ] Identify a biology educator to review content accuracy. Getting teaching material wrong is the
      main risk here and it is not one I can check alone.
- [ ] Choose the stochastic simulation implementation, or write the Gillespie loop directly
- [ ] Decide the fictional organism's scope: how many genes, which processes
- [ ] Confirm licensing for any real parameter sets used
