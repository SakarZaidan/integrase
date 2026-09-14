# 16. Simulation and biophysical modeling accuracy

**Status:** complete
**Date:** 2026-09-14
**Decides:** which deterministic models are implemented, what their real accuracy is, and how an ML
layer is validated against them rather than trusted over them
**Depends on:** `00-research-log.md` section 4, subplans 04, 06
**Feeds:** subplans 08, 17, and the `ModelCard` contents

The brief asks for a focused, honest deep-dive on the biophysics. Honest means the headline is not
flattering: **the field does not have a generalizable predictive model for even the simplest genetic
elements, and this project does not change that.** What it changes is whether the user is told.

---

## What we implement as the deterministic baseline

### RBS: a Salis-style thermodynamic model

The RBS Calculator approach models translation initiation rate as a function of the free energy
change when the 30S ribosomal complex binds the mRNA. The standard decomposition:

```
ΔG_total = ΔG_mRNA:rRNA + ΔG_start + ΔG_spacing − ΔG_standby − ΔG_mRNA

translation initiation rate  ∝  exp(−β · ΔG_total)
```

| Term | What it captures |
|---|---|
| `ΔG_mRNA:rRNA` | Shine-Dalgarno to anti-Shine-Dalgarno hybridization |
| `ΔG_start` | Start codon identity (AUG, GUG, UUG differ) |
| `ΔG_spacing` | Penalty for deviation from optimal SD-to-start spacing |
| `ΔG_standby` | Unfolding of the standby site |
| `ΔG_mRNA` | Folding energy of the mRNA in isolation, which must be paid to unfold |

Source: https://pubmed.ncbi.nlm.nih.gov/21601672/

**What this needs that we do not write ourselves:** RNA secondary structure prediction, to compute
`ΔG_mRNA` and `ΔG_standby`. That is a solved and separately-validated problem (ViennaRNA and
equivalents) and reimplementing it would be exactly the not-invented-here error subplan 23 warns
about. **This is an open dependency question**, since ViennaRNA's licensing needs checking before
we depend on it, and there is no obvious pure-Rust equivalent at the required quality. Listed as an
open item rather than assumed away.

### Promoter: motif and spacer scoring, with lower expectations

Sigma70 promoter strength is modelled from the −35 hexamer, the −10 hexamer, the spacer length
between them, and flanking UP-element contributions.

**The honest framing is different from RBS, and the difference matters.** For RBS there is a
physically-motivated thermodynamic model with a mechanism behind it. For constitutive promoter
strength there is substantially less: position weight matrix scoring against consensus plus spacer
penalties, which is a statistical description rather than a mechanism. The literature is explicit
that predicting promoter strength directly from sequence was "previously almost impossible" and that
thermodynamic proxies were the workaround.

Sources: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6228582/ (PromoterPredict),
https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9701712/ (Sigma70Pred)

**Consequence:** the promoter model ships with wider default intervals than the RBS model, and its
`ModelCard` says why. Presenting both with the same visual confidence would be a lie of omission.

---

## What the accuracy actually is

This table is the most important content in this subplan, and the spread within it is the finding.

| Model | Reported | Source type | How we treat it |
|---|---|---|---|
| RBS Calculator, thermodynamic | **R² from >0.8 down to <0.2** | Published, across datasets | This range is the honest characterisation. Not a single number. |
| SAPIENS / "RBS Predictor" | R² = 0.93 | **Vendor self-report**, ~250k E. coli 5'UTRs | Cited only with that label attached. Not treated as state of the art. |
| Transformer de novo RBS, *P. denitrificans* | Pearson r = 0.80 | Wet-lab validated, **on selected designs** | The most trustworthy figure here, and still a biased subset. |
| Sigma70Pred | 81.5% to 90.4% accuracy | Published, independent test set | **Classification, not strength regression.** Different task. Never quoted as a strength-prediction figure. |

### Three things this table is designed to stop us doing

**1. Quoting the best number.** R² = 0.93 is the highest figure available and the weakest evidence:
a vendor reporting on their own held-out split. Using it in marketing would be technically true and
substantively dishonest.

**2. Conflating classification with regression.** Sigma70Pred's ~90% is for deciding *whether a
sequence is a promoter*, not *how strong it is*. These get conflated constantly in this literature
and the conflation always flatters the tool doing it.

