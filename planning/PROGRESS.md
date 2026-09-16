# Progress

A dated log of what has actually happened, written so that someone returning after a month away
can read this file alone and know exactly where things stand. Complete work, in-flight work, and
direction changes with their reasons. Not a changelog of commits. Commits are in git.

Working title for the project is GENESIS. The real name is decided in subplan 01 and this file
will be updated when it is.

---

## 2026-09-14

### Done

**Research pass (brief section 2).** Written up in `00-research-log.md` with sources and
confidence markers. Five findings materially changed the plan before a single planning document
was written:

1. **`SynBioDex/sbol-rs` exists** (v1.0.0, July 2026, Apache-2.0, actively pushed). The standards
   body shipped a Rust SBOL 2/3 implementation with conversion, fuzzing, and benchmarks. The
   brief's assumption that SBOL support must be built from scratch in Rust is wrong. This turns a
   large subsystem into a dependency decision and weakens one of the project's claimed
   differentiators.
2. **The field's tooling is mostly dormant, and the field is small.** Cello has 875 stars and has
   not been touched since 2022. SBOLDesigner has been dead since 2021. The most-starred
   *maintained* tool in the core ecosystem has 84 stars. Evo 2, on the AI side of the same field,
   has 4,213. The opportunity the brief describes is real. The audience is smaller than the brief
   implies, and subplan 03 has to say so.
3. **Biosecurity screening has a dated compliance target six weeks out.** The OSTP framework
   threshold drops to 50 bp and the sequence-of-concern definition expands in October 2026, with
   IGSC conformance due 2026-10-24. Good news for design: a testable requirement instead of an
   aspiration.
4. **Two credible open-source screening implementations already exist**, one of them
   (`SecureDNA/SecureDNA`) in Rust under Apache-2.0. Writing our own screening matcher would be
   the most dangerous not-invented-here decision available in this project.
5. **This machine has 7 GB of RAM and 12 GB of VRAM.** That contradicts the brief's
   multi-gigabyte-genome-import scenario and rules out local Evo 2 40B work. Subplans 08, 09, and
   11 plan around it rather than asserting targets that cannot be measured here.

**Owner decisions taken.** Recorded here because they constrain everything downstream:

| Decision | Choice |
|---|---|
| Project name | GENESIS is a working title. Subplan 01 selects the real name. Registry check confirmed `genesis` is taken on crates.io, PyPI, npm, and github.com. |
| License | Apache-2.0. Matches `sbol-rs` upstream, patent grant matters in biotech, acceptable to institutional legal review. |
| Remote | `SakarZaidan/<name>` initially, transfer to an organization later. GitHub has no public API for creating organizations. |
| First checkpoint | End of the planning folder. No implementation code before owner review. |
| Commit attribution | Project owner only. No AI co-author trailer, no bot identity, per brief section 5.4. |

**Planning folder created.** `planning/` with `subplans/` and `decisions/`.

**Files written:** `00-research-log.md`, `PROGRESS.md`, `TODO.md`, `REQUESTS.md`.

### Deviations from the brief, with reasons

The brief invites additions under section 15 and asks that any addition be justified here. Three
so far:

- **Two extra subplans, 23 and 24.** Subplan 23 (upstream ecosystem strategy) exists because the
  `sbol-rs` discovery created a build-versus-adopt decision that fits cleanly in none of the
  required 22, and that is simultaneously a technical supply-chain question and a question about
  the project's relationship with the standards body it depends on for legitimacy. Subplan 24
  (biosecurity screening data sourcing and operational policy) exists because subplan 06 covers
  the gateway's architecture and subplan 10 covers its threat model, but neither answers where the
  screening reference data comes from, who may update it, what happens operationally on a hit, or
  how a vulnerability in screening logic gets disclosed. Given the brief makes this subsystem
  first-class, the operational half earns its own document.

- **`planning/decisions/` for architecture decision records.** `FINAL-DECISION.md` is a
  point-in-time artifact. Without somewhere to record decisions made *after* it, the
  `planning-sync` skill has nowhere to write and the planning folder rots exactly the way the
  brief warns about in section 6. Small addition, concrete purpose.

