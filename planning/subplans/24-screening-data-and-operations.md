# 24. Biosecurity screening: data sourcing and operational policy

**Status:** decided. **R-002 substantially resolved.**
**Date:** 2026-09-14
**Decides:** where screening reference data comes from, who may change policy, what happens on a
hit, and how a vulnerability in screening is disclosed
**Why this subplan exists:** subplan 06 covers the gateway's structure and subplan 10 covers its
adversaries. Neither answers where the data comes from or what happens operationally. Given the
brief makes this subsystem first-class, the operational half earns its own document.
**Cross-references:** subplans 06, 10. Read all three together.

---

## The finding that resolves the blocking problem

`REQUESTS.md` R-002 recorded the project's largest external dependency: curated sequence-of-concern
data is deliberately not publicly downloadable, and no amount of engineering routes around that.

**SecureDNA solves this, and it solves it better than having the data would.**

| Property | Detail |
|---|---|
| Licence | MIT OR Apache-2.0. Compatible with ours. |
| Cost | **Free.** Nonprofit, free for all providers, no cost tier. |
| Deployment | `synthclient` runs **on the client's own premises**. Source, precompiled binaries, and container images all available. |
| Protocol | Generates windows from a FASTA input, hashes them via DOPRF (distributed oblivious pseudo-random function) against keyservers, checks results against a hash database (HDB). |
| API | Hand it FASTA, receive JSON naming which hazards were found, where in the input, and an unambiguous flag. |
| Local development | **A test HDB ships in the repository** (`test/data`). Development and testing need no production credentials. |
| Production | Requires certificates, obtained free from securedna.org. |
| Peer review | Published. https://pubmed.ncbi.nlm.nih.gov/42427467/ |
| Framework alignment | Maintains an explicit OSTP Framework compliance position. |

Sources: https://securedna.org , https://securedna.org/faq/ ,
https://github.com/SecureDNA/SecureDNA , https://securedna.org/exemption_certification_system/

### Why the oblivious protocol matters more than convenience

The reason the curated data is not downloadable is that a precise list of screened sequences is
itself a biosecurity problem. DOPRF dissolves that tension: **the client never holds the hazard
database, and the database never sees the client's plaintext sequences.** Windows are hashed
obliviously and checked against the HDB.

This means integrating SecureDNA is not a compromise forced by lack of access. It is a better
design than us holding reference data would be, because a local copy of a sequence-of-concern
database on every user's laptop is exactly the artifact nobody should want to create.

**Decision: `integrase-screen`'s primary backend delegates to a locally-running SecureDNA
`synthclient`. We do not implement screening matching logic.** This follows R-3 and subplan 23's
rule about not reimplementing what the field has already built well. A home-grown matcher would be
the most dangerous not-invented-here decision available in this project, because its failure mode is
a silent false negative.

---

## Backend architecture

```rust
pub trait ScreeningBackend {
    fn screen(&self, req: &ScreeningRequest) -> Result<BackendVerdict, BackendError>;
    fn identity(&self) -> BackendIdentity;   // recorded in every verdict
    fn data_version(&self) -> Option<DataVersion>;
}
```

| Backend | Status | Notes |
|---|---|---|
| `SecureDnaBackend` | Primary | Talks to a local `synthclient` over HTTP/JSON. |
| `CommonMechanismBackend` | Secondary, evaluated later | `ibbis-bio/common-mechanism`, 47 stars, active. Gives operators a choice and gives us a cross-check. |
| `CustomBackend` | Supported | Institutions with their own rules. Identity and policy version recorded in every verdict. |
| `NullBackend` | **Default** | No backend configured. Refuses everything. |

**`NullBackend` remains the default even though SecureDNA is free**, because the user still has to
install and run `synthclient`, and "we could not reach a screener" must never silently mean "clean."

**Running two backends and comparing is available as a configuration.** Disagreement between
independent screeners is high-value signal, and it is exactly the kind of thing this project should
surface rather than resolve silently.

---

## What we send, and what we deliberately do not

The `Screenable` surface is minimal by design, per subplan 10's principle of keeping the audit
surface small.

**Sent:** the nucleotide sequence, its length, and the design identifier for audit correlation.

**Not sent:** design names, user-authored descriptions, annotations, project metadata, user
identity, or anything else. A screening backend needs sequence to do its job. Everything else would
be a privacy leak with no screening benefit, and the DOPRF protocol means even the sequence does
not leave in plaintext.

This is written down because the natural drift is toward sending more context "in case it helps."

---

## Policy versioning is a hard requirement

From the research log: the OSTP framework's flagging threshold drops to **50 bp by October 2026**,
the sequence-of-concern definition **expands in the same month**, IGSC conformance is due
**2026-10-24**, and an interagency group reviews the state of the art before **2026-10-13**.

The rules move on a known cadence. Therefore:

- Thresholds, window sizes, and backend selection are **versioned configuration**, never compiled
  constants.
- Every verdict records `policy_version`, `backend_identity`, and `data_version`.
- A verdict is reproducible: given the same sequence and the same policy version, the same result.
- Policy staleness beyond a configured age surfaces as a warning attached to the verdict, not as a
  silent pass.

**The honest limit on what we can claim:** no government currently mandates nucleic acid synthesis
screening. The mechanism is procurement conditionality on federal research funding plus voluntary
IGSC compliance. **Integrase must never claim to make anyone legally compliant.** It can claim to
implement screening consistent with the framework's approach, which is a different and defensible
statement. This wording goes in the docs and the README verbatim.

---

## What happens on a hit

