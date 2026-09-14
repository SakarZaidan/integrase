# 18. CI/CD and release engineering

**Status:** decided
**Date:** 2026-09-14
**Decides:** the pipeline, the environments, and how a coordinated multi-language release is cut
**Depends on:** subplans 07 (rules), 09 (perf gates), 17 (tests), 24 (screening suite)
**Feeds:** `.github/workflows/`, implemented after the checkpoint

Written before the workflow files exist, per the brief, so the workflows implement a decision rather
than being assembled ad hoc.

---

## Principles

1. **CI enforces what review cannot reliably catch.** Formatting, coverage floors, wire
   compatibility, and performance regressions are mechanical. Reviewers are unreliable at all four
   and their attention is better spent on design.
2. **CI does not depend on third-party services.** Contract tests use the vendored `sbol-rs` corpus,
   not live SynBioHub (which requires auth anyway). Screening tests use the SecureDNA test HDB. CI
   that fails for reasons unrelated to the change teaches people to re-run until green.
3. **Fast feedback first.** Lint and format in under a minute. The slow suites run after.
4. **The screening suite is non-skippable.** No `continue-on-error`, no path filter that could let a
   change through without running it.

---

## Pipeline

### Stage 1: fast checks, target under 2 minutes

| Check | Tool |
|---|---|
| Rust format | `cargo fmt --check` |
| Rust lint | `cargo clippy --all-targets -- -D warnings` |
| Python format and lint | `ruff format --check`, `ruff check` |
| Python types | `mypy --strict` |
| Go format and lint (Phase 3) | `gofmt -l`, `golangci-lint` |
| Proto lint | `buf lint` |
| **Proto breaking change** | `buf breaking --against main` |
| Commit messages | Conventional Commits |
| **Prose voice** | `tools/check-voice.sh` |
| **Mermaid diagrams** | `tools/check-mermaid.sh` |

The last two are unusual in a CI pipeline and they are there for a reason. The brief's voice rules
apply to every document in the repository, and a rule enforced only by good intentions decays. The
mermaid check exists because a diagram that renders as an error block in the README is a visible
failure and "it looked right in my editor" is not verification.

### Stage 2: tests

| Job | Content |
|---|---|
| Rust tests | `cargo nextest run --workspace` |
| Rust doctests | `cargo test --doc`, so documentation that stops compiling fails (R-4.3) |
| **Compile-fail tests** | `trybuild`, proving `export_sbol` rejects an unscreened design |
| Property tests | `proptest`, fixed seed in CI, longer runs nightly |
| **Screening suite** | **Required. Non-skippable.** Fault injection, `NullBackend`, boundary cases, audit integrity. |
| SBOL contract | Semantic comparison against the vendored corpus |
| Python tests | `pytest`, against the built extension rather than a stub |
| Coverage | `cargo llvm-cov` + `pytest-cov`, floors from R-2.1 |

### Stage 3: build verification

Rust workspace across Linux, macOS, and Windows. WASM target. Python wheels via `maturin` for the
supported matrix. Docs build. Go build and `go test -race` from Phase 3.

### Stage 4: security

`cargo deny` (advisories, licenses, bans), `pip-audit`, `govulncheck` from Phase 3, Dependabot
alerts, and secret scanning once the repository is public.

### Stage 5: performance, on a schedule and on relevant PRs

`criterion` against subplan 09's Tier 1 numbers. **10% regression threshold**, memory gated equally
with time. Tier 2 budgets are not gated until measured, because gating an unmeasured budget produces
a flaky check people learn to override.

Baselines are recorded **on CI hardware**, not copied from a developer machine. Subplan 09's numbers
came from a WSL2 laptop and are not interchangeable with a runner's.

### Stage 6: model evaluation, nightly

Calibration coverage, stratified coverage, OOD detection, baseline comparison. Slow, needs the
evaluation dataset, and runs nightly plus on any PR touching `integrase-sim` or model artifacts.

