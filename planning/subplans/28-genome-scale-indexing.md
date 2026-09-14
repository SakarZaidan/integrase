# 28. Genome-scale indexing

**Status:** decided. Restores a capability the first version of subplan 09 cut on a bad measurement.
**Date:** 2026-09-14
**Decides:** the indexing strategy, its memory tiers, and what is honestly supported
**Depends on:** subplan 09 (corrected)
**Feeds:** subplans 11, 21

---

## The correction

Subplan 09 originally concluded that genome-scale indexing was out of scope. That conclusion came
from benchmarking `rust-bio`'s FM-index, which retains a full `u64` suffix array, measuring 39.4
bytes per base, and projecting 113.8 GB for a human genome.

The number was right. The inference was not. Same machine, same synthetic data, same method:

| Index | 20 Mbp build | 1,000 × 20 bp search | Peak RSS | bytes/base |
|---|---:|---:|---:|---:|
| `rust-bio` `FMIndex` | 2,720 ms | 120 µs | 752 MB | 39.4 |
| `fm-index` `FMIndex` | 3,182 ms | 3,873 µs | **77 MB** | **~3.9** |
| `fm-index` `RLFMIndex` | 3,298 ms | 8,099 µs | 81 MB | ~4.1 |

**Ten times less memory for a search that goes from 0.12 µs to 2.9 µs.** In a tool where a human is
waiting, three microseconds is not a cost. I optimised the wrong axis and then cut a feature because
of it.

Revised projections at ~3.9 bytes/base including the text:

| Organism | Genome | Projected | Previously claimed |
|---|---:|---:|---:|
| *E. coli* K-12 | 4.6 Mbp | 18 MB | 0.2 GB |
| *S. cerevisiae* | 12.2 Mbp | 47 MB | 0.4 GB |
| *C. elegans* | 100 Mbp | 0.4 GB | 3.7 GB |
| *D. melanogaster* | 143.7 Mbp | 0.6 GB | 5.3 GB |
| *A. thaliana* | 135 Mbp | 0.5 GB | 5.0 GB |
| Human GRCh38 | 3.1 Gbp | **~12 GB** | 113.8 GB |

The literature reports production FM-indexes under 3 GB for a human genome using 2-bit text packing
and tuned suffix array sampling, which is a further three-to-fourfold available through engineering
rather than invention.

---

## Strategy: tiers, chosen by measurement not by guess

One index does not serve every case. Three tiers, selected automatically from input size and
available memory, overridable explicitly.

| Tier | Structure | bytes/base | Search | Selected when |
|---|---|---:|---|---|
| **T0: scan** | None. Direct scan, SIMD where available. | 1 (the sequence) | O(n) per query | Under ~100 kbp, or a single query. Building an index to search a plasmid once is strictly slower than scanning it. |
| **T1: fast** | `rust-bio` FM-index, suffix array retained | ~39 | 0.12 µs | Under ~10 Mbp with memory available. Bacterial genomes, repeated queries. |
| **T2: compact** | `fm-index` compressed, sampled suffix array | ~3.9 | 2.9 µs | Above ~10 Mbp, or when memory is constrained. Eukaryotic genomes. |

**T0 is the tier the original analysis missed entirely**, and it covers the overwhelming majority of
real use. Subplan 11's scenarios all operate on plasmids and constructs. A 5 kbp plasmid scanned for
a restriction site needs no index at all, and the first version of this analysis spent its time
arguing about a capability most users never reach.

**Tier selection is a documented, inspectable decision**, not hidden magic. The chosen tier and its
reason appear in the API and the CLI, because a user who cannot tell which index they are using
cannot reason about why a query was slow.

### Explicit and honest memory checking

Per subplan 11's edge case E3, the check happens before allocation:

```
Error: cannot index 3.1 Gbp (GRCh38) at tier T2.
  Projected: 12.1 GB at 3.9 bytes/base. Available: 5.9 GB.

  Options:
    - tier T0 (scan): works at any size, O(n) per query, no index
    - index a subset: --region chr1
    - run on a machine with >=16 GB

  Measured on this hardware; see docs/limits.
```

The refusal names the real projection, the real limit, and a route forward. An out-of-memory kill is
not an error message, and "unsupported" is not an answer when T0 works at any size.

---

## What genome-scale search is actually for

The first analysis asked "does any user need this?" and answered no from the four scenarios in
subplan 11. That was the right question and an incomplete survey, because the scenarios were written
before several capabilities were restored.

| Use case | Needs | Tier |
|---|---|---|
| Restriction site search in a plasmid | Nothing | T0 |
| Off-target search for a guide or primer against a host genome | **Genome-scale** | T2 |
| Checking a designed part does not occur in the chassis genome | **Genome-scale** | T2 |
| Genomic context for a Cello placement rule (subplan 25) | **Genome-scale** | T2 |
| Verifying a generated sequence is not a natural sequence (subplan 26) | **Genome-scale** | T2 |
| Screening-adjacent homology search | Delegated to the backend | n/a |

**Rows 2, 4, and 5 are real and two of them only appeared once features were restored.** Off-target
search against a host genome in particular is an ordinary thing a synthetic biologist does, and
Cello 2.0's own extension beyond plasmids to genomic contexts is precisely about placement within a
genome.

Row 5 is worth noting: "is this generated sequence actually just a natural sequence the model
memorised?" is a question the generative era creates, it needs genome-scale search to answer, and it
did not exist as a use case when the first analysis was done.

---

## Implementation

```
integrase-core/index/
├── mod.rs        Index trait, tier selection, memory estimation
├── scan.rs       T0
├── fast.rs       T1, wraps rust-bio
├── compact.rs    T2, wraps fm-index
└── estimate.rs   projected memory, before allocation
```

```rust
pub trait Index {
    fn search(&self, pattern: &[u8]) -> SearchResults;
    fn tier(&self) -> Tier;
    fn memory_bytes(&self) -> usize;
}

/// Called before construction. The whole point is that this cannot fail late.
pub fn estimate_memory(length: usize, tier: Tier) -> usize;
pub fn select_tier(length: usize, available: usize, hint: Option<Tier>)
    -> Result<Tier, TooLarge>;
```

Both index crates are permissively licensed and adopted: `fm-index` at 26,662 recent downloads,
`sucds` at 327,230 underneath it. Neither is written by us, which is the point.

---

## Testing

| Class | Assertion |
|---|---|
| **Cross-tier agreement** | Property test: T0, T1, and T2 return identical results for the same pattern and text. **The most valuable test here**, because a compressed index disagreeing with a scan is a silent correctness bug. |
| Memory estimation | Projected memory is within 20% of measured, across sizes |
| Pre-allocation refusal | An oversized request errors before allocating, and the message names the projection |
| Tier selection | Deterministic and inspectable given size and available memory |
| Performance gates | Subplan 09's Tier 1 numbers, per tier, memory gated equally with time |
| Reverse complement | Searching both strands agrees, which also matters for screening (subplan 17) |

---

## Open items

- [ ] Re-benchmark both indexes against a real *E. coli* genome. Synthetic DNA compresses
      differently from real sequence, so build times are probably pessimistic and memory probably
      optimistic.
- [ ] Measure a 2-bit packed variant to test the literature's sub-3 GB human-genome figure
- [ ] Decide whether index persistence is needed. Rebuilding a 12 GB index per session is
      unacceptable; serialising it is straightforward and adds a format to version.
- [ ] Confirm `fm-index`'s `locate` costs, since the measured figures above are `count` only and
      locate is the operation that needs the sampled suffix array
