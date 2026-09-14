# 04. Resources and data

**Status:** complete, with two blocking gaps identified
**Date:** 2026-09-14
**Decides:** what external data and standards the project depends on, how each is accessed, and
under what terms
**Feeds:** subplans 08, 16, 17, 23, 24, and `REQUESTS.md`

Every entry states what it is, how it is accessed, its license, and whether it is sufficient as-is.
The two entries marked BLOCKING are the ones that decide whether the project's differentiators are
real, so they are listed first rather than buried in alphabetical order.

---

## BLOCKING: the two gaps that matter

### B-1. Curated sequence-of-concern data for biosafety screening

| | |
|---|---|
| **What** | The curated reference set of regulated pathogen and toxin sequences that screening compares designs against. |
| **Access** | Not a public download, by design. Publishing a precise list of screened sequences is itself a biosecurity problem. |
| **License** | n/a |
| **Sufficient?** | **No. This is `REQUESTS.md` R-002.** |

Two credible open-source screening implementations exist and both are candidate routes through
this (subplan 24 evaluates them properly):

- **SecureDNA** (`SecureDNA/SecureDNA`, 21 stars, pushed 2026-07-16, **Rust**, Apache-2.0). A full
  monorepo with crates, a frontend, and certificate infrastructure. Uses a privacy-preserving query
  protocol, which appears to exist specifically so that screening can happen without the client
  holding the database.
- **The Common Mechanism** (`ibbis-bio/common-mechanism`, 47 stars, pushed 2026-09-03, IBBIS).
  Describes itself as a free, open-source, globally available DNA sequence screening tool.

**Position until resolved:** the gateway is built to fail closed. No reference data configured
means no export, stated plainly in the interface rather than silently permitting everything.
Shipping a toy dataset and implying it constitutes real screening is the one outcome that is not
acceptable.

### B-2. Held-out experimental data for calibration validation

| | |
|---|---|
| **What** | Sequences with measured expression values, held out from training, ideally across labs and conditions, to demonstrate that stated confidence intervals actually contain the true value at the stated rate. |
| **Access** | Assembled from published datasets. Requires real curation work, not a download. |
| **License** | Varies per publication. |
| **Sufficient?** | **Partially. The best single candidate is identified below, but one dataset is not enough to demonstrate calibration under distribution shift.** |

This is the quieter of the two blocking gaps and arguably the more dangerous, because it is the
one that can be faked without anyone noticing immediately. Calibrated uncertainty is the project's
central differentiator (subplan 02). Claiming calibration without demonstrating it would be a
worse failure than the silence the existing tools offer, because it converts honest ignorance into
a false assurance.

**Best identified source: Kosuri et al. 2013 (PNAS).** All combinations of 114 promoters and 111
RBSs, 12,653 constructs, measured in E. coli by massively parallel reporter assay. This is the
canonical combinatorial promoter/RBS expression dataset and it is widely reused, which also means
its quirks are documented.
Source: https://www.pnas.org/doi/10.1073/pnas.1301301110

**Why one dataset is not enough.** Calibration measured on a held-out split of a single MPRA
library tests interpolation, not the thing users actually need, which is whether the interval
holds on a sequence from a different library, a different lab, or a different growth condition.
Subplan 08 must plan for at least a second independent dataset for out-of-distribution evaluation,
and must state plainly in the model card if it only ever achieves in-distribution calibration.

Related work worth reading before designing the evaluation: transfer learning for cross-context
prediction of protein expression from 5'UTR sequence, which addresses exactly this generalization
problem. https://academic.oup.com/nar/article/52/13/e58/7691520

---

## Standards

### SBOL 3.1.0 and SBOL 2.3.0

| | |
|---|---|
| **What** | The data model for exchanging genetic designs. SBOL 3 is RDF/graph-based. |
| **Access** | Specifications public at https://sbolstandard.org/ |
| **License** | Open standard |
| **Sufficient?** | Yes, and better than expected. See subplan 23. |

We do not implement SBOL. `sbol-rs` does, it covers both versions, and it ships validators for
109 machine-checkable rules of SBOL 3.1.0 Appendix B and 239 of the SBOL 2.3.0 catalog. Verified
hands-on, see subplan 23.

### SBOL Visual 3.0

| | |
|---|---|
| **What** | The separate standard for diagram glyphs and conventions. Version 3.0 retargets glyph semantics onto the SBOL 3 data model. |
| **Access** | https://sbolstandard.org/visual/ , spec at https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8560346/ |
| **License** | Open standard |
| **Sufficient?** | Yes. The SBOL Visual 2 Ontology is the more useful artifact for implementation. |

