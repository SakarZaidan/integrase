# TODO

Live, prioritized. Checked off or deleted when done, not left to rot. If something here becomes
obsolete because the direction changed, it gets removed with a note rather than quietly ignored.

Last updated: 2026-09-14

---

## Now: planning phase

Target: complete planning folder, then owner review before any implementation code.

### Tracking and setup
- [x] Create `planning/`, `planning/subplans/`, `planning/decisions/`
- [x] `00-research-log.md` with sources and confidence markers
- [x] `PROGRESS.md`
- [x] `TODO.md`
- [x] `REQUESTS.md`
- [x] `planning/decisions/0000-adr-template.md` and `decisions/README.md`

### Subplans
- [x] 01 Project identity and naming  -> **Integrase**
- [x] 02 Idea validation and competitive landscape
- [x] 03 Novelty, uniqueness, and feasibility rating
- [x] 04 Resources and data
- [x] 05 Technology stack, end to end
- [x] 06 Per-subsystem architecture
- [x] 07 Engineering principles and rules
- [x] 08 AI/ML architecture and plan
- [x] 09 Performance engineering
- [x] 10 Security and threat modeling
- [x] 11 Problems, scenarios, use cases, edge cases
- [x] 12 OOP, code organization, API design philosophy
- [x] 13 Frontend and visualization engineering
- [x] 14 Brand and visual design system
- [x] 15 Backend and API design
- [x] 16 Simulation and biophysical modeling accuracy
- [x] 17 Testing strategy
- [x] 18 CI/CD and release engineering
- [x] 19 Contribution and governance model
- [x] 20 Documentation plan
- [x] 21 Roadmap, phases, milestones
- [x] 22 Final rating and verdict
- [x] 23 Upstream ecosystem strategy *(added, see PROGRESS.md)*
- [x] 24 Biosecurity screening data sourcing and operational policy *(added, see PROGRESS.md)*
- [x] 25 Circuit logic compilation *(added, scope restoration)*
- [x] 26 Generative design *(added, scope restoration)*
- [x] 27 Structural visualization *(added, scope restoration)*
- [x] 28 Genome-scale indexing *(added, scope restoration)*
- [x] 29 Verifiable design records *(added, scope restoration)*
- [x] 30 Adoption and distribution *(added 2026-09-15: the gap that kept adoption at 3)*
- [x] 31 Ethics, dual-use oversight, institutional integration *(added, scope restoration)*
- [x] 32 Funding and institutional sustainability *(added 2026-09-15)*
- [x] 33 Education sandbox *(added 2026-09-15: owner-requested feature)*
- [x] 25 Circuit logic compilation *(scope restored, 2026-09-14)*
- [x] 26 Generative design and foundation model integration *(scope restored)*
- [x] 27 Structural and 3D visualization *(scope restored, redefined)*
- [x] 28 Genome-scale indexing *(scope restored, corrects subplan 09)*
- [x] 29 Verifiable Design Records *(new: the primary original contribution)*
- [ ] 30 Adoption strategy *(new)*
- [ ] 31 Sustainability and funding *(new)*

### Repository
- [x] `gh repo create SakarZaidan/integrase --private`
- [x] Apache-2.0 `LICENSE` and `NOTICE`
- [x] Polyglot `.gitignore` (Rust, Go, Python, Node, WSL/editor cruft, `prompt.txt`, screening data)
- [x] Branch protection: PRs required, admins bound, linear history, no force push
- [x] Squash-only merge, merge commits and rebase disabled, delete branch on merge
- [x] Dependabot vulnerability alerts and automated security fixes
- [x] Settings verified by reading back through `gh api`
- [ ] **Secret scanning and push protection**: unavailable on a private repo at this plan tier
      (HTTP 422). Enable when the repository goes public.
- [ ] **Required approvals is 0, not 1.** GitHub forbids a PR author approving their own PR, so
      requiring 1 with a single contributor makes every PR unmergeable. PRs are mechanically
      required; the review itself is enforced by process via the `pr-review` skill. Raise to 1 the
      moment a second contributor exists. See subplan 19.

