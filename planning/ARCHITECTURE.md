# Architecture

**Date:** 2026-09-14
**Status:** authoritative. Synthesises subplans 05, 06, 12, 13, 15, 16, and the calls made in
`FINAL-DECISION.md`.
**Audience:** two. A senior engineer who needs to understand the system's shape in one read, and a
synthetic biologist who needs to judge whether it is trustworthy. The second audience is why the
sections on uncertainty and screening carry as much weight as the module breakdowns.

---

## 1. What the system is

Integrase reads genetic designs, predicts how they will behave, reports how much to trust those
predictions, and refuses to help you order anything that fails biosecurity screening.

Three properties distinguish it structurally from every other tool in the field, and each is
enforced by the type system rather than by convention:

1. **No predicted value exists without its uncertainty.** `Prediction<T>` carries the value, its
   interval, the model that produced it, and a verdict on whether the input was inside that model's
   validated range. There is no API returning a bare number.
2. **No design reaches a synthesis-bound export without screening.** `Cleared<T>` is constructible
   only inside the screening crate, and synthesis-bound export functions accept nothing else.
3. **Confidence rendering is computed, never authored.** A single function maps a domain verdict to
   a visual encoding, so a speculative result cannot be drawn as a validated one.

What it explicitly is not: it does not edit genomes, does not interact with living organisms, does
not order DNA, and does not predict what will happen in a cell. It predicts what published
biophysical models say about a sequence, and reports how much those models know.

---

## 2. System context

```mermaid
flowchart TB
    subgraph ecosystem["Existing ecosystem (we interoperate, not replace)"]
        direction LR
        SBH[("SynBioHub<br/><i>auth required</i>")]
        IGEM[("iGEM Registry<br/><i>CC BY-SA</i>")]
        BIOM[("BioModels<br/><i>CC0</i>")]
        CELLO[/"Cello output<br/><i>UCF + SBOL</i>"/]
        IBIO["iBioSim<br/><i>via SBML export</i>"]
    end

    subgraph surfaces["User surfaces"]
        direction LR
        PY["Python SDK<br/><b>primary</b>"]
        CLI["CLI"]
        WEB["Web UI<br/><i>Phase 2</i>"]
        APP["Desktop<br/><i>Phase 4</i>"]
    end

    subgraph integrase["Integrase"]
        direction TB
        ADAPT["integrase-sbol<br/><i>adapter over sbol-rs</i>"]
        CORE["integrase-core<br/><i>Sequence, Design, Feature</i>"]
        SIM["integrase-sim<br/><i>models + Prediction&lt;T&gt;</i>"]
        VIZ["integrase-viz<br/><i>SBOL Visual + cartography</i>"]
        SCREEN["integrase-screen<br/><b>the gate</b>"]
    end

    subgraph external["External services"]
        SDNA["SecureDNA synthclient<br/><i>local, DOPRF</i>"]
        SBOLRS["sbol-rs<br/><i>348 validation rules</i>"]
    end

    subgraph exits["Leaving the system"]
        INTER["Interchange<br/><i>SBOL, SVG, working files</i>"]
        SYNTH["Synthesis-bound<br/><i>vendor order formats</i>"]
    end

    SBH --> ADAPT
    IGEM --> ADAPT
    CELLO --> ADAPT
    BIOM -.fixtures.-> SIM
    ADAPT <--> SBOLRS
    ADAPT <--> CORE
    CORE --> SIM
    SIM --> VIZ
    SIM --> surfaces
    CORE --> surfaces
    VIZ --> surfaces
    SIM -.SBML.-> IBIO

    CORE --> INTER
    VIZ --> INTER

    CORE --> SCREEN
    SCREEN <--> SDNA
    SCREEN -->|Cleared only| SYNTH

    classDef gate fill:#7a2020,stroke:#d04545,stroke-width:3px,color:#fff
    classDef conf fill:#1d4d4d,stroke:#3fa7a7,stroke-width:2px,color:#fff
    classDef later fill:#2a2a2a,stroke:#666,stroke-dasharray:4 4,color:#bbb
    class SCREEN,SYNTH gate
    class SIM conf
    class WEB,APP later
```

**Read the two edges into `exits`.** Interchange leaves directly, because a design that came from
the ecosystem returns to it and we are not the synthesis boundary there. Synthesis-bound has exactly
one inbound edge, from the gate, and there is no path around it. That is the architecture's central
claim, and it is visible rather than asserted.

---

## 3. The confidence pipeline

The mechanism behind property 1. This is the part a biologist should read most carefully, because it
is where the trustworthiness claim either holds or does not.

