# Coding Role Checklist

Status: canonical source for `coding` role work

Use this tracked file as the source for a per-agent checklist copy. Do not mark
progress in this file directly.

Suggested per-agent copy path:

- `.agent-local/agents/<agent_uid>/checklists/ROLE-coding-checklist.md`

## Startup

- Confirm the registry state and active peers before taking implementation scope. <!-- item-id: coding.startup.registry-state -->
- If this is a fresh chat, claim a fresh `coding` agent identity rather than reusing an older inactive one. <!-- item-id: coding.startup.claim-fresh -->
- Read the newest overlapping coding handoff before continuing implementation work. <!-- item-id: coding.startup.read-handoff -->
- Check the latest completed CI result for the previous push before starting the next coding slice. <!-- item-id: coding.startup.check-latest-ci -->

## Work Cycle

- Begin the user-command work cycle with `scripts/agent_work_cycle.py` before tracked work starts. <!-- item-id: coding.cycle.begin -->
- Run `git status -sb` and avoid unrelated user changes already in the worktree. <!-- item-id: coding.cycle.git-status -->
- Keep implementation scope narrow enough to avoid overlapping another active coding agent's primary files when possible. <!-- item-id: coding.cycle.scope-control -->
- Hand planning-relevant implementation notes to `doc` through the mailbox instead of running `scripts/check-plan-refresh.sh`. <!-- item-id: coding.cycle.mailbox-to-doc -->

## Verification

- Run the most relevant local verification for the changed behavior before committing. <!-- item-id: coding.verify.local-tests -->
- Prefer targeted tests first, then broader validation when the change warrants it. <!-- item-id: coding.verify.targeted-first -->
- If verification could not be run, record that clearly in the mailbox handoff and final report. <!-- item-id: coding.verify.record-gaps -->

## Commit And Push

- Use the agent git identity per commit, for example `gpt-5:coding-N`. <!-- item-id: coding.git.agent-identity -->
- Commit and push serially; do not overlap those steps. <!-- item-id: coding.git.serial-commit-push -->
- If `git push origin main` is rejected, fetch, rebase onto `origin/main`, resolve conflicts conservatively, and retry without force-push. <!-- item-id: coding.git.rebase-on-reject -->
- Preserve user changes first, then already-pushed shared branch changes, then re-apply this chat's work on top during conflicts. <!-- item-id: coding.git.conflict-order -->

## Handoff And Finish

- Leave exactly one open same-role mailbox handoff for the current state before ending the work cycle. <!-- item-id: coding.finish.same-role-handoff -->
- Include planning impact when the change affects roadmap, checklist, progress-page, or issue-triage state. <!-- item-id: coding.finish.include-planning-impact -->
- End the work cycle with `scripts/agent_work_cycle.py` and resolve any reported unchecked items before considering the cycle complete. <!-- item-id: coding.finish.end-cycle -->
