# 25. Circuit logic compilation

**Status:** decided. Restores a capability the first version of subplan 02 cut.
**Date:** 2026-09-14
**Decides:** how Integrase delivers Verilog-to-genetic-circuit compilation without reimplementing
Cello
**Depends on:** subplans 02 (revised), 06, 08
**Feeds:** subplans 21, 30

---

## Why this was cut, and why the cut was wrong

The first version of subplan 02 deferred logic synthesis past v1.0 on the grounds that "Cello
already did it well and reimplementing it is a multi-year project." The first half is right. The
second half assumed reimplementation was the only option, and that assumption was not checked.

| Asset | Language | Licence | Activity | Use |
|---|---|---|---|---|
| `CIDARLAB/cello` | Java | **BSD-2-Clause** | 875 stars, dormant since 2022-09 | Reference implementation, correctness oracle |
| `CIDARLAB/Cello-v2` | Java | none stated | 74 stars, dormant since 2022-05 | The 2.0 feature set |
| **`CIDARLAB/Cello-v2-1-Core`** | **Python** | **Apache-2.0** | 21 stars, pushed 2024-09 | **The integration target** |
| `CIDARLAB/Cello-UCF` | Python | **none stated** | 3 stars, dormant since 2021 | UCF corpus. Licensing is a problem, see below. |

**`Cello-v2-1-Core` is Python, Apache-2.0, and describes itself as a streamlined Cello algorithm
that takes Verilog plus a UCF and produces circuit designs.** Subplan 05 already provisions a
Python service boundary for the ML layer in Phase 3. The marginal cost of a second Python service
speaking the same gRPC contract is small.

The multi-year estimate was for a project nobody needs to do.

---

## What Cello does, so it is clear what we are and are not adding

Four stages:

1. **Logic synthesis.** Verilog in, truth table out, then technology-independent logic
   optimisation (Cello uses ABC/Yosys-class tooling) producing a netlist of NOR and NOT gates.
2. **Technology mapping.** Assign biological gates from a UCF to netlist nodes. This is the
   interesting part and where the search lives: simulated annealing by default, with hill climbing,
   breadth-first search, and random permutation available.
3. **Placing.** Arrange assigned gates into physical DNA layout under the UCF's rules.
4. **Export.** Emit sequences, SBOL, and plots.

A **UCF** (user constraint file) describes the available gates, their response functions, their
constituent DNA parts, and the placement rules for a given organism and genetic context.

**Cello's output is a circuit and a score.** It does not tell you how much to trust that score.

---

## What Integrase adds, which is the entire justification for doing this at all

Wrapping Cello and shipping its output unchanged would be pointless. Three additions, each falling
directly out of infrastructure this project already has:

### 1. Uncertainty on the circuit score

Cello's predicted circuit score is derived from gate response functions measured under specific
conditions. Those measurements have error, the composition of many gates compounds it, and the
score is reported as a number.

Integrase reports it as a `Prediction<CircuitScore>`: a value, an interval propagated through the
gate composition, and a `DomainVerdict` on whether this circuit's topology and gate usage fall
inside the range the UCF's characterisation actually covers.

**This is the single most useful thing we can add**, because circuit failure in practice often
comes from composing gates outside their characterised range, and the current workflow gives a
designer no signal that they have done so. A design whose predicted score is excellent and whose
gates are all being used at the edge of their measured dynamic range should look different from
one that is not.

### 2. Screening the compiled output

A compiled circuit is a sequence. It goes through the same gateway as anything else (subplan 06),
and the synthesis-bound export path requires `Cleared<T>` exactly as it does for a hand-built
construct. Cello has no screening step at all.

### 3. A provenance record for the compilation

The Verilog source, the UCF identity and version, the assignment algorithm and its seed, and the
resulting circuit all go into the Verifiable Design Record (subplan 29) as a derivation step.

**Circuit compilation is non-deterministic**: simulated annealing with a different seed gives a
different assignment. Recording the algorithm, parameters, and seed makes a compilation
reproducible, which it currently is not. That is a real contribution to the field's reproducibility
independent of anything else Integrase does.

