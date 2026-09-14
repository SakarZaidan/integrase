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
- [ ] 01 Project identity and naming  **(gates repo creation)**
- [ ] 02 Idea validation and competitive landscape
- [ ] 03 Novelty, uniqueness, and feasibility rating
- [ ] 04 Resources and data
- [ ] 05 Technology stack, end to end
- [ ] 06 Per-subsystem architecture
- [ ] 07 Engineering principles and rules
- [ ] 08 AI/ML architecture and plan
- [ ] 09 Performance engineering
- [ ] 10 Security and threat modeling
- [ ] 11 Problems, scenarios, use cases, edge cases
- [ ] 12 OOP, code organization, API design philosophy
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
- [ ] 23 Upstream ecosystem strategy *(added, see PROGRESS.md)*
- [ ] 24 Biosecurity screening data sourcing and operational policy *(added, see PROGRESS.md)*

### Repository (after subplan 01 names the project)
- [ ] `gh repo create SakarZaidan/<name> --private`
- [ ] Apache-2.0 `LICENSE` and `NOTICE`
- [ ] Polyglot `.gitignore` (Rust, Go, Python, Node, WSL/editor cruft, `prompt.txt`)
- [ ] Minimal branch protection: PRs required, no direct pushes to default branch
- [ ] Verify settings by reading them back through `gh api`, not by assuming the write worked

### Synthesis
- [ ] `FINAL-DECISION.md`
- [ ] `ARCHITECTURE.md` with top-level plus per-subsystem Mermaid diagrams

### Verification before checkpoint
- [ ] Em dash check passes: `planning/../tools/check-voice.sh` reports no violations (the
      checker must exclude its own pattern definition, or it will always match itself)
- [ ] Banned-vocabulary grep across `planning/` returns nothing outside quoted context
- [ ] Every Mermaid diagram parses under a headless renderer
- [ ] Every subplan's open questions resolve in `FINAL-DECISION.md`, `TODO.md`, or `REQUESTS.md`
- [ ] `git log` shows owner identity only, no AI co-author trailers
- [ ] `prompt.txt` is not tracked

---

## Open research questions (carried from `00-research-log.md`)

Each is assigned to the subplan that has to answer it. Listed here so they cannot be silently
dropped when that subplan gets written.

- [ ] `sbol-rs` API ergonomics, round-trip fidelity on real SynBioHub files, SBOL 2 to 3 conversion
      losslessness, MSRV -> **23**
- [ ] Can SecureDNA be self-hosted, and what is its query protocol -> **24**
- [ ] Does the Common Mechanism ship usable reference data or require the operator to supply it -> **24**
- [ ] SynBioHub rate limits, and whether the public instance is dependable enough for CI contract
      tests -> **17**
- [ ] Which RDF crate `sbol-rs` builds on, so we do not stack two RDF layers -> **05**, **23**
- [ ] Is `bio`'s FM-index fast enough at genome scale for our targets (benchmark, not opinion) -> **09**
- [ ] iGEM Registry of Standard Biological Parts licensing and access terms -> **04**
- [ ] SBML, SED-ML, and BioModels integration surface -> **04**
- [ ] Cello 2.0 UCF format, needed to import Cello outputs -> **06**

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