### Synthesis
- [x] `FINAL-DECISION.md`
- [x] `ARCHITECTURE.md` with top-level plus per-subsystem Mermaid diagrams (5 diagrams, all parse)

### Verification before checkpoint
- [ ] Em dash check passes: `planning/../tools/check-voice.sh` reports no violations (the
      checker must exclude its own pattern definition, or it will always match itself)
- [ ] Banned-vocabulary grep across `planning/` returns nothing outside quoted context
- [x] Every Mermaid diagram parses: `tools/check-mermaid.sh` (verified in both directions)
- [ ] Every subplan's open questions resolve in `FINAL-DECISION.md`, `TODO.md`, or `REQUESTS.md`
- [ ] `git log` shows owner identity only, no AI co-author trailers
- [ ] `prompt.txt` is not tracked

---

## Open research questions (carried from `00-research-log.md`)

Each is assigned to the subplan that has to answer it. Listed here so they cannot be silently
dropped when that subplan gets written.

- [x] ~~`sbol-rs` evaluation~~ Done. Adopt wholesale, see subplan 23.
- [x] ~~SecureDNA self-hosting~~ Yes. Local `synthclient`, DOPRF protocol, test HDB in-repo.
- [ ] Evaluate `common-mechanism` as a second backend, and whether dual-backend comparison earns
      its complexity -> **24**
- [x] ~~SynBioHub CI suitability~~ Moot: API requires auth. Vendor the `sbol-rs` corpus instead.
- [x] ~~Which RDF crate `sbol-rs` uses~~ Oxigraph family. Do not add `sophia`.
- [ ] Is `bio`'s FM-index fast enough at genome scale for our targets (benchmark, not opinion) -> **09**
- [x] ~~iGEM Registry licensing~~ CC BY-SA. ShareAlike applies to redistributed data.
- [x] ~~SBML / SED-ML / BioModels~~ BioModels is CC0. Export target for iBioSim interop.
- [x] ~~Cello UCF import~~ Phase 4 candidate only, and uncommitted: no scenario motivates it.
- [ ] Verify exact license terms on Evo 2 weights before depending on them -> **08**
- [ ] RNA secondary structure dependency: ViennaRNA licensing or an alternative. **Blocks the RBS
      baseline.** -> **16**
- [ ] Benchmark smaller genomic LMs against Evo 2 variants at 12 GB VRAM -> **08**
- [ ] Re-run the FM-index benchmark on a real genome rather than synthetic DNA -> **09**
- [ ] Confirm SecureDNA certificate terms for a design environment, not a synthesis provider -> **24**
- [ ] Measure the WASM bundle early; if it cannot stay near 5 MB the Leptos choice needs revisiting -> **13**
- [ ] Verify hatch/stipple textures stay legible at 12px in both render backends -> **13**, **14**
- [ ] Confirm font licensing (Inter, JetBrains Mono, Source Serif 4) before bundling -> **14**
- [ ] Design the `cleared_token` format: signed, expiring, bound to a design hash -> **15**
- [ ] **Resolve the ViennaRNA licensing question before Phase 1 starts.** Blocks the RBS baseline
      AND the RNA structure view. -> **16**, **08**, **27**
- [ ] **Ask CIDAR Lab to license `Cello-UCF`.** No licence stated means we cannot ship UCFs. -> **25**
- [ ] Draft `integrase-canon-v1`, the canonical design serialization the VDR digest depends on -> **29**
- [ ] Approach SynBioDex re: `DesignProvenance` as an SBOL extension -> **29**
- [ ] Re-benchmark both index tiers on a real genome, not synthetic DNA -> **28**
- [ ] Verify the <5ms loopback gRPC assumption before Phase 3 -> **09**, affects **05**
- [ ] **Find a second independent expression dataset.** On the critical path: without it the
      calibration claim is in-distribution only. -> **08**
- [ ] Decide whether the Python SDK depends on `sbol-py`, pySBOL3, or bundles its own build -> **15**
- [ ] Open the `sbol-py` PyPI conversation with SynBioDex -> blocks Python packaging in **15**
- [ ] Extend dependency vetting to cover data licensing, not just code -> **07**
- [x] ~~Benchmark `bio`'s FM-index~~ Done. 39.4 bytes/base, ~136 Mbp ceiling on this machine.
      Mammalian genomes are out of scope. See subplan 09.
