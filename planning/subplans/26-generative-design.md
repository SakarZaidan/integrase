# 26. Generative design and foundation model integration

**Status:** decided. Restores a capability the first version of subplan 02 cut.
**Date:** 2026-09-14
**Decides:** how generative sequence design works, and what has to be true for it to be responsible
**Depends on:** subplans 08 (calibration), 24 (screening), 29 (provenance)
**Feeds:** subplans 21, 29, 30

---

## The position

**Integrase does not train generative models. It makes other people's generative models
accountable.**

Evo 2 is Apache-2.0 with open weights, 40B parameters, 1 Mb context, trained on 9.3T nucleotides,
and has 4,213 GitHub stars. Anything we trained would be worse. The original cut was right that we
should not compete with it and wrong that the only alternative was to defer.

The gap is not generation. The gap is **everything that should surround generation and currently
does not**: uncertainty on generated sequences, screening before they leave the sandbox, and a
provenance record saying a model wrote them.

---

## What is missing today

A researcher generates a sequence with Evo 2. What they get is a sequence. What they do not get:

| Missing | Consequence |
|---|---|
| Any calibrated confidence | A generated promoter and a characterised one look identical in a FASTA file |
| A screening step | Generation happens in a notebook; screening happens at the vendor, if at all |
| A record that a model produced it | The screening community treats AI-generated sequence as a distinct category, and there is no way to identify it |
| Which region the model wrote | A hand-designed construct with one regenerated 250 bp window is indistinguishable from a wholly generated one |

Commec v2.0's July 2026 release added sensitivity to AI-redesigned proteins split into short
fragments. **The screening side of the field is already adapting to generative models. The design
side has not.**

---

## Architecture: generation is a `Model`, not a special case

The key decision, and it makes almost everything else fall out:

```rust
impl Model for GenerativeModel {
    type Input  = GenerationRequest;   // context, constraints, region to fill
    type Output = GeneratedSequence;
    fn predict(&self, req) -> Prediction<GeneratedSequence>;
    fn domain_check(&self, req) -> DomainVerdict;
}
```

Generation goes through the same contract as prediction (subplan 06). That means R-0.1 applies
without a special case: **a generated sequence cannot be obtained without the uncertainty attached
to it.**

```
GenerationRequest
      │
      ▼
 ml-service (Python, gRPC) ──► Evo 2 or another genomic LM
      │
      ├─ sequence
      ├─ per-token likelihood  ──►  sequence-level confidence
      └─ model id + digest
      │
      ▼
 integrase-sim
      ├─ Prediction<GeneratedSequence>
      ├─ DomainVerdict: is this like anything the model saw?
      └─ downstream: predicted function, with its own interval
      │
      ▼
 integrase-screen   ◄── mandatory before the sequence leaves the sandbox
      │
      ▼
 integrase-vdr  ──► derivation step: { kind: generated, model, digest, region }
```

### What confidence means for a generated sequence, stated carefully

This is subtler than for a prediction and it is easy to get wrong in a way that misleads.

A language model's per-token likelihood says how typical a sequence is **of its training
distribution**. It does not say the sequence will work. A highly likely sequence is one the model
finds unsurprising, which correlates with being biologically plausible and is not the same claim.

So a generated sequence carries **two separate quantities**, and conflating them would be exactly
the dishonesty this project exists to reject:

| Quantity | Means | Source |
|---|---|---|
| **Typicality** | How unsurprising this sequence is to the generating model | Per-token likelihood |
| **Predicted function** | What our calibrated models say it will do, with an interval | `integrase-sim` |

A sequence can be highly typical and predicted to do nothing useful. It can be atypical and
predicted to work. **Reporting only the first, which is what a raw generative workflow does, tells
the user almost nothing they want to know.**