The ontology gives machine-readable glyph constraints, so glyph selection becomes an ontology
lookup rather than a hand-maintained switch statement that drifts from the spec.
https://www.biorxiv.org/content/10.1101/2020.01.24.918417.full.pdf

Existing renderers worth reading for glyph geometry before drawing our own: `dnaplotlib`
(VoigtLab, 304 stars, Python, last push 2024-03-20) and `paraSBOLv` (17 stars, dormant since
2022-08-09).

### SBML, SED-ML, and COMBINE/OMEX

| | |
|---|---|
| **What** | SBML for model exchange, SED-ML for simulation experiment description, OMEX/COMBINE archives for bundling both. |
| **Access** | https://co.mbine.org/standards/ |
| **License** | Open standards |
| **Sufficient?** | Yes, and relevant mainly as an export target. |

These matter because iBioSim (67 stars, still maintained) consumes them. Exporting SBML plus
SED-ML is how Integrase interoperates with the one serious simulation tool in the ecosystem
instead of competing with it, which is the connective-tissue strategy from subplan 02.

---

## Data repositories

### SynBioHub

| | |
|---|---|
| **What** | The field's design repository. REST API plus a SPARQL endpoint. |
| **Access** | **Authentication required, verified 2026-09-14.** |
| **License** | Varies by collection |
| **Sufficient?** | **No, not for anonymous or CI use. Needs an account.** |

This is a concrete finding that changes the testing plan and it was not anticipated by the brief.
Probed six endpoints on the public instance, all returned `401 Login required`:

```
/public/igem/BBa_B0034/1/sbol       401 Login required
/public/igem/BBa_B0034/1/sbolnr     401 Login required
/public/igem/BBa_B0034/1/fasta      401 Login required
/public/igem/BBa_B0034/1/gb         401 Login required
/rootCollections                    401 Login required
```

`/rootCollections` is documented as a public endpoint, so this may be a configuration state of the
public instance rather than a permanent policy. Either way, planning must assume authentication is
required.

**Consequences.** Subplan 17's contract tests cannot hit SynBioHub anonymously in CI. Storing
credentials in CI to fetch test fixtures is exactly the kind of thing subplan 10's threat model
should object to. The better answer is to vendor a licensed test corpus (the `sbol-rs` fixtures are
a strong starting point, see below) and treat live SynBioHub access as an optional, manually-run
integration check rather than a required CI gate.

A SynBioHub account is worth having regardless, for development. Registering an account on an
external service on the owner's behalf is their call, not mine, so this goes to `REQUESTS.md`
rather than being done unilaterally.

### iGEM Registry of Standard Biological Parts

| | |
|---|---|
| **What** | 84,000+ standard biological parts. The field's canonical parts catalogue. |
| **Access** | XML API. https://parts.igem.org/Registry_API , https://registry.igem.org/ |
| **License** | **Creative Commons Attribution-ShareAlike** |
| **Sufficient?** | Yes, with a licensing constraint that must be respected. |

**The ShareAlike term is the thing to get right.** Our code is Apache-2.0. Registry data is
CC BY-SA. Those coexist fine as long as they stay separate: we may query, display, and process
registry data, but any redistributed derivative of that *data* inherits CC BY-SA and requires
attribution. Concretely, this means registry-derived part data must not be silently baked into an
Apache-2.0 crate as if it were our own. Subplan 07's dependency-vetting rules should cover data
licensing as well as code licensing, which is a distinction most dependency policies miss.

### BioModels

| | |
|---|---|
| **What** | 1,000+ manually curated mechanistic models, mostly SBML, with 450+ SED-ML files. |
| **Access** | https://www.ebi.ac.uk/biomodels/ , REST API |
| **License** | **CC0 Public Domain Dedication** |
| **Sufficient?** | Yes. The cleanest licensing of any data source here. |

CC0 means no attribution obligation and no share-alike contamination, so BioModels is the safest
source for bundled test fixtures involving SBML. Useful for validating the SBML export path
against real curated models rather than hand-written toys.

### sbol-rs test fixtures

| | |
|---|---|
| **What** | 343 SBOL data files across cross-implementation, SBOL2, FASTA, and GenBank fixture sets. |
| **Access** | https://github.com/SynBioDex/sbol-rs , `tests/fixtures/` |
| **License** | MIT OR Apache-2.0 |
| **Sufficient?** | Yes, and this is the answer to the SynBioHub CI problem. |

