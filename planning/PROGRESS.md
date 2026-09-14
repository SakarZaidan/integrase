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

### In progress

Subplan 01 (project identity and naming).

### Next

Subplans 01 through 24, then `FINAL-DECISION.md`, then `ARCHITECTURE.md`. Repository creation
slots in immediately after subplan 01. Checkpoint for owner review at the end of the planning
folder.
