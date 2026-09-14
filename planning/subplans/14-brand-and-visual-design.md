# 14. Brand and visual design system

**Status:** decided
**Date:** 2026-09-14
**Decides:** the complete visual identity, with implementable values
**Depends on:** subplan 01 (the name), 06 (`DomainVerdict` drives encoding)
**Feeds:** subplan 13 (implementation), the README, the docs site

---

## The constraint that makes this different from ordinary brand work

**In this product, colour is load-bearing.** Confidence cartography means colour and texture encode
epistemic status: whether a prediction is validated, marginal, or a guess. That is information, not
decoration.

Everything follows from that:

- **The brand palette must not compete with the semantic palette.** If the interface chrome is
  saturated and colourful, the one place colour genuinely means something loses its signal. So the
  base palette is deliberately quiet, and saturation is spent where it carries meaning.
- **Accessibility is a correctness requirement, not a compliance checkbox.** If colour carries
  information and a colourblind user cannot distinguish two states, the tool is lying to them.
- **Colour alone can never be the only encoding.** Measured below, this is not a style preference.

This is why the brief's instruction to avoid generic AI-startup aesthetics is easy to follow here.
A purple-to-blue gradient is not merely unfashionable in this context, it would actively destroy the
product's primary communication channel.

---

## The measurement that settled the encoding

Contrast ratios between the four semantic colours, computed rather than assumed:

| Pair | Ratio |
|---|---:|
| signal vs caution | 1.44:1 |
| signal vs unknown | 1.26:1 |
| signal vs refuse | 1.18:1 |
| caution vs unknown | 1.14:1 |
| caution vs refuse | 1.70:1 |
| unknown vs refuse | 1.49:1 |

All well under 3:1. **These four states are not reliably distinguishable by luminance**, which means
anyone with a colour vision deficiency affecting hue discrimination cannot tell them apart from
colour alone. That is not a flaw to fix by choosing better colours; it is inherent to using four
distinguishable hues at similar lightness, and picking hues far apart in lightness would make the
interface look like a traffic light.

**So texture carries confidence, and hue carries category.** Two orthogonal dimensions, encoded in
two orthogonal channels.

This is not a workaround. It is what cartographers have done for a century, and "confidence
cartography" is the brief's own phrase. Uncertain regions on a real map are hatched, stippled, or
faded. We are borrowing a solved problem's solution.

| Dimension | Channel | Values |
|---|---|---|
| **What a feature is** (promoter, RBS, CDS) | Hue and glyph shape | Follows SBOL Visual conventions |
| **How much we trust a prediction** | Fill texture and opacity | Solid, hatched, stippled, outline-only |

---

## Confidence encoding

The visual grammar for each `DomainVerdict` variant. Computed from the verdict, never authored per
diagram (subplan 06).

| Verdict | Fill | Border | Interval bar | Reads as |
|---|---|---|---|---|
| `InDistribution` | Solid, full opacity | Solid 1px | Narrow, solid caps | Measured and firm |
| `Marginal` | 45° hatch, 8px pitch | Solid 1px | Wider, hatched caps | Provisional |
| `OutOfDistribution` | Stipple, low density | Dashed 1px | Wide, open caps, no endpoints | A guess, and openly so |
| `Uncalibrated` | No fill, outline only | Dotted 1px | **No bar at all** | We have no basis for an interval |

`Uncalibrated` drawing no interval bar is deliberate. A model with no calibration evidence has no
honest interval to draw, and drawing a wide one would imply a quantified uncertainty we have not
earned. Absence is the honest rendering.

**The grammar works in greyscale.** That is the test: print the interface in black and white and
every confidence state remains distinguishable. Subplan 17's SVG snapshot tests include a greyscale
pass for exactly this.

---

## Colour system

Anchored in **paper and ink**, the visual language of scientific publication, rather than in the
glowing-dashboard idiom. Integrase produces figures that end up in papers, and an interface that
looks like where the figure will live is more honest than one that looks like a SaaS dashboard.