Breakdown by serialization: 100 N-Triples, 80 RDF/XML, 76 Turtle, 66 JSON-LD, 21 XML. Fixture
categories include `cross-impl-pysbol3`, `cross-impl-sbol2`, `cross-impl-sbolconverter`, `fasta`,
`genbank`, and `sbol2`. The `cross-impl-pysbol3` set is particularly valuable: it is expected
output generated by pySBOL3, which makes it a cross-implementation oracle, and the licensing is
compatible with ours.

---

## Models and weights

### Evo 2

| | |
|---|---|
| **What** | Genomic foundation model. 40B parameters, 1 Mb context, ~9.3T nucleotides from 128,000+ genomes. Open weights. |
| **Access** | https://arcinstitute.org/tools/evo , `ArcInstitute/evo2` (4,213 stars) |
| **License** | Open weights, exact terms to be verified before any dependency |
| **Sufficient?** | **Not locally. Hardware does not permit it.** |

This machine has 12 GB of VRAM. The 40B model does not fit, and no amount of quantization makes it
fit comfortably. Options are the smaller variants, or a remote API, and subplan 08 must state which
rather than planning around capability that is not available here.

Worth noting for subplan 24: Evo 2's training data was safety-filtered to exclude viral sequences
infecting eukaryotic hosts. That is useful prior art for the position that safety controls belong
inside a system's design rather than in its terms of service.

**Open item:** the exact license terms on the weights have not been verified. Anything that
depends on them needs that checked first, per subplan 07's vetting rules.

---

## Published models we implement rather than consume

These are not datasets. They are published methods that the deterministic baseline implements, and
they are listed here because their accuracy characteristics are a project dependency in the same
way a dataset is.

| Method | Reported accuracy | Notes |
|---|---|---|
| RBS Calculator (thermodynamic, Salis) | R² from >0.8 down to <0.2 | The *range* is the finding. Error is strongly dataset-dependent. https://pubmed.ncbi.nlm.nih.gov/21601672/ |
| SAPIENS / "RBS Predictor" deep model | R² = 0.93 claimed | **Vendor self-report** (De Novo DNA), ~250k E. coli 5'UTRs, not an independent benchmark. Do not cite as state of the art without that caveat. |
| Transformer de novo RBS design (P. denitrificans) | Pearson r = 0.80 | Wet-lab validated, but on selected designs, which is a biased subset. https://pmc.ncbi.nlm.nih.gov/articles/PMC13495406/ |
| Sigma70Pred | 81.5% to 90.4% accuracy | **Classification, not strength regression.** Different task. Do not conflate. https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9701712/ |

Subplan 16 owns the honest write-up of what these models can and cannot do. The point of listing
them here is that the R² > 0.8 to R² < 0.2 spread on the deterministic baseline is precisely why
the confidence layer exists.

---

## Summary table

| Resource | License | Access | Sufficient |
|---|---|---|---|
| SBOL 3.1.0 / 2.3.0 spec | Open standard | Public | Yes |
| SBOL Visual 3.0 + ontology | Open standard | Public | Yes |
| SBML / SED-ML / OMEX | Open standard | Public | Yes |
| `sbol-rs` + fixtures | MIT OR Apache-2.0 | crates.io, GitHub | Yes |
| BioModels | CC0 | REST API | Yes |
| iGEM Registry | CC BY-SA | XML API | Yes, with ShareAlike care |
| SynBioHub | Varies | **Auth required** | No, needs an account |
| Evo 2 weights | To verify | Public | Not on this hardware |
| Calibration data (Kosuri 2013) | Per publication | Public | Partially, needs a second source |
| **Screening reference data** | n/a | **Not public** | **No. R-002.** |

---

## New `REQUESTS.md` entries arising from this subplan

- **R-003: SynBioHub account.** Free to register, but registering an account on an external
  service under the owner's name is their decision. Needed for development and for optional
  integration checks, not for CI.

## Open items for downstream subplans

- [ ] Verify the exact license terms on Evo 2 weights before depending on them -> **08**
- [ ] Identify a second, independent expression dataset for out-of-distribution calibration
      testing -> **08**
- [ ] Decide the vendored test corpus and its provenance documentation -> **17**
- [ ] Extend dependency vetting rules to cover data licensing, not just code -> **07**
