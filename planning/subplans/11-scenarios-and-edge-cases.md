# 11. Problems, scenarios, use cases, and edge cases

**Status:** complete
**Date:** 2026-09-14
**Decides:** the concrete situations the system must handle, written so they become tests
**Depends on:** subplans 02 (scope cut), 04, 06, 09, 24
**Feeds:** subplan 17 (directly: each scenario below becomes a test), subplan 20 (docs structure)

Written with the downstream use in mind, per the brief. Each scenario states the user, what they
are actually trying to do, what happens, and **what must be true** for it to work. The last line of
each is the testable assertion.

Two of the brief's four scenarios changed during planning and are corrected here rather than
restated. Saying so is the point of writing these after the architecture rather than before.

---

## Scenario 1: the working synthetic biologist importing an existing design

**Who:** a postdoc with a plasmid design already in SynBioHub, who wants to know whether their RBS
choice explains the low expression they measured.

**What they do:** fetch the design, import it, run the RBS prediction, compare against what they
observed.

**What happens:**
1. Import SBOL from a file or a SynBioHub URL.
2. `integrase-sbol` reads it through `sbol-rs`, surfacing any conversion report.
3. Features are mapped, SO role IRIs preserved.
4. RBS prediction runs, returning `Prediction<TranslationRate>`.
5. Result carries an interval and a `DomainVerdict`.

**The correction the research forced.** The brief assumed anonymous SynBioHub access. It requires
authentication on every endpoint probed, including `/rootCollections`. So the primary path is
**import from a file the user already has**, with authenticated URL fetch as a configured extra.
This is not a downgrade in practice: a researcher with a design in SynBioHub can export it, and
file import works offline, which matters more to most users than URL convenience.

**Must be true:**
- A real SBOL file from the `sbol-rs` corpus imports with no feature loss.
- The RBS prediction returns an interval, never a bare number.
- If the design is not *E. coli*-like, `DomainVerdict` is `Marginal` or `OutOfDistribution`, not
  silent extrapolation.
- A conversion report, if `sbol-rs` produced one, is visible rather than discarded.

---

## Scenario 2: the student

**Who:** an iGEM student who has never used a design tool.

**The correction, and it is a real scope change.** The brief's scenario is "a student building a
first genetic logic circuit." Subplan 02 deferred circuit-logic compilation past v1.0, because Cello
already did that well and reimplementing it is a multi-year project. **So this scenario as written
is not supported in v1.0, and pretending otherwise in the docs would be the exact overclaiming the
brief warns against.**

**What the student can actually do in v1.0**, which is still genuinely useful for learning:
1. Assemble a simple construct from parts: promoter, RBS, CDS, terminator.
2. See it rendered as SBOL Visual glyphs.
3. Get a predicted expression level **with an interval**, and see that the interval is wide.
4. Change the RBS and watch the prediction move.
5. Export to SBOL and take it elsewhere.

**Why this is arguably better teaching than the briefed scenario.** A student whose first experience
of a design tool is "here is your answer, and here is how much it does not know" learns something
the field badly needs taught. Every other tool teaches the opposite lesson by omission.

**Must be true:**
- Assembling a construct from parts needs no prior SBOL knowledge.
- The interval is visually prominent, not a tooltip.
- Changing the RBS visibly changes both the prediction and its uncertainty.
- Nothing in the interface implies the prediction is a measurement.

---

## Scenario 3: the computational biologist in a notebook

**Who:** someone comfortable in Python who wants to screen 500 RBS variants and rank them.

**What they do:**

```python
import integrase as ig

design = ig.Design.from_sbol("construct.xml")
variants = [design.with_rbs(s) for s in candidate_rbs]
preds = ig.predict_translation_rate(variants)

# Rank by lower interval bound, not by point estimate: the conservative choice
ranked = sorted(preds, key=lambda p: p.interval.lower, reverse=True)

# What the model has not seen
novel = [p for p in preds if p.domain.is_out_of_distribution]
```