- [x] ~~Audit log storage timing~~ No. Hash-chained append-only file suffices until Phase 3.
- [x] ~~Firefox fallback~~ SVG/DOM tier 2. No WebGL2 tier: three render paths means one done well.
- [x] ~~Visual encoding per DomainVerdict~~ Texture carries confidence, hue carries category.
- [x] ~~`Screenable` surface~~ Sequence, length, design ID. Nothing else. See subplan 24.

---

## Phase 0: complete (2026-09-15)

Definition of done was "a fresh clone runs `just setup && just check`, and CI passes on a real PR."
Both verified: 11 of 11 checks, three platforms, first run.

- [x] `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`
- [x] Issue templates, PR template, `CODEOWNERS` (every path verified to exist)
- [x] Six skills under `.claude/skills/`
- [x] CI/CD workflows per subplan 18 (11 jobs)
- [x] Repository scaffold: 9 crates, Python package, `proto/`, `just` targets
- [x] `docs/` with the two pages that make a first release honest
- [x] Root `README.md`
- [x] Full repo configuration: 11 required checks, admins bound, linear history,
      squash-only, Dependabot, topics, description
- [ ] **Secret scanning**: still unavailable on a private repo at this tier. Enable on going public.
- [ ] **Flip the repository to public.** README and scaffold now exist, which was the stated
      condition. Outward-facing and effectively irreversible, so this waits on the owner.

---

## Education sandbox (subplan 33)

Owner-requested. Serves the adoption channel subplan 22 scored as the binding constraint, so its
phase placement is an open question rather than automatically "later".

- [ ] Decide phase placement against Phase 1a
- [ ] Find a biology educator to review content accuracy. Getting teaching material wrong is the
      main risk and not one I can check alone.
- [ ] Scope the fictional organism: gene count, which processes
- [ ] Choose or write the stochastic simulation loop

---

## Upstream contributions owed (subplan 23)

- [ ] **Report RUSTSEC-2026-0194 and -0195 to Oxigraph.** `oxrdfxml 0.2.4` pins `quick-xml ^0.37`;
      the advisories require >= 0.41.0. This is the actual fix location, and it is unfixable
      downstream. Remove the `deny.toml` exceptions when a fixed `oxrdfxml` ships.
- [ ] Note the same to `sbol-rs`, since they inherit it through `sbol-rdf`.
- [ ] Open the `sbol-py` PyPI conversation
- [ ] File the `urn_design.ttl` round-trip observation

---

## Phase 1a: the honest predictor

The gate that can fail and block the phase: an 80% interval contains the truth 80% +/- 5% on
held-out data, **stratified by expression decile**, with **component-wise splits**. Not marginal
coverage, not a random split. Both of those are easy to pass and mean nothing.

- [x] ~~Resolve ViennaRNA licensing.~~ **Done.** Not OSI-approved: forbids redistribution for a fee
      and requires contacting authors for commercial use. Fails R-3.2. Contingency 9 applied; see
      `planning/decisions/0001`.
- [x] ~~`integrase-sim`: the thermodynamic RBS model~~ **Done.** Three of five terms, omissions
      disclosed in the model card, reports `Uncalibrated` until the coverage gate is met.
- [x] ~~`integrase-core`: motif and restriction-site search~~ **Done.** Both strands, IUPAC
      ambiguity, property test asserting a sequence and its reverse complement agree.
- [x] ~~SBOL export~~ **Done.** Round-trips, and `export_for_synthesis` takes `Cleared<Design>`
      with a `trybuild` test proving the unscreened call does not compile.
- [x] ~~`integrase-sbol`: import via `sbol-rs`~~ **Done.** 33/33 vendored pySBOL3 fixtures parse,
      43 designs imported. Conversion notes surfaced rather than discarded.
- [ ] `integrase-sbol`: resolve sub-feature locations. Currently component-level roles only, and
      the import report says so on every affected file.
- [x] ~~Split conformal calibration machinery~~ **Done.** Validated on synthetic data: coverage
      holds at 0.50/0.80/0.90/0.95, and under heavy-tailed noise, which is the distribution-free
      claim actually being tested.
