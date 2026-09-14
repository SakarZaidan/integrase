# 10. Security and threat modeling

**Status:** decided
**Date:** 2026-09-14
**Decides:** the threat model, the attack surface, and the controls
**Cross-references:** subplan 06 (gateway architecture), subplan 24 (screening operations). These
three should be read together; this one covers adversaries, 06 covers structure, 24 covers
operations.
**Feeds:** `SECURITY.md`, the `pr-review` and `merge-decision` skills, subplan 18

---

## Scope and honest framing

This system's highest-consequence failure is not data loss or downtime. It is **a design that should
have been refused reaching a synthesis vendor.** That shapes everything below.

Two things this threat model does **not** claim, stated first because overclaiming here would be
worse than in any other subplan:

1. **Integrase is not a biosecurity control.** It is a design tool with a screening gate. The actual
   control in the real world is the synthesis provider's own screening, which is where the IGSC
   protocol and the OSTP framework operate. Ours is defence in depth, one layer among several, and
   a determined adversary who writes a sequence by hand in a text editor never touches our software
   at all.
2. **We cannot stop a knowledgeable adversary.** Someone who understands the biology does not need
   a CAD tool. Treating our gate as a barrier against expertise would be security theatre. What it
   genuinely does is prevent accidents, catch the uninformed, create an audit trail, and refuse to
   be a convenient instrument.

Being clear about this is itself a security property. A tool that overstates its protective value
encourages people to rely on it.

---

## Adversary model

| Adversary | Capability | Motivation | Do we address them? |
|---|---|---|---|
| **A1. Careless researcher** | Normal user. No evasion intent. | Wants to build something, unaware a component is regulated. | **Yes. This is the primary user the gate protects, and the most common real case.** |
| **A2. Curious user probing the gate** | Normal user, will try obvious workarounds. | Curiosity, or frustration at a refusal. | **Yes.** Obvious evasions must fail. |
| **A3. Determined evader** | Understands screening, will engineer around it. | Deliberate. | **Partially, and honestly: no, not reliably.** See below. |
| **A4. Supply-chain attacker** | Compromises a dependency across three ecosystems. | Broad. | **Yes.** Standard controls. |
| **A5. Attacker on a hosted instance** | Network access to a Phase 3 deployment. | Data theft, resource abuse. | **Yes**, from Phase 3. |
| **A6. Malicious contributor** | Submits a PR weakening screening. | Deliberate. | **Yes.** This is why R-7.3 routes screening changes to human sign-off. |

A3 deserves a direct answer rather than a hedge. **A determined, knowledgeable evader can defeat
sequence-level screening**, and this is well understood in the biosecurity literature rather than a
weakness peculiar to us. Codon reshuffling, fragmenting an order across vendors, and reconstructing
from parts each individually below threshold all defeat naive matching. Our mitigations (below) raise
the effort and leave evidence. They do not reduce the risk to zero and we will not say they do.

---

## Threat: evading the screening gate

The brief asks specifically what happens if someone tries to use the design tool to evade screening.

| Evasion | Mitigation | Effective? |
|---|---|---|
| Export in an unscreened format (FASTA instead of SBOL) | Type-level: **every** export path takes `Cleared<T>` (R-0.2). Adding a new format without screening does not compile. | **Yes.** Structural. |
| Call a lower-level API that skips the gateway | The gateway is the only constructor of `Cleared`. There is no lower-level export. | **Yes.** Structural. |
| Copy the sequence out of the UI by hand | **None. Not addressable, and not worth trying.** | **No**, deliberately. Blocking copy-paste in a design tool would make it unusable and would not stop anyone. |
| Split a design into sub-threshold fragments | Screen sub-assemblies and flag suspicious fragmentation patterns. Partially effective. | **Partially.** The October 2026 threshold drop to 50 bp exists precisely because of this attack. |
| Codon-shuffle to evade exact matching | Delegate to a backend that screens at protein/homology level rather than nucleotide identity. | **Partially**, and entirely dependent on backend quality. Our own matching would be worse. |
| Run a modified local build with screening removed | **None.** It is open-source software on the user's machine. | **No**, unavoidably. Anyone who will patch out the gate was never going to be stopped by it. |
| Configure a permissive custom backend | Allowed by design for institutional self-hosting, but the backend identity and policy version are recorded in every verdict and audit entry. | **Partially.** Visibility rather than prevention. |

**The two "no" rows are load-bearing and are documented in `SECURITY.md` rather than hidden.** A
threat model that claims complete coverage is one nobody should believe.

---

## Threat: the gate fails open

More likely than deliberate evasion, and more dangerous because nobody notices.

| Failure | Control |
|---|---|
| No reference data configured | `NullBackend` **refuses everything** (subplan 06). Unconfigured is the common case, given R-002. |
| Backend unreachable mid-request | Fail closed. A network error is a refusal, never a pass. **Explicitly tested** per subplan 17. |
| Backend returns a malformed response | Fail closed. Unparseable is not "clean." |
| A panic in the screening path | gRPC boundary (subplan 05) converts it into a failed request, which fails closed. A cgo boundary could have taken the process down ambiguously, and this was part of that decision. |
| Stale reference data | Every verdict records `policy_version` and data version. Staleness beyond a configured age is surfaced as a warning on the verdict. |
| Silent policy regression from a code change | R-7.3: screening changes require explicit human sign-off and cannot be auto-merged. |

