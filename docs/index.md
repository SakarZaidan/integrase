# Integrase documentation

Integrase is an open-source platform for designing, simulating, and screening
genetic constructs, built so that every prediction arrives with the uncertainty
that belongs to it.

**Start here if you are new:** [Reading a prediction](concepts/uncertainty.md).
It is the most important page in these docs, because the tool's central claim is
worthless if you do not know how to read what it hands back.

**Read this before trusting anything:** [Limitations](limitations.md).

---

## Status

Early. The workspace builds and the core types exist; the biophysical models,
SBOL import/export, circuit compilation, and rendering do not yet. Version
`0.0.0`, nothing published to a registry.

This documentation describes what exists plus the shape of what is coming, and
marks the difference. Pages describing unbuilt subsystems say so at the top.

## Map

| Section | For |
|---|---|
| [Reading a prediction](concepts/uncertainty.md) | Everyone, first |
| [Limitations](limitations.md) | Everyone, before relying on output |
| Getting started | New users. Being written. |
| Guides | Task-oriented walkthroughs. Being written. |
| Reference | Generated from the code. Not yet published. |
| Contributing | See [`CONTRIBUTING.md`](../CONTRIBUTING.md) |

## How `docs/` and `planning/` differ

They answer different questions, and neither duplicates the other.

**`docs/` is what the system is**, written for people using or working on it.

**[`planning/`](../planning/)** is **why it is that way**: 32 subplans, an
architecture document, and a written record of decisions including the ones that
turned out wrong. If you want to know why a boundary sits where it does, or why a
feature was cut and then restored, that is where the reasoning lives.

The rule: if it answers "how do I use it", it belongs here. If it answers "why is
it like this", it belongs in `planning/`.
