# 01. Project identity and naming

**Status:** decided
**Date:** 2026-09-14
**Decides:** final project name, organization name, package naming scheme, tagline, one-sentence
pitch
**Blocks:** repository creation, every package manifest, the README, the brand subplan (14)

---

## The decision

**The project is named Integrase.**

| Surface | Value |
|---|---|
| Project name | Integrase |
| Written form | `Integrase` in prose, `integrase` in code and package names. Never all-caps. |
| GitHub organization | `integrase` |
| Canonical repository | `integrase/integrase` |
| crates.io | `integrase` umbrella, `integrase-*` for workspace members |
| PyPI | `integrase` |
| npm | `@integrase/*` scope |
| Tagline | Genetic design with the error bars attached. |

**One-sentence pitch:** Integrase is an open-source platform for designing, simulating, and
screening genetic constructs, built so that every prediction it makes arrives with the
uncertainty that belongs to it.

GENESIS, the working title from the project brief, is retired. It survives nowhere in the
codebase, the docs, or the repository.

---

## Why GENESIS had to go

Not a close call. Verified by direct registry queries on 2026-09-14:

| Namespace | `genesis` |
|---|---|
| crates.io | Taken (an ECS library, v0.2.2) |
| PyPI | Taken |
| npm | Taken |
| github.com/genesis | Taken |

Every namespace the project needs is occupied. Worse than the registries, the *software*
collisions are severe and active:

- **Genesis**, the physics and robotics simulation platform, which drew enormous attention on
  release in late 2024. This is the disqualifying one. It is a simulation platform, written partly
  in a systems language, aimed at researchers. A synthetic biology simulation platform sharing its
  name would be confused with it constantly, and we would lose every one of those collisions
  because they got there first with more funding.
- The US Department of Energy **Genesis Mission** AI initiative.
- Genesis Healthcare, Genesis Energy, and a long tail of commercial users.

There is also a softer problem. "Genesis" means *the origin of life*. For a tool that explicitly
does not perform genome editing, does not touch living organisms, and operates only on sequence
data in a sandbox, the name overclaims in precisely the direction that makes working scientists
suspicious. The brief is right that credibility with real biologists is the thing to protect. A
name promising creation-of-life for a CAD tool spends credibility before the README is even read.

---

## Why Integrase

### It is free everywhere, which almost nothing is

Verified 2026-09-14 by direct API query. HTTP 404 means available.

| Namespace | Result |
|---|---|
| crates.io `integrase` | free |
| PyPI `integrase` | free |
| npm `integrase` and `@integrase` scope | free |
| github.com/integrase (user or org) | free |

GitHub turned out to be the scarcest namespace by a wide margin. Of thirty-eight candidates
checked, most biology terms were free on all three package registries and taken on GitHub, usually
by a dormant personal account. Six candidates were free on all four: `integrase`, `resolvase`,
`excisionase`, `periplasm`, `protospacer`, `phagemid`, plus weaker options like `polylinker`,
`bacmid`, and `inoculum`.

Domains `integrase.dev`, `.org`, `.bio`, `.io`, `.com`, and `.sh` all returned no DNS records.
**This is not proof they are unregistered.** A registered domain with no A record resolves to
nothing, and `whois` is not installed in this environment. Domain status needs a real registrar
check before anyone spends money. Logged as a low-priority item, not a blocker, since nothing in
the build depends on a domain existing.

### No trademark conflict found

A search for a registered "Integrase" trademark in software or scientific classes returned
nothing. The near-misses are INTEGRA (registration 5809766, pharmacy workflow software) and
INTEGRASYS (registration 4177976, dead/cancelled as of 2019). Neither is close enough to cause
trouble for an open-source scientific tool.

Caveat stated plainly: this was a web search, not a cleared trademark opinion. If the project ever
takes on commercial weight, that is a question for a lawyer, not for me.

### No software already owns the name

GitHub search returns 83 repositories mentioning integrase. The largest is
`sandialabs/Integrase-On-Demand` at 6 stars. All of them are research artifacts about the enzyme,
not software claiming the name. There is no incumbent to be confused with, which is exactly the
failure mode GENESIS had.

### The meaning is right, on three levels

