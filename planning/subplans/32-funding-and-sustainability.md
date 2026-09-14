# 32. Funding and institutional sustainability

**Status:** decided, with one eligibility blocker found
**Date:** 2026-09-15
**Decides:** how the project is funded and how it survives its first maintainer
**Depends on:** subplans 03, 19, 22, 30
**Feeds:** `REQUESTS.md`, and the re-score in 03 and 22

---

## The gap this closes

Subplan 03 scored sustainability **3 out of 10** and subplan 19 correctly diagnosed it as a
bus-factor problem. Both were right about the diagnosis and neither proposed a funding path, which
is the actual mechanism by which every comparable tool in this field died. Cello, SBOLDesigner,
paraSBOLv, and TinkerCell did not fail technically. They ran out of funded attention.

Subplan 19's mitigations (planning folder as succession artifact, ADRs, low-friction onboarding) are
real and they address the *knowledge* half of the problem. They do nothing about the *attention*
half. This document handles that.

---

## The funding landscape, researched

### NSF PESOSE, and the rename that matters

The NSF POSE programme (Pathways to Enable Open-Source Ecosystems) has been **archived and replaced
by NSF 26-506, PESOSE: Pathways to Enable *Secure* Open-Source Ecosystems.**

| | |
|---|---|
| Track 1 | up to **$300,000** |
| Tracks 2 and 3 | up to **$1,500,000** |
| Most recent deadline | **1 September 2026** |
| Track record | POSE invested over $26M across 19 Phase II projects |

**The rename is the finding.** A programme that was about open-source ecosystems is now explicitly
about *secure* open-source ecosystems. Integrase is an open-source ecosystem play (subplan 02's
connective-tissue positioning) whose distinguishing subsystem is a biosecurity screening gateway
implementing a federal framework. It is difficult to construct a closer fit to a programme's stated
scope.

**The timing is bad and should be stated plainly: the deadline was 1 September 2026, two weeks ago.**
The next cycle is roughly twelve months out. That is not a disaster, because Phase 1 needs to exist
before a credible proposal can be written anyway, and a proposal with a working tool and a
calibration result behind it is far stronger than one with a plan. But it does mean **no NSF money
before late 2027 at the earliest**, and any plan assuming otherwise is wrong.

### The eligibility blocker

PESOSE eligibility is limited to US-based institutions of higher education, non-profit non-academic
organisations, and for-profit organisations with research capability.

**An individual is not eligible.** This is a hard constraint and it was not visible until this
research. Three routes exist:

| Route | What it costs | Assessment |
|---|---|---|
| Academic affiliation, with a PI as the applicant | A relationship, and shared credit | **Most likely to work.** Also directly supplies the institutional relationship subplan 22 identified as the second-highest-leverage action. |
| Form a non-profit | Real legal and administrative work, ongoing | Heavy for a project this early |
| Form a company | Changes the project's character | Contradicts the positioning |

**This is now the single most consequential non-engineering item in the project**, because it gates
both the funding path and the sustainability score. It goes to `REQUESTS.md` as an owner decision,
since choosing an institutional home is not an engineering call.

### Other funders worth approaching, and why

| Funder | Fit | Note |
|---|---|---|
| **NSF Safe-OSE** | Security and privacy in open-source ecosystems, up to $15M programme total | Same family as PESOSE, same eligibility constraint |
| **Chan Zuckerberg Initiative, EOSS** | Essential Open Source Software for Science. Funds maintenance of tools science depends on. | Historically the best fit for *maintenance* rather than novelty, which is exactly the sustainability gap |
| **Open Philanthropy** | Funds biosecurity work substantially, including tooling | The screening gateway and dual-use posture (subplan 31) are the relevant surface |
| **NTI \| bio and IBBIS** | Both work directly on nucleic acid synthesis screening | More likely partners than funders, and partnership has its own value |
| **Wellcome, Schmidt Sciences** | Broader science-tooling funders | Lower fit, non-trivial process |

**The honest ordering:** biosecurity funders are the most natural fit for what makes this project
distinctive, and the most likely to move quickly, because operationalising nucleic acid synthesis
screening at the design stage is squarely inside their stated interests. NSF is larger and slower.
CZI EOSS is the right ask for year three, not year one.

---

## Why this project is fundable, stated as a funder would evaluate it

Worth writing down because it doubles as the skeleton of an actual proposal.

1. **It implements a federal framework.** The OSTP Framework for Nucleic Acid Synthesis Screening
   moves to a 50 bp threshold and an expanded sequence-of-concern definition in October 2026.
   Integrase puts that screening at the design stage, before an order exists.