**Must be true:**
- `Prediction` has no `__float__`. Coercing to a number must be an explicit, visible act, because
  the one-liner that discards uncertainty is the failure mode this API exists to prevent.
- Batch prediction is not 500 sequential round-trips.
- `.domain` is inspectable and filterable, so "which of these is the model guessing about" is one
  line.
- Works with no GPU and no network. The deterministic baseline is the default path.

**This scenario is the primary one.** Subplan 02 concluded the reachable audience is the
ML-for-biology community, and this is what reaching them looks like.

---

## Scenario 4: the research group self-hosting screening

**Who:** a lab with institutional biosafety rules stricter than the default.

**What they do:**
1. Run SecureDNA `synthclient` on institutional infrastructure.
2. Point Integrase at it.
3. Optionally add institution-specific rules through a custom backend.
4. Configure exemption certificates signed by their biosafety officer.

**Must be true:**
- Backend is configuration, not a code change.
- Backend identity and policy version appear in every verdict, so an audit can tell which screener
  produced which decision.
- Custom rules compose with, rather than replace, the primary backend. An institution tightening
  its rules must not accidentally loosen the baseline.
- Exemption certificates pass through to the backend. We do not implement our own exemption logic:
  deciding who may work on what is an institutional and regulatory judgment.

---

## Edge cases

The brief asks for the harder cases. These are where the design gets tested.

### E1. A design partially matches a screening pattern

**The most important edge case in the system**, because it is the common real case, and because
both failure directions are bad.

A 200 bp region shows partial homology to a regulated sequence. Not an exact match. Possibly a
conserved domain shared with something harmless.

**Behaviour:** the decision is the backend's, not ours. SecureDNA returns hazards found, their
locations, and an unambiguous flag. `integrase-screen` **does not second-guess it** and does not
apply its own threshold on top. Adding our own interpretation layer over a peer-reviewed screener's
verdict would be the worst kind of not-invented-here: we would be inventing a false-negative source.

What we do own is presentation: which region, what class of concern, and the route forward
(institutional biosafety officer, exemption certificate).

**Must be true:**
- The verdict is reported verbatim, with the matched region located.
- We never downgrade a backend hit to a pass.
- Refusal is a `Refusal` type explaining itself, not a stack trace.
- The design remains fully usable for analysis. Only export is closed.

### E2. A sequence far outside the model's training distribution

A promoter from a thermophilic archaeon, submitted to a model fitted on *E. coli*.

**Behaviour:** the prediction is still computed. `DomainVerdict::OutOfDistribution` is returned with
a reason. The interval is reported but explicitly flagged as not carrying its nominal coverage,
because conformal guarantees rest on exchangeability and this input breaks it (subplan 08).

**Must be true:**
- The system does not refuse to answer. Refusing to predict anything unfamiliar makes the tool
  useless for novel design, which is what the field is for.
- The output is unmistakably marked as unvalidated, in the API and in the visual encoding.
- The `reason` names the specific trigger (host mismatch, unusual spacer, embedding distance), not
  a generic warning.

### E3. A multi-gigabyte genome import

**Measured, not hypothesised.** Subplan 09 established the FM-index costs 39.4 bytes per base,
giving a practical ceiling near 136 Mbp on a 7 GB machine and 113.8 GB for a human genome.

**Behaviour:** refuse, before allocating. Check the input size, compute the projected requirement,
and return an error naming the actual number.

```
Error: cannot index 3.1 Gbp (GRCh38).
  Projected memory: 113.8 GB at 39.4 bytes/base. Available: 5.9 GB.
  Supported range is up to ~100 Mbp (bacterial genome scale).
  For search over sequences this large, use a dedicated aligner.
```

**Must be true:**
- The check happens before allocation. An OOM kill is not an error message.
- The message states the real number and the real limit.
- It points somewhere useful rather than just refusing.
- Streaming operations that do not need an index (reverse-complement, motif scan by sliding window)
  still work on inputs above the indexing limit. The limit is on indexing, not on the file.

