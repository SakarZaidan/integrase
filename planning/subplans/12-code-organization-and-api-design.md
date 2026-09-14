# 12. Code organization and API design philosophy

**Status:** decided
**Date:** 2026-09-14
**Decides:** how types and interfaces are designed so the system feels coherent from any of three
languages, and what "public API" means
**Depends on:** subplans 05, 06, 07
**Feeds:** subplans 15, 20, and `ARCHITECTURE.md`

---

## The problem this subplan solves

A polyglot system has a characteristic failure mode: it becomes three products that share a
repository. The Rust user gets rich types, the Python user gets dictionaries, the Go user gets
protobuf structs, and the three disagree about what a "design" is. Every bug report then starts
with figuring out which language's model of the world the user was holding.

The fix is not to make all three identical. Idiomatic Rust and idiomatic Python should look
different, and a Python SDK that reads like transliterated Rust is a bad Python SDK. The fix is that
**the three surfaces disagree about syntax and agree about semantics.**

---

## The core discipline: one model, three renderings

`proto/` is the semantic source of truth for anything crossing a language boundary. Not because gRPC
is used everywhere (it is not, until Phase 3), but because a written schema forces the question
"what is this, exactly?" to be answered once instead of three times.

| Concept | Rust | Python | Proto / Go |
|---|---|---|---|
| Construct | `Design` | `Design` | `Design` message |
| Located region | `Feature` | `Feature` | `Feature` message |
| Predicted value | `Prediction<T>` | `Prediction` (generic at runtime) | `Prediction` with a `oneof` value |
| Screened design | `Cleared<Design>` | `ClearedDesign` | `ClearedDesign` message |
| Ontology role | `OntologyTerm` | `str` (IRI) | `string` |

**Naming is identical across languages wherever the concept is identical.** A `Design` is a
`Design` everywhere. Renaming it to `Construct` in Python because that reads better would cost more
in confusion than it gains in elegance.

**Case conventions follow each language**, and that is not an inconsistency. `reverse_complement`
in Rust and Python, `ReverseComplement` in Go, `reverseComplement` in TypeScript. Mechanical
translation, no judgment involved.

---

## Public API surface: what is stable and what is not

**The `integrase` umbrella crate defines the stable Rust surface, by re-export.** Everything a user
should depend on is re-exported there. Everything else, including all of `integrase-core`'s
internals, is either `pub(crate)` or documented as unstable.

The reason for re-export rather than just marking things `pub`: in a workspace, `pub` means "visible
to other crates in this workspace," which is a build concept, not an API promise. Re-export from the
umbrella crate is an explicit act. Someone has to add a line, and that line is reviewable.

```rust
// crates/integrase/src/lib.rs
pub use integrase_core::{Design, Feature, Location, Sequence, SequenceView, Strand};
pub use integrase_sim::confidence::{DomainVerdict, Interval, Prediction};
pub use integrase_screen::{Cleared, Refusal, ScreeningVerdict};

/// Unstable. Anything here may change in a patch release.
/// Depending on it is opting out of the versioning policy in subplan 07.
pub mod unstable {
    pub use integrase_core::index::FmIndex;
}
```

The `unstable` module is a deliberate escape valve. Without one, users depend on internals anyway
and then get broken, and we get blamed. With one, they opt in visibly and we owe them nothing.

**Three stability tiers, stated in the docs for every item:**

| Tier | Promise | Where |
|---|---|---|
| Stable | Semver applies. Breaking changes need a major version. | Re-exported from `integrase` |
| Unstable | May change in any release. | `integrase::unstable`, marked in docs |
| Internal | No promise. Not visible outside the workspace. | `pub(crate)`, private modules |

---

## Error type design

**Rust: one error enum per crate, at the crate boundary.**

```rust
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("sequence contains byte {byte:#04x} at offset {offset}, invalid for {alphabet}")]
    InvalidResidue { byte: u8, offset: usize, alphabet: Alphabet },

    #[error("location {location} exceeds sequence length {length}")]
    LocationOutOfBounds { location: Location, length: usize },
}
```

Three rules that make errors usable rather than decorative:

**R-12.1 Error messages contain the values involved.** `"invalid residue"` tells a user nothing.
`"sequence contains byte 0x55 at offset 91827, invalid for Dna"` tells them where to look. The cost
is a struct variant instead of a unit variant, which is nothing.

**R-12.2 Errors do not leak internal types.** `CoreError` never exposes an `sbol_rs::ReadError` in
its public shape. Wrap it. Otherwise every dependency upgrade is a breaking change to our error
enum, and subplan 23's adapter boundary stops meaning anything.

**R-12.3 `Refusal` is not an error, it is a result.** A screening refusal is the system working
correctly. It is modelled as a distinct type carrying the verdict, the policy version, and what the
user can do about it, not as an `Err` variant lumped with I/O failures. This distinction matters in
the UI: a refusal should be explained, not shown as a stack trace.

**Python errors mirror the Rust enums as an exception hierarchy:**

