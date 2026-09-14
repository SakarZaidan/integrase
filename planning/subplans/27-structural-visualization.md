# 27. Structural and three-dimensional visualization

**Status:** decided. Restores the brief's 3D capability, redefined.
**Date:** 2026-09-14
**Decides:** where 3D rendering belongs and where it does not
**Depends on:** subplans 13 (rendering pipeline), 14 (visual grammar), 16 (structure in the models)
**Feeds:** subplan 21

---

## The distinction the first cut got half right

Subplan 13 declined the brief's "rendering designed constructs in 3D" and the reasoning holds:
genetic constructs are linear, SBOL Visual specifies a 2D glyph language because that is what the
field reads and journals accept, perspective makes lengths ambiguous and occludes features, and a 3D
construct view would be a screenshot feature that anyone doing real work turns off.

**What the cut got wrong was stopping there.** Declining 3D for linear sequence data is correct.
Concluding that the project therefore has no 3D is not, because parts of this domain are genuinely
three-dimensional and the system already computes some of them.

**The rule: 3D where the data is three-dimensional. Never as decoration on data that is not.**

---

## Where 3D is genuinely the right representation

### 1. RNA secondary structure

**This is the one with a hard requirement behind it.** Subplan 16 established that the thermodynamic
RBS model needs RNA secondary structure prediction for its `ΔG_mRNA` and `ΔG_standby` terms. The
system is already computing a structure in order to make its headline prediction.

Right now that computation is invisible. The user sees a number and an interval; they do not see
that the interval is wide **because a hairpin is occluding the ribosome binding site**. Showing the
structure turns an opaque number into an explanation.

| Representation | Use |
|---|---|
| 2D secondary structure diagram (arc, dot-bracket, or a force-directed layout) | Default. This is what the field reads. |
| 3D tertiary rendering | Available, secondary. Genuinely useful for judging spatial accessibility. |

**And the confidence grammar applies.** Base pairs predicted with low probability render with the
same hatching and stippling that uncertain predictions get everywhere else (subplan 14). A structure
diagram where every pair looks equally certain is the same lie as a prediction without an interval,
and it is one every existing structure viewer tells.

That last point is a genuine contribution. Structure prediction confidence is available from the
partition function and is almost never shown.

### 2. Protein structure

When a design contains a coding sequence, its product has a structure, and predicted structures are
now routine. Rendering a CDS's predicted product, coloured by predicted-confidence, connects the DNA
design to the thing it makes.

**Scope discipline:** we do not predict protein structure. We display structures obtained elsewhere
and carry their confidence through our grammar. Adding a structure predictor would be a second
enormous subsystem with no relationship to this project's contribution.

### 3. DNA supercoiling and topology

Plasmid topology is three-dimensional and affects expression. Lower priority, real, and honestly
speculative in value until a user asks for it. Listed for completeness rather than committed.

---

## Where 3D stays declined

| Proposal | Verdict |
|---|---|
| 3D rendering of linear constructs | **Declined.** The data is 1D. SBOL Visual is 2D for good reasons. Perspective actively degrades the task. |
| A 3D "genome browser" | **Declined.** Same argument at larger scale. |
| Decorative double helices | **Declined.** Every synthetic biology project has one. It is illegible and says nothing. |

---

## Implementation

This costs less than it appears, because the pipeline already exists.

`integrase-viz` already targets `wgpu` for native and WASM (subplan 05, 13), which is a 3D-capable
renderer being used for 2D work. Adding a structural view is a new scene type against the same
backend, not a new rendering stack.

```
integrase-viz/
├── glyph/           SBOL Visual, 2D          (Phase 2)
├── structure/
│   ├── rna2d.rs     arc / force-directed     (Phase 2, with the RBS model)
│   ├── rna3d.rs     tertiary                 (Phase 4)
│   └── protein.rs   cartoon / surface        (Phase 4)
├── confidence/
│   └── cartography.rs   shared by all views
└── backend/{wgpu,svg}.rs
```

**`rna2d.rs` moves into Phase 2**, alongside the RBS model, because it explains the model's own
output rather than being an addition to it. The 3D views are Phase 4.

**SVG output applies to structure diagrams too**, which matters more here than for glyphs: RNA
structure diagrams go into papers constantly, and one carrying pairing confidence would be novel in
a figure.

---

## Testing

| Class | Assertion |
|---|---|
| Structure determinism | The same sequence and energy model produce an identical diagram |
| Confidence encoding | Low-probability base pairs render distinguishably, **verified in greyscale** like every other confidence encoding |
| SVG snapshot | Structure diagrams are snapshot-tested, same as glyphs |
| Consistency | The structure shown is the structure the RBS model used. **A view disagreeing with the model that produced it would be worse than no view.** |

The last row is the one that would actually bite. If the display recomputes structure with different
parameters than the prediction used, the user sees an explanation of a number that was not computed
that way.

---

## Open items

- [ ] The RNA structure dependency is the same open question as subplan 16's: ViennaRNA licensing or
      an alternative. **This subplan and the RBS model share a blocker.**
- [ ] Decide the source of protein structures. Display-only, so this is a question about which
      formats and services to read, not which predictor to build.
- [ ] Confirm base-pair probabilities are available from whichever structure implementation is
      chosen. **Without them the confidence encoding is impossible and this becomes an ordinary
      structure viewer.**
