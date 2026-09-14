# Research log

Working notes. Messy by design, per the project's own rules: this file is a scratchpad, not a
deliverable. Everything else in `planning/` is held to a higher bar. What this file owes you is
sources and honesty about what was actually verified versus what came from a search summary.

**Started:** 2026-09-14
**Status:** open, appended to as research continues

## How to read this

Each entry states the finding, the source, and the line that matters: **what this changes**.
A finding with no "what this changes" line did not earn its place and should be deleted.

Confidence markers used throughout:
- `[verified]` I fetched the primary source or queried the API directly and read the result.
- `[secondary]` Came from a search-result summary. Reliable enough to plan against, not to quote.
- `[unverified]` Stated in the project brief or assumed. Needs checking before anything depends on it.

---

## 1. SBOL and the standards layer

### 1.1 SBOL 3.1.0 is current; SBOL Visual 3.0 exists separately

SBOL 3 is a graph-based (RDF) data model, a real departure from SBOL 2's structure. Version
3.1.0 is the current data-model spec. SBOL Visual is a *separate* standard covering diagram
glyphs and conventions, currently at 3.0, and the 3.0 revision exists mainly to retarget the
glyph semantics onto the SBOL 3 data model rather than SBOL 2. `[secondary]`

Sources:
- https://sbolstandard.org/ , https://sbolstandard.org/visual/
- SBOL 3 paper: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7516281/
- SBOL 3.1.0: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10063177/
- SBOL Visual 3.0: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8560346/
- SBOL Visual 2 Ontology (machine-readable glyph constraints): https://www.biorxiv.org/content/10.1101/2020.01.24.918417.full.pdf

**What this changes:** "SBOL support" is two separate implementation tracks, not one. The data
model (import/export/round-trip) and the visual language (glyph rendering, layout conventions)
have different specs, different failure modes, and different test strategies. The project brief
treats them as one line item. Subplans 06 and 13 need to split them. The SBOL Visual *ontology*
is the useful artifact for the renderer: glyph selection becomes an ontology lookup instead of a
hand-maintained switch statement.

### 1.2 `SynBioDex/sbol-rs` exists. This is the single biggest finding of the research pass.

The official SBOL standards organization (SynBioDex) published a Rust implementation. `[verified]`

| Fact | Value |
|---|---|
| Repo | https://github.com/SynBioDex/sbol-rs |
| Created | 2026-05-09 |
| Last push | 2026-09-08 (six days before this log entry) |
| Version | v1.0.0 |
| Stars | 5 |
| License | Apache-2.0 (repo carries both LICENSE-APACHE and LICENSE-MIT) |
| Coverage | SBOL 3.1.0 and SBOL 2.3.0, bidirectional conversion |
| Crates | `sbol`, `sbol3`, `sbol-convert`, `sbol-cli`, `sbol-utilities` |
| Repo contents | `crates/`, `tests/`, `benches/`, `fuzz/`, `spec/`, `docs/`, `CITATION.cff` |

Verified by `gh api repos/SynBioDex/sbol-rs` and by querying the crates.io API for each crate.
The presence of `fuzz/` and `benches/` alongside a `spec/` directory is a real engineering
posture. Whoever wrote this was not producing a demo.

**What this changes:** A lot.

1. The project brief assumes SBOL parsing in Rust must be built from scratch. That assumption is
   now wrong, and the brief's framing of "native SBOL2/SBOL3 import and export from day one" goes
   from a large subsystem to a dependency plus a conformance test suite.
2. It creates a build-versus-adopt decision the brief could not anticipate. A v1.0.0 that is four
   months old with 5 stars is simultaneously the obvious right dependency (it is the standards
   body's own implementation) and a genuine supply-chain question (one maintainer group, no
   adoption track record, no evidence yet of how it handles breaking spec revisions).
3. It weakens one of the project's claimed differentiators. "No modern Rust tooling exists for
   SBOL" was going to be part of the pitch. It is no longer true.
4. It suggests a relationship worth having. SynBioDex is the standards body whose goodwill the
   project's entire positioning depends on. Being a serious downstream user of, and contributor
   to, their Rust library is a much better opening move than shipping a competing implementation.

