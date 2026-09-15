---
name: writing-voice
description: The voice every document, commit message, PR description, code comment, and issue reply in this repository is held to. Load before writing prose of any kind for Integrase, and when reviewing someone else's prose. Other skills reference this one rather than restating the rules.
---

<!-- voice-check: teaches-by-example -->

This document quotes the patterns it forbids, as negative examples. The voice
checker skips it for that reason; the prose outside the examples still follows
the rules.
# Writing voice

This is the single definition. `pr-review`, `issue-comment-response`, and
`planning-sync` all defer here rather than each carrying a slightly different
version that drifts.

## Why this is enforced rather than suggested

Integrase's entire product thesis is that it will not overstate what it knows. A
tool that reports calibrated uncertainty while its README promises the world has
undermined itself before anyone runs it. The prose and the product make the same
claim, so the prose is held to the same standard.

`tools/check-voice.sh` catches the mechanical subset in CI. It cannot catch tone,
rhythm, or a sentence that would fit any project. That is what this skill is for.

## Hard rules

### No em dashes

Use a period, a comma, a colon, or restructure. If you reach for one, that is
the signal to rewrite the sentence, not to keep the dash.

### Banned vocabulary

`delve`, `seamless`, `furthermore`, `moreover`, `it's worth noting that`,
`unlock`, `elevate`, `game-changing`, `cutting-edge` as filler, `robust` as a
filler adjective, `leverage` **as a verb**.

Two documented exceptions, both already in the checker:

- `leverage` as a **noun** is ordinary precise English. "Ordered by leverage" and
  "the highest-leverage action" both stay. Rewording them to satisfy a regex
  would make the writing worse.
- `robustness analysis` and similar defined technical uses are fine. The checker
  flags bare `robust` for a human to look at rather than hard-failing it.

### No reflexive rule-of-three

"Fast, flexible, and future-proof" is a cadence, not a thought. If three parallel
adjectives genuinely are clearest, keep them. The checker warns rather than
fails here for exactly that reason.

### No hedging filler

"It is important to note that" and "it should be mentioned that" carry no
information. If it is worth saying, say it.

### No manufactured enthusiasm

No exclamation points used to supply energy the content does not have.

### Vary sentence length

Real careful writing has rhythm. A run of same-length sentences reads as
generated even when the content is good.

### State uncertainty specifically

Not "this may have limitations" but "this model's accuracy on out-of-distribution
sequences has not been validated."

### Nothing that would fit another project

If a sentence could be pasted unchanged into a different repository's README,
rewrite it to say something about this one.

## Before and after

Real examples, drawn from this repository's own history.

**A feature was cut**

> Bad: We've made the difficult but necessary decision to streamline our scope in
> order to deliver a more focused, robust MVP.

> Good: Cut circuit logic synthesis from v1.0. Cello already did it well and
> reimplementing it is a multi-year project.

The first says nothing checkable. The second names the thing and the reason.

**Reporting a benchmark**

> Bad: Our high-performance indexing delivers blazing-fast search at genome
> scale.

> Good: 120 ns per 20 bp query, flat across a twentyfold increase in index size.
> Memory is the constraint, not speed: 39.4 bytes per base, which projects to
> 113.8 GB for a human genome.

**Admitting an error**

> Bad: We've identified an opportunity to enhance our benchmarking methodology.

> Good: I benchmarked one library's default configuration and concluded a
> capability was impossible. The measurement was right and the inference was
> wrong, and it cost a feature.

**A commit message body**

> Bad: This commit refactors the screening module to improve code quality and
> maintainability.

> Good: Screening moves to its own crate so "does this PR touch screening?" is
> answerable from the diff path alone, which is what the pr-review skill needs to
> trigger extra scrutiny.

**Describing an unfinished thing**

> Bad: Integrase is a next-generation platform that empowers researchers to
> unlock the potential of synthetic biology.

> Good: Integrase is early. The planning is thorough and the implementation has
> just started. If you are looking for something to use today, this is not yet
> that.

## Applying this to specific surfaces

**Commit messages.** Conventional Commits for the subject. The body explains
**why**, because the diff already shows what. A body that narrates the diff in
prose has told the reader nothing.

**PR descriptions.** Lead with the decision or the finding, not a summary of
files changed. If the PR corrected an earlier mistake, say so plainly and say
what the mistake was.

**Code comments.** Document why, invariants, units, and coordinate conventions.
`// increment the counter` above `counter += 1` is noise, and the
`ai-code-review` skill treats it as a defect.

**Planning documents.** State what was decided and why, and what was rejected
and why. A planning document that only records what survived is missing the half
that stops questions being re-litigated.

**Issue replies.** See `issue-comment-response`, which is built on this skill.

## The test

Read it aloud. If it sounds like a person who understands this specific project
explaining something to a colleague they respect, it passes. If it sounds like a
press release, an assistant being helpful, or a template with the project name
substituted in, rewrite it.
