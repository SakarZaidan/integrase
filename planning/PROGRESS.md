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