This is significant enough that I am adding a dedicated subplan (23) for upstream ecosystem
strategy rather than burying the decision inside the stack subplan.

**Still to check:** actual API ergonomics of `sbol-rs`, its round-trip fidelity on real-world
SynBioHub files, whether it handles SBOL 2 -> 3 conversion losslessly, and its MSRV. None of
these are answerable from metadata. This needs reading the code and running it against real
files, which happens in subplan 23.

### 1.3 Python SBOL tooling is the live path today

`[verified]` via PyPI and GitHub API:

| Package | Version | Repo activity |
|---|---|---|
| `sbol3` | 1.2.0.post0 | SynBioDex/pySBOL3, 48 stars, pushed 2026-09-09 |
| `sbol2` | 1.4.1 | maintained |
| `sbol-utilities` | 1.0a17 | SynBioDex/SBOL-utilities, 22 stars, pushed 2025-10-26 |
| `biopython` | 1.88 | n/a |

pySBOL3 at 48 stars and a push six days ago is the most actively maintained SBOL library in the
ecosystem, and it is Python.

**What this changes:** The Python SDK is not a second-class afterthought bolted onto a Rust core.
It is the surface where this project's users already live. Subplan 15 should treat the Python SDK
as a primary API consumer with equal standing to the desktop app, and subplan 17 should use
pySBOL3 as the cross-implementation oracle for round-trip conformance testing: if our Rust path
and pySBOL3 disagree about a file, that is a bug worth investigating regardless of which one is
wrong.

### 1.4 SynBioHub API surface

RESTful HTTP API with search, fetch, and submit endpoints. Token-based auth returned on login and
passed in the `X-Authorization` header. SPARQL endpoint exposed for federated queries against the
underlying RDF store. Also offers Java and Python client APIs. Plugin system where submit-plugins
transform uploaded files into SBOL before ingestion. `[secondary]`

Sources:
- https://synbiohub.github.io/api-docs
- https://wiki.synbiohub.org/apidocumentation/
- SBOL Stack paper: https://pubs.acs.org/doi/10.1021/acssynbio.5b00210

**What this changes:** The SPARQL endpoint is more interesting than the REST API for our purposes.
It means "find me every part in the registry matching these constraints" is a query we can push
upstream rather than a bulk download we have to index locally. Relevant to subplan 09, because it
changes the performance story for registry search from "index 100k parts locally" to "federate the
query," and to subplan 11, because the network-partition edge case now has a second failure mode.

**Still to check:** rate limits, whether the public synbiohub.org instance is stable enough to
depend on in CI (contract tests in section 8 of the brief call for testing against real SynBioHub
files, which either means vendoring a corpus or hitting the network in CI, and hitting the network
in CI is usually a mistake).

---

## 2. Competitive landscape: the field's tooling is mostly dormant

This is the most sobering data in the research pass. All figures `[verified]` via GitHub API on
2026-09-14.

| Tool | Stars | Last push | Language | Read |
|---|---|---|---|---|
| CIDARLAB/cello | 875 | 2022-09-19 | Java | Dormant 4 years. Field's flagship. |
| CIDARLAB/Cello-v2 | 74 | 2022-05-16 | Java | Dormant 4 years. |
| SynBioHub/synbiohub | 84 | 2026-09-06 | JavaScript | Active. |
| MyersResearchGroup/iBioSim | 67 | 2026-04-07 | Java | Moderately active. |
| SynBioDex/pySBOL3 | 48 | 2026-09-09 | Python | Active. |
| SynBioDex/SBOLDesigner | 31 | 2021-06-04 | Java | Dormant 5 years. |
| SynBioDex/SBOL-utilities | 22 | 2025-10-26 | JavaScript | Slow. |
| SynBioDex/SBOLCanvas | 17 | 2026-09-10 | TypeScript | Active. |
| SynBioDex/libSBOLj3 | 2 | 2026-04-16 | Java | Low attention. |
| SynBioDex/sbol-rs | 5 | 2026-09-08 | Rust | New, active. |
| VoigtLab/dnaplotlib | 304 | 2024-03-20 | Python | Slowing. |
| BiocomputeLab/paraSBOLv | 17 | 2022-08-09 | Python | Dormant. |