### E4. Network partition mid-operation

The brief's version is between the Go layer and the Python ML service. Subplan 05 deferred both to
Phase 3, so the version that matters in Phase 1 and 2 is **the screening backend becoming
unreachable mid-request**, which is the same failure with much higher stakes.

**Behaviour:** fail closed. A timeout, connection reset, malformed response, or partial response is
a refusal. Never a pass.

**Must be true:**
- Every one of those failure modes is fault-injected in tests, per subplan 24.
- The refusal distinguishes "screening said no" from "screening could not be reached," because they
  mean different things to the user even though both block export.
- No retry loop silently degrades into a pass after N attempts.
- The ML service being unreachable degrades differently and correctly: the deterministic baseline
  still runs, and the result reports that the ML correction was unavailable rather than silently
  omitting it.

That last point is the asymmetry worth noting. **Screening unavailable means stop. ML unavailable
means continue with less.** Confusing the two in either direction is a bug.

### E5. A design with no features

An imported SBOL file with a sequence and no annotations.

**Behaviour:** valid input. Sequence-level operations work. Feature-dependent predictions return a
clear "no RBS annotated" rather than guessing where the RBS is. **Must be true:** we do not silently
infer features and then predict on our own guess, which would attribute our inference error to the
model.

### E6. Conflicting or overlapping feature annotations

Real SBOL files have overlapping features, features on both strands, and occasional contradictions.

**Behaviour:** preserved as-is, not normalised away. Overlap is biologically real. Where a
prediction needs an unambiguous feature and finds several, it reports the ambiguity rather than
choosing one.

### E7. A sequence containing ambiguity codes

`N`, `R`, `Y`, and the rest of IUPAC. Common in real data.

**Behaviour:** accepted at the sequence level. Predictions requiring exact residues return
`Marginal` or refuse, depending on how many ambiguous positions fall in the region that matters.
**Must be true:** ambiguity is never silently resolved to a concrete base.

### E8. Round-tripping a design that came from SBOL 2

**Measured in subplan 23:** SBOL2 to 3 and back is semantically faithful but not syntactically
idempotent. Ontology URIs are canonicalised and `persistentIdentity`/`version` cannot survive,
because SBOL 3 removed them.

**Must be true:**
- Round-trip tests compare semantically. A triple-equality assertion fails on correct behaviour and
  then gets weakened until it passes, which is how a test suite rots.
- The user sees the conversion report rather than silently losing `persistentIdentity`.

---

## What each scenario contributes to the test plan

Direct mapping, so subplan 17 inherits this rather than re-deriving it.

| Item | Test type | Home |
|---|---|---|
| S1 | End-to-end, against the vendored `sbol-rs` corpus | `tests/e2e/import_predict.rs` |
| S2 | End-to-end, construct assembly | `tests/e2e/first_construct.rs` |
| S3 | Integration, via the Python SDK | `python/tests/test_notebook_flow.py` |
| S4 | Integration, backend configuration | `tests/integration/custom_backend.rs` |
| E1 | Screening suite, against the SecureDNA test HDB | `integrase-screen/tests/partial_match.rs` |
| E2 | Model evaluation | `integrase-sim/tests/domain_verdict.rs` |
| E3 | Unit, pre-allocation size check | `integrase-core/tests/size_limits.rs` |
| E4 | **Fault injection**, screening and ML separately | `integrase-screen/tests/fail_closed.rs` |
| E5-E7 | Property and unit | per crate |
| E8 | Contract, semantic comparison | `integrase-sbol/tests/roundtrip.rs` |

---

## Open items

- [ ] Does any scenario here require genome-scale search? **Answer: no.** All four user scenarios
      operate on plasmids and constructs. This closes the question subplan 02 raised and confirms
      the subplan 09 scope decision.
- [ ] Cello UCF import: which scenario motivates it, and does one exist in v1.0? Currently none of
      the four does, which suggests it belongs later than the brief implies.