And there is a compounding problem worth naming: a generated sequence is, almost by construction,
more likely to be out-of-distribution for our downstream predictors, which were calibrated on
natural and hand-designed libraries. **Generation and prediction pull in opposite directions.** The
honest output for a novel generated sequence is frequently a wide interval and an
`OutOfDistribution` verdict, and the interface must not soften that because it is disappointing.

---

## The safety position

Generative design is the highest-risk capability in this project, and the brief's principle applies
exactly: never restrict thinking, designing, or simulating inside the sandbox; restrict only at the
boundary toward physical synthesis.

| Stage | Policy |
|---|---|
| Generating in the sandbox | **Unrestricted.** This is thinking. A model proposing a sequence is not a hazard. |
| Viewing, analysing, iterating | **Unrestricted.** |
| Interchange export | Not gated, per subplan 21's split, **with one addition: the generated region is marked in the exported record.** |
| Synthesis-bound export | **Gated**, identically to any other design. `Cleared<T>` required. |

### Two additions specific to generation

**1. The derivation chain records the region, not a flag.** Subplan 29's format records
`{ kind: generated, model, model_digest, prompt_digest, region: [start, end] }`. "Which 250 bp did
the model write" is a different and more useful question than "did a model touch this," and it is
the question a screener actually has.

**2. We inherit upstream safety filtering and say so.** Evo 2's training data was filtered to
exclude viral sequences infecting eukaryotic hosts. That is a meaningful property of the model and
it belongs in the `ModelCard`, because a user should know which model's filtering they are relying
on. **It is not a substitute for screening**, and the pipeline screens regardless.

### What we will not build

| Not building | Why |
|---|---|
| A model trained by us | Evo 2 is better. Subplan 08's data constraint is unchanged. |
| An unfiltered generation endpoint | Generation is unrestricted in the sandbox; providing a hosted API that generates and returns sequence with no screening path attached is a different product and not one we want to be. |
| Guarantees about generated sequence safety | A model's training filter is not a screen. The screen is the screen. |

---

## Hardware

Subplan 08 established the constraint and it applies harder here: **12 GB of VRAM.** Evo 2 at 40B
does not fit.

| Path | Assessment |
|---|---|
| Smaller Evo 2 variants locally | Default. Works on this hardware. |
| Remote inference | Supported, opt-in, with the network dependency stated. Never the default, because a design tool whose pitch is local honest analysis should not silently require an API key. |
| Larger models on user hardware | Supported. The `ml-service` boundary means the model is a deployment choice, not a code change. |

The `ml-service` gRPC boundary from subplan 05 is what makes this tractable: swapping Evo 2 for its
successor is a service configuration change, which is exactly the property the brief asked for when
it said a better model should be a drop-in replacement rather than a rewrite.

---

## Testing

| Class | Assertion |
|---|---|
| Uncertainty attached | No API path returns a generated sequence without `Prediction` wrapping. Compile-fail test. |
| Typicality and function are distinct | Both present, separately labelled, never summed into one score |
| OOD honesty | A generated sequence unlike the calibration set returns `OutOfDistribution`, and the test asserts we do **not** quietly widen and call it in-distribution |
| Screening mandatory | Generated sequence cannot reach synthesis-bound export uncleared. Fault injection included. |
| Provenance | Region-level derivation survives a VDR round-trip |
| Degradation | With `ml-service` unreachable, the rest of the system works and reports generation as unavailable (subplan 11, E4) |

---

## Open items

- [ ] Verify Evo 2 weight licence terms precisely before depending on them (R-3.2, carried from
      subplan 08)
- [ ] Benchmark which Evo 2 variant fits 12 GB usefully, and what its quality cost is
- [ ] Decide how per-token likelihood maps to a reportable typicality figure. **This needs to be a
      defined, documented statistic, not a raw log-prob sum that varies with length.**
- [ ] Determine whether downstream predictors can be calibrated on generated sequences at all, or
      whether `OutOfDistribution` is the permanent honest answer for novel generation. **If it is
      permanent, say so in the docs rather than implying the interval is meaningful.**
