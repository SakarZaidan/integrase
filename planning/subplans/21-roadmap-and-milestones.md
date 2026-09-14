# 21. Roadmap, phases, and milestones

**Status:** decided
**Date:** 2026-09-14
**Decides:** build order, with checkable definitions of done
**Depends on:** all preceding subplans
**Feeds:** `FINAL-DECISION.md`, which finalises this

---

## A refinement to subplan 06, found while roadmapping

Sequencing the work exposed a real gap in the gateway design, and it is worth stating plainly
because it is the kind of thing that only surfaces when you try to order the work.

**The problem.** Subplan 06 says `NullBackend` is the default and refuses everything, and that
export functions take `Cleared<T>`. Taken literally, a Phase 1 user with no SecureDNA instance
configured cannot export anything at all, including saving their own design back to SBOL. That
would make the tool useless for its primary Phase 1 purpose, and it also over-applies the brief,
which says restriction happens "only at the boundary where a design would leave the system toward
physical synthesis."

**The refinement.** Export splits into two classes.

| Class | Examples | Gated? |
|---|---|---|
| **Interchange** | SBOL round-trip, saving a working file, sharing with a collaborator, SVG figures | **No.** The design came from the ecosystem and goes back to it. We are not the synthesis boundary here, and gating it would just push people to a different tool. |
| **Synthesis-bound** | An explicit "prepare for ordering" flow, vendor-format order sheets, FASTA or GenBank produced by that flow | **Yes.** `Cleared<T>` required. This is the boundary the brief describes. |

The distinction is the **flow**, not the file format. The same FASTA bytes are interchange when a
user saves a working file and synthesis-bound when produced by the ordering path. That is why the
gate lives on the function, not the serializer.

**Is this a loophole?** Partly, and the honest answer is recorded in subplan 10's evasion table
already: a user who copies a sequence out of the interface by hand defeats any gate, and nothing
can prevent that. Gating interchange would not close that hole; it would only make the tool worse
for the researchers who are the overwhelming majority. The gate's value is that the convenient path
to ordering passes through screening, not that every byte is imprisoned.

`FINAL-DECISION.md` records this as the resolved position, and subplan 06 is amended.

---

## Phases

Each phase has a **single question it answers** and a definition of done that is checkable rather
than aspirational.

### Phase 0: foundation

**Question: can someone contribute to this?**

- Repository configured, branch protection, CI green
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, templates, `CODEOWNERS`
- Six skills under `.claude/skills/`
- Workspace scaffold: Rust crates, Python package, `proto/`, `docs/`
- `just` targets mirroring CI exactly
- README

**Done when:** a fresh clone runs `just setup && just check` successfully on Linux, and CI passes on
a trivial PR. Not "when the files exist."

### Phase 1: the honest predictor

**Question: does this tell the truth about what it knows?**

This is the wedge from subplan 03: one narrow thing that is unambiguously the best available.

| Deliverable | Detail |
|---|---|
| `integrase-core` | `Sequence`, `SequenceView`, `Design`, `Feature`, coordinate arithmetic, motif and restriction search |
| `integrase-sbol` | Import/export via `sbol-rs`, conversion reports surfaced |
| `integrase-sim` | `Prediction<T>`, `DomainVerdict`, `ModelCard`, thermodynamic RBS model |
| Calibration | Split conformal on Kosuri, component-wise splits |
| `integrase-screen` | Type-level gate, `Cleared<T>`, `NullBackend`, SecureDNA backend |
| `integrase-py` | Python SDK, wheels |
| `integrase-cli` | Import, predict, screen, export |
| SVG output | Figures with the confidence grammar, and snapshot testing |
| Docs | `uncertainty.md`, `limitations.md`, getting started |

**Done when all of these are true:**

1. A real SBOL file from the vendored corpus imports with no feature loss, verified semantically.
2. `predict_translation_rate` returns a calibrated interval, and the calibration test passes:
   an 80% interval contains the truth 80% ± 5% on held-out data, **stratified by expression decile**,
   not just marginally.
3. `float(prediction)` raises in Python.
4. The `trybuild` compile-fail test proves `export_for_synthesis` rejects an unscreened design.
5. Fault injection confirms every screening failure mode produces a refusal, never a pass.
6. An out-of-distribution sequence returns `OutOfDistribution` with a specific named reason.
7. `pip install integrase` then the notebook flow from scenario S3 works end to end, with no GPU and
   no network.
