# Calibration data

Integrase does not ship the data its models are calibrated against. This page
says why, and how to obtain it.

## Why it is not included

The canonical dataset is **Kosuri et al. 2013**, which measured 12,653
promoter and ribosome binding site combinations in *E. coli*.

> Kosuri S, Goodman DB, Cambray G, Mutalik VK, Gao Y, Arkin AP, Endy D, Church GM.
> *Composability of regulatory sequences controlling transcription and translation
> in Escherichia coli.* PNAS 2013;110(34):14024-14029.
> DOI [10.1073/pnas.1301301110](https://doi.org/10.1073/pnas.1301301110) ·
> [PMC3752251](https://pmc.ncbi.nlm.nih.gov/articles/PMC3752251/)

Two reasons it is not vendored, and the second is the binding one.

**Automated retrieval is blocked.** PMC returns an interstitial page rather than
the file for scripted requests. Working around an access control the host
deliberately placed there is not something this project does.

**The redistribution terms are unresolved.** The article is copyright the
National Academy of Sciences. Free to read is not the same as free to
redistribute. Subplan 07's rule R-3.3 holds that data licences are checked like
code licences, and shipping this inside an Apache-2.0 repository without
establishing the terms would be precisely what that rule exists to prevent.

## What happens without it

Nothing breaks, and nothing pretends.

Models report `DomainVerdict::Uncalibrated` and return **no interval**, because a
model that has not demonstrated coverage has no honest interval to offer. That is
the truthful state rather than a degraded one.

Calibration tests that need the data report that they were **skipped and why**,
rather than passing green and implying calibration was demonstrated.

## How to obtain it

1. Open [PMC3752251](https://pmc.ncbi.nlm.nih.gov/articles/PMC3752251/) in a browser.
2. Under Supplementary Materials, download:
   - **Dataset S1** (`1301301110_sd01.xls`), promoter strength calculations
   - **Dataset S2** (`1301301110_sd02.xls`), RBS strength calculations
   - **Dataset S3** (`1301301110_sd03.xls`, 15.8 MB), all intermediate and final values
3. Convert the sheet you need to TSV or CSV with a header row.
4. Place it at `data/calibration/` (gitignored), or set `INTEGRASE_DATA_DIR`.

### Expected columns

A header row is required, so column order does not matter and a differently
ordered file is read correctly rather than silently misinterpreted.

| Column | Required | Meaning |
|---|---|---|
| `id` | yes | Construct identifier, so a row traces back to the source |
| `rbs` | yes | Ribosome binding site sequence |
| `protein` | yes | Measured protein output. For Kosuri, a GFP/RFP fluorescence ratio. |
| `promoter` | no | Promoter identifier or sequence |
| `rna` | no | Measured RNA level |

**Units are not normalised across datasets, deliberately.** A calibration fitted
on one dataset's units does not transfer to another's, and rescaling to make them
look comparable would hide exactly the distribution shift that matters.

## The second dataset, and why it matters

One dataset is not enough, and this is the honest limit on any calibration claim
Integrase can currently make.

A held-out split of a single library measures **interpolation**: can the model
predict a sequence drawn from the same design space as its calibration data? The
question a user actually has is different. Will the interval hold for *their*
sequence, from *their* library, in *their* conditions?

Calibration measured on one library can look excellent and mean very little,
because the calibration set shares the training set's biases.

Until a second, independent dataset is in place:

- Calibration is **in-distribution only**
- The `ModelCard` must say so
- Sequences unlike the calibration set return
  `DomainVerdict::OutOfDistribution`, and their stated coverage does not hold

That is a real, useful, limited product. Claiming general calibration from a
single library would not be.

Tracked in `planning/REQUESTS.md` as R-005.

## Splitting

Use `split_by_component`, not a random split.

Combinatorial libraries reuse the same RBS across many constructs. A random split
puts near-duplicates on both sides, so the model is evaluated on sequences it has
effectively already seen, and every metric is inflated.

Component-wise splitting evaluates on an RBS the model has never seen. It
produces worse numbers, and they are the true ones.