For contrast, from the AI-for-biology side of the same field:

| Tool | Stars | Last push |
|---|---|---|
| ArcInstitute/evo2 | 4,213 | 2026-06-19 |

**What this changes:** Two things, pulling in opposite directions, and subplan 03 has to hold
both honestly.

The opportunity is real. Cello, the tool the brief names as the field's logic compiler, has not
been touched in four years. SBOLDesigner has been dead for five. The brief's claim that the
ecosystem is "fragmented, largely from the 2010s, with no unified engineering-grade platform" is
not marketing language. It is accurate, and the GitHub activity data supports it more strongly
than the brief itself does.

The problem is also real, and the brief does not mention it. **The entire addressable community
is small.** The most-starred genetic CAD tool in the field has 875 stars and is abandoned. The
most-starred *maintained* one has 84. A project whose stated goal is "become the reference
implementation in its field" is aiming at a field where the incumbent reference implementations
have double-digit and low-triple-digit star counts. Meanwhile Evo 2 has 4,213 stars, which says
the attention and funding in computational biology has moved decisively toward foundation models
and away from CAD tooling.

That asymmetry is the single most important input to subplan 02 (is the differentiation real?)
and subplan 03 (is the adoption path realistic?). It does not kill the project. It does mean any
plan that assumes a large, eager user base waiting for better tooling is planning against a
fantasy. Subplan 03 must score adoption honestly on this evidence, and subplan 02 should consider
whether the project's wedge is better aimed at the active, well-funded ML-for-biology audience
than at the dormant CAD audience.

**Newer entrants found:**
- GCAD, a computational framework for mammalian genetic program CAD, ACS Synthetic Biology, March
  2026. https://pubs.acs.org/asbcd6/article/15/3/1033/5091519/GCAD-A-Computational-Framework-for-Mammalian
  `[secondary]` Mammalian focus, so adjacent rather than directly competing with an E. coli-first
  system, but it shows the CAD space is not entirely static.
- GP-write CAD platform (Genome Project-write with Twist Bioscience), notable because it offers
  **direct ordering of synthetic DNA from industrial partners**. `[secondary]` That is precisely
  the design-to-synthesis boundary where our biosafety gateway sits, and it means the "screen at
  the export boundary" pattern has a working commercial precedent to study.
- No successor to Cello found. Community forks exist (`ginomcfino/CELLO-3.0`, 2 stars, 2023) but
  nothing with momentum. `[verified]` via GitHub search.

---

## 3. Biosecurity screening: current practice and a live deadline

### 3.1 The regulatory picture

The 2024 OSTP Framework for Nucleic Acid Synthesis Screening (published 2024-04-29) conditions US
federal life-sciences research funding on procuring synthetic nucleic acids only from providers
that comply with the framework. The May 5, 2025 Executive Order "Improving the Safety and Security
of Biological Research" directs agencies to revise or replace the 2024 Framework. `[secondary]`

Dates that matter:
- Requirement effective for purchases on or after **2025-04-26**.
- Flagging threshold drops to **50 bp by October 2026**.
- Sequence-of-concern definition **expands in October 2026** to include sequences known to
  contribute to pathogenicity or toxicity.
- IGSC members must conform to the shorter threshold by **2026-10-24**.
- An OSTP-designated interagency group assesses the state of the art before **2026-10-13**.

Important nuance, and the kind of thing that is easy to get wrong: **no government currently
mandates nucleic acid synthesis screening outright.** The mechanism is procurement conditionality
plus voluntary industry compliance through the IGSC, whose members represent a majority of the
global market. `[secondary]`

Sources:
- https://aspr.hhs.gov/S3/Pages/OSTP-Framework-for-Nucleic-Acid-Synthesis-Screening.aspx
- https://aspr.hhs.gov/S3/Documents/OSTP-Nucleic-Acid-Synthesis-Screening-Framework-508.pdf
- https://genesynthesisscreening.centerforhealthsecurity.org/key-policies
- IGSC Harmonized Screening Protocol v3.0: https://genesynthesisconsortium.org/wp-content/uploads/IGSC-Harmonized-Screening-Protocol-v3.0-1.pdf
- IBBIS international screening standards: https://ibbis.bio/our-work/international-screening-standards/

