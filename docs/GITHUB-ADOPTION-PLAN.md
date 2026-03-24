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
- classic branch protection on `main`
- three required status checks on `main`:
  - `rust-and-validation`
  - `ast-grep-quality`
  - `code-quality-hotspots-warning`
- one required pull-request approval on `main`
- `.github/CODEOWNERS`
- `.github/dependabot.yml`
- vulnerability alerts
- the `MycelLayer/Mycel` organization-owned repository

The main gaps worth addressing first are:

- `main` still uses classic branch protection rather than `rulesets`
- code owner reviews are not required yet
- admin enforcement is still off for the branch protection
- delete-branch-on-merge is still off
- auto-merge remains disabled

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

## 4. Revisit Auto-Merge After The Merge Gate Exists

Enable auto-merge only after step 1 is working well.

Why later:

- auto-merge is most useful when pull requests must wait on checks or reviews
- enabling it before the team is comfortable with the current merge gate still
  provides limited operational value

Main tradeoff:

- small convenience gain, but it can hide merge timing if the team is not yet
  comfortable with enforced review rules

## 5. Treat Projects As An Optional Planning Upgrade

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

1. configure `main` rulesets / branch protection
2. refine `CODEOWNERS` and decide whether code owner reviews should be required
3. tune Dependabot plus vulnerability-alert usage
4. optionally enable auto-merge

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
- decide whether GitHub Projects should mirror the existing multi-agent workflow
  or stay out of scope