- [x] ~~Locate the Kosuri data.~~ Found: PMC3752251, supplementary Datasets S1-S3. PMC blocks
      automated retrieval and the redistribution terms are unresolved (NAS copyright), so it is
      loaded from a local path and gitignored. See `docs/calibration-data.md` and R-005.
- [x] ~~Dataset loading and component-wise splitting~~ **Done.**
- [ ] **Fit the calibration to the real data.** Blocked on R-005: the file has to be downloaded by
      hand.
- [x] ~~Evaluation harness with stratified coverage~~ **Done.** Demonstrated that marginal coverage
      of 0.805 can hide strata running from 1.000 down to 0.520, which is the argument for the
      Phase 1a gate being stratified.
- [x] ~~`integrase-py`: the SDK backed by the real core~~ **Done.** PyO3 bindings, `float()` and
      `int()` both refused, batch path, hand-written stub checked against the extension in CI.
- [ ] Identify a second independent dataset for out-of-distribution evaluation -> R-005

---

## Blocked

Tracked in full in `REQUESTS.md`. Summary:

- **GitHub organization creation.** Needs the owner. GitHub exposes no public API for it. Not
  blocking anything right now because the personal repo is the agreed interim.
- ~~**Curated sequence-of-concern reference data.**~~ **RESOLVED** by subplan 24. SecureDNA is free,
  runs locally, uses an oblivious protocol so no one holds the database, and ships a test HDB so CI
  needs no credentials.

---

## Per-subplan open items

Each subplan ends with its own open items: **80 across 21 subplans**. They are not
duplicated here, deliberately. This file is a prioritised near-term list, and pasting 80 items
into it would make it unreadable and therefore unused, which is how a TODO file stops being trusted.

**The rule:** a subplan's own "Open items" section is authoritative for that subsystem. An item gets
promoted into the list above when it blocks near-term work, and into `REQUESTS.md` when only the
owner can resolve it. Everything else stays where the context lives.

To see them all:

```
for f in planning/subplans/*.md; do
  sed -n '/^## Open items/,$p' "$f" | grep '^- \[ \]' | sed "s|^|$(basename $f): |"
done
```

Counts by subplan:

- `01-project-identity-and-naming.md`: 3 open
- `04-resources-and-data.md`: 4 open
- `05-technology-stack.md`: 5 open
- `06-subsystem-architecture.md`: 5 open
- `07-engineering-principles.md`: 3 open
- `08-ml-architecture.md`: 6 open
- `09-performance-engineering.md`: 4 open
- `10-security-and-threat-model.md`: 5 open
- `11-scenarios-and-edge-cases.md`: 2 open
- `12-code-organization-and-api-design.md`: 3 open
- `13-frontend-and-visualization.md`: 4 open
- `14-brand-and-visual-design.md`: 3 open
- `15-backend-and-api-design.md`: 4 open
- `16-biophysical-modeling-accuracy.md`: 3 open
- `17-testing-strategy.md`: 4 open
- `18-cicd-and-release.md`: 5 open
- `19-contribution-and-governance.md`: 3 open
- `20-documentation-plan.md`: 3 open
- `21-roadmap-and-milestones.md`: 3 open
- `23-upstream-ecosystem-strategy.md`: 4 open
- `24-screening-data-and-operations.md`: 4 open

### Blocking, raised 2026-09-15

- [ ] **R-004: decide the institutional route.** An individual cannot apply to NSF PESOSE. This
      gates the entire funding path. Owner decision.
- [ ] Identify a second independent calibration dataset. Caps data availability at 7.

### Blocking before Phase 1a starts

Promoted here because `FINAL-DECISION.md` names them as gating:

- [ ] **ViennaRNA licensing.** Blocks the RBS baseline, the headline Phase 1 feature. If it cannot
      be used, ship the model without the structure-dependent terms and widen the intervals, per
      `FINAL-DECISION.md` decision 9.
- [ ] **Second calibration dataset.** Without it the central claim is in-distribution only.
- [ ] **Open the SynBioDex conversation** (`sbol-py` on PyPI, the `urn_design.ttl` observation).
