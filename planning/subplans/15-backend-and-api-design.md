# 15. Backend and API design

**Status:** decided
**Date:** 2026-09-14
**Decides:** the API surface, versioning, auth, and how the Python SDK and app are both first-class
**Depends on:** subplans 05, 06, 07, 12
**Feeds:** `proto/`, subplan 18

---

## What exists when

The brief describes a gRPC/REST service layer. Subplan 05 deferred the service to Phase 3, because
after the scope cut there is nothing to orchestrate. That does not defer the **API design**, which
happens now.

| Phase | Surface |
|---|---|
| 1 | Rust library API, Python SDK, CLI. **`proto/` contracts written but not served.** |
| 2 | WASM bindings for the web UI |
| 3 | Go gRPC service, REST gateway, Python ML service |

**Writing the contracts in Phase 1 without serving them is the point.** It forces the semantic model
to be pinned down once (subplan 12), it makes the Phase 3 service an implementation rather than a
redesign, and it can be validated against a stub server long before the real one exists.

---

## The Python SDK is not a wrapper

Subplan 02 concluded the reachable audience is the ML-for-biology community, and subplan 11 made the
notebook the primary scenario. So the SDK is designed as a first-class surface, not generated
bindings with a thin veneer.

```python
import integrase as ig

design = ig.Design.from_sbol("construct.xml")

pred = ig.predict_translation_rate(design)
pred.value              # 4200.0
pred.interval           # Interval(lower=1900.0, upper=9100.0)
pred.coverage           # 0.80
pred.domain             # DomainVerdict.in_distribution(similarity=0.91)
pred.model              # ModelId('rbs-thermo-v1')
float(pred)             # TypeError, deliberately

preds = ig.predict_translation_rate(variants)   # batched, one crossing

cleared = ig.screen(design)          # raises ScreeningRefused on a hit
cleared.to_sbol("out.xml")           # only reachable via a cleared design
```

### Design rules

**`Prediction.__float__` raises, and this is the single most important line in the SDK.** The
one-liner that discards uncertainty is `rate = float(predict(design))`, and in a notebook it would
be written within a minute of installation. Making it a `TypeError` with a message pointing at
`.value` costs a user three seconds once and preserves the product's entire thesis.

**Batch calls cross the boundary once.** `predict(list)` is one PyO3 crossing, not N. Subplan 11's
scenario is 500 variants.

**Errors mirror the Rust hierarchy** (subplan 12), so a user reading Rust docs and a user reading
Python docs learn the same failure model.

**`ScreeningRefused` is an exception, not a return value.** In Rust a refusal is a typed `Refusal`
result because Rust callers must handle it. In Python, an exception is what stops a notebook cell
from proceeding as though nothing happened, and that is the behaviour that matters. Same semantics,
idiomatic shape in each language.

### The packaging problem, unresolved

From subplan 23: `sbol-py` provides Rust-backed SBOL for Python, unifying SBOL 2 and 3, but is **not
yet on PyPI**.

| Option | Cost |
|---|---|
| Depend on `sbol` once published | Best. Not currently available. |
| Bundle our own maturin build of the bindings | Packaging burden, duplicates upstream work |
| Use pySBOL3 in Python while using `sbol-rs` in Rust | **Rejected.** Two SBOL implementations with different behaviour in one product is the exact interoperability bug we exist to remove. |

**Action: ask SynBioDex about publication (subplan 23).** Until it resolves, the SDK keeps its SBOL
dependency behind our own types so the choice is swappable. A downstream project wanting to depend
on a library is the most useful thing a maintainer can hear about publication priority, so this is a
reasonable thing to ask rather than an imposition.

---

## gRPC contracts

`proto/` at the repository root, not under `services/`, because Rust, Go, and Python all consume it
and burying it in one language's tree implies ownership that does not exist.

```protobuf
// proto/integrase/v1/prediction.proto
syntax = "proto3";
package integrase.v1;

message Prediction {
  double        value            = 1;
  Interval      interval         = 2;   // absent only when uncalibrated
  double        nominal_coverage = 3;
  string        model_id         = 4;
  DomainVerdict domain           = 5;
  Provenance    provenance       = 6;
}

message DomainVerdict {
  oneof verdict {
    InDistribution     in_distribution     = 1;
    Marginal           marginal            = 2;
    OutOfDistribution  out_of_distribution = 3;
    Uncalibrated       uncalibrated        = 4;
  }
}
```