**What this changes:** The biosafety subsystem has a concrete, dated compliance target roughly six
weeks out from the start of this project rather than a vague ethical aspiration. That is much
better for design: "flag at 50 bp, use the post-October-2026 SOC definition" is a testable
requirement. It also means the screening logic must be built assuming the rules change on a known
cadence, so the reference data and the threshold policy have to be versioned and hot-swappable,
not compiled in. That is a real architectural constraint and it belongs in subplan 06 and 24.

The "no mandate, procurement conditionality" nuance matters for honest documentation. The project
must not claim it makes anyone legally compliant. It can claim it implements screening consistent
with the framework's stated approach, which is a different and defensible claim.

### 3.2 Screening tooling that already exists, and one of it is Rust

`[verified]` via GitHub API:

| Tool | Stars | Last push | Language | License |
|---|---|---|---|---|
| SecureDNA/SecureDNA | 21 | 2026-07-16 | **Rust** | Apache-2.0 |
| ibbis-bio/common-mechanism | 47 | 2026-09-03 | (Python) | open source |

SecureDNA is a Rust monorepo: `crates/`, `bin/`, `frontend/`, `certs/`, `docs/`, built with
Earthly, roughly 28 MB of repo. There is also an independent security analysis of it:
https://arxiv.org/pdf/2512.09233 `[secondary]`

The Common Mechanism (IBBIS) describes itself as "a free, open-source, globally available tool for
DNA sequence screening."

**What this changes:** This is the second-biggest finding after `sbol-rs`, and it points the same
direction: do not build what the field has already built well.

Building our own screening algorithm from scratch would be the single most dangerous piece of
not-invented-here in this entire project. A false negative in home-grown screening code is the
worst failure this system can produce. Two credible, open-source, actively maintained screening
implementations exist, one of them in the same language as our core and under the same license we
have chosen.

Subplan 24 should seriously evaluate integrating or delegating to these rather than implementing
screening logic ourselves, and subplan 10's threat model should treat "we wrote our own screening
matcher and it has a gap" as a named risk with this integration as the mitigation.

**Still to check:** whether SecureDNA can be run self-hosted or requires their hosted service,
what its query protocol looks like, and whether the Common Mechanism ships a usable reference
dataset or expects you to bring your own. The second question is the crux, see below.

### 3.3 The hard problem: screening reference data is not freely downloadable

Not yet directly verified, but strongly implied by the structure of the field: the curated
sequence-of-concern databases that screening depends on are not a public download, for the obvious
reason that publishing a precise list of the sequences you screen for is itself a biosecurity
problem. SecureDNA's design (a hashed, privacy-preserving query protocol against a hosted database)
appears to exist specifically to solve this. `[unverified, needs confirming]`

**What this changes:** This is the project's single biggest external dependency and it is not
something engineering effort can route around. It goes in `REQUESTS.md` immediately, and subplan
24 has to have a credible answer for what the system does in the common case where the operator
has no access to a curated SOC dataset. "Ship with no reference data and refuse to export" is an
honest answer. "Ship with a toy dataset and imply it is real screening" is not, and would be
exactly the kind of dishonesty-by-construction the brief warns about.

---

## 4. The ML layer: what is actually achievable

### 4.1 Evo 2 changed the baseline

Arc Institute's Evo 2: 40B parameters, 1 Mb context, trained on ~9.3T nucleotides across 128,000+
genomes spanning bacteria, archaea, phages, humans, and plants. Open weights. Published March 2026
(preprint February 2025). Training data was safety-filtered to remove viral sequences infecting
eukaryotic hosts. 4,213 GitHub stars. `[secondary]` for the model details, `[verified]` for repo
stats.

Sources:
- https://arcinstitute.org/tools/evo
- https://www.biorxiv.org/content/10.1101/2025.02.18.638918v1
- https://pubmed.ncbi.nlm.nih.gov/41781614/
- Interpretability work: https://www.goodfire.com/research/interpreting-evo-2

