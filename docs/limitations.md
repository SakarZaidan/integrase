# Limitations

What Integrase cannot do, in one place.

This page is a feature, not a disclaimer. Every competing tool in this field has
most of these same limitations and states none of them. A user who reads this and
uses Integrase anyway knows what they are getting, which is the only kind of
trust worth having with scientists.

---

## What the software does not do at all

- **Does not edit genomes.** It operates on sequence data in files.
- **Does not interact with living organisms.**
- **Does not order or produce physical DNA.**
- **Does not perform wet-lab work**, and nothing it outputs substitutes for
  experimental validation.

---

## What the predictions are, and are not

**Predictions are model outputs, not measurements.** Integrase computes what
published biophysical models say about a sequence. That is a different claim from
what the sequence will do in a cell, and the gap between them is large.

### The intervals describe model uncertainty, not biological uncertainty

This is the single most important thing on this page.

A calibrated 80% interval means: on data resembling the calibration set, an
interval constructed this way contains the true measured value about 80% of the
time. It does **not** mean there is an 80% chance your construct will express at
that level in your organism, in your conditions, on your plasmid.

**Factors invisible to any sequence-only model:**

| Factor | Why it is invisible |
|---|---|
| Genomic context | Upstream transcription and neighbouring genes affect expression and are not in the sequence being scored |
| DNA supercoiling | A physical property of the molecule in the cell |
| Host physiology | Growth rate, stress state, and resource competition |
| Growth conditions | Media, temperature, induction timing |
| Plasmid copy number | Scales expression independently of the construct |
| Metabolic burden | A strong construct changes the cell that hosts it |

No interval computed from sequence accounts for any of these. Widening the
interval would not fix it either, because the error is not random with respect to
the model.

### Published accuracy is wide, and dataset-dependent

For ribosome binding site strength, thermodynamic model performance ranges from
**R² above 0.8 down to below 0.2** depending on which dataset it is evaluated on.
That spread is the honest characterisation, and a single headline number would
misrepresent it.

We report figures with their source type, because a vendor's self-reported
accuracy on their own held-out split is weaker evidence than an independent
wet-lab validation:

| Claim | Figure | Evidence quality |
|---|---|---|
| RBS Calculator, thermodynamic | R² 0.2 to 0.8 | Published, across datasets |
| Vendor deep model | R² 0.93 | **Vendor self-report.** Not independently benchmarked. |
| Transformer de novo RBS design | Pearson r = 0.80 | Wet-lab validated, on selected designs |

### Promoter prediction is weaker than RBS prediction

RBS prediction has a physically motivated thermodynamic model with a mechanism
behind it. Constitutive promoter strength prediction has position weight matrix
scoring plus spacer penalties, which is a statistical description rather than a
mechanism.

Promoter predictions therefore ship with **wider default intervals**, and the
model card says why. Rendering both with the same visual confidence would be a
lie of omission.

### Out of distribution means the guarantee stops applying

Calibration uses split conformal prediction, which gives a coverage guarantee
under *exchangeability* between the calibration data and your input. An input
unlike anything in the calibration set breaks that assumption, and the guarantee
evaporates exactly where you most need it.

This is why `DomainVerdict` is a separate field from the interval rather than
folded into it. The interval answers "how wrong am I likely to be, given this
input is like my calibration data?" The verdict answers "is it?"

When the verdict is `OutOfDistribution` or `Uncalibrated`, treat the number as a
guess with a plausible magnitude, not as a prediction with a confidence level.

### Validated on *E. coli*

Models are fitted on *E. coli* data. Other hosts return
`DomainVerdict::OutOfDistribution` with the host mismatch named. The prediction
is still computed, because refusing to answer anything unfamiliar would make the
tool useless for novel design, but its stated coverage does not hold.

---

## Screening

**Integrase is not a biosecurity control.** It is a design tool with a screening
gate. The actual control is the synthesis provider's own screening, where the
IGSC protocol and the OSTP framework operate on real orders.

Ours is defence in depth. It prevents accidents, catches the uninformed, creates
an audit trail, and declines to be a convenient instrument.

**Two things it cannot prevent**, documented rather than hidden:

1. **Copying a sequence out of the interface by hand.** No gate stops this.
   Blocking copy-paste would make the tool unusable and stop nobody.
2. **Running a modified local build with screening removed.** It is open-source
   software on your machine.

**It also cannot reliably stop a determined, knowledgeable adversary.** Codon
reshuffling, splitting an order across vendors, and reconstructing from
individually sub-threshold fragments all defeat sequence-level screening. This is
well understood in the biosecurity literature and is not peculiar to us.

**It does not make you compliant with anything.** No government currently
mandates nucleic acid synthesis screening; the mechanism is procurement
conditionality on federal research funding plus voluntary industry standards.
Integrase implements screening consistent with the framework's approach, which is
a different and smaller claim than compliance.

**With no backend configured, it refuses everything** on the synthesis-bound
path. That is deliberate: passing everything when unconfigured would let users
believe they had been screened.

---

## Scale

Sequence indexing costs roughly 3.9 bytes per base with the compressed index.

| Genome | Approximate memory |
|---|---:|
| *E. coli*, 4.6 Mbp | 18 MB |
| *S. cerevisiae*, 12.2 Mbp | 47 MB |
| *D. melanogaster*, 144 Mbp | 0.6 GB |
| Human, 3.1 Gbp | ~12 GB |

A human genome needs a workstation, not a laptop. Inputs projected to exceed
available memory are refused **before allocation**, with the projection in the
error message, because an out-of-memory kill is not an error message.

---

## Maturity

Integrase is at version `0.0.0`. Nothing is published to crates.io or PyPI. The
biophysical models, SBOL import/export, circuit compilation, and rendering are
not written yet.

Pre-1.0, breaking changes ship in minor versions. That is stated rather than
pretended otherwise.

---

## If you find a limitation not listed here

That is a documentation bug worth filing. This page is meant to be complete, and
a limitation a user discovers themselves is one we should have told them about.