---

## What blocks a merge

| Check | Blocking |
|---|---|
| Format, lint, type check | Yes |
| Proto breaking change | Yes, unless the PR bumps the API version |
| **Screening suite** | **Yes, always** |
| Unit, integration, contract, doctests | Yes |
| Coverage floor, `integrase-screen` | **Yes, hard gate** |
| Coverage floor, other crates | Advisory once, blocking on repeat |
| Performance regression | Yes, overridable with a written justification in the PR |
| Model evaluation | No, nightly signal |
| Voice and mermaid | Yes |

The coverage distinction is deliberate. A mechanical gate that fires on a legitimate refactor
teaches people to route around it, so only the highest-consequence crate gets a hard gate. The
screening crate never gets the advisory treatment.

---

## Release engineering

The hard problem: one product, three package ecosystems, each with its own registry, versioning
culture, and immutability rules.

### Coordinated versioning

**One version number across all three** (R-6.1). `integrase` 0.4.0 on PyPI matches
`integrase-core` 0.4.0 on crates.io matches the Go module tag. A user should never reason about a
compatibility matrix.

**The cost, stated honestly:** a Python-only bugfix still bumps the Rust crates. That publishes
versions with no changes in them. That is the correct trade: a wasted version number is free, and a
user discovering that Python 0.4.2 needs Rust 0.4.1 is not.

### Cutting a release

1. `release/x.y.z` branch.
2. Version bumped everywhere by a script, not by hand. Hand-editing nine manifests is how one gets
   missed.
3. `CHANGELOG.md` generated from Conventional Commits, then **edited by a human**. Generated
   changelogs are accurate and unreadable; the edit pass is what makes them useful.
4. Full CI including nightly-only suites.
5. PR, reviewed, merged.
6. Tag `vX.Y.Z` triggers publication.

### Publication order matters

```
crates.io  →  PyPI  →  Go module tag  →  GitHub Release
```

**crates.io is first and it is irreversible.** A published version cannot be replaced, only yanked.
So the Rust publish is the commit point, and everything downstream can be re-cut if it fails.

**Go modules are just tags**, immutable via the module proxy once fetched. **This is why subplan 01
flagged the Go module path migration:** the path is baked into every import statement, so no Go
module version gets published from the personal namespace before the transfer in R-001.

### Pre-1.0

Breaking changes in minor versions, with changelog entries saying so (R-6.2). Being clear about
instability is better than pretending stability while breaking things, which is what version numbers
are for.

### What is released where

| Artifact | Registry |
|---|---|
| `integrase`, `integrase-core`, and the rest | crates.io |
| `integrase` Python wheels | PyPI |
| `integrase-cli` binaries | GitHub Releases, per platform |
| Go module | Git tag |
| **Model weights** | GitHub Releases, versioned, checksummed, with the `ModelCard` alongside |
| WASM bundle | With the web deployment |

Model weights are never in git (`.gitignore`). They are separately versioned artifacts, and a
`ModelId` in a `Prediction` resolves to a specific released artifact so any past result is
reproducible.

---

## Environments

| Environment | Purpose |
|---|---|
| Local | `just` targets matching CI exactly, so "works locally" means something |
| CI | Every PR |
| Nightly | Model eval, extended property runs, full cross-platform matrix |
| Staging (Phase 3) | Hosted deployment pre-production |
| Production (Phase 3) | Operator-run. We publish images; we do not operate a service in v1.0. |

**`just` targets mirroring CI is the small thing that matters most for a polyglot repo.** A
contributor who fixes a typo in the Python SDK should be able to run `just check` and see what CI
will see, without learning three toolchains.

---

## Open items

- [ ] Write the workflows, post-checkpoint
- [ ] Record CI-hardware performance baselines
- [ ] The version-bump script
- [ ] Decide the wheel build matrix, which drives CI time significantly
- [ ] Confirm secret scanning once the repository is public