**What this changes:** Two things.

First, capability. The brief specifies training CNN/BiLSTM/Transformer models for sequence-to-
expression prediction. That was the right answer in 2023. In 2026, the stronger baseline is
probably "use a genomic foundation model's embeddings and fit a small calibrated head on top,"
which needs far less training data and is far more likely to generalize off-distribution. Subplan
08 has to compare these honestly rather than defaulting to the brief's architecture.

Second, precedent. Evo 2's training-data safety filtering is the same design philosophy as our
biosafety gateway, executed by a well-resourced team, published, and scrutinized. Subplan 24
should cite it as prior art for the position that safety controls belong inside the system's
design rather than in a terms-of-service document.

**Constraint, and it is a hard one:** see section 6. 40B parameters does not fit in 12 GB of VRAM.
Any plan involving Evo 2 locally means the smaller variants, quantization, or an external API, and
subplan 08 must say which.

### 4.2 RBS and promoter prediction: the numbers do not agree with each other

This is where honest calibration matters most, because the published figures span a range wide
enough that picking the flattering one would be a lie by selection.

| Claim | Figure | Source type | Notes |
|---|---|---|---|
| RBS Calculator (thermodynamic) | R² from >0.8 down to <0.2 | `[secondary]` | Range depends entirely on dataset. This spread *is* the finding. |
| "RBS Predictor" / SAPIENS deep model | R² = 0.93 | `[secondary]` | **Vendor claim** (De Novo DNA docs), ~250k E. coli 5'UTRs. Treat with caution: not an independent benchmark. |
| Transformer de novo RBS design, P. denitrificans | Pearson r = 0.80 | `[secondary]` | Experimentally validated on selected designs. More trustworthy because wet-lab validated, but on selected designs, which is a biased subset. |
| sigma70 promoter *classification* (Sigma70Pred) | 81.5% to 90.4% acc. on independent data | `[secondary]` | Classification, not strength regression. Different task. Do not conflate. |

Sources:
- RBS Calculator: https://pubmed.ncbi.nlm.nih.gov/21601672/ , https://docs.denovodna.com/docs/rbs-calculator
- Paracoccus denitrificans de novo RBS design: https://pmc.ncbi.nlm.nih.gov/articles/PMC13495406/
- ML-guided batched RBS design: https://pubs.acs.org/doi/10.1021/acssynbio.2c00015
- Sigma70Pred: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9701712/
- PromoterPredict: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6228582/

**What this changes:** Subplan 08 and subplan 16 must state which of these numbers the project
holds itself to and why, and must not present the 0.93 figure as the state of the art without
flagging that it is a vendor's self-reported number on their own held-out split. The defensible
public claim is closer to "r ≈ 0.8 on in-distribution sequences in the organism the model was
trained on, substantially worse off-distribution, and we report a calibrated interval with every
prediction." Anything stronger than that is not supportable from the literature I found.

The R² > 0.8 to R² < 0.2 spread for the thermodynamic baseline is the most useful single fact
here. It means the deterministic model's error is highly dataset-dependent, which is exactly the
situation where the brief's "confidence cartography" idea earns its keep rather than being
decoration.

---

## 5. Rust, Go, and the boundary between them

### 5.1 Rust bioinformatics crates, with real download numbers

`[verified]` via crates.io API, recent-downloads figures:

| Crate | Version | Recent downloads | Use |
|---|---|---|---|
| `oxigraph` | 0.5.11 | 538,745 | SPARQL + RDF toolkit. Relevant: SBOL 3 is RDF. |
| `rio-turtle` | 0.8.6 | 344,432 | **Unmaintained**, upstream says use `oxttl`. Do not adopt. |
| `sophia` | 0.10.0 | 292,486 | RDF toolkit, alternative to oxigraph. |
| `bio` (rust-bio) | 4.0.1 | 95,060 | FM-index, alignment, parsers. |
| `noodles` | 0.116.0 | 86,853 | Bioinformatics format I/O, very broad format coverage. |
| `rust-htslib` | 1.0.1 | 73,846 | HTSlib bindings. |
| `needletail` | 0.7.3 | 72,587 | FASTA/FASTQ parsing, k-mers. Minimal-copy. |
| `bio-seq` | 0.15.1 | 955 | Bit-packed typed sequences. Low adoption, evaluate carefully. |

