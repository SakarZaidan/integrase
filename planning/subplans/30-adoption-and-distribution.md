# 30. Adoption and distribution strategy

**Status:** decided
**Date:** 2026-09-15
**Decides:** how this tool reaches users, given that the previous plan had no answer
**Depends on:** subplans 02, 03, 22 (which established the problem and did not solve it)
**Feeds:** subplans 21, 32, and the re-score in 03 and 22

---

## The gap this subplan exists to close

Subplans 03 and 22 scored adoption **3 out of 10** and said, correctly, that planning had moved it
by exactly zero. That was an honest assessment of the plan at the time. It was not an honest
assessment of the problem, because **the plan contained no distribution strategy at all.** Its
entire answer was "publish a paper," which is one channel, and a slow one.

Scoring a dimension low and then not working on it is not calibrated honesty. It is giving up and
calling it rigour. This document is the work that should have been done before assigning that score.

The evidence that motivated the low score stands and is not being explained away: `sbol-rs` is
objectively excellent, comes from the standards body, and has 5 stars. **Quality does not produce
adoption in this field.** The conclusion to draw is that adoption needs a deliberate mechanism, not
that it is impossible.

---

## The channel the plan missed entirely: iGEM

Verified figures from the 2025 Grand Jamboree (Paris, October 2025): **4,600+ participants, 400+
teams, 50+ countries**, with registration for 2026 reported as a record. The Registry of Standard
Biological Parts holds **84,000+ parts** and belongs to iGEM.

That is not a niche. It is, by a wide margin, the largest concentration of people doing exactly what
Integrase is built for, and the previous plan did not mention it once.

### Why the fit is unusually exact

This is not "a big audience exists, let us market to it." Every single thing an iGEM team must do
maps onto a subsystem we are already building:

| What an iGEM team must do | Integrase subsystem |
|---|---|
| Source parts from the Registry | `integrase-sbol` + Registry import (subplan 04) |
| Compose parts into a construct | `integrase-core` |
| Design a genetic logic circuit | Circuit compilation (subplan 25) |
| Predict whether it will work | `integrase-sim` with calibrated intervals |
| **Complete a Team Safety Form, signed by their supervisor** | **`integrase-screen`** |
| **Check parts against iGEM's White List** | **The screening gateway is literally this** |
| Produce figures for the wiki and the Jamboree | SVG export with confidence grammar (subplan 13) |
| Document the design reproducibly | Verifiable design records (subplan 29) |

**The safety row is the one that matters most.** iGEM requires every team to submit a Team Safety
Form that a supervisor signs off, and maintains a White List of parts and organisms considered
safe. Today that process is substantially manual. Our screening gateway does the sequence-level
half of it automatically, produces an auditable record, and fails closed. That is not a feature we
would build for iGEM; it is a feature we are building anyway that happens to solve a problem 400
teams have every single year.

### The second-order reason this channel is worth more than its headcount

iGEM participants are overwhelmingly undergraduates at the start of their careers. **The tools they
learn become the tools they use as graduate students, and specify as PIs.** Cello, SBOLDesigner, and
SynBioHub all reached whatever adoption they have partly through this path.

More pointedly, for a project whose thesis is honest uncertainty: a student whose first genetic
design tool reports intervals and says when it is guessing learns a habit that the field currently
does not teach. That is a durable contribution independent of whether Integrase itself survives,
and it is worth stating because it is the strongest non-commercial argument for building this.

### What we actually do about it

Concrete, ordered by effort against return.

| Action | Effort | What it gets |
|---|---|---|
| **Registry import that works flawlessly**, including the Registry's real-world data quirks | Medium | Table stakes. A tool that chokes on Registry parts is unusable for this audience. |
| **iGEM Safety Form support**: produce the sequence-screening evidence a team attaches | Low, given the gateway exists | The single highest-value integration available |
| **White List as a first-class screening policy**, versioned alongside iGEM's annual revision | Low | Directly implements a rule teams must follow |
| A worked example built as an iGEM-style project, published as a tutorial | Low | Removes the "how do I start" barrier |
| Offer a workshop or office hours during the competition season | Medium | Direct contact with teams, and the fastest feedback loop available |
| Approach iGEM about listing in team resources | Low, but not in our control | Distribution at the source |

**Honest caveat:** the last row depends on a third party and cannot be planned into existence. The
rows above it do not, and they are worth doing regardless of whether iGEM ever endorses anything.

