# 13. Frontend and visualization engineering

**Status:** decided
**Date:** 2026-09-14
**Decides:** rendering pipeline, component architecture, state management, and how confidence
cartography becomes implementable UI
**Depends on:** subplans 05 (wgpu/WASM/Tauri), 06 (`integrase-viz`), 14 (visual grammar)
**Feeds:** `ARCHITECTURE.md`, subplan 21

---

## Delivery order, reversed from the brief

**Web first. Desktop later. And this is evidence-based rather than preference.**

In this exact ecosystem, delivery model correlates with survival:

| Tool | Model | Last push |
|---|---|---|
| SBOLCanvas | Web | 2026-09-10, active |
| SynBioHub | Web | 2026-09-06, active |
| SBOLDesigner | Java desktop | 2021-06-04, dead |
| TinkerCell | Desktop | abandoned |

Combined with subplan 02's finding that the reachable audience is the notebook-and-browser crowd,
the order is: **Python SDK and CLI (Phase 1), web visualization (Phase 2), Tauri desktop (Phase 4).**

---

## Rendering pipeline

```
integrase-core Design
        │
integrase-sim Prediction<T> ──┐
        │                     │
integrase-viz                 │
  ├── glyph/ontology.rs   ← SBOL Visual ontology lookup
  ├── glyph/geometry.rs   ← glyph paths
  ├── layout.rs           ← linear layout, coordinate mapping
  └── cartography.rs      ← DomainVerdict ──┘  becomes texture + opacity
        │
   ┌────┴────┐
 wgpu       svg
   │          │
native +    docs, papers,
 WASM       snapshot tests
```

**Two backends, one scene description.** Layout and glyph selection happen once and produce a
backend-agnostic scene. `wgpu` rasterises it interactively; `svg` emits it deterministically.

### Why SVG is a first-class backend, not a fallback

Three reasons, and only the first is obvious.

1. **Scientists put figures in papers.** An SVG export that carries the confidence grammar means the
   uncertainty encoding survives into publication, which is where it reaches the widest audience.
2. **It makes visual regression testing possible.** SVG is deterministic and text-diffable. A
   rendering change shows up as a readable diff. GPU output requires image comparison with a
   tolerance, which is flaky and which nobody trusts enough to act on.
3. **It is the Firefox answer.** See below.

### The Firefox problem, addressed rather than ignored

WebGPU reached candidate recommendation in March 2026 at roughly 85% global support, shipping by
default in Chromium and Safari 26. **It is not in stable Firefox.**

A scientific tool that does not work in a reviewer's browser has a credibility problem with exactly
the audience it needs. So:

| Tier | Path | Capability |
|---|---|---|
| 1 | WebGPU via `wgpu` | Full interactive rendering |
| 2 | **SVG/DOM** | Full static rendering, pan and zoom, selection. No GPU-accelerated large-scale interaction. |

**Tier 2 is not degraded to the point of uselessness**, and that is the design requirement. For
constructs of the size real users work with (kilobases, tens of features), SVG rendering is entirely
adequate. The GPU path matters for large genomic views, which subplan 09's measurement showed are
out of scope anyway. **A WebGL2 intermediate tier is deliberately not built**, because maintaining
three rendering paths for a tool this size means maintaining one well and two badly.

The detection is capability-based, not user-agent based, and the tier in use is visible in the UI
rather than silent.

---

## Confidence cartography as implementable UI

Subplan 14 defines the visual grammar. This is how it becomes code.

**The rule: encoding is computed from `DomainVerdict`, never authored per view.** There is no code
path where a developer chooses how confident something looks.

```rust
/// The only place visual confidence is decided. A single function so it cannot
/// drift between views, and so subplan 17's snapshot tests cover all of it.
pub fn encoding_for(verdict: &DomainVerdict) -> ConfidenceEncoding {
    match verdict {
        DomainVerdict::InDistribution { .. } =>
            ConfidenceEncoding { fill: Fill::Solid,   border: Border::Solid,  interval: IntervalCap::Solid },
        DomainVerdict::Marginal { .. } =>
            ConfidenceEncoding { fill: Fill::Hatch45, border: Border::Solid,  interval: IntervalCap::Hatched },
        DomainVerdict::OutOfDistribution { .. } =>
            ConfidenceEncoding { fill: Fill::Stipple, border: Border::Dashed, interval: IntervalCap::Open },
        DomainVerdict::Uncalibrated =>
            ConfidenceEncoding { fill: Fill::None,    border: Border::Dotted, interval: IntervalCap::None },
    }
}
```

### Three UI rules that follow from the product principle

**1. Uncertainty is visible at rest.** Never behind a hover, a tooltip, an expander, or a details
pane. A tooltip is where information goes to be ignored, and this information is the product.

