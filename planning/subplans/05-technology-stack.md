# 05. Technology stack, end to end

**Status:** decided
**Date:** 2026-09-14
**Decides:** every language, library, boundary, and build choice
**Depends on:** subplans 02 (scope cut), 03 (constraints), 04 (data), 23 (adopt `sbol-rs`)
**Feeds:** subplans 06, 07, 09, 12, 13, 15, 18, and `ARCHITECTURE.md`

---

## Summary of decisions

| Layer | Choice | Built in |
|---|---|---|
| Core library | Rust 2024 edition, MSRV 1.93 | Phase 1 |
| SBOL interop | `sbol-rs` as a dependency, thin adapter | Phase 1 |
| Python SDK | PyO3 + maturin, wheels to PyPI | Phase 1 |
| CLI | Rust, `clap` | Phase 1 |
| Screening gateway | Rust crate, delegating to an external screener | Phase 2 |
| Visualization | Rust + `wgpu`, WASM for web | Phase 2 |
| Orchestration API | Go + gRPC (`tonic` on the Rust side) | **Phase 3, when a service exists** |
| ML service | Python, isolated behind gRPC | Phase 3 |
| Desktop app | Tauri v2 wrapping the web frontend | Phase 4 |
| Database | None until Phase 3, then PostgreSQL | Phase 3 |
| Build | Cargo workspace + `uv` + `just` | Phase 1 |

The unusual entry is Go at Phase 3 rather than Phase 1. That is argued below and is the most
likely thing in this subplan to be disagreed with.

---

## Rust: the core

**Edition 2024. MSRV 1.93**, set by `sbol-rs` rather than by preference. Local toolchain is 1.98,
so there is headroom. MSRV is a published compatibility promise and subplan 07 owns the policy for
raising it.

### Crates, with reasoning

| Crate | Purpose | Why this one |
|---|---|---|
| `sbol`, `sbol3`, `sbol2`, `sbol-convert` | SBOL interop | Standards body's own implementation. 348 validation rules. See subplan 23. |
| `needletail` | FASTA/FASTQ parsing | Minimal-copy, 72k recent downloads, the ecosystem default. |
| `bio` (rust-bio) | FM-index, alignment | Ships the FM-index the brief asked us to build. 95k recent downloads. **Benchmark before committing**, see subplan 09. |
| `thiserror` | Library error types | Typed errors at library boundaries. Subplan 07 sets the policy. |
| `anyhow` | Binary error handling | In binaries and tests only, never in library public APIs. |
| `serde` | Serialization | Unavoidable and correct. |
| `proptest` | Property testing | Sequence manipulation is exactly where example-based tests miss whole bug classes. |
| `criterion` | Benchmarking | Needed for subplan 09's regression gates. |
| `clap` | CLI | Derive API, standard. |
| `tracing` | Structured logging | Needed for the screening audit trail in subplan 24. `log` is not enough. |
| `pyo3` + `maturin` | Python bindings | The only serious option. |
| `wgpu` | Rendering | Native plus WASM from one codebase. |
| `tonic` | gRPC, Phase 3 | Standard Rust gRPC. |

### Crates deliberately not used

- **`bio-seq`** for bit-packed sequences. 955 recent downloads. That number is below the bar subplan
  07 sets for a dependency in a correctness-critical path. Revisit if adoption grows; hand-roll the
  small amount of bit-packing we need in the meantime, or stay with plain byte slices until
  profiling says otherwise.
- **`sophia`** for RDF. `sbol-rs` uses the Oxigraph family (`oxrdf`, `oxttl`, `oxrdfxml`,
  `oxjsonld`), verified in subplan 23. Adding a second RDF implementation to the same binary would
  be a self-inflicted wound. If we need RDF primitives directly, we use Oxigraph.
- **`rio-turtle`.** Unmaintained; upstream says use `oxttl`. Listed only so nobody adds it later.
- **`rust-htslib`.** Wraps HTSlib in C. We have no alignment-file use case, and a C dependency
  complicates every build target including WASM. Not now.

---

## The Rust/Go boundary: gRPC, not FFI

The brief asks for a real engineering decision here rather than a default. This is it.

**Decision: a gRPC service boundary. No cgo, no C ABI between Rust and Go.**

Go 1.26 cut cgo call overhead by roughly 30%, which makes FFI more viable than it was, and this
machine runs Go 1.26.1. The overhead argument is therefore weaker than it used to be. It still
loses, for four reasons that have nothing to do with per-call latency.

1. **The Python ML service already forces a network hop.** Once one boundary in the system is a
   service boundary, making a second one an in-process FFI boundary buys inconsistency, not
   performance. Two different integration mechanisms means two different failure modes, two
   debugging stories, and two things a contributor has to learn.

2. **Crash isolation matters more here than latency.** The biosafety screening gateway is the
   highest-consequence component in the system. A panic in Rust reached through cgo can take down
   the Go process; there is no clean unwinding across that boundary. A gRPC boundary turns the same
   failure into a failed request that the caller can detect, log, and fail closed on. For a
   component whose entire job is to refuse things safely, "fails closed and says so" beats "fast."