---

## Architecture

```
Verilog  ──►  integrase-logic (Rust)
                 │  parse, validate, truth table
                 ▼
              gRPC ──►  cello-service (Python)
                 │         wraps Cello-v2-1-Core
                 │         synthesis + technology mapping + placing
                 ◄─────── netlist + assignment + score
                 │
              integrase-logic
                 │  wrap score as Prediction<CircuitScore>
                 │  propagate gate-level uncertainty
                 │  compute DomainVerdict from UCF characterisation bounds
                 ▼
              Design  ──►  integrase-screen  ──►  Cleared  ──►  export
                              └──►  integrase-vdr: derivation step recorded
```

`integrase-logic` is a thin Rust crate. The heavy algorithmic work stays in Python where a working,
published implementation already lives.

### Why a service rather than a port, initially

| Approach | Verdict |
|---|---|
| **Wrap `Cello-v2-1-Core` over gRPC** | **Chosen for Phase 3.** Working implementation, Apache-2.0, published and validated results. Fastest path to the capability, and the boundary already exists for the ML service. |
| Port technology mapping to Rust | **Phase 5 candidate, only if measurement demands it.** Simulated annealing over gate assignments is the compute-bound stage and would benefit, but "would benefit" is not a reason without a profile (R-5.4). |
| Reimplement from the papers | Rejected. This is what the original cut assumed and it is strictly worse than using the authors' own code. |
| Port the Java original | Rejected. `Cello-v2-1-Core` is Python, newer, and explicitly streamlined. |

**The port, if it happens, has an unusually good correctness oracle:** the original Cello is BSD-2
and its published results are reproducible, so a Rust technology mapper can be differentially tested
against the Python implementation on the same UCF and seed.

---

## The licensing problem, which is real

`CIDARLAB/Cello-UCF`, the UCF collection, **states no licence.** Dormant since 2021.

No licence means all rights reserved. We cannot redistribute those UCFs, bundle them, or ship them
as test fixtures. This is exactly the data-licensing case subplan 07's R-3.3 exists to catch, and it
would be easy to miss because the code is permissively licensed and the data sits beside it.

| Path | Status |
|---|---|
| Ask CIDAR Lab to state a licence | **Preferred.** Goes to `REQUESTS.md`. A short email, and the likely answer is that no licence was an oversight. |
| Users supply their own UCFs | Always supported. The tool reads UCFs; it need not ship them. |
| Ship only UCFs with clear licensing | Whatever `Cello-v2-1-Core`'s Apache-2.0 `library/` directory actually covers. Needs checking file by file. |
| Bundle them anyway | **No.** |

**Until licensing is clear, Integrase reads UCFs and ships none.** A first-run message points at
where to obtain them. Slightly worse onboarding, and correct.

---

## Testing

| Class | Assertion |
|---|---|
| Differential against Cello | Same Verilog, same UCF, same seed produces the same assignment as the reference implementation |
| Determinism | Same seed reproduces exactly; different seeds are allowed to differ and both are recorded |
| Uncertainty propagation | Property test: composing gates never shrinks the interval |
| Domain detection | A circuit using a gate outside its characterised input range returns `Marginal` or worse |
| Screening integration | A compiled circuit cannot reach synthesis-bound export uncleared |
| VDR round-trip | A compilation's derivation step verifies |

The differential test is the important one. It is the reason to wrap rather than reimplement: we
inherit a validated implementation and can prove we have not broken it.

---

## Open items

- [ ] `REQUESTS.md`: ask CIDAR Lab to state a licence for `Cello-UCF`
- [ ] Audit which UCFs `Cello-v2-1-Core`'s Apache-2.0 grant actually covers
- [ ] Determine whether gate response functions in UCFs carry enough error information to propagate
      uncertainty honestly. **If they do not, the interval is uninformative and we must say so
      rather than manufacture one.** This is a genuine risk to addition 1.
- [ ] Confirm `Cello-v2-1-Core` runs headless and scriptably without its `app/` layer
