# Code of Conduct

## The short version

Be decent to people. Argue about the work, not the person. If you would not say
it to a colleague you respect, do not put it in a pull request.

This document exists so that when something goes wrong there is a written
standard to point at, and a named person who acts on it. A code of conduct with
no enforcer is a decoration.

## Scope

Applies in the repository (issues, pull requests, discussions, commit messages,
code comments) and in any space where someone is representing the project.

## Expected behaviour

- Assume competence and good faith. The person you are replying to has context
  you may not.
- Criticise work specifically. "This will fail on empty input" is useful.
  "This is sloppy" is not.
- Accept that maintainers say no. Scope is finite, and a rejected feature is not
  a rejected person.
- Be honest about uncertainty. This project's entire thesis is that overstating
  confidence is harmful, and that applies to how we talk to each other as well as
  to what the software reports.

## Unacceptable behaviour

- Harassment, whether sustained or a single serious incident.
- Demeaning, discriminatory, or unwelcome comments about a person, including
  about their identity or their level of experience.
- Publishing others' private information.
- Deliberate intimidation, stalking, or disruption.
- Sustained derailing of technical discussion.

## The dual-use clause

This clause is specific to this project, and it is not boilerplate.

Integrase is a tool for designing biological sequences. It carries a biosecurity
screening gateway for good reasons. The following are grounds for **immediate and
permanent removal**, without the graduated process below:

- Requesting help using Integrase, or any part of it, to design something
  intended to cause harm.
- Using the issue tracker, discussions, or pull requests to solicit that help
  from others.
- Contributing code whose purpose is to weaken, bypass, or silently disable the
  screening gateway.

This is in the code of conduct rather than only in the threat model because it
governs **conduct**, not architecture. The threat model describes what an
adversary might attempt; this describes what participation in this community
requires.

**What this clause is not.** It is not a prohibition on discussing biosecurity,
on reporting vulnerabilities in the screening path, or on legitimate research
involving regulated sequences. Security research is welcome and has its own
process in [`SECURITY.md`](SECURITY.md). Researchers work with controlled agents
legitimately every day, under institutional oversight, and a tool that treated
that as suspicious would be useless to them.

The distinction is intent, and in the rare ambiguous case we will ask before we
act.

## Reporting

Report to the project owner, **Sakar Zaidan**, via
[a private security advisory](https://github.com/SakarZaidan/integrase/security/advisories/new)
or by direct message on GitHub.

Reports are handled confidentially. You will get an acknowledgement within a
week. If your report concerns the owner, and there is currently no second
maintainer to escalate to, say so and it will be handled through a neutral third
party rather than ignored.

**This is a bus-factor problem and it is acknowledged rather than hidden.** A
single-maintainer project cannot offer a genuinely independent appeal route. When
a second maintainer exists, this section changes, and that is tracked as part of
the governance plan.

## Enforcement

| Step | What happens |
|---|---|
| 1. Correction | A private note explaining what was wrong and why. Most things end here. |
| 2. Warning | A formal warning, with a defined period of no interaction with the people involved. |
| 3. Temporary ban | Removal from all project spaces for a stated period. |
| 4. Permanent ban | Permanent removal. |

The owner decides the step. Steps may be skipped for serious incidents, and the
dual-use clause above skips directly to step 4.

If you think an enforcement decision was wrong, say so. You will get a
considered answer rather than silence, and the limits on independent appeal are
stated honestly above.

## Attribution

Adapted from the [Contributor Covenant](https://www.contributor-covenant.org),
version 2.1, with the dual-use clause, the named enforcement route, and the
honest statement about single-maintainer appeal added for this project.