**The single design rule: every uncertain outcome in the screening path resolves to refusal.** There
is no code path where "we could not determine" becomes "allowed." Subplan 17 tests this by fault
injection rather than by inspection, because this is exactly the property that rots silently.

---

## Threat: supply chain across three ecosystems

The brief correctly flags this as a real surface: three package managers, three vulnerability
databases, three sets of maintainer accounts.

| Control | Detail |
|---|---|
| Vetting before adoption | R-3.1, R-3.2. Justified in the PR, and the bar is highest in correctness-critical paths. |
| Lockfiles committed | All three ecosystems. Reproducible builds. |
| Automated scanning | `cargo deny`, `pip-audit`, `govulncheck` in CI. Dependabot alerts enabled. |
| `sbol-rs` pinned exactly | R-3.4. Not automerged. 348 validation rules we depend on. |
| Dependabot does not automerge into `integrase-screen` | Screening changes need human review regardless of source, and "it was a bot" is not an exemption. |
| Minimal dependencies in the gateway | R-12.6: `integrase-screen` depends only on `integrase-core`, keeping the audit surface for the highest-consequence component as small as possible. |
| Secret scanning + push protection | **Currently unavailable** on a private repo at this plan tier. Enable when public. Tracked in `TODO.md`. |

**The highest-value control here is R-12.6.** A compromise in a transitive dependency of the
screening crate is the worst supply-chain outcome available, so that crate has almost no
dependencies to compromise.

---

## Threat: the hosted deployment (Phase 3)

Not built yet, so these are requirements rather than implemented controls.

| Surface | Requirement |
|---|---|
| gRPC/REST API | AuthN required for anything but health checks. AuthZ per design, per organisation. |
| Multi-tenancy | A design belongs to an owner. Cross-tenant reads are the obvious high-severity bug class and need explicit tests, not just careful code. |
| Resource exhaustion | Index construction is memory-hungry (39.4 bytes/base, subplan 09). Per-request size limits are a **denial-of-service control, not a nicety**: a single 200 Mbp upload would exhaust a modest server. |
| Screening credentials | Backend credentials are server-side only, never reachable by a client. |
| Audit log | Append-only, tamper-evident, separately backed up. Subplan 24 owns the retention policy. |
| Transport | TLS everywhere. No plaintext gRPC off-loopback. |

**Open architectural question, flagged rather than assumed:** the screening audit log may need
durable, tamper-evident storage from the moment screening exists, which would pull a storage
decision earlier than subplan 05's Phase 3 plan. Subplan 24 decides.

---

## Threat: malicious or careless contributor

| Control | Detail |
|---|---|
| No direct pushes to `main` | Branch protection, `enforce_admins` true. Verified. |
| Screening changes need human sign-off | R-7.3. The `merge-decision` skill refuses to auto-merge them regardless of how clean the diff is. |
| `Cleared` test constructor is `#[cfg(test)]` | Compiled out of release. Widening it is a sign-off change. |
| CODEOWNERS on the screening path | Once the org exists. Tracked in R-001. |
| Structural detectability | Screening is its own crate, so "does this PR touch screening?" is answerable from the diff path. This is why it is a crate and not a module. |

---

## Secrets

| Secret | Handling |
|---|---|
| Screening backend credentials | Server-side env or secret manager. Never in git, never client-side, never in CI logs. |
| SynBioHub credentials (R-003) | Local `.env` only. **Never in CI**, which is why subplan 04 vendors a fixture corpus instead. |
| Model weight storage credentials | Release tooling only. |
| Signing keys | Hardware-backed or CI-managed, never in the repository. |

`.gitignore` excludes `.env`, `*.pem`, `*.key`, and `credentials.json`, with narrow allowances for
test fixtures under `testdata/`.

---

## Vulnerability disclosure

`SECURITY.md` at the repository root, with a private reporting channel (GitHub Security Advisories).

**One project-specific rule that differs from normal practice:** a vulnerability in the screening
gateway that would allow a design to bypass or defeat screening is **not** disclosed publicly on the
normal timeline. It is coordinated with the screening backend providers first, because a public
writeup of how to evade screening is itself a harm. This is a real departure from standard
open-source disclosure norms and it is deliberate. Subplan 24 owns the full policy.

---

## Open items

- [ ] `SECURITY.md`, including the two honest "we cannot stop this" rows -> post-checkpoint
- [ ] Does the audit log need durable storage before Phase 3? -> **24**, affects **05**
- [ ] Fault-injection tests for every fail-closed path -> **17**
- [ ] Enable secret scanning when the repository goes public
- [ ] CODEOWNERS on `crates/integrase-screen/` once the org exists -> R-001
