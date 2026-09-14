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
- [ ] `planning/decisions/0000-adr-template.md` and `decisions/README.md`

### Subplans
- [x] 01 Project identity and naming  -> **Integrase**
- [x] 02 Idea validation and competitive landscape
- [x] 03 Novelty, uniqueness, and feasibility rating
- [x] 04 Resources and data
- [x] 05 Technology stack, end to end
- [x] 06 Per-subsystem architecture
- [x] 07 Engineering principles and rules
- [ ] 08 AI/ML architecture and plan
- [ ] 09 Performance engineering
- [ ] 10 Security and threat modeling
- [ ] 11 Problems, scenarios, use cases, edge cases
- [x] 12 OOP, code organization, API design philosophy
- [ ] 13 Frontend and visualization engineering
- [ ] 14 Brand and visual design system
- [ ] 15 Backend and API design
- [ ] 16 Simulation and biophysical modeling accuracy
- [ ] 17 Testing strategy
- [ ] 18 CI/CD and release engineering
- [ ] 19 Contribution and governance model
- [ ] 20 Documentation plan
- [ ] 21 Roadmap, phases, milestones
- [ ] 22 Final rating and verdict
- [x] 23 Upstream ecosystem strategy *(added, see PROGRESS.md)*
- [ ] 24 Biosecurity screening data sourcing and operational policy *(added, see PROGRESS.md)*

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
- [ ] `FINAL-DECISION.md`
- [ ] `ARCHITECTURE.md` with top-level plus per-subsystem Mermaid diagrams

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
- [ ] Can SecureDNA be self-hosted, and what is its query protocol -> **24**
- [ ] Does the Common Mechanism ship usable reference data or require the operator to supply it -> **24**
- [x] ~~SynBioHub CI suitability~~ Moot: API requires auth. Vendor the `sbol-rs` corpus instead.
- [x] ~~Which RDF crate `sbol-rs` uses~~ Oxigraph family. Do not add `sophia`.
- [ ] Is `bio`'s FM-index fast enough at genome scale for our targets (benchmark, not opinion) -> **09**
- [x] ~~iGEM Registry licensing~~ CC BY-SA. ShareAlike applies to redistributed data.
- [x] ~~SBML / SED-ML / BioModels~~ BioModels is CC0. Export target for iBioSim interop.
- [ ] Cello 2.0 UCF format specifics, needed for the import path -> **11**
- [ ] Verify exact license terms on Evo 2 weights before depending on them -> **08**
- [ ] Find a second independent expression dataset for out-of-distribution calibration -> **08**
- [ ] Decide whether the Python SDK depends on `sbol-py`, pySBOL3, or bundles its own build -> **15**
- [ ] Open the `sbol-py` PyPI conversation with SynBioDex -> blocks Python packaging in **15**
- [ ] Extend dependency vetting to cover data licensing, not just code -> **07**
- [ ] Benchmark `bio`'s FM-index at realistic sizes before committing -> **09**
- [ ] Does the screening audit log need durable storage before Phase 3 -> **24**
- [ ] Firefox / WebGL2 fallback strategy -> **13**
- [ ] Exact visual encoding per `DomainVerdict` variant -> **13**, **14**
- [ ] `Screenable` trait surface: what a backend gets to see -> **24**

---

## After the checkpoint (brief section 16, steps 8 through 13)

Not started, listed so the shape of the next phase is visible.

- [ ] `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`
- [ ] Issue templates, PR template, `CODEOWNERS`
- [ ] Six skills under `.claude/skills/`: `pr-review`, `merge-decision`, `ai-code-review`,
      `issue-comment-response`, `planning-sync`, `writing-voice`
- [ ] CI/CD workflows per subplan 18
- [ ] Full repo configuration: required status checks, Dependabot across cargo/gomod/pip, secret
      scanning and push protection, topics and description
- [ ] Repository scaffold: Rust workspace, Go module, Python package, desktop app shell
- [ ] `docs/` per subplan 20
- [ ] Root `README.md`
- [ ] Flip repository to public once README and scaffold exist
- [ ] Phase 1 implementation per `FINAL-DECISION.md`

---

## Blocked

Tracked in full in `REQUESTS.md`. Summary:

- **GitHub organization creation.** Needs the owner. GitHub exposes no public API for it. Not
  blocking anything right now because the personal repo is the agreed interim.
- **Curated sequence-of-concern reference data for biosafety screening.** The project's largest
  external dependency. Blocks any claim that screening is real rather than illustrative. Does not
  block architecture or the gateway's interface design.
