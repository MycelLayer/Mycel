# GitHub Adoption Plan

Status: proposed short rollout for repository governance and collaboration

This plan focuses on GitHub-native features that can improve Mycel's safety,
review flow, and planning visibility without changing product behavior.

## Current Snapshot

As of 2026-03-24, the repository already uses:

- GitHub Actions workflows for CI and Pages
- issue forms and a pull request template
- GitHub Discussions for open-ended conversation
- GitHub Projects as an available planning surface
- secret scanning and secret scanning push protection
- private vulnerability reporting
- classic branch protection on `main`
- three required status checks on `main`:
  - `rust-and-validation`
  - `ast-grep-quality`
  - `code-quality-hotspots-warning`
- one required pull-request approval on `main`
- `.github/CODEOWNERS`
- `.github/dependabot.yml`
- vulnerability alerts
- automated security fixes
- the `MycelLayer/Mycel` organization-owned repository

The main gaps worth addressing first are:

- `main` still uses classic branch protection rather than `rulesets`
- code owner reviews are not required yet
- admin enforcement is still off for the branch protection
- delete-branch-on-merge is still off
- auto-merge remains disabled
- code scanning is still not configured

## 1. Strengthen Branch Governance

Keep improving the `main` merge gate before adding more automation.

Target settings:

- keep pull requests and the existing required CI checks in place
- decide whether to move from classic branch protection to GitHub `rulesets`
- decide whether admins should also be fully enforced by the merge gate
- decide whether code owner reviews should become required now that
  `CODEOWNERS` exists

Why first:

- this tightens the real merge gate that already exists around the workflows we
  maintain
- later features such as auto-merge become more useful only after merge
  requirements exist
- this is the highest-leverage safety improvement with the lowest ongoing
  maintenance cost

Main tradeoff:

- maintainers lose some direct-to-main speed in exchange for a safer default

## 2. Refine Ownership Routing

Refine `.github/CODEOWNERS` now that the first-pass file exists.

Suggested first-pass ownership split:

- `apps/` -> application owners
- `crates/` -> core Rust owners
- `scripts/` and `.github/workflows/` -> workflow/process owners
- `docs/` and `pages/` -> docs/public-surface owners

Why second:

- review routing matters more now that pull-request review is already part of
  the merge path
- Mycel already has clear directory boundaries that map well to ownership

Main tradeoff:

- ownership needs periodic maintenance as responsibilities evolve

## 3. Mature Dependabot And Security Adoption

Keep the existing Dependabot and vulnerability-alert setup small and
intentional.

Recommended scope:

- keep the current `dependabot.yml` focused on the ecosystems we already use
  (`cargo`, GitHub Actions, and the root `npm` workspace)
- keep private vulnerability reporting enabled for the public repository
- keep secret scanning and secret scanning push protection enabled unless
  signal quality becomes a real burden
- decide whether host-side Dependabot behavior should stay on grouped low-churn
  updates or narrow further
- keep version updates intentional until the team decides how much update churn
  it wants
- review grouped update settings if alert volume becomes noisy

Why third:

- the repository now has secret protection, Dependabot config, and vulnerability
  alerts, so the next value is tuning signal quality rather than merely turning
  the features on
- security-update pull requests fit well now that branch protections and review
  routing already exist

Main tradeoff:

- maintainers will need to triage additional automated pull requests

## 4. Add Code Scanning Only With A Deliberate Workflow Choice

Code scanning is now the main remaining GitHub-native security feature that is
still unset.

Current repo fit on 2026-03-24:

- GitHub's repository default-setup API reports `state: not-configured` and
  detects `actions`, `python`, and `rust` as eligible CodeQL languages for
  this repository
- the repository does not currently carry a dedicated CodeQL workflow under
  `.github/workflows/`
- existing CI already covers formatting, Clippy, compile, tests, ast-grep, and
  hotspot warnings, so CodeQL would be an additive security-analysis layer
  rather than a replacement for current checks
- the current Pages and docs-tooling surface is not the main reason to enable
  code scanning here; the best immediate fit is the Rust codebase, GitHub
  Actions workflows, and Python-based repo scripts

Recommended scope:

- choose a lightweight first pass instead of enabling the broadest possible
  default without review