2. **It is an ecosystem, not an application.** SBOL interop, Cello import, SBML export, SynBioHub
   integration. PESOSE funds ecosystems specifically.
3. **It addresses a documented scientific problem.** Published prediction accuracy for genetic parts
   ranges from R² above 0.8 to below 0.2 depending on dataset, and no tool reports that uncertainty
   to users. That is a reproducibility problem with a concrete remedy.
4. **It reaches students at scale.** 4,600+ iGEM participants annually (subplan 30), which is a
   broader-impacts story that writes itself.
5. **It is already open, licensed, and governed.** Apache-2.0, branch protection, a threat model, a
   dual-use policy, and a disclosure policy. Most proposals promise these.

**The weakness a reviewer will find immediately:** no users, no publication, and one contributor.
That is precisely why Phase 1 and subplan 30 come before the proposal rather than after it.

---

## Sustainability beyond grants

Grants are attention with an expiry date. Every dead tool in this field was grant-funded once.

| Mechanism | What it does | Honest assessment |
|---|---|---|
| **Institutional home** | A lab or centre that treats the tool as infrastructure | The single most durable option, and the hardest to arrange |
| **Consortium** | Several institutions running their own screening backends, each with a stake | Plausible only after real adoption |
| **Support and integration contracts** | Companies paying for institutional deployment | Real revenue, and changes the project's character. Needs a decision before it is accepted, not after. |
| **Fiscal host** | NumFOCUS or similar handling the legal and financial layer | Removes the non-profit formation burden, requires demonstrated adoption to join |
| **Contributor pipeline** | iGEM participants and students who stay | Slow, real, and the only mechanism that fixes bus factor rather than deferring it |

**What we explicitly will not do:** an open-core model that puts the screening gateway or the
uncertainty layer behind a paid tier. Those are the two things that make the tool worth having, and
gating either would make the project's stated principles into marketing.

---

## Bus factor, concretely

Subplan 19 identified the risk. These are the mitigations, ordered by how much they actually help.

1. **The planning folder.** Thirty-one subplans recording decisions, rejected alternatives, and
   reasoning. A successor can reconstruct intent without the author. Already done.
2. **An institutional home.** Converts a person into a position. Not done, and it is the real fix.
3. **A second maintainer with merge rights**, before it is urgent. Subplan 19's escalation from 0 to
   1 required approvals depends on this existing.
4. **Documented, reproducible release process** (subplan 18), so releasing does not require the
   original author's undocumented knowledge.
5. **No private infrastructure.** Everything needed to build, test, and release is in the repository
   or is a documented free service. A project that depends on one person's laptop configuration dies
   with that laptop.

Point 5 is worth emphasising because it is cheap and frequently neglected. CI running the same
`just` targets a contributor runs locally is a bus-factor control, not just a convenience.

---

## The plan, sequenced

| When | Action |
|---|---|
| **Now** | Decide the institutional route. Owner decision, `REQUESTS.md` R-004. |
| **During Phase 1** | Build the thing. A proposal without a working tool is weak, and the eligibility question needs answering regardless. |
| **After Phase 1a** | Publish the calibration result. This is the credential everything else rests on. |
| **After publication** | Approach biosecurity funders. Faster and better-fitting than NSF. |
| **~Sept 2027** | PESOSE cycle, if an eligible entity exists by then. |
| **Year 3** | CZI EOSS for maintenance, if adoption is demonstrable. |

---

## Honest limits

1. **No funding before late 2027 at the earliest**, and the PESOSE deadline for this year has passed.
2. **The eligibility blocker is real and unresolved.** Without an institution, the largest funding
   source is simply unavailable.
3. **US-centric.** Most identified sources require US affiliation, which constrains the options.
4. **Funding is not a business model.** It buys time and ends.
5. **Everything here is a plan.** The project currently has no funding, no institution, and one
   contributor. The re-score must reflect the plan's quality, not pretend the plan is executed.

---

## New `REQUESTS.md` entry

**R-004: decide the institutional route.** Academic affiliation, non-profit, or company. This gates
NSF eligibility, is the highest-leverage sustainability action available, and is a personal and
strategic decision rather than an engineering one.

## Open items

- [ ] Confirm PESOSE's next cycle dates when published
- [ ] Verify CZI EOSS's current cycle and eligibility
- [ ] Identify specific labs where an affiliation would be plausible
- [ ] Draft the one-page project summary that every funder conversation starts with