```mermaid
flowchart LR
    SEQ["Sequence<br/>+ Features"] --> DET["Deterministic model<br/><i>thermodynamic</i>"]
    SEQ --> DOM["Domain check"]

    DET --> RAW["Raw estimate"]
    DOM --> VERDICT{"DomainVerdict"}

    subgraph checks["Domain signals"]
        direction TB
        H["host organism"]
        S["start codon"]
        SP["spacer length"]
        ST["secondary structure"]
        EMB["embedding distance<br/><i>Phase 3</i>"]
    end
    checks --> DOM

    RAW --> CONF["Split conformal<br/><i>calibration set</i>"]
    CONF --> INT["Interval<br/><i>nominal coverage</i>"]

    ML["ML correction<br/><i>Phase 3, optional</i>"] -.-> COMPARE{"Disagrees with<br/>baseline?"}
    RAW --> COMPARE
    COMPARE -->|yes| FLAG["Surface the<br/>disagreement"]

    INT --> PRED["Prediction&lt;T&gt;"]
    VERDICT --> PRED
    FLAG -.-> PRED
    MC["ModelCard<br/><i>accuracy, datasets,<br/>failure modes</i>"] --> PRED

    PRED --> ENC["encoding_for(verdict)"]
    ENC --> VIS["Visual encoding<br/><i>fill, border, caps</i>"]
    PRED --> API["SDK / gRPC / CLI"]

    classDef conf fill:#1d4d4d,stroke:#3fa7a7,stroke-width:2px,color:#fff
    class PRED,INT,VERDICT conf
```

### Why the interval and the verdict are separate fields

This is the single most important design decision in the system and the easiest one to get wrong.

Split conformal prediction gives a **finite-sample coverage guarantee under exchangeability** between
the calibration set and the test point. Out-of-distribution inputs are precisely where
exchangeability fails, so the guarantee evaporates exactly where a user most needs it.

Collapsing the two into one widened interval would hide that. So:

| Field | Answers |
|---|---|
| `interval` | Given that this input resembles my calibration data, how wrong am I likely to be? |
| `domain` | Does this input resemble my calibration data at all? |

A wide interval on an in-distribution input means the model is honestly uncertain. A narrow interval
on an out-of-distribution input means the interval is not trustworthy. **Those are different
situations and a single number cannot express both.**

### What the intervals do not cover

Stated here rather than only in the docs, because an architecture document that omits it is
misleading a reader who is deciding whether to trust the system.

Genomic context, supercoiling, host physiology, growth conditions, plasmid copy number, and
metabolic burden are real, substantial, and invisible to any sequence-only model. **The intervals
quantify uncertainty about the model's own prediction, not about what will happen in a cell.** No
interval computed from sequence can fix that, and the system does not claim otherwise.

---

## 4. The screening gateway

The mechanism behind property 2.

```mermaid
flowchart TB
    D["Design"] --> CLASS{"Which export<br/>class?"}

    CLASS -->|interchange| FREE["write_sbol()<br/>write_svg()<br/><i>not gated</i>"]
    CLASS -->|synthesis-bound| GATE

    subgraph GATE["integrase-screen"]
        direction TB
        REQ["ScreeningRequest<br/><i>sequence, length, design id</i><br/><i>nothing else</i>"]
        BACKEND{"ScreeningBackend"}
        SDNA["SecureDnaBackend<br/><i>local synthclient</i>"]
        CM["CommonMechanism<br/><i>candidate</i>"]
        CUST["CustomBackend<br/><i>institutional</i>"]
        NULL["NullBackend<br/><b>default: refuses all</b>"]
        VERD{"Verdict"}
        AUDIT[("Audit log<br/><i>hash-chained,<br/>sequence hashes only</i>")]
    end

    REQ --> BACKEND
    BACKEND --> SDNA & CM & CUST & NULL
    SDNA & CM & CUST & NULL --> VERD

    VERD -->|clean| CLEARED["Cleared&lt;Design&gt;<br/><i>private field,<br/>constructible only here</i>"]
    VERD -->|hit| REFUSE["Refusal<br/><i>what, where, route forward</i>"]
    VERD -->|unreachable| REFUSE
    VERD -->|malformed| REFUSE
    VERD -->|timeout| REFUSE
    VERD --> AUDIT

    CLEARED --> EXPORT["export_for_synthesis()"]
    REFUSE --> STILL["Design stays fully usable<br/><i>edit, simulate, interchange</i>"]

    classDef gate fill:#7a2020,stroke:#d04545,stroke-width:3px,color:#fff
    class GATE,CLEARED,REFUSE,EXPORT gate
```

