---
name: issue-comment-response
description: Write replies to issues, issue comments, and PR comments. Use when responding to a bug report, question, feature request, or review comment on Integrase. Produces replies in the project's voice, specific to what was actually asked.
---

<!-- voice-check: teaches-by-example -->

This document quotes the patterns it forbids, as negative examples. The voice
checker skips it for that reason; the prose outside the examples still follows
the rules.
# Issue and comment responses

Built on `writing-voice`. Read that first; this adds what is specific to replying
to people.

## The standard

Reply like a maintainer who read the issue carefully and has something specific
to say. Not like a support system acknowledging a ticket.

Three things every reply should do:

1. **Answer what was actually asked.** Not the adjacent question that is easier.
2. **Be honest about uncertainty and scope.** Vague agreement is worse than a
   clear no, because it wastes the reporter's time waiting.
3. **Say what happens next**, if anything does.

## Never do these

- Open with "Thanks for reaching out!" or "Great question!". Thanking someone for
  a genuinely useful report is fine and specific. Reflexive gratitude is filler.
- Restate the issue back at the reporter. They wrote it.
- Say "we'll look into it" without saying who, when, or what would change.
- Agree vaguely with a feature request that is out of scope. Say it is out of
  scope, and why.
- Close an issue without explaining why, even when the reason feels obvious.
- Use the banned vocabulary from `writing-voice`.

## Before and after

### A bug report that is actually correct behaviour

> **Bad:** Thanks for reporting this! You're right that this behavior can be
> confusing. We'll look into improving the documentation around round-tripping.

> **Good:** This is working as intended, though the documentation should have
> told you that, so the docs are the bug here.
>
> SBOL 2 to 3 conversion canonicalises ontology URIs, and SBOL 3 removed
> `persistentIdentity` and `version` entirely, so a 2 to 3 to 2 round trip cannot
> reconstruct the original triples. We measured this: 7 of 7 fixtures differ, and
> the differences are almost all symmetric pairs on `role` and `type`.
>
> What you should get is a conversion report telling you what changed. If you did
> not see one, that is a real bug and worth a separate issue.
>
> Opened #47 to document this in the SBOL guide.

The good version answers the question, gives the evidence, separates the real
bug from the non-bug, and names the follow-up.

### A feature request that is out of scope

> **Bad:** That's an interesting idea! We'll add it to our roadmap for
> consideration in a future release.

> **Good:** Not planning to build this, and I want to be clear about why rather
> than leaving it open indefinitely.
>
> Integrase operates on sequence data and deliberately does not order DNA. Adding
> vendor ordering would put us on the other side of the boundary our screening
> gateway exists to guard, and the gateway is most of why this project is worth
> building.
>
> What is in scope, and may solve your actual problem: exporting a screened
> design in a vendor-ready format, so you paste it into their ordering flow
> yourself. That keeps the screening gate meaningful and saves you the manual
> step. If that would work, say so and I will file it.

Declines clearly, gives the real reason, offers the nearest thing that is in
scope, and asks rather than assumes.

### A question you cannot answer

> **Bad:** The model should work well for most bacterial promoters.

> **Good:** Honestly, I do not know, and the tool should tell you rather than me
> guessing.
>
> The RBS model is fitted on *E. coli* data. For *B. subtilis* it will return a
> prediction with `DomainVerdict::OutOfDistribution` and a reason naming the host
> mismatch, which means the interval is reported but its stated coverage does not
> hold.
>
> If you have measured expression data for *B. subtilis* constructs, that is
> exactly what we need to extend calibration, and I would be glad to talk about
> it.

Admits ignorance, explains what the tool does in that case, and turns it into a
specific ask.

### A screening-related report

> **Bad:** Thanks for flagging! We take security very seriously. Please email us
> privately.

> **Good:** Moving this to a private advisory, and I have deleted the sequence
> from the thread.
>
> Screening-bypass reports do not get a public reproduction recipe on any
> timeline, which is a deliberate departure from normal disclosure practice and
> is documented in SECURITY.md. To be clear about the limit: we withhold the
> recipe, never the fact. If deployed versions are not screening correctly, users
> get told promptly.
>
> Advisory opened, and I will have an assessment within 14 days.

Acts first, explains the unusual policy, states its limit, gives a timeline.

### A first-time contributor whose PR needs changes

> **Bad:** Thanks for the contribution! Just a few small changes needed before we
> can merge.

> **Good:** This is close. Two things before it merges, one of which is a rule
> you had no way of knowing about.
>
> `predict_strength` returns `f64`. Every predicted quantity has to come back as
> `Prediction<T>` so the interval and the domain verdict travel with it. That is
> R-0.1 in CONTRIBUTING and it is the least negotiable rule in the project. If
> the model has no calibration data yet, `Prediction::uncalibrated` is the right
> constructor.
>
> The test at `rbs.rs:88` asserts the function returns `Ok`, which cannot fail
> for that input. Worth replacing with one that pins the actual value, since an
> arithmetic error in the free energy calculation is the bug most likely to hide
> here.
>
> Neither is a big change and the rest looks good.

Specific, cites the rule, explains why the rule exists, says what to write
instead, and is honest that one rule was not discoverable.

## Tone calibration

- **Reporting a real bug:** thank them specifically for the part that was useful,
  usually the reproduction.
- **Frustrated:** address the substance. Do not mirror the frustration and do not
  perform patience at them.
- **Confused about scope:** be direct about what the tool does and does not do.
  Most scope confusion is our documentation's fault.
- **Proposing something good but not now:** say so, say what would change the
  answer.
- **Hostile:** reply once, factually, briefly. If it continues, the Code of
  Conduct applies and you stop replying.
