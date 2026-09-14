# 31. Ethics, dual-use oversight, and institutional integration

**Status:** decided
**Date:** 2026-09-14
**Decides:** the ethical architecture, the regulatory frameworks it implements, and the governance
that holds it
**Cross-references:** subplans 10 (threat model), 24 (screening operations), 26 (generative
constraints). Those cover mechanism. This covers what the mechanism is for, and who decides.

---

## Why restoring scope raised the stakes

Subplan 02 restored circuit compilation, generative design, and genome-scale search. Each of those
is individually reasonable and together they change what this tool is.

A calibrated RBS predictor is a measurement instrument. **A platform that compiles a specification
into a working genetic circuit, proposes optimised sequences for its parts, and searches genomes at
scale is a design automation system**, and design automation lowers the expertise required to do
the thing being automated. That is the entire point, and it is also the entire risk. Lowering the
barrier is good when the barrier was tedium and bad when the barrier was expertise that functioned
as a safeguard.

This subplan exists because "we added a screening gate" is not a sufficient answer to that.

---

## The principle, and its limit

**Never restrict thinking, designing, simulating, or analysing. Restrict only the boundary toward
physical realisation, and the automation of harm.**

The first half is from the brief and it is right. A tool that refuses to let a scientist think is
useless and will be abandoned for one that does not. Analysis is not action.

**The second clause is added here**, and it is the part the brief did not anticipate. Automation is
different from analysis. A user analysing a dangerous sequence is learning. A system optimising a
dangerous sequence on the user's behalf is *doing work toward* a harmful outcome, and the fact that
it happens in a sandbox does not change what the work is.

So the constraint is not uniform:

| Activity | Constraint |
|---|---|
| Read, import, view, annotate any sequence | **None.** Including regulated ones. Analysis is not action. |
| Simulate, predict, analyse | **None.** |
| Compile a circuit from a specification | None, with DURC category detection (below) |
| **Generate or optimise sequences** | **Task-scoped, output screened** (subplan 26) |
| Export toward physical synthesis | Screening gate (subplan 24) |
| **Automate optimisation of a flagged agent** | **Refused.** This is the added clause. |

The last row is the one that departs from a pure boundary-only model, and it does so deliberately.
Running an optimiser to improve a property of a select agent is not thinking about biology. It is
the automation doing the work.

---

## Implementing the DURC and PEPP framework

The brief asks for biosafety architected in rather than bolted on. The strongest available version
of that is to implement the framework that actually governs this research, rather than inventing our
own ethics.

**The United States Government Policy for Oversight of Dual Use Research of Concern and Pathogens
with Enhanced Pandemic Potential**, released 2024-05-06 and effective 2025-05-06, unifies the 2012
and 2014 DURC policies and the 2017 HHS P3CO framework. It enumerates 90+ agents and toxins, nine
categories of experiment, and two categories of research requiring proactive assessment by the
principal investigator at grant application and annual reporting.