**`domain` is required, and `interval` is absent only for `Uncalibrated`.** A proto with optional
uncertainty fields is a proto where a client can ignore them, and proto3's field presence rules make
"forgot to set it" and "deliberately empty" indistinguishable if we are careless. The `oneof` for
`DomainVerdict` means a client must handle the variants rather than reading a string it can ignore.

### Services

```protobuf
service DesignService {          // import, export, validate
  rpc ImportSbol   (ImportSbolRequest)   returns (Design);
  rpc ExportSbol   (ExportSbolRequest)   returns (ExportSbolResponse);
}

service SimulationService {      // predictions, streamed for sweeps
  rpc PredictTranslationRate (PredictRequest) returns (Prediction);
  rpc PredictBatch           (PredictBatchRequest) returns (stream Prediction);
}

service ScreeningService {       // the gate
  rpc Screen (ScreenRequest) returns (ScreenResponse);
}
```

**`ExportSbolRequest` carries a `cleared_token`**, not a raw design. The type-level guarantee from
subplan 06 cannot cross a wire boundary, so it degrades to a server-side check: the server verifies
the token references a real, unexpired, current-policy screening verdict for that exact design
hash. **This is the same weakening documented at the Python boundary in subplan 12**, and it gets
the same treatment: explicit tests that an export request with a missing, forged, stale, or
mismatched token is refused.

**`PredictBatch` streams.** Subplan 11's scenario is 500 variants, and a user should see results
arrive rather than wait for all of them. This is also the one place Go's concurrency model earns its
place in Phase 3.

### REST

**Generated from the same protos via `grpc-gateway`, never hand-maintained.** A hand-written REST
layer alongside gRPC drifts, and then the two disagree about what a `Prediction` is, which is the
failure subplan 12 exists to prevent.

---

## Versioning

**Package version is shared across ecosystems** (R-6.1). `integrase` 0.4 in Python matches
`integrase-core` 0.4 in Rust. A user should never have to reason about a compatibility matrix.

**API version is in the proto package path** (`integrase.v1`), separate from the release version.
Breaking wire changes mean `v2` served alongside `v1`, not a silent change.

**`buf breaking` in CI** (R-6.4). Wire compatibility is the thing that breaks users silently and at
a distance, and it is the one class of regression that a human reviewer reliably fails to catch.

---

## Authentication and authorization, Phase 3

Not built yet, so these are requirements rather than implemented controls.

| Concern | Decision |
|---|---|
| Local use | **No auth.** A single-user library and CLI on your own machine. Adding auth would be security theatre with a usability cost. |
| Hosted, service-to-service | mTLS |
| Hosted, user | OIDC against the operator's identity provider. **We do not build a user database.** Institutions have one, and building our own would mean owning password resets and credential storage for no benefit. |
| Authorization | Per-design ownership, per-organisation scoping. Cross-tenant reads are the obvious high-severity bug class and get explicit tests, not just careful code (subplan 10). |
| Screening credentials | Server-side only. Never reachable by a client, never in CI logs. |

**Rate limiting is a memory-safety control, not just an abuse control.** At 39.4 bytes per base
(subplan 09), a single large upload can exhaust a modest server. Request size limits are enforced
before allocation, mirroring edge case E3.

---

## API principles

Five rules specific enough to be checkable in review.

1. **Uncertainty is never optional in a response.** No endpoint returns a bare predicted value in
   any protocol. This is R-0.1 crossing every boundary.
2. **Screening is never bypassable, even when the compiler cannot enforce it.** Where the type system
   does not reach (Python, gRPC), the check is explicit and explicitly tested.
3. **Batch by default for anything a user will do more than once.** Round-trips per variant is the
   shape that makes an SDK unusable at the scale people actually work.
4. **Errors are specific and actionable.** Per R-12.1, the values involved are in the message. A
   gRPC status with "invalid argument" and no detail is a bug report we will receive instead of a
   problem the user could have solved.
5. **The CLI, the SDK, and the service expose the same capabilities.** If something is only
   reachable from one surface, that is a gap, not a feature, and it gets recorded rather than
   rationalised.

---

## Open items

- [ ] Resolve the `sbol-py` PyPI question -> **23**, blocks Python packaging
- [ ] Design the `cleared_token` format: signed, with an expiry and a design hash binding
- [ ] Validate the `proto/` contracts against a stub server in Phase 1, so Phase 3 is implementation
      rather than discovery
- [ ] Confirm the < 5 ms loopback gRPC budget, since subplan 05's boundary decision rests on it