A hit is not an accusation. The overwhelmingly likely case is A1 from subplan 10's adversary model:
a researcher who did not know a component was regulated. Designing the response around the rare
malicious case would make the tool hostile to the common legitimate one.

| Step | Behaviour |
|---|---|
| 1. Export refused | A `Refusal`, not an error. Modelled as a distinct type (R-12.3) so the UI explains rather than showing a stack trace. |
| 2. The user is told what and where | Which region matched, and what class of concern. SecureDNA's API returns exactly this. |
| 3. The design stays fully usable | Editing, simulating, inspecting, and saving locally all continue to work. **Only the export path is closed.** |
| 4. The route forward is named | Institutional biosafety officer, and SecureDNA's exemption certificate system. |
| 5. Logged locally | Append-only audit entry. |
| 6. **Nothing is reported to anyone** | See below. |

### We do not report users

**Integrase does not transmit hit reports to any authority, vendor, or third party.** This is a
deliberate design decision and it deserves its reasoning stated.

A design tool that silently reports its users to authorities is a design tool that researchers will
not use, and they would be right not to. It would also be counterproductive: the legitimate
researcher who hits a match is exactly the person we want to keep inside a system that gives them
correct information and a route to proper approval. Driving them to an unscreened tool is a worse
outcome for everyone.

The actual enforcement point is the synthesis provider, where the IGSC protocol and the OSTP
framework operate and where the order is real. Our layer informs and refuses. It does not police.

### Legitimate work on controlled agents

Real researchers legitimately work with regulated sequences, and a tool that treats that as
impossible is useless to them. SecureDNA's exemption certificate system handles it: a researcher
specifies the controlled agent, and their **institutional biosafety officer approves and signs**
using the institution's certificate. Lab-wide tokens can be derived from an existing research
registration document.

Integrase supports exemption certificates by passing them through to the backend, which applies them
automatically. **We do not implement our own exemption mechanism.** Deciding who may work on what is
an institutional and regulatory judgment, and it is not ours to make or to reimplement.

---

## The audit log

| Property | Decision |
|---|---|
| Content | Timestamp, design ID, sequence hash (**not the sequence**), verdict, backend identity, policy version, data version, exemption certificate reference if used. |
| Append-only | Yes. Entries are never modified or deleted. |
| Local by default | Written where the user runs the software. Not transmitted. |
| Sequence storage | **Hash only.** An audit log full of plaintext sequences of concern would be a new hazard. |
| Retention | Local: user-controlled. Hosted (Phase 3): operator-configured with a documented default. |
| Tamper evidence | Hash chain, each entry including the previous entry's hash. |

**This resolves the open question from subplans 05 and 10 about whether storage is needed before
Phase 3.** It is not. A hash-chained append-only file is sufficient for local single-user operation
and needs no database. PostgreSQL arrives in Phase 3 with the hosted deployment, as originally
planned. The file format is designed so migration is an import, not a redesign.

---

## Vulnerability disclosure: a deliberate departure from the norm

Standard open-source practice is coordinated disclosure on a fixed timeline, then publish. **For one
specific class of vulnerability, we do not follow that.**

| Class | Handling |
|---|---|
| Ordinary vulnerabilities (API, parsing, dependencies) | Standard coordinated disclosure, 90 days, public advisory. |
| **Screening bypass or evasion technique** | **Not published on a fixed timeline.** Reported privately, coordinated with the affected backend provider (SecureDNA, IBBIS) first, because a public writeup of how to evade screening is itself a harm. Fixed quietly, credited to the reporter with the technique withheld if they agree. |

This is a real departure from norms that many open-source contributors hold strongly, so it is
stated up front in `SECURITY.md` rather than discovered during an incident. A reporter who disagrees
is entitled to that view; they should know the policy before they report, not after.

**The limit of this policy, stated honestly:** we do not withhold information that users need in
order to protect themselves. If a bypass means deployed versions are not screening correctly, users
are told that promptly and clearly. What is withheld is the reproduction recipe, not the fact.

---

## Testing, which subplan 17 implements

This is the most rigorously tested component in the system (95% floor, R-2.1). Specific requirements:

| Test class | What it asserts |
|---|---|
| Fail-closed fault injection | Backend unreachable, timing out, returning malformed JSON, returning a partial response, and panicking all produce a refusal. Never a pass. |
| `NullBackend` | Refuses every input including empty ones. |
| Type-level gate | A compile-fail test (`trybuild`) proving `export_sbol` cannot be called with an unscreened `Design`. |
| Python runtime gate | Attempting to export an unscreened design from Python raises. This is the weakened guarantee from subplan 12 and needs explicit coverage. |
| Boundary conditions | Sequences at exactly the threshold length, one below, one above. Matches straddling window boundaries. |
| Partial matches | Subplan 11 owns the scenarios; this is where they are exercised. |
| Policy versioning | A verdict produced under policy v1 is reproducible under policy v1 after v2 exists. |
| Audit integrity | Tampering with any entry breaks the hash chain and is detected. |

**These run against the SecureDNA test HDB**, which is why its presence in their repository matters
so much: the highest-consequence test suite in the project can run in CI with no credentials and no
production data.

---

## Open items

- [ ] Evaluate `common-mechanism` as a second backend, and whether dual-backend comparison is worth
      the complexity
- [ ] Confirm SecureDNA certificate terms for a tool that is a design environment rather than a
      synthesis provider. **We are an unusual consumer of their system and should ask rather than
      assume the fit.**
- [ ] Design the hash-chain audit format
- [ ] Draft `SECURITY.md` with the disclosure departure stated plainly -> post-checkpoint