**What this changes:** The brief's suggestion to build "suffix-array or FM-index-backed" sequence
search is partly solved: `bio` ships an FM-index. Whether it is fast enough at genome scale for our
targets is an open question for subplan 09, and it is the sort of thing to benchmark before
committing rather than assume in either direction.

`oxigraph` at 538k recent downloads matters more than it looks. SBOL 3 is RDF, so a real SBOL
implementation needs an RDF layer underneath. That is presumably what `sbol-rs` is doing, and it
is worth checking which one they picked before we pick a different one and end up with two RDF
stacks in the same binary.

`bio-seq` at 955 recent downloads is a yellow flag under any sane dependency-vetting policy.
Subplan 07 should set the bar that catches this case explicitly.

### 5.2 The Rust/Go boundary: cgo got cheaper, but that is not the whole argument

Go 1.26 reduced cgo call overhead by about 30%, which makes FFI more practical than it was.
`[secondary]` Relevant because this machine has Go 1.26.1 installed. The countervailing view from
the same sources is that the common production pattern is service-level separation over gRPC
rather than mixing languages in one address space, with `tonic` on the Rust side and
`google.golang.org/grpc` on the Go side. `[secondary]`

Sources:
- https://blog.arcjet.com/calling-rust-ffi-libraries-from-go/
- https://words.filippo.io/rustgo/
- https://en.ihcblog.com/rust2go-cgo-asm/
- tonic vs grpc-go benchmark: https://dev.to/speed_engineer/grpc-performance-tonic-rust-vs-grpc-go-benchmarked-at-scale-2ojl

**What this changes:** Nothing yet, deliberately. This is subplan 05's decision and I do not want
to prejudge it here. What the research does establish is that both options are viable in 2026 and
the choice has to be made on this project's specific characteristics (crash isolation for a
screening gateway, deployment shape, whether the Python ML service already forces a network hop
anyway) rather than on generic performance folklore. Noting for subplan 05: if the Python ML
service is already over gRPC, the marginal complexity of putting Rust behind gRPC too is small,
and the crash-isolation argument for the biosafety gateway may dominate the performance argument
entirely.

**Note:** `protoc` is not installed on this machine. If the answer is gRPC, that is a toolchain
prerequisite for the dev environment docs in CONTRIBUTING.md. Rust's `prost`/`tonic` can vendor
protoc, Go's tooling generally cannot without setup.

---

## 6. Hardware and environment constraints

`[verified]` directly on this machine, 2026-09-14:

| Component | Value |
|---|---|
| Rust | 1.98.0 |
| Go | 1.26.1 |
| Python | 3.12.3, with `uv` 0.11.7 |
| Node / pnpm | 24.14.1 / 9.15.9 |
| Docker | available via Docker Desktop (WSL passthrough) |
| `protoc` | **not installed** |
| CPU | 20 cores |
| GPU | NVIDIA RTX 5070 Ti Laptop, **12,227 MiB VRAM** |
| **System RAM** | **7 GB total, ~6 GB available** |
| Disk | 710 GB free |
| Platform | WSL2, Linux 6.6.87.2-microsoft-standard-WSL2 |

**What this changes:** The 7 GB RAM figure is the constraint that bites, and it directly
contradicts one of the brief's own scenarios.

1. **"A multi-gigabyte genome import" cannot be tested honestly on this machine.** The human
   genome as raw FASTA is roughly 3 GB. Loading it naively into 7 GB of RAM alongside an OS, a
   Rust toolchain, and a Go service is not going to work. This is actually useful: it forces the
   sequence core to be designed for streaming and memory-mapped access from the start instead of
   discovering the problem later on a bigger machine. Subplan 09 should set a memory budget as a
   first-class performance target, not just latency and throughput.
2. **12 GB VRAM rules out Evo 2 40B locally.** Smaller variants, quantization, or a remote API.
   Subplan 08 must state which and must not plan around capability this machine does not have.
