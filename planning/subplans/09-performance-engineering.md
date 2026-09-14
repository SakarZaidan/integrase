# 09. Performance engineering

**Status:** decided, with one measured finding that changes scope
**Date:** 2026-09-14
**Decides:** numeric performance targets, benchmarking methodology, regression detection
**Depends on:** subplans 05, 06, 07
**Feeds:** subplans 11, 18, 21, and `FINAL-DECISION.md`

The brief asks for specific numbers rather than "fast." Every number below is either measured on
this machine or explicitly marked as unverified, because a target nobody can measure is a target
nobody will meet.

---

## The measurement that changes scope

Open question from the research log: is `bio`'s FM-index fast enough at genome scale? Answered by
benchmark rather than opinion, on this machine (20 cores, 7 GB RAM), release build, deterministic
synthetic DNA.

| Text size | Suffix array build | FM-index build | 1,000 × 20 bp search | Peak RSS |
|---:|---:|---:|---:|---:|
| 1 Mbp | 61 ms | 25 ms | 113 µs | 54 MB |
| 5 Mbp | 408 ms | 86 ms | 134 µs | 189 MB |
| 20 Mbp | 2,297 ms | 423 ms | 120 µs | 752 MB |

**Search performance is excellent and it is the easy half.** Roughly 120 ns per 20 bp query, flat
across a twentyfold increase in index size, which is the behaviour an FM-index is supposed to have.
Nothing about query latency is a problem.

**Memory is the finding.** The index costs about **39.4 bytes per base** asymptotically. Projected:

| Organism | Genome | Projected RSS |
|---|---:|---:|
| *E. coli* K-12 | 4.6 Mbp | 0.2 GB |
| *S. cerevisiae* | 12.2 Mbp | 0.4 GB |
| *D. melanogaster* | 143.7 Mbp | 5.3 GB |
| Human GRCh38 | 3.1 Gbp | **113.8 GB** |

**Practical ceiling on this machine: roughly 136 Mbp.**

### What this means

The brief specifies "fast motif and restriction-site search at genome scale" and a
"multi-gigabyte genome import" scenario. **Neither is achievable with this approach, on this
hardware, and the human-genome case is not achievable on most hardware.** 113.8 GB of RAM to index
one genome is a cluster-scale requirement, not a laptop feature.

Three honest responses, and we take the first:

1. **Scope it to what the users actually work on.** Plasmids and constructs are kilobases. Bacterial
   genomes are megabases. Both are comfortably inside the ceiling, with room to spare. This
   confirms subplan 02's assessment that genome-scale search was a capability without a
   demonstrated user, and now there is a number behind that judgment instead of an opinion.
2. Use a memory-efficient index (compressed suffix array, sampled suffix array, FM-index over a
   2-bit packed alphabet). Real engineering, weeks of work, for a use case nobody has asked for.
3. Claim it and hope nobody tries. Not an option.

**Decision: the documented supported range is up to bacterial genome scale (targeting 100 Mbp), and
the documentation says so with this number in it.** Anything larger returns a clear error naming
the limit, not an out-of-memory kill. Subplan 11 owns the edge case where a user imports something
too large, and the correct behaviour is a useful refusal.

This is a scope reduction driven by measurement, and it is the kind the brief asks for: a claim
that could not be verified got checked, and it did not survive.

---

## Targets

### Tier 1: measured on this machine, gated in CI

| Operation | Target | Measured | Headroom |
|---|---|---|---|
| FM-index build, 5 Mbp | < 1 s | 494 ms | 2.0× |
| FM-index build, 20 Mbp | < 4 s | 2,720 ms | 1.5× |
| Motif search, 20 bp pattern | < 1 µs | ~0.12 µs | 8× |
| Index memory | < 45 bytes/base | 39.4 | 1.14× |
| SBOL round-trip, typical design | < 10 ms | not yet measured | |
| Reverse-complement, 1 Mbp | < 5 ms | not yet measured | |

Targets are set with headroom above measurement so the gate catches regressions rather than noise.
The memory target is deliberately tight, at 14% headroom, because memory is the binding constraint
and a regression there reduces the supported genome size directly.

### Tier 2: budgets, not yet measured

Stated so they are designed for, marked so nobody mistakes them for results.