3. **The latency budget is human-scale.** This is a design tool. A user clicks, waits, and reads a
   result. The budget is milliseconds, and gRPC round-trips on loopback are well inside it. Paying
   real complexity for microseconds nobody perceives is the wrong trade. Subplan 09 sets the actual
   numbers.

4. **cgo poisons the build.** It complicates cross-compilation, breaks pure-Go static linking,
   slows builds, and degrades the debugging experience on both sides. That cost is paid by every
   contributor on every build, forever, in exchange for a benefit measured in microseconds.

**Rejected alternative: in-process FFI via `cbindgen` or `cxx`.** Genuinely faster per call, and
the right answer if the Go layer were making millions of fine-grained calls into Rust. It is not.
It orchestrates jobs, which means coarse-grained calls where the work dominates the transport by
orders of magnitude.

**Consequence:** `protoc` is not installed on this machine. It becomes a documented prerequisite in
`CONTRIBUTING.md`. Rust's `prost`/`tonic` can vendor protoc; Go's tooling generally cannot without
setup, so the dev-environment docs have to cover it rather than assuming it.

---

## Go, and why it arrives in Phase 3

The brief specifies Go for orchestration, concurrent multi-user design sessions, and service
coordination. That is the right language for those jobs. The disagreement is about *when*, not
*whether*.

**After subplan 02's scope cut, there is nothing for Go to orchestrate in Phase 1.** The Phase 1
deliverable is a Rust library, a Python SDK, and a CLI. A single-user library has no jobs to
schedule, no sessions to coordinate, and no services to route between. Adding Go now would mean a
third toolchain, a third CI matrix, a third dependency ecosystem, and a third set of supply-chain
surface, for zero user-visible value.

**What we do instead:** design the gRPC boundary now, in `.proto` files, so the orchestration layer
slots in rather than requiring a rewrite. The Rust core exposes its capabilities through an
interface that a service can wrap the moment there is a reason to. Subplan 15 owns that interface
design, and it gets designed in Phase 1 even though it is not served in Phase 1.

**When Go arrives (Phase 3), the libraries are:**

| Library | Purpose |
|---|---|
| `google.golang.org/grpc` | The service boundary |
| `grpc-ecosystem/grpc-gateway` | REST from the same protos, so the HTTP API is generated rather than hand-maintained and drifting |
| `log/slog` | Structured logging, standard library, no dependency |
| `golang-migrate` | Schema migrations |
| `pgx` | PostgreSQL driver |

Standard library first. Go's ecosystem rewards restraint and the brief's dependency-vetting
requirement (subplan 07) applies hardest to the language with the easiest `go get`.

**Honest risk:** deferring Go means the polyglot architecture is partly unexercised until Phase 3,
so integration problems surface late. Mitigation is that the `.proto` contracts exist from Phase 1
and can be validated with a stub server well before the real one is written.

---

## Python

Two distinct Python surfaces that should not be confused.

### The SDK (Phase 1)

Built with **PyO3 and maturin**, shipping wheels to PyPI as `integrase`. This is where subplan 02
concluded the users actually are: pySBOL3 at 48 stars is the most active library in the ecosystem,
and the ML-for-biology audience lives in notebooks.

**Open dependency question, from subplan 23.** `sbol-py` provides Rust-backed SBOL for Python,
unifying SBOL 2 and 3, but is not yet on PyPI. Three options, none clean:

1. Depend on `sbol` once published. Best outcome, not currently available.
2. Bundle our own maturin build of the bindings. Packaging burden, and duplicates upstream.
3. Use pySBOL3 for the Python path while using `sbol-rs` in Rust. **Rejected**: two SBOL
   implementations with different behaviour inside one product is exactly the interoperability bug
   we exist to eliminate.

Path taken: ask SynBioDex about PyPI publication (subplan 23), and design the SDK so the SBOL
dependency is swappable behind our own types until that resolves.

### The ML service (Phase 3)

Isolated behind gRPC, exactly as the brief specifies, so a better model is a drop-in replacement
rather than a rewrite. Frameworks decided in subplan 08, not here, because the architecture choice
depends on evaluation methodology rather than the reverse.

**`uv` for dependency management**, not pip or poetry. Already installed (0.11.7), resolves fast,
and lockfiles are committed.

---

## Frontend and visualization

**Web first, desktop later.** This reverses the brief's ordering, on evidence: SBOLCanvas is
web-based and actively maintained (pushed 2026-09-10), while the Java desktop tools of the same
ecosystem are dead (SBOLDesigner last touched 2021-06-04). Delivery model correlates with survival
in this field.

**`wgpu`, targeting native and WASM from one codebase.** WebGPU reached candidate recommendation in
March 2026 with roughly 85% global browser support, shipping by default in Chromium browsers and
Safari 26.

**It is not in stable Firefox.** That is planned for, not ignored: a scientific tool that does not
work in a reviewer's browser has a credibility problem with exactly the audience it needs. Subplan
13 owns the fallback path.

