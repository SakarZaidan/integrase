# 20. Documentation plan

**Status:** decided
**Date:** 2026-09-14
**Decides:** what goes in `docs/`, its information architecture, and how it avoids rotting
**Depends on:** subplans 11 (scenarios), 16 (honest limits), 19
**Feeds:** `docs/`, built after the checkpoint

---

## How `docs/` and `planning/` relate

The brief asks this to be settled explicitly so the two do not duplicate and drift. The split is
by **tense and audience**, which makes it easy to apply without judgment calls.

| | `planning/` | `docs/` |
|---|---|---|
| Tense | Why we decided, what we rejected, what is next | What the system **is**, now |
| Audience | Future maintainers, including future-us | Users and contributors |
| Changes when | A decision is made | The code changes |
| Contains rejected alternatives | **Yes, that is the point** | No |
| Contains phasing | Yes | No. A user does not care what Phase 3 holds. |

**The rule: if it answers "why is it like this?", it is planning. If it answers "how do I use it?",
it is docs.**

The one deliberate overlap is architecture. `planning/ARCHITECTURE.md` is a planning artifact
recording structural decisions with their reasoning. `docs/contributing/architecture.md` describes
the same system for someone reading to work in it. Same system, different question, and neither is a
copy of the other. The `planning-sync` skill keeps them consistent when structure changes.

---

## Information architecture

```
docs/
├── index.md                     what this is, who it is for, what it does not do
├── getting-started/
│   ├── install.md               three languages, honest about prerequisites
│   ├── first-design.md          scenario S2, end to end
│   ├── first-prediction.md      and how to read the interval
│   └── screening-setup.md       configuring SecureDNA, because unconfigured refuses
├── guides/
│   ├── importing-sbol.md        scenario S1
│   ├── notebook-workflows.md    scenario S3, the primary audience
│   ├── self-hosting-screening.md  scenario S4
│   └── exporting-figures.md     SVG with the confidence grammar intact
├── concepts/
│   ├── designs-and-features.md  the data model
│   ├── uncertainty.md           ★ the most important page in the docs
│   ├── confidence-cartography.md  reading the visual grammar
│   ├── screening-gateway.md     what it does, what it cannot do
│   └── standards.md             SBOL, SBOL Visual, SBML, and what we support
├── reference/
│   ├── python/                  generated from docstrings
│   ├── rust/                    rustdoc
│   ├── cli/                     generated from clap
│   └── api/                     generated from proto
├── contributing/
│   ├── architecture.md          the system, for someone working in it
│   ├── development.md           environment setup
│   └── testing.md               how to run what CI runs
└── limitations.md               ★ what this tool cannot do
```

The four getting-started pages map to the scenarios in subplan 11, which were written with this in
mind. Documentation derived from scenarios describes tasks people actually attempt, rather than
features that happen to exist.

---

## The two starred pages

### `concepts/uncertainty.md`

**The most important page in the documentation**, because the product's central differentiator is
useless if users do not understand what they are being shown.

It has to answer, in order:

1. What the interval means. Specifically: a calibrated 80% interval contains the true value about
   80% of the time **on data like the calibration set**.
2. What `DomainVerdict` means and why it is separate from the interval. The conformal exchangeability
   argument from subplan 08, written for a biologist rather than a statistician.
3. **What the interval does not cover.** Genomic context, host physiology, growth conditions, copy
   number. Our intervals quantify uncertainty about the model's prediction, not about the biological
   outcome (subplan 16).
4. How to use it: rank by lower bound rather than point estimate, and treat `OutOfDistribution` as
   "the model is guessing."

If a reader finishes this page believing the intervals are biological confidence intervals, the page
has failed and so has the product.

### `limitations.md`

A prominent, linked-from-the-README page listing what Integrase cannot do. Lifted from subplan 16's
"what this subsystem does not claim" plus subplan 09's measured ceilings plus subplan 10's two
honest "we cannot stop this" rows.

**This is a feature, not a disclaimer.** Every competing tool has the same limitations and states
none of them. A user who reads it and uses the tool anyway knows what they are getting, which is the
only kind of trust worth having with scientists. It is also the page most likely to be cited
approvingly by a skeptical reviewer, which is exactly the audience that decides whether this tool
gets adopted.

---

## Generated versus hand-written

| Content | Source | Why |
|---|---|---|
| API reference, Python | docstrings | Hand-maintained reference drifts within one release |
| API reference, Rust | rustdoc | Same |
| CLI reference | `clap` | Same |
| gRPC reference | `proto/` comments | Same |
| Model cards | **`ModelCard` structs** | Accuracy claims must come from the code that makes them, or the docs and the model disagree and the docs are wrong |
| Concepts, guides, getting started | Hand-written | Explanation cannot be generated |

**Model cards being generated from code is the anti-drift mechanism that matters most.** A hand-
written accuracy number in the docs is a number that will eventually be wrong, and in this project
a wrong accuracy claim is worse than no documentation at all.

---

## How documentation avoids rotting

Six mechanisms, in rough order of how much work they do:

1. **Doctests** (R-4.3). Examples in doc comments are compiled and run in CI, so an example that
   stops working fails the build. This catches most reference rot automatically.
2. **Same-PR requirement** (R-4.4). A change to user-facing behaviour updates the docs in the same
   PR. Follow-ups do not happen.
3. **Generated reference.** Cannot drift from the code because it is the code.
4. **Docs build in CI.** Broken links and bad references fail.
5. **`planning-sync`.** Architectural change updates `ARCHITECTURE.md` and its
   `docs/contributing/architecture.md` counterpart together.
6. **Scenario tests are also doc tests.** The getting-started pages follow scenarios that are
   covered by end-to-end tests (subplan 17). If the test breaks, the tutorial is wrong, and we find
   out from CI rather than from a user.

Mechanism 6 is the one most projects miss. A tutorial nobody executes is a tutorial that stops
working quietly.

---

## Tone

Per the brief's section 11, enforced by `tools/check-voice.sh` in CI. Three additions specific to
documentation:

- **State limitations where they are relevant, not only on the limitations page.** The RBS prediction
  guide says what the model cannot do, at the point where someone is using it.
- **Never imply a prediction is a measurement.** No "the expression level is 4,200." It is
  "predicted translation initiation rate 4,200, 80% interval 1,900 to 9,100."
- **Write for a biologist who is not a programmer, and a programmer who is not a biologist.** Both
  are real users, and each needs the other's vocabulary explained. A glossary covers both
  directions.

---

## What ships when

| Phase | Docs |
|---|---|
| 1 | `index`, install, first-design, first-prediction, `uncertainty.md`, `limitations.md`, generated Python and Rust reference |
| 2 | Visualization guides, confidence cartography, figure export |
| 3 | Self-hosting, API reference, deployment |

**`uncertainty.md` and `limitations.md` ship in Phase 1**, with the first release. They are not
polish to add later; they are the parts that make the first release honest.

---

## Open items

- [ ] Choose the docs generator. Needs to handle Rust, Python, and proto reference together.
      `mdBook` plus generated sub-sites is the leading candidate.
- [ ] Write the glossary, covering both directions
- [ ] Wire scenario tests to the tutorials they document so a broken tutorial fails CI
