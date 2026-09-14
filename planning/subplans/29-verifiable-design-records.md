# 29. Verifiable Design Records

**Status:** decided. **This is the project's primary original contribution.**
**Date:** 2026-09-14
**Decides:** the format, the trust model, and the standards path for cryptographically verifiable
design provenance
**Depends on:** subplans 06 (`Prediction<T>`, `Cleared<T>`), 24 (screening), 08 (calibration)
**Feeds:** `FINAL-DECISION.md`, subplans 30 (adoption), 31 (sustainability), and `ARCHITECTURE.md`

---

## The problem nobody is solving

Three questions are being asked in synthetic biology in 2026, with increasing urgency, and no tool
answers any of them:

1. **Where did this sequence come from?** Evo 2 has 4,213 stars and generates DNA. Commec v2.0's
   July 2026 release specifically added sensitivity to AI-redesigned proteins split into short
   fragments, which tells you the screening community already regards machine-generated sequence as
   a distinct category of problem. Yet a FASTA file arriving at a synthesis provider carries no
   indication of whether a human designed it, a model generated it, or a model modified something a
   human designed.

2. **Was it screened, and against what?** The OSTP framework's threshold drops to 50 bp and the
   sequence-of-concern definition expands in **October 2026**, with IGSC conformance due
   **2026-10-24**. Providers must screen. But screening happens at the order boundary, redundantly,
   with no way to know whether the designer already screened it, under which policy version, or
   against which database. A design screened in January under one policy and ordered in November
   under another leaves no trace of the discrepancy.

3. **How much should I trust the numbers attached to it?** A design arrives with a predicted
   expression level. From which model? Calibrated on what? Was the input inside that model's
   validated range? Currently this information exists only in the session that produced it, and
   evaporates the moment the sequence is copied into an email.

**All three are provenance questions, and provenance is a solved problem in other domains.**

---

## The precedent, and why it matters that this is not a new idea

This is deliberately not an invention. It is the application of a proven, deployed mechanism to a
field that needs it and does not have it.

**C2PA** exists because AI-generated media created exactly this problem for images and video. Its
answer is a signed manifest: a block of metadata containing assertions about an asset's origin and
editing history, cryptographically bound to the asset. Its 2026 conformance program tests
interoperability across creation tools, publishing platforms, and verification services. And its
adoption is driven by regulation, specifically EU AI Act Article 50 and California SB 942, both
requiring machine-readable disclosure of AI-generated content.

**in-toto and SLSA** solve the same problem for software supply chains. The architecture is three
layers: a **DSSE envelope** carrying the signature, an **in-toto Statement** binding the attestation
to a subject by digest, and a **predicate** holding the domain-specific payload. SLSA provenance is
one predicate type among many.

The parallel to biology is close enough to be uncomfortable:

| | Media (C2PA) | Software (SLSA) | **Engineered DNA** |
|---|---|---|---|
| Artifact | Image, video | Build output | Genetic design |
| Generative pressure | Diffusion models | Code assistants | Evo 2 and successors |
| Regulatory forcing function | EU AI Act Art. 50, CA SB 942 | US EO 14028, SSDF | **OSTP framework, Oct 2026** |
| Attestation | Signed manifest | in-toto + DSSE | **Nothing exists** |

**The mechanism is proven and the gap is real.** Building "C2PA for engineered DNA" is a smaller
technical claim than it sounds and a larger strategic one.

---

## The format

A Verifiable Design Record is an **in-toto attestation**, using the existing three-layer
architecture rather than inventing a container.

```
DSSE Envelope
└── in-toto Statement
    ├── subject: [{ name, digest: { sha256: <canonical design digest> } }]
    ├── predicateType: "https://integrase.dev/DesignProvenance/v1"
    └── predicate: { ... }
```

### Subject: content-addressing a biological design

The subject digest is the SHA-256 of a **canonical serialization** of the design: sequence plus
features plus roles, in a defined normal form.

This is the one genuinely hard technical problem in the format, and it deserves stating plainly.
Two SBOL files can describe the same design and differ byte-for-byte: ontology URIs canonicalize
differently (measured directly in subplan 23, where SBOL 2 to 3 round-trips produced symmetric
differences on `sbol:role` and `sbol:type`), feature ordering is not specified, whitespace and
namespace prefixes vary, and SBOL 3 dropped `persistentIdentity` entirely.