Sources:
[USG Policy](https://bidenwhitehouse.archives.gov/wp-content/uploads/2024/05/USG-Policy-for-Oversight-of-DURC-and-PEPP.pdf),
[Implementation Guidance](https://bidenwhitehouse.archives.gov/wp-content/uploads/2024/05/USG-DURC-PEPP-Implementation-Guidance.pdf),
[NIH OSP announcement](https://osp.od.nih.gov/us-government-releases-policy-for-oversight-of-dual-use-research-of-concern-and-pathogens-with-enhanced-pandemic-potential/).

### Category detection: screening the experiment, not just the sequence

This is the genuinely novel contribution of this subplan, and it follows from the observation that
**sequence screening answers the wrong question for DURC.**

Sequence screening asks: is this DNA on a list? The DURC policy does not regulate sequences. It
regulates *experiments*: taking an agent on the list and doing one of nine things to it. An
experiment can be entirely DURC Category 1 while every individual sequence in it screens clean.

Integrase knows things a sequence screener does not. It knows the host organism, the design intent
expressed as a circuit specification, the function of the parts, and what an optimisation objective
was set to. That is enough to detect the *shape* of a category-1 or category-2 experiment.

```rust
pub struct DurcAssessment {
    /// Agents in the design matching the policy's enumerated list.
    agents: Vec<AgentMatch>,
    /// Which of the policy's experiment categories this design's structure and
    /// stated intent resembles.
    category_signals: Vec<CategorySignal>,
    /// Never a determination. Always a routing decision.
    recommendation: OversightRoute,
    policy_version: PolicyVersion,
}

pub enum OversightRoute {
    /// Nothing matched. Proceed.
    NoSignal,
    /// Signals present. Route to the institutional review body. Integrase does
    /// not decide; it makes sure the right humans are asked.
    InstitutionalReview { signals: Vec<CategorySignal>, draft: ReviewSubmission },
    /// Category 1 or 2 shape plus a listed agent. Export gated pending
    /// documented institutional approval.
    RequiresApproval { signals: Vec<CategorySignal>, draft: ReviewSubmission },
}
```

**Three design rules that make this useful rather than obstructive:**

1. **Integrase never makes a DURC determination.** It is not qualified to and the policy does not
   give software that authority. Determinations are made by an Institutional Review Entity. Our job
   is detection and routing, and being clear about the difference is the whole point.
2. **It drafts the paperwork.** `ReviewSubmission` pre-fills what the institution needs: the agents
   involved, which category signals fired, the design, and the intent. A researcher who would
   otherwise skip review because the form is tedious is far more likely to submit a form that is
   already written.
3. **False positives are acceptable, false negatives are not.** The cost of an unnecessary review is
   a researcher's irritation. The cost of a missed one is the thing this exists to prevent. The
   detector is deliberately over-sensitive and says so to the user.

**Honest limitation:** detection operates on stated intent and design structure, and both can be
withheld or misstated. A user who does not tell the tool what they are building defeats category
detection entirely. This catches the researcher who did not realise their work was DURC, which is
the common and important case, and it does not catch someone concealing it. Same honest posture as
subplan 10.

---

## Institutional integration

Ethics that route to a person only work if the routing reaches a real process.

| Body | Integration |
|---|---|
| **Institutional Biosafety Committee (IBC)** | `ReviewSubmission` exports in a form an IBC submission can consume. Approvals are recorded against the design and gate export. |
| **Institutional Review Entity (DURC/PEPP)** | Same mechanism, different form, per the 2024 policy's institutional oversight requirements. |
| **iGEM Safety and Security Committee** | 448 teams in 2026, and any part not on the White List requires an approved Check-In Form before use. **Integrase validates against the White List and drafts the Check-In.** See subplan 28. |
| **Synthesis provider screening** | The real enforcement point. We are a prior layer, not a replacement. |

**The iGEM integration is the highest-leverage ethics work in the project**, and that is not a
marketing observation. Ten thousand students a year, most encountering biosafety review for the
first time, currently meeting it as a form that blocks them. A tool that makes the safety step feel
like part of designing rather than an obstacle to it shapes how an entire generation relates to
biosecurity. That is worth more than any control we could add to the export path.

---

## Governance of the project itself

A project making these decisions should not make them alone, and a single maintainer deciding
biosecurity policy is a governance failure regardless of how carefully they decide.

### Biosecurity Advisory Group

**Purpose:** review and approve changes to screening logic, DURC detection, generative task
definitions, and the disclosure policy. Not a rubber stamp and not a logo on a website.

**Composition sought:** someone with biosecurity policy expertise, a practising synthetic biologist,
an institutional biosafety officer, and a security engineer. Three to five people.

**Authority, stated concretely:**

- Changes to screening or DURC logic require review. This extends R-7.3 from "human sign-off" to
  "sign-off by someone qualified."
- The group may require a capability be removed. That authority is real or the group is decorative.
- Disagreement is published. If the group advises against something and the project does it anyway,
  that is on the record.

**Honest status: this does not exist and cannot be created unilaterally.** It requires people agreeing to
serve. It is `REQUESTS.md` R-004, and until it exists the project says it does not have one rather
than implying informal review is equivalent.

**Interim, until it exists:** changes in these areas are held to the same written criteria, the
reasoning is published in the PR, and the change is announced rather than merged quietly. That is
weaker than review by qualified people and should be described as weaker.

### Red-teaming

Announced, scheduled, and published, because a safety property nobody has attacked is a safety
property nobody has tested.

| Target | Question |
|---|---|
| Export gate | Can a design reach a synthesis-bound path unscreened, through any surface? |
| Generative tasks | Can permitted operations compose into detectability reduction (subplan 26)? |
| Category detection | What DURC-shaped designs pass without signal? |
| Oracle behaviour | Does refusal behaviour leak what the screener catches? |
| Fail-open | Can any fault make the gate pass rather than refuse? |

Findings follow subplan 24's disclosure policy: bypasses coordinated with backend providers before
publication, everything else on the normal timeline. **Red-team results are published even when
unflattering**, because a project that only publishes results making it look good has made its
transparency worthless.

### Transparency report

Annually: screening volume and refusal counts in aggregate, red-team findings, advisory group
composition and any published disagreements, and any government or institutional request received
and how it was handled.

**No user-identifying information, ever.** Subplan 24 already commits to not reporting users, and a
transparency report that deanonymised them would break that commitment in the name of openness.

---

## What we refuse, and what we do not

A capability list is more honest than a values statement, so both directions are enumerated.

### Refused, by construction

| Refused | Enforcement |
|---|---|
| Optimising a sequence to reduce screening detectability | No task expresses it; red-teamed for composition (subplan 26) |
| Automated optimisation of a listed agent's properties | Category detection blocks the optimisation path |
| Synthesis-bound export without screening | Type system, plus tested checks at every boundary |
| Generating without screening the output | Not exposed, not configurable |
| Showing a screened-out generated candidate | Oracle surface |

### Explicitly permitted, and defended

Equally important, because a tool that refuses too much is abandoned and a tool nobody uses protects
nobody.

| Permitted | Why |
|---|---|
| Importing, viewing, annotating a regulated sequence | Analysis is not action. Researchers legitimately work with these. |
| Simulating and predicting on any sequence | Same. |
| Working on select agents with institutional approval | The approval mechanism exists precisely for this. Blocking approved work would be substituting our judgment for an IBC's. |
| Self-hosting with institutional policy | Subplan 24. Institutions have rules and legitimate authority. |
| Full offline operation | A tool requiring a network call to function is a tool that can be switched off remotely, and that is its own governance problem. |

**The last row is a deliberate trade against control.** Offline operation means we cannot revoke
access or push a policy update to a running instance. We accept that, because software that
phones home to check whether a scientist is allowed to compute is a worse precedent than the
marginal control it buys.

---

## Ethical risks we carry and do not fully mitigate

Written because a project that lists only mitigated risks has not looked hard.

1. **Design automation lowers the expertise barrier.** That is the product's purpose and its
   principal risk, and no control removes it. What we do is ensure the lowered barrier comes bundled
   with screening, category detection, and routing to review, so the path of least resistance
   includes the safeguards.
2. **We cannot stop a determined, knowledgeable adversary.** Subplan 10 says this about screening
   and it is equally true here. Our controls raise effort and leave evidence.
3. **Open source means the controls are removable.** Anyone can fork and delete the gate. Apache-2.0
   permits it. The alternative, a closed tool, would be less trustworthy and no more effective
   against the adversary who would bother.
4. **Category detection depends on user-stated intent.** Withholding intent defeats it.
5. **We may be wrong about where the lines are.** Biosecurity policy is contested and evolving. We
   implement a current framework rather than a timeless ethic, and the framework will change. That
   is why policy is versioned configuration and why the advisory group matters.

---

## Open items

- [ ] **R-004: recruit a Biosecurity Advisory Group.** Cannot be done unilaterally.
- [ ] Read the full 2024 USG DURC/PEPP policy and implementation guidance. This subplan works from
      summaries and the agent list and nine experiment categories must be implemented from the
      primary source, not from a search result.
- [ ] Verify the current status of the framework: the May 2025 Executive Order directed revision of
      the related nucleic-acid synthesis framework, and DURC policy may move too.
- [ ] Design `ReviewSubmission` against a real IBC form
- [ ] Establish the red-team schedule and find external participants