**Every uncertain outcome resolves to refusal.** Unreachable, malformed, timed out, and panicked all
converge on the same edge. There is no code path where "could not determine" becomes "allowed", and
subplan 17 tests this by fault injection rather than inspection, because it is exactly the property
that rots silently.

### Why we delegate rather than implement

A false negative in screening is the worst failure this system can produce. SecureDNA is
peer-reviewed, free, MIT/Apache-2.0, and runs on the client's own premises using a distributed
oblivious pseudo-random function, so **the client never holds the hazard database and the database
never sees plaintext sequences.** That is why the curated data is not downloadable, and the protocol
dissolves the tension rather than working around it.

Writing our own matcher would mean inventing a false-negative source to solve a problem someone else
has already solved correctly.

### The three places the guarantee weakens

`Cleared<T>` is a compile-time guarantee in Rust. It cannot cross every boundary, and where it
cannot, the degradation is documented and separately tested.

| Boundary | Enforcement | Test |
|---|---|---|
| Rust | Compile-time. `Cleared` has a private field. | `trybuild` compile-fail test |
| Python | Runtime check. Python cannot express "constructible only there". | Export an unscreened design, assert it raises |
| gRPC | Server-side. A signed `cleared_token` bound to a design hash, with expiry. | Missing, forged, stale, and mismatched tokens all refused |

**This list is not allowed to grow silently.** A fourth boundary means a fourth test, decided
deliberately.

---

## 5. Crate structure

```mermaid
flowchart TB
    UMB["integrase<br/><i>umbrella: the stable API surface</i>"]
    CORE["integrase-core<br/><i>no internal deps</i>"]
    SBOL["integrase-sbol"]
    SIM["integrase-sim"]
    SCREEN["integrase-screen"]
    VIZ["integrase-viz"]
    PY["integrase-py"]
    CLI["integrase-cli"]
    RS["sbol-rs<br/><i>external</i>"]

    UMB --> CORE & SBOL & SIM & SCREEN
    SBOL --> CORE
    SBOL --> RS
    SIM --> CORE
    SCREEN --> CORE
    VIZ --> CORE & SIM
    PY --> UMB & VIZ
    CLI --> UMB & VIZ

    classDef gate fill:#7a2020,stroke:#d04545,stroke-width:3px,color:#fff
    classDef base fill:#22303a,stroke:#5a8fa8,stroke-width:2px,color:#fff
    class SCREEN gate
    class CORE base
```

Two structural constraints, both load-bearing:

**`integrase-core` depends on nothing internal.** That keeps it reusable and keeps its compile and
test times low. If something in `core` needs `sim`, the design is wrong and the thing belongs in
`sim`.

**`integrase-screen` depends only on `integrase-core`.** Not on `sim`, not on `sbol`. The
highest-consequence component in the system stays auditable in isolation: a reviewer can read that
crate and its one dependency and see everything affecting a verdict. Every dependency added there
widens the audit surface, and it is also the most attractive target for a supply-chain attack.

Screening being its own crate rather than a module is what makes "does this PR touch screening?"
answerable from the diff path alone, which the `pr-review` and `merge-decision` skills rely on.

---

## 6. Core data model

```rust
pub struct Sequence { data: Vec<u8>, alphabet: Alphabet }

/// Borrowed, strand-aware. Reverse-complement is a view, not a copy, because
/// the memory budget does not permit casual cloning.
pub struct SequenceView<'a> { data: &'a [u8], strand: Strand, alphabet: Alphabet }

pub struct Design { id: DesignId, sequence: Sequence,
                    features: Vec<Feature>, provenance: Provenance }

pub struct Feature { id: FeatureId, location: Location,
                     role: OntologyTerm,       // an IRI, deliberately not an enum
                     name: Option<String>, children: Vec<FeatureId> }
```

**`role` is an ontology IRI, not an enum.** The Sequence Ontology has thousands of terms, gains
more, and is maintained by people who are not us. An enum forces a breaking change every time
biology adds a category and an `Other(String)` escape hatch that becomes the most-used variant. IRIs
also round-trip through SBOL without translation.

**Coordinates are 0-based half-open, internally, everywhere.** GenBank is 1-based inclusive, BED is
0-based half-open, and SBOL has its own rules. Mixed conventions are the endemic off-by-one source
in bioinformatics, and the only defence is stating the convention once and converting at every
adapter, with tests on the conversion.

---

## 7. Measured limits

An architecture document that omits its own operating envelope is not describing a real system.
These numbers were measured on a 20-core, 7 GB WSL2 machine (subplan 09).