- **Repository creation moved earlier than the brief's step ordering.** The brief puts repo setup
  at step 7, after all planning. The owner chose "personal repo now" so that planning work has a
  real remote and the PR discipline from section 5.5 applies to planning commits rather than being
  retrofitted. Repo gets created as soon as subplan 01 produces a name, with minimal configuration
  only. Full configuration (CODEOWNERS enforcement, required status checks, Dependabot, templates)
  waits until CI exists, because requiring status checks that do not exist yet would block every
  planning PR.

### Planning folder complete

All 24 subplans, `FINAL-DECISION.md`, and `ARCHITECTURE.md` are written and merged. Delivered
across 11 pull requests, each self-reviewed against the `pr-review` criteria with findings recorded
as a PR comment before merge.

**The project is named Integrase.** GENESIS is retired. `github.com/SakarZaidan/integrase`, private,
Apache-2.0, branch protection on, squash-only, Dependabot enabled.

### What the research changed, beyond the five findings above

Six more things were established by measurement rather than assumption during the subplans:

- **`sbol-rs` is far larger than its listing suggested.** 14 crates, 348 machine-checkable validation
  rules, Python bindings, a CLI, FASTA and GenBank conversion. A probe built against the real crates
  confirmed 33 of 33 pySBOL3 fixtures round-trip losslessly. A year of planned work became a
  dependency.
- **The FM-index costs 39.4 bytes per base.** Measured. That projects to 113.8 GB for a human genome
  and a practical ceiling near 136 Mbp here, which killed the brief's genome-scale search feature
  with a number rather than an argument.
- **SynBioHub requires authentication on every endpoint probed**, including one the docs describe as
  public. Changed the testing strategy to a vendored corpus.
- **SecureDNA resolves R-002**, the project's largest external dependency. Free, local, oblivious
  protocol, and a test database ships in their repository so the highest-consequence test suite runs
  in CI with no credentials.
- **The four semantic colours are not distinguishable by luminance** (1.14:1 to 1.70:1). That settled
  the visual grammar: texture carries confidence, hue carries category.
- **One palette value failed WCAG** at 4.20:1 against a required 4.5:1, and was corrected.

### Deviations from the brief, with reasons

Beyond the three recorded above:

- **Scope cut hard (subplan 02).** Circuit logic synthesis, generative design, 3D construct
  rendering, the desktop app, and genome-scale search are all out of v1.0. TinkerCell attempted this
  exact unification with a team and institutional backing and did not survive. The failure is
  structural, not a matter of effort.
- **Go arrives in Phase 3, not Phase 1 (subplan 05).** After the scope cut there is nothing for it to
  orchestrate. The `proto/` contracts are written in Phase 1 so the layer slots in.
- **3D rendering declined (subplan 13).** Constructs are linear; SBOL Visual specifies 2D because
  that is what journals accept.
- **The brief's student scenario is not supported in v1.0 (subplan 11)**, a downstream consequence of
  the scope cut that took several documents to notice.
- **Branch protection requires 0 approvals, not 1 (subplan 19).** GitHub forbids self-approval, so
  requiring 1 with one contributor makes every PR unmergeable. PRs are mechanically required and the
  review runs as recorded process instead.
- **Two extra subplans and an ADR directory**, justified above.

### The honest verdict

Subplan 22 re-scored the project against subplan 03's rubric: **4.7 to 5.6 out of 10.** Feasibility
and data availability improved by removing work. **Adoption did not move at all**, and that is the
binding constraint. `sbol-rs` is objectively excellent, from the standards body, and has 5 stars. If
that repository cannot get attention, quality alone will not get this one attention either.

The recommendation is to build the small version and publish the calibration result, because a
citable artifact is worth more to this project's survival than any additional feature.

### Next

Checkpoint. Owner review of the planning folder before any implementation code. After that, section
16 steps 8 through 13: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, templates, the six skills, CI
workflows, the repository scaffold, `docs/`, the README, and Phase 0.