### Light mode: paper

| Token | Hex | Contrast on paper | Use |
|---|---|---:|---|
| `--paper` | `#FAF9F7` | base | Background. Warm off-white, not `#FFFFFF`: pure white on a backlit display is harsher than paper and reads as cheap. |
| `--paper-2` | `#F1EFEB` | | Raised surfaces, table stripes |
| `--ink` | `#16161A` | 17.15:1 AAA | Primary text. Near-black with a trace of blue-violet, not `#000000`: pure black on off-white vibrates. |
| `--ink-2` | `#3B3B42` | 10.56:1 AAA | Secondary text |
| `--ink-3` | `#6E6E78` | 4.79:1 AA | Tertiary text, axis labels |

### Dark mode: slate

Not an inversion. A genuinely neutral dark, avoiding the blue-black that every developer tool has
converged on.

| Token | Hex | Contrast on slate | Use |
|---|---|---:|---|
| `--slate` | `#0E1013` | base | Background |
| `--slate-2` | `#17191D` | | Raised surfaces |
| `--chalk` | `#E8E6E1` | 15.28:1 AAA | Primary text. Warm, not pure white. |
| `--chalk-2` | `#A8A6A1` | 7.83:1 AAA | Secondary text |

### Semantic palette

The only saturated colours in the system. Each is verified against its background.

| Token | Light | on paper | Dark | on slate | Meaning |
|---|---|---:|---|---:|---|
| `--signal` | `#1A6B5A` | 6.06:1 AA | `#4FB49B` | 7.56:1 AAA | Validated, in-distribution |
| `--caution` | `#9C6019` | 4.87:1 AA | `#D9A05B` | 8.28:1 AAA | Marginal, provisional |
| `--unknown` | `#6E6E78` | 4.79:1 AA | `#8A8894` | 5.47:1 AA | Out of distribution, uncalibrated |
| `--refuse` | `#9B2C2C` | 7.15:1 AAA | `#E06C6C` | 5.93:1 AA | Screening refusal |

**Every value passes WCAG AA for body text, verified by computation rather than by eye.** `--caution`
was originally `#A86A1F` and measured 4.20:1, failing AA. It was darkened to `#9C6019` for 4.87:1.
That correction is recorded because "it looked fine" is how accessibility failures ship.

**Reasoning behind the specific hues:**

- **`--signal` is a deep green-teal, not blue.** Blue is the default "primary" of every developer
  tool and carries no meaning. Green reads as verified without the cheerful connotation of a
  brighter green, and the teal shift keeps it from reading as a success toast.
- **`--unknown` is deliberately the same value as `--ink-3`.** Out-of-distribution is not an alarm
  state, it is an absence of knowledge, and it should read as quiet rather than alarming. Rendering
  uncertainty in a warning colour would push users away from novel design, which is what the field
  exists to do.
- **`--refuse` is a muted brick, not a bright red.** A screening refusal is usually a researcher who
  did not know a component was regulated (subplan 24). The interface should read as "stop and check
  this," not as an accusation.
- **No gradients anywhere.** Not as an aesthetic rule but a functional one: a gradient makes fill
  saturation ambiguous, and fill saturation carries meaning here.

### What is deliberately absent

No purple. No blue-to-violet gradient. No neon accent on dark. No glassmorphism. Not because they
are overused, though they are, but because every one of them adds visual noise to channels this
product needs to keep clean.

---

## Typography

| Role | Typeface | Reasoning |
|---|---|---|
| Interface | **Inter** | Designed for screen UI at small sizes. Its tabular figures matter: numeric predictions in a column must align, and proportional figures make a results table unreadable. |
| Sequence and code | **JetBrains Mono** | The critical property is unambiguous `0`/`O`, `1`/`l`/`I`, and clear `ACGT` at small sizes. Misreading a base is a real error mode. |
| Documents and long-form docs | **Source Serif 4** | Long-form reading, and it signals "publication" rather than "landing page". |

