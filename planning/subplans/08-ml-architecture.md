# 08. AI/ML architecture and plan

**Status:** decided
**Date:** 2026-09-14
**Decides:** model architecture, training data, evaluation methodology, and how calibrated
uncertainty reaches the user
**Depends on:** subplans 04 (data), 06 (`Prediction<T>`), 16 (baseline accuracy)
**Feeds:** subplans 09, 15, 17, 21

The brief asks for a completely honest statement of current accuracy ceilings with no rounding up.
That statement is in section 6 and it is the part of this document to read first if reading only
one part.

---

## The architecture decision

**Foundation model embeddings plus a small calibrated head, not a from-scratch CNN/BiLSTM.**

The brief specifies "CNN/BiLSTM/Transformer approaches for RBS strength prediction." That was the
right answer when the brief was written. In 2026 it is not, for a specific reason rather than
fashion.

| Approach | Data needed | Generalization | Verdict |
|---|---|---|---|
| From-scratch CNN/BiLSTM | Large labelled set, and we have roughly 12,653 constructs | Poor off-distribution. Learns the library it was trained on. | Rejected as primary |
| **Foundation model embeddings + calibrated head** | Small labelled set. The representation is pretrained on 9.3T nucleotides. | Better, though not solved | **Chosen** |
| Full fine-tune of a genomic foundation model | Moderate, plus serious compute | Best potential | Not on 12 GB VRAM |

The binding constraint is data, not architecture. Kosuri et al. 2013 gives 12,653 constructs, which
is a small dataset by deep-learning standards. Training a transformer from scratch on it produces a
model that has memorised one MPRA library. Using a pretrained representation and fitting a small
head on top is the standard answer to exactly this situation, and it is the one place where the
2025-2026 shift in the field genuinely helps us.

### The hardware reality, stated rather than wished away

This machine has **12 GB of VRAM**. Evo 2 at 40B parameters does not fit and no quantization makes
it fit comfortably. Three options:

1. **Smaller Evo 2 variants** for local embedding extraction. Preferred. Embeddings are computed
   once per sequence and cached, so inference cost is amortised.
2. **Remote API** for the large model. Introduces a network dependency and a cost per call into the
   critical path of a tool whose whole pitch is local, honest analysis. Avoid for the default path.
3. **A smaller genomic language model** entirely. Viable and needs benchmarking against option 1.

**Decision: option 1, with embeddings cached and the extraction step optional.** The system must
work with the deterministic baseline alone, because a user without a GPU is a normal user, not an
edge case. The ML layer is an enhancement, never a requirement.

**Open item flagged honestly:** Evo 2's exact weight licensing has not been verified (subplan 04).
Nothing depends on it until that is checked, per R-3.2.

---

## Training and evaluation data

### What we have

**Kosuri et al. 2013 (PNAS):** all combinations of 114 promoters and 111 RBSs, 12,653 constructs,
*E. coli*, measured by massively parallel reporter assay.
https://www.pnas.org/doi/10.1073/pnas.1301301110

### Why one dataset is not enough, and this is the crux of the whole plan

A random held-out split of a single MPRA library measures **interpolation**: can the model predict a
sequence drawn from the same design space as its training data? That is a much easier question than
the one users actually have, which is whether the prediction holds for *their* sequence, from
*their* library, in *their* conditions.

A model can achieve excellent held-out R² on Kosuri and be useless in practice, and the calibration
would look perfect while being meaningless, because the calibration set shares the training set's
biases.

**Requirement: at least one independent dataset from a different study, used only for
out-of-distribution evaluation and never for training or calibration fitting.** Identifying it is an
open item and it is on the critical path. Related work on cross-context 5'UTR transfer addresses
exactly this problem and is the right starting point:
https://academic.oup.com/nar/article/52/13/e58/7691520

**If no second dataset can be assembled, the honest outcome is that the model ships reporting
in-distribution calibration only, with `OutOfDistribution` returned for anything unlike Kosuri, and
the `ModelCard` saying so.** That is a real, useful, limited product. Claiming general calibration
from a single library would not be.

### Splitting

Random splits leak. Promoter-RBS combinatorial libraries share components across rows, so a random
split puts near-duplicates on both sides and inflates every metric.

**Splits are by component, not by construct.** Promoters and RBSs in the test set do not appear in
any training construct. This will produce worse numbers than a random split, and those numbers will
be the true ones.

---

## Calibrated uncertainty: how it is actually produced

This is the project's central claim, so the method has to be principled rather than a heuristic
widening of error bars.

### Split conformal prediction

**Chosen method.** Given a fitted model and a held-out calibration set, conformal prediction
produces intervals with a finite-sample marginal coverage guarantee: an 80% interval contains the
true value at least 80% of the time, without assuming the model is correct or that errors are
Gaussian.

The procedure is simple enough to implement correctly and test:

1. Fit the model on the training split.
2. On a disjoint calibration split, compute conformity scores (absolute residuals, or normalised
   residuals for heteroscedastic data, which expression data is).
3. The interval half-width at level α is the appropriate quantile of those scores.

**Why conformal rather than the alternatives:**

| Method | Problem for us |
|---|---|
| Gaussian error bars from model variance | Assumes a noise model that expression data violates. Usually badly. |
| Ensemble spread | Measures disagreement between models, not error. Confidently wrong ensembles are common. |
| MC dropout / Bayesian NN | Approximate posteriors with no coverage guarantee, harder to test. |
| **Split conformal** | Distribution-free, finite-sample guarantee, and directly testable. |

### The assumption that breaks, stated plainly

**Conformal coverage guarantees hold under exchangeability between the calibration set and the test
point.** Out-of-distribution inputs are precisely the case where exchangeability fails. So the
guarantee evaporates exactly where the user most needs it.

