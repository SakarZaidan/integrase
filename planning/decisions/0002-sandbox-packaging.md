# 0002. Keep the sandbox a separate import root inside one Python project

**Status:** accepted
**Date:** 2026-09-16
**Amends:** `subplans/33-education-sandbox.md`
**Deciders:** project owner

## Context

Subplan 33 requires the education sandbox to be "a separate surface from the professional tool,
sharing the sequence core", and says explicitly that it is "not a mode hidden inside the CAD tool".
The reasoning is sound: different users, different fidelity expectations, and a different safety
profile. A student sandbox that can be reached from inside the design tool would let a sandbox
number be mistaken for a design prediction.

The subplan did not say how to package it. Two readings were available.

**A separate uv project.** `sandbox/` at the repository root with its own `pyproject.toml`,
lockfile, and CI job. Maximum separation, and the packaging boundary enforces the surface boundary.

**A separate import root inside the existing project.** `python/src/integrase_sandbox/`, sharing
the dev tooling, lockfile, and CI job with `integrase`.

## Decision

The second, for now.

The separation subplan 33 asks for is about what a user can reach and what claims a number carries,
not about how many lockfiles exist. `import integrase_sandbox` is a different name from
`import integrase`, and nothing in `integrase` imports the sandbox. A student running the sandbox
cannot reach a design prediction, and a scientist running the design tool never loads a sandbox
parameter.

A second uv project would add a second lockfile, a second ruff and mypy configuration, a second CI
job, and a second dependency audit, for a repository with one maintainer and nothing published yet.

## The property this depends on, and how it is verified

**The published wheel must not contain the sandbox.** If it did, a `pip install integrase` would
put teaching-grade parameters inside the professional package, and the separation would exist only
in the source tree.

Verified on 2026-09-16 by building the wheel and listing it: top-level entries are `integrase` and
`integrase-0.0.0.dist-info`, with no `integrase_sandbox`. This follows from `tool.maturin`'s
`module-name = "integrase._integrase"`, which makes maturin package the `integrase` directory alone
rather than everything under `python-source`.

That is a property of maturin's behaviour rather than of an explicit exclusion, so it could change
without anyone editing this repository. Before the first release, the packaging job must assert it
rather than assume it.

## Revisit when

- There is something to publish. A released sandbox needs its own distribution, its own version,
  and its own README, and at that point the second project stops being overhead and starts being
  the point.
- The sandbox gains a dependency the design tool must not have, such as a plotting or notebook
  stack. A shared lockfile would then put it in the design tool's dependency tree.
- Content authoring begins in earnest. Subplan 33 names it as the larger half of the work, and it
  may want its own review process and its own contributors.

## Alternatives rejected

**`integrase.sandbox` as a subpackage.** Rejected outright. It is exactly the mode-inside-the-CAD-
tool that subplan 33 forbids, and `from integrase.sandbox import ...` reads as an endorsement that
sandbox output is of a piece with design output.

**A separate repository.** It would break the "sharing the sequence core" half of the requirement,
or force the core to be published before it is ready.