**So the canonical form is defined over the semantic content, not the file.** Sequence bytes,
features sorted by (start, end, role IRI), roles normalised to preferred ontology IRIs, hierarchy
flattened deterministically. Subplan 23's measurement is what tells us this layer is necessary; a
naive digest over file bytes would make every record unverifiable after a format round-trip.

### Predicate: what the record actually asserts

```jsonc
{
  "design": {
    "canonicalization": "integrase-canon-v1",
    "length_bp": 4238,
    "feature_count": 7,
    "alphabet": "dna"
  },

  // Where it came from. A chain, not a flag.
  "derivation": [
    { "kind": "imported",  "source": "synbiohub:BBa_K1234567", "digest": "sha256:..." },
    { "kind": "edited",    "by": "human", "operations": 12 },
    { "kind": "generated", "model": "evo2-7b", "model_digest": "sha256:...",
      "prompt_digest": "sha256:...", "region": [1204, 1456] }
  ],

  // Every prediction, with the evidence needed to judge it.
  "predictions": [
    { "quantity": "translation_initiation_rate",
      "value": 4200.0, "interval": [1900.0, 9100.0], "nominal_coverage": 0.80,
      "model_id": "rbs-thermo-v1.2",
      "model_card_digest": "sha256:...",
      "calibration_set": "kosuri2013-componentwise-split-a",
      "domain_verdict": "in_distribution", "similarity": 0.91 }
  ],

  // The screening attestation. The part a synthesis provider cares about.
  "screening": {
    "verdict": "clear",
    "backend": "securedna",
    "backend_version": "...",
    "policy_version": "ostp-2026-10",
    "threshold_bp": 50,
    "hazard_db_version": "...",
    "screened_at": "2026-09-14T10:04:22Z",
    "exemptions": []
  },

  "producer": { "tool": "integrase", "version": "0.4.0" }
}
```

### Three properties this buys

**The derivation chain is a chain.** Not a boolean "AI-generated" flag. A design imported from
SynBioHub, hand-edited, then partially regenerated by Evo 2 has all three steps recorded with the
region the model touched. That is strictly more useful than a flag and it is what the screening
community actually needs, because "which 250 bp did the model write" is a different question from
"did a model touch this."

**Predictions carry their own evidence.** A `model_card_digest` means a claimed accuracy figure is
bound to the specific card that made it. Subplan 07's R-7.3 already routes `ModelCard` accuracy
changes to human sign-off; this makes a past claim independently checkable rather than merely
governed.

**Screening is attested, not asserted.** The verdict, the backend, the policy version, the
threshold, and the database version. A provider receiving this can verify that the design was
screened under the policy in force at the time, and can see immediately if a design cleared under
the pre-October-2026 threshold is being ordered afterwards.

---

## Trust model

Three tiers, because the right answer differs by deployment and pretending otherwise produces a
format nobody can use.

| Tier | Signing | Verifier needs | Use |
|---|---|---|---|
| **Local** | Ed25519 key generated on first run | The public key | A researcher's own records. Integrity, not identity. |
| **Institutional** | Key bound to an institutional identity, optionally the same certificate infrastructure SecureDNA already uses for exemptions | The institution's public key | A lab or biofoundry. This is the tier that matters for compliance. |
| **Transparency-logged** | Sigstore keyless, with a Rekor-style transparency log entry | Network access at verify time | Public designs, publications, anything where non-repudiation matters |

**Tier 1 is the default and it is deliberately weak.** A locally generated key proves that a record
was not altered after signing. It proves nothing about who made it. Saying so plainly matters: a
format that implies more assurance than it delivers is worse than no format, and this is exactly the
failure mode the rest of this project exists to avoid.

**We do not operate a certificate authority.** SecureDNA already runs certificate infrastructure for
exemption tokens, and institutions already have identity providers. Building a third would be both
redundant and a governance liability we are not equipped to carry.

### What a VDR does not prove

Stated here rather than in a footnote, because the whole value of the format depends on people not
over-reading it:

- **Not that the design is safe.** It records a screening verdict from a named backend under a named
  policy. If that backend has a gap, the record faithfully attests to a verdict that was wrong.
- **Not that the predictions are correct.** It records what a model said and how much that model
  knows. Subplan 16 is explicit that no sequence-only model captures genomic context.
- **Not that a human reviewed it.** Unless a human signature is present, in which case it proves
  exactly that and nothing more.