| Operation | Budget | Rationale |
|---|---|---|
| Single RBS prediction, deterministic | < 50 ms | Interactive. A user edits a sequence and expects the number to move. |
| gRPC round-trip, loopback | < 5 ms | Subplan 05's argument for gRPC over cgo assumes this. If it fails, revisit that decision. |
| Python SDK call overhead over native Rust | < 1 ms | PyO3 boundary should not dominate. |
| Screening a 10 kbp design | < 2 s | Depends entirely on the backend. Unverifiable until subplan 24 resolves. |
| Web UI first render | < 2 s | Below this, a tool feels responsive. |

### Explicitly not targeted

| Thing | Why not |
|---|---|
| Mammalian-genome indexing | 113.8 GB. Out of scope, stated with the number. |
| ML inference latency | Depends on model and hardware. A budget here would be fiction until subplan 08's model exists. |
| Multi-user throughput | No service until Phase 3. |

---

## Methodology

**`criterion` for Rust microbenchmarks.** Statistical, reports confidence intervals, detects
regressions. Using a benchmark harness that reports uncertainty when the whole project is about
reporting uncertainty is the obvious choice.

**Measurement rules**, so numbers are comparable across runs:

1. Release builds with the repository's release profile. A debug-build benchmark is noise.
2. Deterministic inputs. The benchmark above uses a seeded xorshift so reruns compare like with
   like.
3. Memory measured as peak RSS from `/proc/self/status`, reported alongside time. A speedup bought
   with memory is not a speedup on a 7 GB machine, and time-only benchmarks hide that trade.
4. Every reported number names the hardware. The CI runner and this laptop are different machines
   and their numbers are not interchangeable.

**Where benchmarks live:** `crates/*/benches/`, and `crates/integrase-bench/` for cross-crate
scenarios.

---

## Regression detection

**Gated in CI on Tier 1 numbers only.** Tier 2 budgets are not gated until they are measured,
because gating an unmeasured budget produces a flaky check that people learn to re-run until it
passes.

**Threshold: 10% slower than the recorded baseline fails the build.** Below 10%, CI runner variance
dominates and the signal is not real. Chosen to be loose enough that the gate is trusted; a gate
people routinely override is worse than no gate.

**On a failure, two legitimate outcomes:** fix the regression, or change the threshold in the same
PR with a written justification. Silently re-running until green is neither, and the `pr-review`
skill treats a threshold change without justification as a request-changes.

**Baselines are committed**, so a regression is visible in the diff rather than living in CI state
that nobody can inspect.

**Memory regressions are gated equally.** Given that memory sets the supported genome size, a
memory regression is a scope regression.

---

## The profiling posture

Per R-5.4, complexity added for performance requires a before-and-after measurement in the PR.

**The first optimisation is the algorithm, the second is the allocation, and the third is
everything else.** The 39.4 bytes per base result is an algorithmic property of using a full
suffix array, and no amount of micro-optimisation changes it. If the supported size ever needs to
grow, the answer is a different index structure, not faster code.

Tools: `cargo flamegraph` for CPU, `heaptrack` or `dhat` for allocation, `criterion` for
regressions. `perf` is available.

---

## Honest limitations of this whole section

- **Every Tier 1 number comes from one machine**, a WSL2 laptop with 7 GB of RAM and 20 cores. CI
  runners will differ, possibly substantially. The numbers are real but they are not universal, and
  CI baselines have to be recorded on the CI hardware rather than copied from here.
- **Synthetic DNA compresses differently from real genomes.** Real sequence has repeats and biased
  composition, which affects both suffix array construction time and index size. The build times
  above are probably pessimistic and the memory probably optimistic. Benchmarking against a real
  *E. coli* genome is an open item.
- **The Tier 2 budgets are guesses informed by what feels responsive**, not requirements derived
  from user research. They are marked as budgets for exactly that reason.

---

## Open items

- [ ] Re-run the FM-index benchmark against a real *E. coli* genome, not synthetic DNA
- [ ] Measure SBOL round-trip and reverse-complement to move them into Tier 1
- [ ] Record CI-hardware baselines separately from local ones
- [ ] Verify the < 5 ms loopback gRPC assumption before Phase 3, since subplan 05's boundary
      decision rests on it
