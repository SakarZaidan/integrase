# 0001. Ship the RBS model without structure-dependent terms

**Status:** accepted
**Date:** 2026-09-15
**Amends:** `subplans/16-biophysical-modeling-accuracy.md`, `subplans/08-ml-architecture.md`
**Deciders:** project owner (via `FINAL-DECISION.md` contingency 9)

## Context

The thermodynamic RBS model decomposes translation initiation into five free-energy terms:

```
ΔG_total = ΔG_mRNA:rRNA + ΔG_start + ΔG_spacing − ΔG_standby − ΔG_mRNA
```

Two of those, `ΔG_standby` and `ΔG_mRNA`, require RNA secondary structure prediction. Subplan 16
flagged this as an open dependency and named ViennaRNA as the standard implementation, with its
licensing unverified.

**Verified 2026-09-15, and it fails our dependency bar.**

The ViennaRNA licence reads: "Permission is granted for research, educational, and commercial use
and modification so long as 1) the package and any derived works are not redistributed for any fee,
other than media costs, 2) proper credit is given to the authors and the Institute for Theoretical
Chemistry of the University of Vienna."

It is **not an OSI-approved open source licence**. It restricts redistribution for a fee, and
commercial incorporation requires contacting the authors.

Source: https://viennarna.readthedocs.io/en/latest/license.html

That is incompatible with bundling into an Apache-2.0 project, which permits redistribution for a
fee, and it fails subplan 07's R-3.2 requirement that a dependency in a correctness-critical path be
permissively licensed.

Rust alternatives were checked. `rna-algos` (620 recent downloads) and `rna-ss-params` (684) exist
but sit below R-3.2's adoption bar for a correctness-critical path, and neither is a drop-in for
Turner-parameter minimum-free-energy folding.

## Decision

**Ship the RBS model with the three terms we can compute, and omit the two we cannot.**

`ΔG_mRNA:rRNA`, `ΔG_start`, and `ΔG_spacing` are computed from sequence directly and need no
structure prediction. Those ship.

`ΔG_standby` and `ΔG_mRNA` are omitted by default. **The intervals widen to reflect the missing
physics**, and the model card states plainly which terms are absent.

ViennaRNA may be used as an **optional runtime dependency** if the operator has installed it
themselves. We call it; we never bundle or redistribute it, so no licence obligation attaches. When
present, the full five-term model runs and the model card says so.

## Consequences

**What gets easier.** No licence entanglement, no vendored C library, no build complexity in CI, and
Phase 1a is unblocked today rather than after a licensing negotiation.

**What gets harder.** The default model is less accurate than the published five-term version,
particularly on sequences with strong secondary structure, which is precisely the regime where the
published model's own accuracy is weakest (subplan 16 records R² ranging 0.2 to 0.8 across
datasets).

**What this commits us to.** Reporting the omission honestly in every prediction rather than in a
footnote. A model missing a term is a model with more uncertainty, and this system exists to say so.
`DomainVerdict::Marginal` is returned with a specific reason when a sequence looks likely to be
structure-dominated, because that is exactly when the missing terms matter most.

**Cost to reverse.** Low. The term is additive. Adding structure prediction later is a new
`Process`-shaped contribution to the same sum, not a redesign.

## Alternatives rejected

| Alternative | Why not |
|---|---|
| Bundle ViennaRNA | Licence forbids it for an Apache-2.0 project that permits redistribution for a fee. Fails R-3.2. |
| Depend on `rna-algos` or `rna-ss-params` | Below the adoption bar for a correctness-critical path (620 and 684 recent downloads), and not a drop-in for Turner-parameter MFE folding. Revisit if adoption grows. |
| Implement Zuker MFE folding ourselves | Substantial, well-trodden work with a high bar for correctness. Reimplementing a solved problem badly is worse than omitting a term honestly. Contradicts subplan 23's rule. |
| Delay Phase 1a until this resolves | The contingency existed precisely so this would not block. |
| Ship the full model with a guessed structure term | The worst option. An invented number presented as physics. |

## Verification

The model card for `rbs-thermo-v1` lists the omitted terms explicitly, and a test asserts that the
card's `known_failure_modes` mentions the omission. If someone later adds structure prediction and
forgets to update the card, that test fails.

Honest limit on this verification: it checks that we *said* the terms are missing, not that the
resulting intervals are correctly widened. That is a calibration question and is answered by the
Phase 1a coverage gate, not here.