**A second honest caveat:** iGEM teams are seasonal, transient, and largely stop using tools after
the Jamboree. This is a high-throughput, low-retention channel. It is valuable for reach and for
the generational effect, and it will not produce a stable contributor base by itself. Subplan 32
handles that separately.

---

## The other channels, ranked honestly

### 1. Being inside the tools people already use

The strongest form of distribution for infrastructure is not being a destination. SynBioHub is
active (pushed within the last month) and has a documented plugin system where submit-plugins
transform uploaded files. A plugin that attaches calibrated predictions and a screening verdict to
designs in SynBioHub puts Integrase in front of that audience without asking anyone to adopt a new
tool.

Same argument for exporting SBML that iBioSim consumes, and for reading Cello output. **Subplan 02's
"connective tissue" positioning is a distribution strategy, not just an architectural one**, and
the previous plan never made that connection.

### 2. Teaching

Synthetic biology courses need a tool that is free, installable, and does not require a cluster. The
honest-uncertainty angle is pedagogically valuable in a way that is genuinely rare: it lets an
instructor show, in one screen, that a prediction and a measurement are different things.

Deliverable: a course module with exercises, released under a permissive license, that an instructor
can adopt without writing anything.

### 3. Publication

The previous plan's only channel, and still a real one. Two distinct papers, not one:

- **A tools paper** in a venue the field reads (ACS Synthetic Biology, Bioinformatics). This is how
  a tool becomes citable, and citation is how it becomes standard here.
- **A methods contribution**: calibrated uncertainty for genetic part prediction, with the
  evaluation methodology public. This is the one with scientific weight, and subplan 22 already
  identified it as the highest-leverage single action available.

### 4. The biosecurity community

A distinct audience the previous plan ignored. IBBIS, NTI|bio, the Johns Hopkins Center for Health
Security, and the iGEM safety committee all care about screening reaching the design stage. This
audience is small but disproportionately influential, well-funded, and actively looking for tools
that operationalise the OSTP framework. It also connects directly to subplan 32's funding path.

### 5. Conferences

SEED, SynBioBeta, and the iGEM Jamboree. Low cost, slow return, worth doing once there is something
to show. Not a priority before Phase 1 ships.

---

## What we measure

Vanity metrics are worse than no metrics, because they let you feel successful while failing. GitHub
stars are explicitly not a target: `sbol-rs` has 5 and is excellent.

| Metric | Why it is real |
|---|---|
| **iGEM teams using it in a season** | Countable, and directly tied to the primary channel |
| **Citations** | The currency this field actually runs on |
| **PyPI downloads, excluding CI** | Weak but directional, and harder to game than stars |
| **Designs screened** (count only, never content) | Measures the differentiating feature actually being used |
| **External contributors who land a second PR** | The bus-factor metric. First PRs are cheap; second ones mean something. |
| **Institutions running their own screening backend** | Deepest form of adoption available |

---

## The honest limits of this strategy

1. **None of this is executed yet.** This is a plan for adoption, not adoption. The distinction
   matters and subplan 22's re-score must respect it.
2. **iGEM is seasonal and transient.** High reach, low retention.
3. **iGEM endorsement is not ours to grant ourselves.** We can build the integration; we cannot make
   them list us.
4. **Publication takes 12 to 18 months** from submission to influence, and depends on the
   calibration result actually working (subplan 08's critical path).
5. **A tool still has to be good.** Distribution amplifies quality; it does not substitute for it,
   and the field's dormant repositories are full of tools that had distribution and stopped being
   maintained.

---

## What this changes for the score

Adoption was scored 3 on the basis that the plan had no mechanism. It now has a specific one, aimed
at a verified audience of 4,600+ people per year whose mandatory safety workflow our differentiating
subsystem implements directly.

**That is a change in the plan's quality, not in the project's current adoption, which remains
zero.** Subplan 03's re-score must separate those two things explicitly, because conflating them is
how a planning document starts lying.

---

## Open items

- [ ] Verify the iGEM White List's machine-readable form, if one exists
- [ ] Verify what the Team Safety Form actually requires, field by field
- [ ] Confirm SynBioHub's plugin API is suitable for attaching predictions
- [ ] Build the worked iGEM-style example once Phase 1 ships
- [ ] Identify the right venue and format for the tools paper