```
IntegraseError
├── CoreError
│   ├── InvalidResidueError
│   └── LocationOutOfBoundsError
├── SbolError
├── SimulationError
└── ScreeningRefused        # not an error condition, a documented outcome
```

Same names, same semantics, idiomatic shape in each language.

---

## Module boundaries

**R-12.4 A module is a boundary, not a folder.** If `seq/` and `design/` freely reach into each
other's internals, they are one module in two folders and should be honest about it.

**R-12.5 Dependencies between crates are acyclic and stated:**

```
integrase (umbrella)
    ├── integrase-core        (no internal deps)
    ├── integrase-sbol        -> core
    ├── integrase-sim         -> core
    ├── integrase-screen      -> core
    ├── integrase-viz         -> core, sim
    ├── integrase-py          -> all
    └── integrase-cli         -> all
```

`integrase-core` depends on nothing internal. That is the constraint that keeps it reusable and
keeps its compile times and test times low. If something in `core` needs `sim`, the design is
wrong and the thing belongs in `sim`.

**R-12.6 `integrase-screen` depends only on `core`.** Not on `sim`, not on `sbol`. The screening
gateway must be auditable in isolation, and a reviewer should be able to read that crate and its
one dependency to understand everything that affects a screening verdict. Every dependency added
there widens the audit surface for the highest-consequence component in the system.

---

## Designing across the language boundary

### Rust to Python: what translates and what does not

**Generics do not translate.** `Prediction<T>` is generic in Rust and dynamically typed in Python.
The Python binding exposes `Prediction` with a `.value` whose type depends on what produced it, plus
`.interval`, `.coverage`, `.model`, and `.domain`. That is idiomatic Python and it preserves the
semantics, which is the actual requirement.

**Ownership does not translate.** `SequenceView<'a>` is a borrowed view. Python has no lifetimes, so
the binding either copies or holds a reference to the owning object. The choice is copy-on-access
with a documented cost, because a dangling reference in a notebook is a segfault, and a segfault in
a scientist's notebook is an unrecoverable trust failure.

**The type-level screening gate does not translate, so it is enforced at runtime.** Python cannot
express "this function only accepts a value constructed by that module." `ClearedDesign` therefore
validates on construction and the export functions check, raising `ScreeningRequired` otherwise.
**This is a genuine weakening of R-0.2 at the Python boundary and it must be tested explicitly**,
because a guarantee the compiler used to provide is now a runtime check that can be forgotten.
Subplan 17 owns the test that tries to export an unscreened design from Python and asserts it fails.

### Rust to gRPC

**R-12.7 Proto messages are not internal types.** They are a wire format with its own evolution
rules. Conversions live in explicit `From`/`TryFrom` implementations in a boundary module, never by
making internal types `#[derive(prost::Message)]`. Otherwise wire-format constraints (field
numbering, optionality, no enums with data) leak into the domain model and quietly ruin it.

---

## Naming conventions

Small things, decided once, so they stop being decided repeatedly in review.

| Rule | Example |
|---|---|
| No abbreviations except domain-standard ones | `sequence` not `seq` in public API; `rbs`, `cds`, `utr` are fine, they are what biologists say |
| Constructors say what they make | `Design::from_sbol`, `Design::from_fasta`, not `Design::new` with a format flag |
| Predicates read as questions | `is_palindromic`, `has_feature`, `contains_site` |
| Conversions follow Rust convention | `as_` borrowed and cheap, `to_` expensive, `into_` consuming |
| Units in the name where ambiguity is possible | `length_bp`, `free_energy_kcal_per_mol` |
| Coordinates state their convention | Documented on every type: 0-based, half-open. Stated once, everywhere. |

That last row is worth more than it looks. Off-by-one errors from mixed coordinate conventions are
endemic in bioinformatics, and every format we touch has an opinion. GenBank is 1-based inclusive,
BED is 0-based half-open, SBOL locations have their own rules. **Internally everything is 0-based
half-open, converted at the boundary, and the conversion is the adapter's job and is tested.**

---

## On object orientation

The brief asks for an OOP philosophy. The honest answer is that this codebase is not
object-oriented and should not be.

Rust favours data plus traits over inheritance, and the domain suits it: a `Design` is data, a
`Model` is a behaviour, and nothing here forms a natural is-a hierarchy. `Feature` does not inherit
from `Sequence`. A promoter is not a subclass of a feature; it is a feature whose role IRI happens
to be a promoter term, which is exactly why `role` is data rather than a type (subplan 06).

**What we take from OO:** encapsulation is real and enforced (`Cleared`'s private field is the
architecture), and interface-based polymorphism is real (`Model`, `ScreeningBackend`,
`ScreeningGateway` are traits with multiple implementations).

**What we reject:** inheritance hierarchies, and the instinct to model every noun as a class.
Python bindings expose classes because that is idiomatic Python, but they are façades over the same
flat data model, not a second type system with its own hierarchy.

---

## Open items

- [ ] Python binding ergonomics for `Prediction` and generics -> **15**
- [ ] Runtime enforcement test for the Python screening gate -> **17**
- [ ] Whether `unstable` needs a feature flag as well as a module -> revisit at 1.0