An integrase is an enzyme that inserts genetic material into a host genome at a specific site.
Serine integrases in particular (Bxb1, phiC31) are workhorse tools of modern synthetic biology,
used for genome engineering, DNA assembly, and building logic and data-storage devices. Sources:
[ACS Synthetic Biology on serine integrases](https://pubs.acs.org/doi/10.1021/acssynbio.7b00308),
[the ins and outs of serine integrase site-specific recombination](https://pmc.ncbi.nlm.nih.gov/articles/PMC4267755/),
[Bxb1 as best-of-fifteen for human genome integration](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4015280/).

1. **Literal.** It names a real tool that the people we are building for use to assemble the exact
   circuits this platform designs. Naming a genetic design tool after a genetic assembly enzyme is
   the kind of choice that reads as domain literacy rather than branding.
2. **Structural.** The project's entire positioning is that it is the missing integration layer for
   a fragmented ecosystem, joining SBOL, Cello, SynBioHub, and simulation tooling that currently do
   not talk to each other. An integrase is, definitionally, the thing that integrates. The name
   states the thesis.
3. **Behavioural.** Serine integrase recombination is highly directional and does not spontaneously
   reverse without an accessory factor. That is a pleasant accident rather than a reason to pick the
   name, but it maps to how the system should behave: operations are precise, deliberate, and do
   not quietly undo themselves.

The `-ase` suffix denotes an agent that performs an action. Integrase is the thing that integrates.
For an infrastructure project, a name that is a noun-of-agency beats an abstract noun.

### It does not overclaim

"Integrase" describes a mechanism, not a miracle. Compare with GENESIS. A tool named after a
specific, well-characterized enzyme makes a modest, checkable claim about what it is. That fits a
project whose core product principle is refusing to present predictions without their uncertainty.

---

## The honest problem with Integrase

**HIV integrase inhibitors are a major drug class.** Raltegravir, dolutegravir, and the rest of the
INSTI class mean that a general web search for "integrase" returns HIV pharmacology, not synthetic
biology. This is a genuine SEO cost and it should be stated rather than glossed.

Three reasons it is acceptable:

1. **The collision is with a drug class, not with software.** Nobody searching for an HIV
   antiretroviral will install a Rust crate by accident, and nobody looking for genetic design
   software will land on a pharmacology paper and stay. GENESIS's collision was with an actively
   promoted simulation platform competing for the same developers. That is the collision that
   actually costs you users.
2. **Developers do not search bare nouns.** They search "integrase github", "integrase rust",
   "integrase sbol", or they arrive from a link. Every one of those disambiguates on the first
   result once the project exists.
3. **The package namespaces are clean**, which is where the collisions would actually hurt.
   `cargo add integrase` and `pip install integrase` are unambiguous today and will stay that way.

A name with zero collisions anywhere would have to be invented, and invented names carry a
different cost: they signal nothing, so every reader needs the tagline to understand what the tool
is. For a scientific tool trying to earn trust from domain experts, a real domain term that
experts recognize is worth more than a clean search result.

---

## Alternatives considered and rejected

| Candidate | Namespaces | Why rejected |
|---|---|---|
| **Resolvase** | free on all four | Genuinely close second. A resolvase resolves recombination intermediates, and "resolves" is a decent metaphor. Rejected because it is a narrower, less recognized enzyme than integrase, and because "resolver" already means something specific and different in software (DNS, dependency resolution), which invites the wrong mental model. |
| **Excisionase** | free on all four | Wrong metaphor entirely. Excision means cutting out and removing. This project joins things together. |
| **Periplasm** | free on all four | The space between the inner and outer bacterial membranes, so "the layer in between" is an apt metaphor for middleware. Rejected on sound: three syllables ending in a consonant cluster, awkward to say aloud, and it names a location rather than an action. |
| **Protospacer** | free on all four | CRISPR-specific. Ties the project's identity to one technology the platform is not specifically about, and would date badly. |
| **Phagemid, Bacmid, Polylinker, Inoculum** | free on all four | All available, all weak. They name passive objects or substrates rather than an action, and none carries a metaphor connected to what the software does. |
| **Ligase** | crates and npm free, PyPI and GitHub taken | The best metaphor of the entire list. A ligase joins DNA fragments, which is exactly the project's thesis. Rejected purely on availability: PyPI and the GitHub namespace are both occupied, and a project that cannot own its own name on PyPI has a permanent papercut. Worth recording that this was rejected on logistics, not on merit. |
| **Cistron, Operon, Replicon** | GitHub or npm taken | Good synbio terms, all blocked on at least one required namespace. |
| **Chassis, Anneal, Locus, Motif, Primer, Vector, Codon, Helix** | taken everywhere | All taken, several by prominent software (Codon is Exaloop's compiler). Generic enough that the collisions will never clear. |

---

## Naming scheme for packages and crates

Consistency here prevents the slow drift that makes polyglot projects feel like three projects
wearing a trenchcoat. The rule: **one name, three registries, predictable suffixes.**

### Rust (crates.io)

A workspace with an umbrella crate and subsystem crates. Names mirror the subsystems from subplan
06 rather than being invented separately.

```
integrase            umbrella, re-exports the stable public surface
integrase-core       sequence types, indexing, motif and restriction-site search
integrase-sbol       SBOL interop layer, wrapping sbol-rs (see subplan 23)
integrase-sim        biophysical simulation, thermodynamic models
integrase-screen     biosafety screening gateway
integrase-viz        wgpu rendering and SBOL Visual glyph layout
integrase-ffi        C ABI surface, only if subplan 05 chooses FFI over gRPC
```

`integrase-screen` is deliberately its own crate rather than a module. The brief requires the
screening gateway to be a first-class subsystem with its own threat model and test suite, and a
crate boundary makes that structurally true rather than aspirational. It also means a reviewer can
see at a glance whether a PR touches screening, which the `pr-review` skill depends on.

### Python (PyPI)

```
integrase            the SDK, single distribution
```

One package, not a family. The Python surface is an SDK for scientists working in notebooks, and
splitting it across distributions would make `pip install` a research project. Internal structure
lives in modules under `integrase.*`, not in separate packages.

### npm

```
@integrase/viz       WASM visualization bindings
@integrase/ui        shared UI components, if subplan 13 concludes they are needed
```

Scoped from the start. The bare `integrase` npm name is free and gets claimed defensively to stop
someone else taking it, pointing at the docs. Namespace squatting by third parties is a real risk
for a project with a public roadmap.

### Go

```
github.com/integrase/integrase/services/...
```

Go module paths follow the repository, so this resolves once the org exists. Until the transfer in
`REQUESTS.md` R-001 completes, the path is `github.com/SakarZaidan/integrase/...`. **This is the
one naming decision with a real migration cost.** Go module paths are baked into every import
statement, so the transfer should happen before any external user imports the Go packages. Flagged
for subplan 18's release plan: do not publish a Go module version from the personal namespace.

---

## Tagline and pitch

**Tagline:** Genetic design with the error bars attached.

Short, states the differentiator, and states it as a property rather than a promise. It says what
the tool does differently in six words: other tools give you a number, this one gives you the
number and how much to trust it. The phrase "error bars" is the right register for the audience,
because scientists already know what an error bar is and already distrust software that omits
them.

Rejected taglines, and why:
- *"The unified platform for synthetic biology."* Says nothing a reader can check, and every
  platform claims to unify something.
- *"Design. Simulate. Build."* The reflexive rule-of-three cadence the brief bans, and the third
  verb is a lie because the project explicitly does not do physical construction.
- *"Genetic engineering, engineered."* Cute, empty, and invites exactly the genome-editing
  misreading the project needs to avoid.

**One-sentence pitch:** Integrase is an open-source platform for designing, simulating, and
screening genetic constructs, built so that every prediction it makes arrives with the uncertainty
that belongs to it.

**Longer positioning, for the README and the docs landing page:**

> Integrase is a design and simulation environment for engineered genetic systems. It reads and
> writes SBOL, so the designs you already have in SynBioHub, Cello, or iBioSim come in without
> conversion work. It simulates promoter and ribosome binding site behaviour using published
> biophysical models, and it reports what those models actually know: a prediction, a calibrated
> interval, and an honest marker when a sequence falls outside the range where the model has been
> validated. Every construct is screened against biosecurity criteria before it can be exported
> toward physical synthesis.
>
> Integrase does not edit genomes, does not touch living organisms, and does not order DNA. It
> operates on sequence data. Physical synthesis and wet-lab work happen elsewhere, by people, with
> the oversight that entails.

That last paragraph is not a disclaimer to be buried. It goes near the top of the README. A tool
in this domain that is vague about what it does not do will be assumed to be hiding something, and
the assumption will be fair.

---

## What this constrains downstream

- **Subplan 14 (brand)** inherits a name rooted in enzymology and precision, not in creation myth.
  The visual identity should follow: instrument-like, restrained, closer to a good oscilloscope
  than a biotech startup deck. The purple-to-blue gradient the brief bans would fight the name.
- **Subplan 06 (architecture)** inherits the crate names above, which means the subsystem
  boundaries in that document and the crate boundaries in the workspace have to agree. If they
  diverge, one of them is wrong.
- **Subplan 18 (release)** inherits the Go module path migration problem. No Go module release
  from the personal namespace.
- **Repository creation** is unblocked. Next action: `gh repo create SakarZaidan/integrase`.

## Open items

- [ ] Domain registration status needs a real registrar or `whois` check. Not a blocker, and the
      spend is the owner's call. Only relevant once there is a docs site to point at.
- [ ] Claim the bare `integrase` npm name defensively when the org exists.
- [ ] Reserve `integrase` on crates.io and PyPI before announcing publicly. Registry squatting
      against a project with a public roadmap is a known pattern and the defence is cheap.