**Sequences are always monospace, always.** A nucleotide sequence in a proportional face is a bug,
because position matters and a user counting bases needs a fixed grid.

**Numbers with uncertainty have a fixed format**, so scanning a column is possible:

```
4.2e3  [1.9e3 – 9.1e3]  80%  in-dist
```

Value, interval, coverage, domain. Same order everywhere, aligned on the value.

### Scale

A 1.25 ratio from a 16px base. Small ratios suit dense information; 1.5 would waste vertical space
a design tool needs.

```
12 / 14 / 16 / 20 / 25 / 31 / 39 px
```

---

## Spacing and layout

**4px base unit**, scale `4 / 8 / 12 / 16 / 24 / 32 / 48 / 64`.

**Density is a feature.** This is a professional tool used for hours, not a marketing page. Generous
whitespace is right for a landing page and wrong for a workbench. The reference is a well-designed
IDE or a good oscilloscope UI, not a startup homepage.

**Layout principles:**

1. **The sequence is the spine.** In any construct view, the linear sequence runs horizontally and
   everything else aligns to its coordinate system. Position is meaningful, so the layout preserves
   it.
2. **Uncertainty is never behind an interaction.** Intervals and domain verdicts are visible at
   rest. A tooltip is where information goes to be ignored, and the entire product thesis is that
   this information is not optional.
3. **One accent per view.** If two things are highlighted, neither is.
4. **Borders over shadows.** 1px hairlines rather than elevation. Shadows imply physical layering
   that a data-dense tool does not have, and they blur edges where precision matters.

---

## Logo direction

Described precisely enough to execute, though no artwork is generated here.

**The concept: an integration site.** A serine integrase recognises two attachment sites, `attB` and
`attP`, and recombines them. The mark is that junction, abstracted.

**Form:** two horizontal strokes of unequal weight approaching from left and right on the same
baseline, meeting at a vertical seam. They do not merge into a single continuous line; the seam
stays visible as a hairline gap or a subtle offset. Reading as "two things joined, and the join is
visible" is the whole idea, and it matches the positioning from subplan 01: Integrase is connective
tissue, and it does not pretend the seams are not there.

**Proportion:** strokes at 2 units, gap at 1 unit, total mark on a 16-unit square grid. Optically
centred, not mathematically centred.

**Colour:** `--ink` on `--paper`, `--chalk` on `--slate`. Single colour, never gradient. Must work at
16px as a favicon, which is why the form is two strokes and a gap rather than anything with detail.

**Wordmark:** "Integrase" in Inter Medium, standard tracking, sentence case. Never all caps. The name
is a real enzyme and typesetting it as a shouted acronym would undercut the domain literacy it was
chosen for (subplan 01).

**What to avoid:** a DNA double helix. Every synthetic biology project uses one, it is illegible at
small sizes, and it says "biology" rather than saying anything about this tool.

---

## Consistency across surfaces

| Surface | Application |
|---|---|
| README | Paper palette. The architecture diagram uses the semantic colours with their real meanings, so the diagram teaches the visual grammar before the user sees the app. |
| Docs site | Source Serif 4 body, Inter headings, same tokens. |
| Application | Density-first. Same tokens, same confidence grammar. |
| Figures exported by users | The SVG output path (subplan 06) emits the same grammar, so a figure in a paper carries the uncertainty encoding with it. |

That last row matters more than it looks. **A figure exported into a publication keeps the
confidence encoding.** The uncertainty travels with the result instead of being stripped at the
point where it would reach the widest audience.

---

## Open items

- [ ] Verify the palette against deuteranopia, protanopia, and tritanopia simulation. The greyscale
      test covers the worst case, but simulation should confirm.
- [ ] Hatch and stipple patterns need to render correctly at small sizes in both wgpu and SVG.
      Texture that turns to mush at 12px would silently defeat the whole encoding.
- [ ] Font licensing: Inter (OFL), JetBrains Mono (OFL), Source Serif 4 (OFL) all need confirming
      before bundling, per R-3.3.