- use GitHub code scanning only if we want persistent SARIF-backed findings in
  the Security tab rather than relying solely on existing CI and `ast-grep`
- prefer a single clear initial path, such as GitHub's default CodeQL setup,
  before layering on more languages or custom packs

Why now:

- the lower-friction security switches are already on, so code scanning is the
  next meaningful GitHub-native visibility upgrade
- this is the first remaining item that adds new analysis coverage rather than
  just governance or routing changes

Main tradeoff:

- better static-analysis visibility in exchange for extra CI time and alert
  triage overhead

Practical decision options:

- enable default CodeQL setup now as the smallest repo-native next step
- keep code scanning out of scope for now if the team wants to avoid more
  Actions runtime and security-alert triage this quarter
- move to advanced setup only if default setup proves too noisy, misses needed
  customization, or needs tighter event/path control than the UI-managed
  defaults provide

Current recommendation:

- enable GitHub's default CodeQL setup first if we want to continue the GitHub
  security-adoption track soon
- keep the initial language set aligned with GitHub's detected `actions`,
  `python`, and `rust` coverage unless the first runs show a clear reason to
  narrow it
- treat advanced setup as a second-step escalation, not the starting point,
  because this repository already has a stable CI baseline and does not yet
  show a strong need for a hand-maintained CodeQL workflow
- if the team decides against more scan time or alert queue overhead right now,
  explicitly leave code scanning deferred and rely on the existing CI plus
  `ast-grep` checks instead of leaving the decision ambiguous

## 5. Revisit Auto-Merge After The Merge Gate Exists

Enable auto-merge only after step 1 is working well.

Why later:

- auto-merge is most useful when pull requests must wait on checks or reviews
- enabling it before the team is comfortable with the current merge gate still
  provides limited operational value

Main tradeoff:

- small convenience gain, but it can hide merge timing if the team is not yet
  comfortable with enforced review rules

## 6. Treat Projects As An Optional Planning Upgrade

GitHub Projects is worth adopting only if we want a GitHub-native planning view
for issue, PR, and role-based work tracking.

Good fit signals:

- we want one place to see coding, delivery, and doc work together
- we want custom fields such as role, scope, planning impact, or priority
- we want roadmap or status views tied directly to issues and pull requests

Why not earlier:

- Projects improves coordination clarity, but it does not reduce merge or
  security risk as directly as the first three steps
- Mycel already has local multi-agent coordination, so Projects would be an
  additive planning layer rather than a prerequisite

Main tradeoff:

- better visibility in exchange for setup and ongoing curation work

## Keep, But Do Not Expand Yet

GitHub Discussions should remain enabled for design questions, early
exploration, and public conversation, but it does not need immediate process
expansion.

Keep the current boundary:

- Discussions for open-ended design or community conversation
- Issues for tracked work
- pull requests for mergeable changes

## Not A Near-Term Candidate

Merge queue should stay deferred for now.

Reason:

- it is a strong fit for busy protected branches, but the current branch
  governance still looks light enough that merge queue would be extra process
  weight
- Mycel is organization-owned now, but current throughput still looks too small
  to justify merge-queue overhead yet

## Minimal Adoption Sequence

If we want the smallest practical rollout, use this sequence:

1. strengthen `main` governance, including the rulesets vs classic-branch-protection decision
2. refine `CODEOWNERS` and decide whether code owner reviews should be required
3. keep tuning Dependabot and the newly enabled security features
4. decide whether to add code scanning
5. optionally enable auto-merge

This sequence keeps the change surface small while improving safety and review
discipline quickly.

## Follow-Up Work

Concrete next implementation tasks for a future work item:

- draft the exact required status checks for `main`
- decide whether classic branch protection should move to `rulesets`
- decide whether code owner reviews and admin enforcement should be required
- refine the first-pass `.github/CODEOWNERS`
- record which maintainers can bypass rulesets, if any
- decide whether Dependabot should stay on grouped low-churn updates or narrow
  further
- decide whether GitHub code scanning should use default CodeQL setup now or
  stay explicitly out of scope until scan-time and triage budget increase
- decide whether GitHub Projects should mirror the existing multi-agent workflow
  or stay out of scope