8. Coverage floors met, screening at 95%.

**Criterion 2 is the one that can fail and block the phase.** It depends on the calibration dataset
and on conformal prediction actually working on this data. If it does not, Phase 1 does not ship as
"the honest predictor," and the honest response is to ship reporting `Uncalibrated` and say so
rather than to weaken the test.

### Phase 2: seeing it

**Question: can a user read the uncertainty without being told how?**

- `integrase-viz`: SBOL Visual glyph rendering via ontology lookup
- Confidence cartography: hatch, stipple, outline per `DomainVerdict`
- Web UI (Leptos/WASM), SVG tier-2 fallback for Firefox
- Promoter model, with its wider intervals

**Done when:**
1. A construct renders with correct SBOL Visual glyphs, snapshot-tested per SO role.
2. All four `DomainVerdict` states are distinguishable **in greyscale**, snapshot-tested.
3. The UI works in Firefox via the SVG tier.
4. WASM bundle under 5 MB gzipped. **If this fails, the Leptos decision gets revisited before it is
   entrenched** (subplan 13).
5. Someone unfamiliar with the tool can correctly identify which of two predictions is less
   trustworthy, without explanation. This is a usability test, not a code test, and it is the real
   measure of whether confidence cartography works.

### Phase 3: the service

**Question: can more than one person use it at once?**

Go orchestration, gRPC serving the Phase 1 contracts, REST via grpc-gateway, Python ML service,
foundation-model embeddings, PostgreSQL, auth.

**Done when:** the Phase 1 `proto/` contracts are served unchanged (if they need changing, Phase 1's
design was wrong and that is worth knowing), loopback gRPC is under 5 ms confirming subplan 05's
boundary assumption, cross-tenant isolation is explicitly tested, and the ML layer beats the
deterministic baseline on the out-of-distribution set or is not enabled.

### Phase 4: desktop and beyond

Tauri shell. Cello UCF import **if a real user asks for it**, which no current scenario does
(subplan 11). Generative design via Evo 2. Additional screening backends.

**Nothing here is committed.** This phase is a list of candidates, and each needs a scenario before
it earns a place.

---

## What is deliberately not on the roadmap

| Item | Why |
|---|---|
| Circuit logic synthesis | Cello did it well. Import its output instead. Multi-year to reimplement. |
| Genome-scale indexing | Measured at 113.8 GB for a human genome (subplan 09). Out of scope, stated with the number. |
| 3D construct rendering | Constructs are linear. SBOL Visual is 2D because that is what journals accept. |
| Our own screening matcher | A false negative is the worst failure available. Delegate to a peer-reviewed implementation. |
| Our own SBOL implementation | 348 validation rules already exist upstream. |
| A hosted service we operate | We publish images. Operating a service is a different business. |

---

## Sequencing risks

| Risk | Phase | Mitigation |
|---|---|---|
| **Calibration does not work on available data** | 1 | Ship `Uncalibrated` and say so. Do not weaken the test. This is the single biggest risk to the project's thesis. |
| Second dataset never materialises | 1 | In-distribution calibration only, stated in the `ModelCard`. |
| ViennaRNA licensing blocks the RBS model | 1 | Open item. Needs resolving early, since it blocks the headline feature. |
| WASM bundle too large | 2 | Revisit Leptos before entrenchment. Measure early. |
| `sbol-py` never reaches PyPI | 1 | Ask SynBioDex. Keep the SBOL dependency swappable behind our types. |
| Nobody uses it | all | **Not mitigable by engineering.** Subplan 03 scored adoption 3/10. The lever is a citable artifact, and that is a Phase 1 output, not a Phase 4 one. |

The last row is why Phase 1 is scoped as "the honest predictor" rather than "the platform
foundation." A phase that produces something publishable is worth more to this project's survival
than a phase that produces good infrastructure nobody sees.

---

## Open items

- [ ] Resolve the ViennaRNA question before Phase 1 starts in earnest
- [ ] Identify the second calibration dataset
- [ ] Amend subplan 06 with the interchange/synthesis-bound export split