3. Performance targets in subplan 09 must be split into "verifiable here" and "requires
   representative hardware, currently unverified." Stating a target that cannot be measured on the
   only machine available is how benchmarks quietly become fiction.

---

## 7. Frontend and visualization

WebGPU reached candidate recommendation in March 2026 with roughly 85% global browser support, and
ships by default in Chromium browsers and Safari 26. **It is not in stable Firefox**, so a
production web target needs a WebGL2 or CPU fallback. `[secondary]` `wgpu` targets Windows, macOS,
Linux, and the browser via WASM from a single codebase. Tauri v2 is the general 2026 default
recommendation over Electron on bundle size (roughly 3 to 10 MB vs 120 to 200 MB) and idle memory
(roughly 30 to 85 MB vs ~450 MB). `[secondary]`

Sources:
- https://github.com/gpuweb/gpuweb/wiki/Implementation-Status
- https://web.dev/blog/webgpu-supported-major-browsers
- https://www.w3.org/TR/webgpu/

**What this changes:** The brief's "Rust plus wgpu, exposed to web via WASM" is technically sound
in 2026, which was not obviously true two years ago. The Firefox gap is the thing to plan for, not
ignore. Subplan 13 needs a stated position on the fallback path, because "scientific software that
does not work in my browser" is a credibility problem with exactly the academic audience this
project needs.

Also relevant for SBOL Visual rendering: `dnaplotlib` (304 stars, VoigtLab) and `paraSBOLv` (17
stars, dormant since 2022) are the existing glyph-rendering implementations, both Python. Worth
reading for glyph geometry before drawing our own.

---

## 8. Naming

`[verified]` by direct registry queries, 2026-09-14. HTTP 200 means taken, 404 means available.

| Candidate | crates.io | PyPI | npm | github.com/NAME |
|---|---|---|---|---|
| genesis | taken | taken | taken | taken |
| cistron | free | free | free | taken |
| operon | free | taken | taken | taken |
| replicon | free | free | taken | taken |
| anneal | taken | taken | taken | taken |
| chassis | taken | taken | taken | taken |
| protospacer | free | free | free | **free** |
| riboforge | free | free | free | taken |
| synthase | free | taken | taken | taken |
| plastid | free | taken | free | taken |

Beyond the registries, "Genesis" collides with the Genesis robotics/physics simulator (which drew
very large attention in late 2024), the US DOE "Genesis Mission" AI initiative, Genesis Healthcare,
and Genesis Energy.

**What this changes:** GENESIS is not viable as a package or org name. Decision taken with the
project owner on 2026-09-14: GENESIS is the internal working title, subplan 01 selects the real
name. This table is the starting shortlist, not the answer. Subplan 01 needs to widen it, check
trademark and scientific-literature collisions (a name that collides with a gene or a protein is
worse than one that collides with a startup), and check that the name does not read as
over-claiming.

---

## Open questions carried forward

Things I could not resolve from metadata and search, listed so they do not get silently dropped.

1. `sbol-rs` API ergonomics, round-trip fidelity on real SynBioHub files, SBOL 2 to 3 conversion
   losslessness, MSRV. Needs hands-on evaluation. -> subplan 23.
2. Whether SecureDNA can be self-hosted, and its query protocol. -> subplan 24.
3. Whether the Common Mechanism ships usable reference data or expects the operator to supply it.
   -> subplan 24, and almost certainly `REQUESTS.md`.
4. SynBioHub rate limits and whether the public instance is stable enough for CI contract tests.
   -> subplan 17.
5. Which RDF crate `sbol-rs` builds on. -> subplan 05 and 23, to avoid stacking two RDF layers.
6. Whether `bio`'s FM-index performance is adequate at genome scale for our targets, which needs a
   benchmark, not an opinion. -> subplan 09.
7. Current licensing and access terms for the iGEM Registry of Standard Biological Parts. Not yet
   researched. -> subplan 04.
8. SBML and SED-ML integration surface. The brief mentions BioModels and SED-ML but I have not
   researched them. -> subplan 04.
9. Cello 2.0's UCF (user constraint file) format, which is the thing we would need to read to
   import Cello outputs. -> subplan 06.
