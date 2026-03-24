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
- the `MycelLayer/Mycel` organization-owned repository

The main gaps worth addressing first are:

- `main` has no ruleset or classic branch protection
- no required status checks or review gate protect merges to `main`
- no `CODEOWNERS` file exists yet
- no repo-side Dependabot configuration exists yet
- auto-merge is disabled and would not add much value until merge requirements
  exist

## 1. Branch Governance First

Adopt GitHub `rulesets` or branch protection for `main` before adding more
automation.

Target settings:

- require pull requests before merge
- require the existing CI checks to pass
- restrict direct pushes to `main`
- optionally require at least one review

Why first:

- this creates a real merge gate around the workflows we already maintain
- later features such as auto-merge become more useful only after merge
  requirements exist
- this is the highest-leverage safety improvement with the lowest ongoing
  maintenance cost

Main tradeoff:

- maintainers lose some direct-to-main speed in exchange for a safer default

## 2. Add Ownership Routing

Add `.github/CODEOWNERS` after branch governance is in place.

Suggested first-pass ownership split:

- `apps/` -> application owners
- `crates/` -> core Rust owners
- `scripts/` and `.github/workflows/` -> workflow/process owners
- `docs/` and `pages/` -> docs/public-surface owners

Why second:

- review routing matters more once pull-request review is part of the merge path
- Mycel already has clear directory boundaries that map well to ownership

Main tradeoff:

- ownership needs periodic maintenance as responsibilities evolve

## 3. Enable Dependabot Security Updates

Turn on Dependabot security updates for vulnerable dependencies and GitHub
Actions, and add a minimal repository config to keep version-update churn small.

Recommended scope:

- start with security updates first
- add a minimal `dependabot.yml` for the ecosystems we already use (`cargo`,
  GitHub Actions, and the root `npm` workspace)
- keep version updates optional until the team decides how much update churn it
  wants
- review grouped update settings if alert volume becomes noisy

Why third:

- the repository already has secret protection enabled, so dependency security
  is the next natural GitHub-native safety layer
- security-update pull requests fit well once branch protections and review
  routing are in place

Main tradeoff:

- maintainers will need to triage additional automated pull requests

## 4. Revisit Auto-Merge After The Merge Gate Exists

Enable auto-merge only after step 1 is working well.

Why later:

- auto-merge is most useful when pull requests must wait on checks or reviews
- enabling it before branch requirements exist provides little operational
  value

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

- it is a strong fit for busy protected branches, but branch governance is not
  in place yet
- Mycel is organization-owned now, but current throughput still looks too small
  to justify merge-queue overhead yet

## Minimal Adoption Sequence

If we want the smallest practical rollout, use this sequence:

1. configure `main` rulesets / branch protection
2. add a minimal `CODEOWNERS`
3. add a minimal `dependabot.yml` and enable Dependabot security updates
4. optionally enable auto-merge

This sequence keeps the change surface small while improving safety and review
discipline quickly.

## Follow-Up Work

Concrete next implementation tasks for a future work item:

- draft the exact required status checks for `main`
- draft or refine the first-pass `.github/CODEOWNERS`
- record which maintainers can bypass rulesets, if any
- decide whether Dependabot should stay on grouped low-churn updates or narrow
  further to security-only host-side behavior
- decide whether GitHub Projects should mirror the existing multi-agent workflow
  or stay out of scope