**Desktop: Tauri v2** when it arrives in Phase 4, wrapping the same web frontend. Roughly 3 to
10 MB bundles against Electron's 120 to 200, and 30 to 85 MB idle memory against roughly 450. The
Rust backend is already our core, so Tauri is a thinner adapter than Electron would be.

**Rejected: a native GUI toolkit** (egui, iced, Qt). Would mean two separate UI implementations for
web and desktop, which for a one-person project is a reliable way to end up with one good UI and
one neglected one.

---

## Database: none, until there is a reason

**Phase 1 and 2 have no database.** Designs are SBOL documents, which are files. A library and a
CLI operating on files need persistence the way a compiler needs persistence, which is to say they
do not.

**Phase 3, when the orchestration layer arrives: PostgreSQL.** Job state, user sessions, and the
screening audit log all want transactions and durability. Postgres also handles the JSON-ish shapes
we will have without a second store.

**Rejected: SQLite for local state in Phase 1.** Tempting, and wrong. It would mean inventing a
storage model before we know what needs storing, and the first thing anyone would put in it is
caching that profiling has not yet justified.

**The screening audit log is the one exception worth flagging early.** Subplan 24 may conclude it
needs durable, append-only, tamper-evident storage from the moment screening exists, which could
pull a storage decision earlier than Phase 3. Cross-referenced there rather than pre-empted here.

---

## Repository layout

Monorepo, per-language workspaces. One repository because the polyglot boundaries are the thing
most likely to break, and a monorepo makes a cross-boundary change one atomic commit instead of a
coordination problem across three repos.

```
integrase/
├── crates/                      Rust workspace
│   ├── integrase/               umbrella, re-exports the stable surface
│   ├── integrase-core/          sequence types, indexing, search
│   ├── integrase-sbol/          adapter over sbol-rs (subplan 23)
│   ├── integrase-sim/           biophysical models + uncertainty
│   ├── integrase-screen/        biosafety gateway (own crate, deliberately)
│   ├── integrase-viz/           wgpu rendering, SBOL Visual glyphs
│   ├── integrase-py/            PyO3 bindings
│   └── integrase-cli/           CLI
├── proto/                       .proto contracts, source of truth for Phase 3
├── services/                    Go, Phase 3
├── ml/                          Python ML service, Phase 3
├── web/                         web frontend, Phase 2
├── app/                         Tauri shell, Phase 4
├── docs/                        user and contributor docs
├── planning/                    decisions and phasing
├── tools/                       repo tooling (check-voice.sh lives here)
└── testdata/                    vendored fixtures, provenance documented
```

`integrase-screen` is its own crate rather than a module inside `integrase-core`. The brief requires
the screening gateway to be a first-class subsystem with its own threat model and tests, and a crate
boundary makes that structurally true rather than aspirational. It also means the `pr-review` skill
can detect "this PR touches screening" from the diff path alone, which is exactly what that skill
needs to trigger extra scrutiny.

`proto/` sits at the root rather than under `services/` because the contracts are shared by Rust,
Go, and Python. Burying them in one language's directory would imply ownership that does not exist.

---

## Build and tooling

| Concern | Tool | Why |
|---|---|---|
| Rust | Cargo workspace | No alternative, and none wanted. |
| Python | `uv` | Fast, lockfiles committed, already installed. |
| Web | `pnpm` | Already installed. Strict node_modules layout catches phantom dependencies. |
| Task runner | `just` | One `justfile` so `just test` works regardless of which language a contributor came from. A polyglot repo where each language has its own incantation has a contributor onboarding problem. |
| Containers | Docker Compose, Phase 3 | Only when there are services to compose. |
| Protobuf | `buf` | Linting and breaking-change detection on the contracts. Breaking an API contract silently is a failure mode worth tooling against, per subplan 07's versioning policy. |

**Rejected: Bazel or Nix.** Both solve polyglot builds properly, and both would cost more in
contributor friction than they return at this size. A synthetic biologist who wants to fix a typo
in the Python SDK should not have to learn a build system first. Revisit if the repo ever gets
large enough that native tooling stops working.

---

## Hardware constraints this stack has to respect

From the research log: 7 GB RAM, 12 GB VRAM, 20 cores.

- **7 GB RAM rules out naive whole-genome loading.** `integrase-core` is designed for streaming and
  memory-mapped access from the start rather than discovering this later on a bigger machine.
  Subplan 09 sets a memory budget as a first-class target alongside latency.
- **12 GB VRAM rules out local Evo 2 at 40B.** Subplan 08 must plan for smaller variants or a remote
  API and say which.
- **20 cores makes the parallelism story easy** for local work, which weakens the case for the Go
  orchestration layer in Phase 1 further: `rayon` inside the Rust core handles embarrassingly
  parallel simulation sweeps without any service at all.

---

## Open items

- [ ] Benchmark `bio`'s FM-index at realistic sizes before committing to it -> **09**
- [ ] Resolve the `sbol-py` PyPI question -> **15**, **23**
- [ ] Decide whether the screening audit log pulls storage earlier than Phase 3 -> **24**
- [ ] Firefox/WebGL2 fallback strategy -> **13**
- [ ] ML framework selection -> **08**