**3. Collapsing a range to a point.** "R² ≈ 0.8" for the thermodynamic RBS model is a defensible
number on some datasets and a fabrication on others. The range is the result. A model whose error
varies by a factor of four depending on which library you test it on is telling you something
important about its generalization, and reporting the good end hides exactly that signal.

---

## Known failure modes

These are what `DomainVerdict` has to detect. Each one is a concrete, checkable condition rather
than a vague caveat.

| Failure mode | Why it breaks | Detection |
|---|---|---|
| **Strong mRNA secondary structure** | `ΔG_mRNA` dominates, and secondary structure prediction is itself approximate. Errors compound. | Compute structure; flag when folding energy is large relative to the hybridization term. |
| **Non-*E. coli* host** | The anti-SD sequence, ribosome, and codon context differ. A model fit on *E. coli* is out of domain elsewhere, and the *P. denitrificans* work exists precisely because of this. | Host is an explicit model input. A mismatch is `OutOfDistribution`, not a silent extrapolation. |
| **Non-canonical start codons** | `ΔG_start` is parameterised on a small number of observations for GUG and UUG. | Flag `Marginal` for non-AUG starts. |
| **Unusual spacer lengths** | `ΔG_spacing` is a fitted penalty curve, poorly constrained at the tails. | Flag `Marginal` outside the well-sampled range. |
| **Genomic and compositional context** | Upstream transcription, supercoiling, and neighbouring genes affect real expression and appear nowhere in the model. | **Not detectable from sequence alone.** Stated as a standing limitation in every `ModelCard`, not as a per-prediction flag. |
| **Very short or very long 5'UTRs** | Outside the fitted range. | Length bounds check. |

The fifth row is the uncomfortable one. Context effects are real, substantial, and invisible to any
sequence-only model. No confidence interval computed from sequence can account for them. **The
honest position is that our intervals describe uncertainty about the model's own prediction, not
uncertainty about what will happen in a cell**, and that distinction has to be stated plainly in the
docs rather than buried. Overstating it would be the exact failure the brief warns against.

---

## How the ML layer is validated against the baseline, not trusted over it

The brief asks specifically how the ML correction layer is validated rather than simply trusted.
Four rules.

**V-1. The deterministic model is always computed.** Even when an ML model is available, the
thermodynamic prediction is computed and retained. It is cheap, it is mechanistically interpretable,
and it is the reference against which the ML output is judged.

**V-2. Disagreement is a signal, and it is surfaced.** When the ML correction and the deterministic
baseline disagree by more than a threshold, that is reported to the user rather than silently
resolved in the ML model's favour. Large disagreement on an in-distribution input usually means one
of them is wrong, and the user is better placed than we are to know which.

**V-3. The ML layer must beat the baseline on held-out data to be enabled at all.** Not "on the
training distribution," which is trivial, but on the out-of-distribution evaluation set from subplan
08. A correction layer that improves interpolation and degrades extrapolation is worse than no
correction layer, because it moves error into exactly the regime where users are least able to
notice.

**V-4. An ML model with no calibration evidence reports `Uncalibrated` and does not override the
baseline.** It may be shown alongside. It may not be presented as the answer.

---

## What this subsystem does not claim

Written explicitly so it can be lifted into the README and the docs verbatim.

- It does not predict what will happen in a living cell. It predicts what published biophysical
  models say about a sequence.
- It does not account for genomic context, host physiology, growth conditions, plasmid copy number,
  or metabolic burden.
- Its intervals quantify uncertainty about the model's prediction, not about the biological outcome.
- It is validated on *E. coli* data. Other hosts are out of distribution and are labelled as such.
- It is not a substitute for experimental validation and nothing in it should be read as one.

**This list is a feature, not a disclaimer.** Every competing tool has the same limitations and
states none of them. A user who reads this list and then uses the tool anyway is a user who knows
what they are getting, which is the only kind of trust worth having with scientists.

---

## Open items

- [ ] RNA secondary structure dependency: ViennaRNA licensing, or a viable alternative. **Blocks the
      RBS model.** -> **08**, and possibly `REQUESTS.md`
- [ ] Threshold for V-2 disagreement reporting, which needs data to set -> **08**
- [ ] Whether a promoter model is worth shipping in Phase 1 given how much weaker the science is
      than for RBS -> **21**