This is not a reason to abandon conformal prediction. It is the reason `DomainVerdict` exists as a
separate field rather than being folded into the interval width. The two answer different questions:

- **The interval** answers "given that this input is like my calibration data, how wrong am I
  likely to be?"
- **`DomainVerdict`** answers "is this input like my calibration data at all?"

Collapsing them into one widened interval would hide the second question, and the second question is
the one that distinguishes a sequence the model understands from one it has never seen anything like.
This is the technical core of what the brief calls confidence cartography, and getting it right is
why the two fields are separate in `Prediction<T>` (subplan 06).

### Domain detection

`DomainVerdict` is computed, not guessed. Candidate signals, to be validated in implementation:

- Distance from the training distribution in foundation-model embedding space.
- Explicit feature bounds from subplan 16's failure-mode table: host organism, start codon, spacer
  length, UTR length, secondary structure magnitude.
- Conformity score distribution: an input whose nearest calibration neighbours all had large
  residuals is a warning regardless of embedding distance.

The feature-bounds checks are shipped first because they are interpretable and directly traceable to
documented model limitations. Embedding-distance thresholds require the calibration work to exist
before they can be set.

---

## Evaluation: what CI actually checks

Per subplan 07's R-2.6, calibration is tested rather than asserted.

| Test | Asserts | Fails when |
|---|---|---|
| **Coverage** | An 80% interval contains the truth 80% ± tolerance of the time on held-out data | Calibration has drifted or the split leaked |
| **Coverage by stratum** | Coverage holds within expression-level deciles, not only on average | The model is well-calibrated on average and badly calibrated on weak promoters, which averaging hides |
| **OOD detection** | Sequences from the independent dataset are flagged `Marginal` or `OutOfDistribution` at a stated rate | Domain detection is decorative |
| **Baseline comparison** | ML layer beats the deterministic baseline on the OOD set, per subplan 16's V-3 | The correction layer improves interpolation and degrades extrapolation |
| **Interval sanity** | Intervals are finite, ordered, and contain the point estimate | Arithmetic or serialization bugs |

The stratified coverage test is the one most likely to catch a real problem. Marginal coverage is
easy to achieve while being useless: a model that is overconfident on weak signals and
underconfident on strong ones can hit 80% overall and be wrong in a systematic, user-visible way.

---

## How uncertainty reaches the user

Unbroken, by construction. The chain is:

```
Model::predict  ->  Prediction<T>  ->  gRPC Prediction message  ->  SDK / UI
```

- **Rust:** `Prediction<T>` is the only return type for a predicted quantity (R-0.1).
- **gRPC:** the `Prediction` message carries `value`, `interval`, `nominal_coverage`, `model_id`,
  and `domain`. All required fields. A proto with optional uncertainty would let a client ignore it.
- **Python:** `Prediction` object with `.value`, `.interval`, `.coverage`, `.domain`. There is no
  `float(prediction)` coercion, deliberately, because that is exactly how the uncertainty would get
  discarded in a notebook one line after it arrives.
- **Visualization:** `DomainVerdict` drives the visual encoding, computed rather than authored
  (subplan 06). A speculative element cannot be rendered as a validated one, because nobody chooses
  how it is rendered.

---

## Retraining and model lifecycle

**Models are versioned artifacts, not code.** A `ModelId` identifies architecture, training data
version, and calibration set version. A `Prediction` records which `ModelId` produced it, so any
result is reproducible and any past claim is auditable.

**Weights are not in git** (excluded in `.gitignore`). They are released as versioned artifacts with
checksums, and the `ModelCard` travels with them.

**Retraining triggers:** a new public dataset of sufficient quality, a validated dataset contributed
by a user, or measured calibration drift. Retraining is never automatic. A new model version goes
through the same evaluation gate as the first one, and a model that fails the gate is not shipped
regardless of how much better its headline R² looks.

---

## The honest accuracy ceiling

No rounding up, as the brief requires.

**What the literature supports:** for RBS strength prediction on *E. coli* sequences similar to the
training distribution, correlation around **r ≈ 0.8** is achievable and has been independently
demonstrated with wet-lab validation. The deterministic thermodynamic baseline ranges from R² > 0.8
to R² < 0.2 depending on the dataset, and that variance is itself the honest characterisation.

**What the literature does not support:**

- That any model generalizes reliably to other host organisms without retraining.
- That R² = 0.93 is the state of the art. That figure is a vendor self-report on their own held-out
  split, and we cite it only with that label.
- That promoter strength prediction is anywhere near as good as RBS prediction. It is not, and
  subplan 16 explains why the underlying science is weaker.
- That sequence-only models can account for genomic context, host physiology, or growth conditions.
  They cannot, and no interval computed from sequence can fix that.

**What we will claim, and only this:**

> Predicts translation initiation rate for *E. coli* ribosome binding sites, reporting a calibrated
> interval and an explicit verdict on whether the input falls within the range where the model was
> validated. In-distribution performance is comparable to published methods. Out-of-distribution
> behaviour is flagged rather than silently extrapolated.

That is a smaller claim than the brief's framing implies, and it is the largest one the evidence
supports. Subplan 22 should check whether this document kept that promise.

---

## Open items

- [ ] **Identify a second, independent expression dataset.** On the critical path for any
      out-of-distribution claim. -> blocks the calibration story
- [ ] Verify Evo 2 weight licensing before depending on it -> R-3.2
- [ ] RNA secondary structure dependency: ViennaRNA licensing or an alternative -> **16**, blocks the
      RBS baseline
- [ ] Benchmark smaller genomic LMs against Evo 2 variants at 12 GB VRAM
- [ ] Set the V-2 baseline-disagreement threshold, which needs data -> **16**
- [ ] Embedding-distance thresholds for domain detection, after calibration data exists