---

## 2026-09-15

### Direction change from the owner

Instruction: keep the full feature set, compensate with deeper planning, and push the rating up.
That reverses the scope cut subplan 02 made on 2026-09-14. It was the right call, and the evidence
now says so: restoring scope moved novelty from 6 to 8.5 and defensibility from 6 to 8.

### What was wrong with the previous round, in order of severity

1. **Adoption was scored 3 and then not worked on.** The plan contained no distribution strategy at
   all, so the score was measuring the absence of work rather than the difficulty of the problem.
   Self-fulfilling: I concluded planning could not help, so I did not plan, so it did not help.
   Subplan 30 fixes it and moves the score to 8.5.
2. **Genome-scale indexing was declared infeasible on one library's benchmark.** `rust-bio`'s
   FM-index keeps a full `u64` suffix array. Tuning its `Occ` sampling rate from k=3 to k=64 drops
   that component from 29.33 to 2.11 bytes/base, and a compressed index measures ~3.9 bytes/base
   against the original 39.4. The measurement was right and the inference was wrong, and it cost a
   feature.
3. **Scope was cut partly because the build-versus-adopt work had not been done.** Nearly every
   large subsystem turns out to have an existing licence-compatible component underneath it.

### Work done today

- **Reconciled a duplicated planning state.** Two parallel attempts had produced conflicting files
  at subplan numbers 25, 26, and 27. Main's merged versions are authoritative; the duplicate at 26
  was removed, and the parallel 27 (ethics and dual-use oversight, a genuinely different topic from
  main's structural visualization) was renumbered to 31 and kept.
- **Subplan 30, adoption and distribution.** The iGEM channel: 4,600+ participants, 400+ teams,
  50+ countries at the 2025 Jamboree. Every step of a team's workflow maps to a subsystem we are
  building, and their mandatory Team Safety Form and White List map directly onto the screening
  gateway.
- **Subplan 32, funding and sustainability.** NSF POSE is now PESOSE, "Pathways to Enable *Secure*
  Open-Source Ecosystems", Tracks 2 and 3 up to $1.5M. Found an eligibility blocker that was
  invisible before: **an individual cannot apply.** Raised as R-004.
- **Subplan 22 rewritten** around three rubrics instead of one, because the original number was
  hiding three different questions.

### The rating, honestly

| Rubric | Score |
|---|---|
| Plan quality | **≈9.3** |
| Prospects if executed | **7.7** (was 5.6) |
| Current state | near zero: no users, no code, no funding, one contributor |

The ceiling this plan can reach is about **9.2**, and the four steps to it are named in subplan 22.
Three require execution rather than planning. **9.5 across the board is not written down**, because
the evidence does not support it and a planning folder about honest uncertainty cannot end by
reporting a figure it cannot defend.

### Next

Owner decisions on R-001 (organisation) and R-004 (institutional route). Then Phase 0.

---

## 2026-09-15 (later): Phase 0 complete

Phase 0 asked one question: **can someone contribute to this?** The answer is
verifiable rather than asserted. A fresh clone runs `just setup && just check`
and gets a green result, and CI passes on every platform.

### Definition of done, checked

| Criterion | Status |
|---|---|
| Fresh clone runs `just setup && just check` | Verified locally |
| CI passes on a real PR | 11 of 11 checks, three platforms, first run |
| Contributor can find the rules | `CONTRIBUTING.md` front-loads both tier-0 rules |
| Repository configured deliberately | 11 required checks, admins bound, linear history, squash-only |

### What landed

**Scaffold (PR #14).** Nine crates. The two tier-0 guarantees are structural
rather than aspirational: `Prediction<T>` has private fields, no `Deref`, and no
`into_value()`, so getting the number out means naming the field. `Cleared<T>`
has no public constructor, so the export gate is a compile-time property.

21 Rust tests and 6 Python tests. The property tests were mutation-checked in
both directions: injecting a revcomp that does not complement, and an off-by-one
in reverse-strand slicing, both fail the suite.

**Governance and CI (PR #15).** `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` with the
dual-use clause, `SECURITY.md` with the disclosure departure stated up front,
`CODEOWNERS`, issue and PR templates, and 11 CI jobs.

The screening job guards the type-level gate itself, which runtime tests cannot:
it fails the build if a public constructor for `Cleared<T>` appears outside
`cfg(test)`.

**Six skills (PR #16).** Under `.claude/skills/`, available to any future session.

**README and docs (PR #17).** `docs/limitations.md` and
`docs/concepts/uncertainty.md` ship now because they are what makes a first
release honest.

### Three decisions the work forced

1. **`rust-toolchain.toml` pins stable, not the 1.93 MSRV.** Pinning the floor
   would make every contributor build with the oldest supported compiler. The
   floor is checked mechanically in CI instead.
2. **An unrecognised `DomainVerdict` renders as the *least* confident encoding.**
   The enum is `#[non_exhaustive]`, so a future variant must land somewhere.
   Defaulting to solid fill would draw an unknown state as validated, which is
   the worst rendering error this system could make.
3. **macOS and Windows tests are required, not advisory.** Leaving them
   non-blocking would let them rot, which is the advisory-check failure mode
   subplan 18 criticises.

### Two lint rules overridden with reasons rather than silenced

Ruff's `N818` wants `ScreeningRefused` renamed to end in `Error`. Subplan 12
holds that a refusal is not an error but the system working correctly, and filing
it alongside I/O failures would lose the distinction that matters in the UI.
Suppressed at the point of exception with the reasoning attached.

`DomainReason` derives `PartialEq` without `Eq` because two variants carry `f64`
and NaN is not reflexive.

### Things the tooling caught in my own work

Worth recording, because it is the argument for having built the checkers:

- The voice checker failed my own prose three times, most recently on four em
  dashes in `uncertainty.md` written minutes earlier.
- Two false positives in the checker itself surfaced while writing the skills:
  Rust macros (`assert!`, `panic!`) tripping the exclamation rule, and documents
  that teach the rules by quoting them. Both fixed and verified in both
  directions.
- A speculative `cleared_for_test` constructor was written, found unused, and
  removed. Unused API added "for later" is exactly what the `ai-code-review`
  skill exists to catch.

### Next

Phase 1a: the honest predictor. Its gate is a real criterion that can fail and
block the phase, an 80% interval containing the truth 80% plus or minus 5% on
held-out data, stratified by expression decile, with component-wise splits.

Two owner decisions remain open and neither blocks engineering:
**R-001** (GitHub organisation) and **R-004** (institutional route, which gates
NSF eligibility entirely).

One item blocks Phase 1a in earnest rather than Phase 0: the ViennaRNA licensing
question, since the RBS baseline needs RNA secondary structure prediction.

---

## 2026-09-15 (later still): education sandbox planned

Owner requested a student-facing simulation sandbox: edit DNA and environment, see how a body is
built up from small parts, understand how small changes cause traits and disease, and design
medicine in a virtual environment. Written up as subplan 33.

### What research changed about it

The realistic ceiling is higher than I assumed. Karr et al. 2012 built the first whole-cell model of
a complete organism (*M. genitalium*: 401 genes, 1,857 reactions, **28 separate submodels**), and
whole-cell *E. coli* models now exist. The architectural finding matters more than the scale:
**those models compose different simulation methods per subsystem** rather than using one engine.

So the owner's instinct, semi-realistic and built from composed pieces, is not a compromise. It is
what the state of the art does, and subplan 33 follows that pattern deliberately.

### Three things pushed back on, with reasons

**Eye colour cannot be taught the simple way, because the simple way is wrong.** It involves roughly
16 genes, and the dominant factor (rs12913832) is not even inside a pigment gene: it is a regulatory
variant controlling how much of a neighbouring gene is expressed. Two blue-eyed parents can have a
brown-eyed child. It ships as the lesson in why biology resists simple rules, which the uncertainty
machinery is uniquely able to teach. Sickle cell, cystic fibrosis and Huntington's are genuinely
single small changes and carry full accuracy.

**Students will not discover cures, and presenting it as discovery would be a harm.** A simulator
contains only what was put in it. The better design is letting them experience why treatments fail:
about 90% of drug candidates entering clinical trials never reach approval, and only 8.1% make it
through all phases. Those numbers are the curriculum.

**"Restrict to good ideas only" is not implementable.** Intent is not in the data, and the edit that
disables a harmful gene is the same edit reversed. What works instead is curation: the sandbox
contains only the genes and molecules we ship, the organism is fictional, and there is no export
path toward physical synthesis at all. Deciding what exists is stronger than guessing at intent.

### The architecture decision

Composition and interfaces, not inheritance. A shared `CellState` with independent `Process`
objects stepping it, generalising Karr's 28 submodels. `Process` is a `Protocol` rather than an
abstract base class specifically so a student can write one without inheriting anything, which is
the best exercise the sandbox offers.

Cell type is data on a cell, not a subclass of it, for the same reason a feature's role is an
ontology IRI rather than an enum in `integrase-core`. Biology does not sort into a clean tree.

This is a deliberate departure from subplan 12, which argued the codebase is not object-oriented and
should not be. That argument still holds for the professional tool. The sandbox is a different case,
because a containment hierarchy is genuinely present in the domain being modelled.

Python leads here rather than follows, which is the only part of the project where that is true.
Students read and write the model.

---

## 2026-09-15: Phase 1a underway, and CI earned its keep

### Built

**The thermodynamic RBS model** (PR #20) and **SBOL 3 import** (PR #21). 33 of 33 vendored pySBOL3
fixtures parse, yielding 43 designs. Workspace at 55 tests.

### ViennaRNA resolved, and the answer was no

Its licence is not OSI-approved: it forbids redistribution for a fee and asks for author contact
before commercial incorporation, which is incompatible with Apache-2.0 and fails R-3.2.
`FINAL-DECISION` contingency 9 applied, recorded as ADR 0001. Three of five free-energy terms ship;
the two needing structure prediction are omitted and the model card says so, with a test asserting
it keeps saying so.

### cargo-deny found three real things on PR #21

The supply-chain surface subplan 10 warned about turned out to be real, and all three arrived with
`sbol-rs`.

**Two live advisories against `quick-xml 0.37.5`** (RUSTSEC-2026-0194 and -0195), both
denial-of-service: quadratic runtime on duplicate attribute names, and unbounded
namespace-declaration allocation. Reached through `sbol-rdf -> oxrdfio -> oxrdfxml`.

**Not fixable downstream.** The advisories require `quick-xml >= 0.41.0`. `oxrdfxml 0.2.4` is the
latest release and pins `^0.37`, and `oxrdfio` depends on it unconditionally rather than behind a
feature, so no version selection or feature flag on our side resolves it.

**Not currently reachable**, which is the part that matters. Our only entry point is `from_turtle`,
which parses through `oxttl`. The XML parser is linked and never called.

Three responses rather than one. A documented, conditional exception in `deny.toml` stating what it
is, why it cannot be fixed here, and what must happen before it is removed. A 64 MiB input cap
before parsing, which blunts both vectors for any parser this module feeds. And a **reachability
test** that fails the build if an RDF/XML entry point is added, because the reachability argument is
the part most likely to quietly stop being true while the exception silently becomes load-bearing.
Verified in both directions: adding an RDF/XML function fires it, and renaming `from_turtle` fires
the mirror guard that stops it passing vacuously.

**A licence rejection: CDLA-Permissive-2.0** on `webpki-roots`, via `ureq`, via `sbol-ontology`.
This is a good illustration of R-3.3's point that data carries its own terms: the licence covers
Mozilla's CA certificate set shipped inside the crate. It is permissive with no copyleft or
share-alike, so it was added to the allow list with that reasoning recorded rather than waved
through.

### Owed upstream

Reporting the `quick-xml` chain to Oxigraph is now the highest-value contribution available, since
that is the actual fix location and every downstream SBOL user inherits it. Added to `TODO.md`
alongside the `sbol-py` and `urn_design.ttl` items.

---

## 2026-09-16: rendering, and the first of the sandbox

Four merged pull requests and a fifth in progress. Recording what the work found rather than what
it shipped, since the findings are the part that changes later decisions.

### SVG rendering (#28, #31)

Designs render to SVG with the confidence encoding carried into the figure. Shape and hue say what
a part is, fill texture and border style say how much to trust the prediction about it, and the two
channels never cross. Convert a figure to greyscale and both survive.

The golden files earned their place immediately. Two bugs were invisible until the generated output
was read: reverse-strand features were mirrored about their own start coordinate and drawn at
entirely the wrong place on the backbone, and stroke-only glyphs emitted pattern definitions nothing
referenced. Neither would have failed a test written against the API.

Row packing came later and needed a second attempt. Greedy interval partitioning in sequence order
produces a correct packing that reads wrongly: a component-level role spanning the whole construct
claims the second row before most parts are placed, and the parts then scatter above and below it.
Placing shortest spans first puts the parts in a line on the backbone with the container above.
**The first version was correct and useless**, which is worth remembering the next time a layout
looks finished.

### The voice check had a hole in it (#29)

`tools/check-voice.sh` read markdown only. Most of this project's prose now lives in Rust doc
comments, so the section 11 rules went unenforced exactly where the writing happens. Coverage went
from 5 files to 111.

The extension needed quote tracking rather than a naive search for the comment marker, because this
codebase is mostly ontology IRIs and `"https://identifiers.org/SO:0000167"` would otherwise read as
a comment starting at the `//`. Rust tracks double quotes only: a lifetime like `'a` opens a
single-quote state that never closes.

### SBOL import was producing designs out of coordinate pairs (#30)

`BBa_F2620` imported as one undifferentiated 1023 bp box, and the same file produced 17 designs
where it holds 8.

The cause is worth recording because it will recur. **`try_from_object` in `sbol-rs` is a
projection, not a type test.** Every typed constructor returns `Some` for any object, reading
whatever properties happen to be present. A `sbol:Range` legitimately carries `sbol:hasSequence`,
pointing at the sequence it indexes into, so projecting one as a Component produced a design with a
real sequence attached and nothing about it looking wrong. Any future use of that API has to match
the typed `SbolObject` variant first.

The same pass found two protein sequences importing as DNA designs. The single-letter amino acid
alphabet overlaps the IUPAC nucleotide ambiguity codes almost completely, so a protein sequence
passes nucleotide validation. Only the declared encoding catches it, and only if checked before
validation rather than after.

### The education sandbox engine

Subplan 33's architecture, built: one `CellState`, four independent processes, a `Simulator` that
steps them, and a fictional four-gene organism to run.

Two things went in that the subplan asked for and that are easy to lose later. Every constant is a
`Parameter` carrying a provenance, and a measured or derived one with no source is refused at
construction. A test asserts that no shipped process holds a bare float, because adding one is easy,
looks harmless, and removes the only thing telling a student the number was invented.

`Process` is a `Protocol`, and a test asserts that a class with a `step` method and a `name` works
with no base class, no import, and no registration. If that test ever needs one of those to pass,
the design has regressed into something a beginner cannot extend.

**A test failure worth keeping.** Message levels settle at 2.45 where the analytic answer is 2.5. My
first instinct was a loose tolerance. The 2 percent is the discretisation: transcription runs before
degradation in the same step, so the discrete fixed point is `(k/d)(1 - d*dt)`. The test now asserts
the discrete value exactly and a second test asserts that halving `dt` halves the error, which is
what a first-order scheme should do and would catch a silent change to the integration.

### Packaging

`ADR 0002` records why the sandbox is a separate import root inside one Python project rather than a
second uv project, and names the property that decision rests on: the published wheel must not
contain it. Verified by building the wheel and listing it. That is maturin's behaviour rather than
an explicit exclusion, so the packaging job has to assert it before the first release.
