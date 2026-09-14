# Requests to the project owner

Things only the project owner can provide: access, credentials, datasets that are not freely
downloadable, and decisions that are genuinely business or personal preference rather than
engineering calls.

Rules for this file:
- Every entry states **what** is needed, **why**, and **what happens once it arrives**.
- Resolved entries are marked resolved **in place**. Nothing is deleted, so the history of what
  was asked and answered survives.
- This file is not a place to park decisions that engineering judgment can settle. Freely
  available data gets fetched, not requested.

Last reviewed: 2026-09-14

---

## OPEN

### R-004: Decide the institutional route

**Status:** OPEN, raised 2026-09-15
**Blocks:** NSF eligibility entirely. The largest identified funding source is unavailable without
this.

**What is needed.** A decision on whether this project lives under an academic affiliation, a
non-profit you form, or a company.

**Why.** Research in subplan 32 found that NSF PESOSE (the successor to POSE, now "Pathways to
Enable *Secure* Open-Source Ecosystems", NSF 26-506) restricts eligibility to US-based institutions
of higher education, non-profit non-academic organisations, and for-profit organisations with
research capability. **An individual is not eligible.** Track 1 is up to $300,000 and Tracks 2 and 3
up to $1,500,000.

This was not visible until the funding research and it is now the most consequential non-engineering
item in the project. It gates the funding path, and subplan 22 identified an institutional
relationship as the second-highest-leverage action available for sustainability.

| Route | Cost | Assessment |
|---|---|---|
| Academic affiliation with a PI applicant | A relationship, shared credit | Most likely to work, and supplies the institutional relationship anyway |
| Form a non-profit | Real legal and ongoing administrative work | Heavy this early |
| Form a company | Changes the project's character | Contradicts the positioning |

**Timing note:** the most recent PESOSE deadline was 1 September 2026, two weeks ago. The next cycle
is roughly twelve months out, so there is time, and a proposal is stronger after Phase 1 exists
anyway. But there is **no NSF funding before late 2027 at the earliest**.

**What happens once resolved.** The funding sequence in subplan 32 becomes actionable, and
sustainability stops being scored on mitigations alone.

---

### R-001: GitHub organization creation

**Status:** OPEN, raised 2026-09-14, interim workaround agreed
**Blocks:** nothing at present

**What is needed.** A GitHub organization created by the owner, and the repository transferred
into it once it exists.

**Why.** Brief section 5.1 requires the project to live under an organization rather than a
personal account, and explicitly forbids silently falling back to a personal repo. Two independent
reasons this cannot be done from here: GitHub exposes no public REST API for creating
organizations on github.com (the only creation endpoints are GitHub Enterprise Server admin
endpoints), and the `gh` token in this environment carries scopes `gist, read:org, repo, workflow`
with no `admin:org`.

**Interim, agreed with the owner on 2026-09-14.** Work proceeds on `SakarZaidan/<name>` and the
repository is transferred into the organization later. GitHub transfers preserve stars, issues,
pull requests, and set up redirects from the old URL, so the cost of deferring is low.

**What happens once resolved.** Transfer the repository into the org, update every documentation
link and package metadata field that names the personal account, set up CODEOWNERS against real
org teams, and configure org-level settings that need `admin:org`.

**What the owner needs to do.** Create an empty organization at
`https://github.com/organizations/new` (free tier is sufficient) and tell me the name. If the org
enforces OAuth app access restrictions, the `gh` OAuth app may also need approval for the token to
operate inside it.

---

---

### R-003: SynBioHub account

**Status:** OPEN, raised 2026-09-14
**Blocks:** nothing. Optional live integration checks only.

**What is needed.** A registered account on synbiohub.org, or a decision that we do not want one.

**Why.** Verified on 2026-09-14 that the public SynBioHub API requires authentication for every
endpoint probed, including `/rootCollections`, which the documentation describes as public. All six
returned `401 Login required`.

This does **not** block CI or testing. Subplan 04 resolves that by vendoring the `sbol-rs` fixture
corpus (343 files, MIT OR Apache-2.0) as the test set, which is a better answer than putting
credentials in CI regardless. An account is useful for development and for optional manual
integration checks against real registry content.

Registering an account on an external service under the owner's name is their decision, not an
engineering call, which is why this is here rather than done.

**What happens once resolved.** Live SynBioHub import becomes testable manually. Credentials go in
a local `.env`, never in CI, per subplan 10.

---

## RESOLVED

### R-000: Initial project decisions

**Status:** RESOLVED 2026-09-14

Four decisions were genuinely the owner's rather than engineering calls, and were put to them
before planning started.

| Question | Resolution |
|---|---|
| Keep the name GENESIS despite collisions on crates.io, PyPI, npm, github.com, and against the Genesis robotics simulator and DOE Genesis Mission? | **No.** GENESIS is the internal working title. Subplan 01 researches and selects the real name. |
| How to handle the GitHub organization requirement given no API exists for it? | **Personal repo now, transfer later.** See R-001. |
| Which license? | **Apache-2.0.** Matches `sbol-rs` upstream, carries an express patent grant (which matters in a patent-dense field), and passes institutional legal review without friction. AGPL was considered and rejected: it is banned by corporate policy at many of the exact industrial adopters a standards-layer tool needs. |
| Where does the first checkpoint fall in the brief's 13-step sequence? | **End of the planning folder.** No implementation code until the owner has read the planning documents. |

---

### R-002: Access to a curated sequence-of-concern reference dataset

**Status:** **RESOLVED 2026-09-14** by subplan 24. No owner action needed.
**Blocks:** any honest claim that biosafety screening is real rather than illustrative
**Does not block:** gateway architecture, its interfaces, its test harness, or subplans 06/10/24

**What is needed.** A usable reference dataset of sequences of concern, or credentialed access to
a screening service that holds one, for the biosafety screening gateway.

**Why.** This is the project's largest external dependency and engineering effort cannot route
around it. Screening compares designed sequences against a curated set of regulated-pathogen and
toxin sequences. That curated set is deliberately not a public download, for the obvious reason
that publishing a precise list of what you screen for is itself a biosecurity problem. Research so
far found two credible open-source screening implementations (`SecureDNA/SecureDNA`, Rust,
Apache-2.0; `ibbis-bio/common-mechanism`, IBBIS) but has not yet confirmed whether either ships
usable reference data or expects the operator to bring their own. Subplan 24 investigates this
first, and this request may narrow or resolve on its own once that research is done.

**What happens once resolved.** The screening gateway moves from "architecturally complete,
refuses to export because it has no reference data" to actually functional. Until then the system
will be built to fail closed: no reference data means no export, stated plainly in the UI and the
docs rather than silently passing everything.

**Note on scope.** If no access is obtainable, that is an acceptable outcome and not a project
blocker. The correct behaviour is to ship the gateway with a documented integration point, fail
closed by default, and be explicit in the README that screening requires the operator to supply or
connect their own reference data. What is *not* acceptable is shipping a toy dataset and implying
it constitutes real screening.
