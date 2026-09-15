# Security Policy

## Reporting a vulnerability

**Do not open a public issue.** Use
[GitHub Security Advisories](https://github.com/SakarZaidan/integrase/security/advisories/new).

You will get an acknowledgement within 7 days and an assessment within 14.

## One class of report is handled differently, and you should know before you report

Standard open-source practice is coordinated disclosure on a fixed timeline,
then publish. We follow that for most things. **We do not follow it for
screening-bypass vulnerabilities**, and this is a deliberate departure from norms
that many contributors hold strongly.

| Class | Handling |
|---|---|
| Ordinary vulnerabilities (API, parsing, dependencies, auth) | Coordinated disclosure, 90 days, public advisory, credit as you prefer |
| **Screening bypass or evasion technique** | Reported privately, coordinated with the affected screening backend provider first, fixed quietly. **No public reproduction recipe, on no timeline.** Credit offered with the technique withheld, if you agree. |

The reasoning: a public writeup of how to evade biosecurity screening is itself a
harm. The vulnerability is not only in our code, it is in a control that exists
to make a class of misuse harder.

### The limit of that policy, stated explicitly

**We do not withhold information users need to protect themselves.** If a bypass
means deployed versions are not screening correctly, users are told promptly and
clearly that this is the case, which versions are affected, and what to do.

What is withheld is the **reproduction recipe**, never the **fact**.

If you disagree with this policy, that is a reasonable position to hold. We would
rather you knew about it before reporting than discovered it afterwards, which is
why it is stated here rather than in a footnote.

## What is in scope

- The screening gateway and anything that could cause it to fail open
- The uncertainty layer, where a bug could cause a prediction to be presented as
  more confident than it is
- Memory safety, parsing, and denial of service in `integrase-core`
- Dependency vulnerabilities across the Rust and Python ecosystems
- The hosted service and its authentication, once Phase 3 exists

## What is out of scope, and why

Two things are documented non-goals rather than unreported weaknesses, and both
appear in our threat model:

1. **Copying a sequence out of the interface by hand.** No gate can prevent this.
   Blocking copy-paste in a design tool would make it unusable and would stop
   nobody.
2. **Running a modified local build with screening removed.** It is open-source
   software on the user's own machine. Anyone willing to patch out the gate was
   never going to be stopped by it.

Reports of either are welcome as discussion, but they are known and accepted,
not vulnerabilities.

## What Integrase is not

**Integrase is not a biosecurity control.** It is a design tool with a screening
gate. The actual control is the synthesis provider's own screening, which is
where the IGSC protocol and the OSTP framework operate on real orders.

Our layer is defence in depth: it prevents accidents, catches the uninformed,
creates an audit trail, and declines to be a convenient instrument. It does not
stop a determined, knowledgeable adversary, and we will not claim it does.
Overstating a tool's protective value encourages people to rely on it.

## Supported versions

Pre-1.0. Only the latest release receives fixes. This changes at 1.0.