**2. The interval is drawn, not just printed.** A number with a bracket beside it is easy to skim
past. A visible interval bar whose width is proportional to the uncertainty is not. When a user
changes an RBS and the bar visibly widens, they have learned something that a numeric readout would
not have taught them.

**3. There is no "hide uncertainty" setting.** This will be requested. It is declined, and the
reason is recorded here so it does not need re-litigating: a toggle that hides uncertainty converts
this tool into every other tool, and the person who most wants that toggle is the person who most
needs the information.

---

## Component architecture

**Leptos (Rust/WASM)** rather than React or Svelte.

| Option | Assessment |
|---|---|
| **Leptos** | One language across core, rendering, and UI. `Design`, `Prediction`, and `DomainVerdict` cross into the UI as real types, not JSON that can drift. Fine-grained reactivity suits a dense tool. **Chosen.** |
| React + TS | Larger ecosystem, more hireable. But it means a serialization boundary, duplicated type definitions, and the risk that the UI's model of `DomainVerdict` drifts from the core's. For a one-person project, the duplication is the bigger cost. |
| Svelte | Same boundary problem, smaller ecosystem than React. |

**The honest risk:** Leptos has a much smaller community than React, and a contributor who knows web
frontend probably knows React. This narrows the contributor pool for the UI specifically, which
subplan 19 should weigh against the bus-factor concern. The counterweight is that the alternative
means every core type exists twice, and two definitions of `DomainVerdict` that can disagree is a
correctness risk in the one place this product cannot afford one.

```
web/
├── components/
│   ├── construct_view/    linear construct, the primary surface
│   ├── prediction/        value, interval bar, domain badge
│   ├── screening/         verdict display, refusal explanation
│   └── primitives/        buttons, tables, inputs, per subplan 14
├── state/
└── wasm_bridge/           typed bindings into integrase-viz
```

---

## State management

**Three kinds of state, deliberately separated**, because conflating them is how design tools
develop mysterious bugs.

| Kind | Example | Lives | Undo? |
|---|---|---|---|
| **Document** | The design itself | Owned by the core, WASM side | **Yes.** Command pattern. |
| **Derived** | Predictions, layout, screening verdict | Cached, invalidated by document changes | No. Recomputed. |
| **View** | Zoom, selection, panel sizes | UI only | No. |

**Derived state is never edited directly.** A prediction is a function of a design and a model. If a
user could edit a prediction, the system would be lying about where the number came from. This
sounds obvious and is exactly the invariant that erodes when someone wants to add a "manual
override" feature.

**Undo applies only to document state**, implemented as invertible commands rather than
snapshotting, because snapshots of large sequences would be memory-expensive on a 7 GB budget.

---

## The 3D rendering question, answered

The brief asks for "rendering designed constructs in 3D." **This is declined, with reasoning.**

1. **Genetic constructs are linear.** The information is sequence, position, and orientation. That
   is 1D data conventionally drawn in 2D.
2. **SBOL Visual specifies a 2D glyph language** because that is what the field reads and what
   journals accept. Rendering in 3D would break the standard we position ourselves as supporting.
3. **3D is worse at the actual task.** Perspective makes lengths ambiguous and occludes features.
   For data where position is the meaning, a projection that distorts position is a downgrade.
4. **It would be a demo, not a tool.** It would look impressive in a screenshot and be turned off by
   anyone doing real work.

**Where 3D would genuinely help, and is therefore a possible later addition:** RNA secondary
structure, protein structure, and DNA supercoiling. All are real 3D phenomena, and none is in v1.0
scope. The distinction is that 3D should be used where the data is three-dimensional, not as
decoration on data that is not.

---

## Performance

| Target | Budget | Status |
|---|---|---|
| First render, typical construct | < 2 s | Tier 2 budget, unmeasured (subplan 09) |
| Interaction latency, pan/zoom | < 16 ms (60 fps) | Tier 2, unmeasured |
| Re-render after an edit | < 100 ms | Tier 2, unmeasured |
| WASM bundle | < 5 MB gzipped | Needs verification. Rust WASM bundles grow quietly, and this is the one that will be hardest to hold. |

The bundle-size budget is flagged as the risky one. Leptos plus `wgpu` plus the core is a lot of
Rust to ship to a browser, and a 15 MB download would be a real adoption problem for the exact
audience we want.

---

## Open items

- [ ] Measure the WASM bundle early. If it cannot be held near 5 MB, the Leptos decision needs
      revisiting before it is entrenched.
- [ ] Verify hatch and stipple textures stay legible at 12px in both backends. Texture that turns to
      mush defeats the confidence encoding silently.
- [ ] Prototype the construct view with real SBOL data before committing to the layout model.
- [ ] Decide whether the SVG backend ships in Phase 1 for the CLI, ahead of the web UI. It is
      cheap, it makes figures possible immediately, and it enables snapshot testing early.