| Property | Measured | Consequence |
|---|---|---|
| FM-index search, 20 bp | ~120 ns, flat across a 20× index size increase | Search is not a bottleneck |
| FM-index memory | **39.4 bytes per base** | The binding constraint |
| *E. coli*, 4.6 Mbp | 0.2 GB | Comfortable |
| *D. melanogaster*, 144 Mbp | 5.3 GB | At the edge |
| Human, 3.1 Gbp | **113.8 GB** | **Out of scope** |

**Supported range is bacterial genome scale, targeting 100 Mbp.** Anything larger returns an error
naming the projection and the limit, checked before allocation, because an out-of-memory kill is not
an error message. Streaming operations that need no index still work above the limit.

This killed a feature the brief asked for. It was killed by a measurement rather than an argument,
which is the standard this project holds itself to.

---

## 8. Phase map

```mermaid
flowchart LR
    P0["Phase 0<br/>Foundation<br/><i>can someone contribute?</i>"]
    P1A["Phase 1a<br/>The result<br/><i>does it tell the truth?</i>"]
    P1B["Phase 1b<br/>The tool<br/><i>can someone use it?</i>"]
    P2["Phase 2<br/>Seeing it<br/><i>readable unaided?</i>"]
    P3["Phase 3<br/>The service<br/><i>more than one user?</i>"]

    P0 --> P1A --> P1B --> P2 --> P3

    P1A -.->|"gate: 80% interval covers<br/>80% ±5%, stratified"| PUB["Publish the<br/>calibration result"]

    classDef gate fill:#7a2020,stroke:#d04545,stroke-width:2px,color:#fff
    class PUB gate
```

| Phase | Contains |
|---|---|
| 0 | Repo config, CI, scaffold, skills, `just` targets, README |
| 1a | `core`, `sbol` import, `sim` with `Prediction<T>` and the RBS model, conformal calibration, Python SDK, evaluation harness |
| 1b | CLI, `screen` with the gate and SecureDNA backend, SVG output, SBOL export, docs |
| 2 | Glyph rendering, confidence cartography, web UI, promoter model |
| 3 | Go orchestration, gRPC serving the Phase 1 contracts unchanged, Python ML service, PostgreSQL, auth |

The `proto/` contracts are written in **Phase 1** and validated against a stub server, even though
nothing serves them until Phase 3. If they need changing in Phase 3, Phase 1's semantic model was
wrong, and that is worth finding out early rather than late.

---

## 9. Why a biologist should trust this

The technical audience has section 5 and 6. This section is for the other one.

1. **It tells you when it does not know.** Every prediction carries an interval and an explicit
   verdict on whether your sequence resembles anything the model was validated on. No other tool in
   this field does this.
2. **Its accuracy claims come from the code that makes them.** `ModelCard` is a machine-readable
   structure containing published accuracy, validation datasets, and known failure modes, and the
   documentation is generated from it. The docs cannot drift from the model.
3. **Its limitations are written down.** `limitations.md` ships with the first release, listing what
   the tool cannot do, including that its intervals describe model uncertainty rather than
   biological outcome.
4. **It does not pretend to be a biosecurity control.** It is a design tool with a screening gate.
   The real control is the synthesis provider's screening. The threat model says so, including two
   attacks it cannot prevent.
5. **It does not report you.** A screening hit is explained, logged locally, and transmitted to
   nobody. The likely case is a researcher who did not know a component was regulated, and the route
   forward named is an institutional biosafety officer.
6. **It does not claim to make you compliant.** No government mandates synthesis screening; the
   mechanism is procurement conditionality and voluntary industry standards. Integrase implements
   screening consistent with the framework's approach, which is a different and smaller claim.

---

## 10. Where to read more

| Question | Document |
|---|---|
| Why these subsystem boundaries? | `subplans/06-subsystem-architecture.md` |
| Why these crates and languages? | `subplans/05-technology-stack.md` |
| Why depend on `sbol-rs`? | `subplans/23-upstream-ecosystem-strategy.md` |
| How accurate are the models, really? | `subplans/16-biophysical-modeling-accuracy.md` |
| How is calibration produced and tested? | `subplans/08-ml-architecture.md` |
| Where does screening data come from? | `subplans/24-screening-data-and-operations.md` |
| What can an attacker do? | `subplans/10-security-and-threat-model.md` |
| What are the performance limits? | `subplans/09-performance-engineering.md` |
| What was decided, and what was rejected? | `FINAL-DECISION.md` |
| Is this project actually a good idea? | `subplans/03` and `subplans/22` |