- **Not that the sequence was not derived from something undeclared.** Someone can retype a sequence
  into a new session and sign a record claiming clean human authorship. The record is as honest as
  its producer. **It raises the cost of lying and creates an audit trail; it does not make lying
  impossible.** Subplan 10's threat model already establishes that a determined adversary defeats
  sequence-level controls, and this is the same limit.

---

## Why this is defensible in a way the rest of the project is not

Subplan 03 scored defensibility low because calibrated uncertainty and a screening gate, while
novel, are copyable. A competitor adds a field and a check.

A record format is different, for three reasons that compound:

**1. Standards inertia.** The path is to submit `DesignProvenance` as an in-toto predicate type and
as an SBOL extension. Standards bodies move slowly and incumbency is most of the game. Being the
reference implementation of a format is a position, not a feature.

**2. Network effects on the verify side.** The value of emitting VDRs is proportional to how many
parties verify them. The value of verifying is proportional to how many exist. That is a two-sided
market, and the way in is to make verification free, trivial, and independent of us: a small
standalone verifier, permissively licensed, that a synthesis provider can run without adopting any
other part of Integrase. **Give away the verifier, and the format spreads whether or not our design
tool does.**

**3. A regulatory tailwind with a date on it.** C2PA's adoption curve is driven by the EU AI Act and
California SB 942. The OSTP framework plays the same role here, and the October 2026 milestones are
weeks away rather than hypothetical.

---

## Implementation

All primitives exist as well-adopted Rust crates. Verified on crates.io, 2026-09-14:

| Need | Crate | Recent downloads |
|---|---|---|
| Hashing | `sha2` | 249,056,414 |
| Signing | `ed25519-dalek` | 54,107,496 |
| COSE / CBOR encoding | `coset` | 3,443,576 |
| Keyless signing, transparency log | `sigstore` | 304,304 |
| Merkle structures | `rs-merkle` | 723,299 |
| in-toto types | `in-toto` | 1,554 |

The `in-toto` crate at 1,554 downloads sits below subplan 07's adoption bar (R-3.2) for a
correctness-critical path. **Decision: implement the Statement and DSSE envelope directly against
the published specifications rather than depending on it.** Both are small, stable, well-specified
formats, and the alternative is a low-adoption dependency in the component whose entire purpose is
trustworthiness. This is one of the few places in the whole plan where building beats depending, and
the reasoning is adoption risk rather than capability.

New crate: **`integrase-vdr`**, depending only on `integrase-core` and the primitives above. The
same isolation rule as `integrase-screen` (R-12.6) applies for the same reason: a component whose
job is trust must be auditable in a single sitting.

```
crates/integrase-vdr/
├── canonical.rs     canonical serialization, the hard part
├── statement.rs     in-toto Statement
├── envelope.rs      DSSE
├── predicate.rs     DesignProvenance/v1
├── sign.rs          local, institutional, sigstore
└── verify.rs        the standalone verifier
```

---

## Testing

This joins `integrase-screen` at the **95% coverage floor** (R-2.1), because a provenance format
that can be forged is worse than none.

| Class | Assertion |
|---|---|
| **Canonicalization stability** | Property test: a design surviving an SBOL 2 to 3 to 2 round-trip produces an identical canonical digest. This is the test that subplan 23's measured URI canonicalization exists to threaten. |
| Canonicalization discrimination | Any semantic change (one base, one feature bound, one role) changes the digest |
| Signature verification | Valid verifies; tampered payload, tampered subject, wrong key, and truncated envelope all fail |
| **Downgrade resistance** | A record with the signature stripped, or with `screening` removed, fails verification rather than verifying as unscreened |
| Cross-version | A v1 record still verifies after v2 exists |
| Independence | The standalone verifier validates records without any other Integrase crate present |

The downgrade test matters most. The obvious attack on any attestation format is removing the
inconvenient assertion and hoping the verifier treats absence as acceptable.

---

## Open items

- [ ] Draft `integrase-canon-v1` precisely enough to be implemented twice and agree. This is the
      specification work the format lives or dies on.
- [ ] Approach SynBioDex about `DesignProvenance` as an SBOL extension, alongside the `sbol-py`
      conversation from subplan 23
- [ ] Approach the in-toto community about registering the predicate type
- [ ] Talk to SecureDNA about whether their exemption certificate infrastructure can carry
      institutional signing identity, rather than us building a parallel one
- [ ] Decide whether the standalone verifier is a separate repository. Arguments both ways: a
      separate repo signals independence to verifiers, a monorepo keeps it honest against the
      emitter.
