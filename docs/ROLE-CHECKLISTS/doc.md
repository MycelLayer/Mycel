# Doc Role Checklist

Status: canonical source for `doc` role work

Use this tracked file as the source for a per-agent checklist copy. Do not mark
progress in this file directly.

Suggested per-agent copy path:

- `.agent-local/agents/<agent_uid>/checklists/ROLE-doc-checklist.md`

## Startup

- Confirm the registry state and active peers before taking documentation or planning scope. <!-- item-id: doc.startup.registry-state -->
- If this is a fresh chat, claim a fresh `doc` agent identity rather than reusing an older inactive one. <!-- item-id: doc.startup.claim-fresh -->
- Check recent active, paused, and recently inactive mailboxes before planning-sync work. <!-- item-id: doc.startup.scan-mailboxes -->

## Work Cycle

- Begin the user-command work cycle with `scripts/agent_work_cycle.py` before tracked work starts. <!-- item-id: doc.cycle.begin -->
- Run `git status -sb` and avoid unrelated user changes already in the worktree. <!-- item-id: doc.cycle.git-status -->
- Treat `ROADMAP.*` and `IMPLEMENTATION-CHECKLIST.*` as the higher planning authority when surfaces disagree. <!-- item-id: doc.cycle.source-of-truth-order -->
- Use `docs/PLANNING-SYNC-PLAN.md` as the entry point for `sync doc`, `sync web`, and `sync plan` batches. <!-- item-id: doc.cycle.planning-entry-point -->

## Planning Sync

- Run `scripts/check-plan-refresh.sh` after each completed doc work item while preparing next items. <!-- item-id: doc.plan.run-refresh-check -->
- If planning refresh is due, include the due surfaces in the next items before the sync batch starts. <!-- item-id: doc.plan.include-due-surfaces -->
- Scan registry mailboxes for recent planning-relevant handoffs before `sync doc`, `sync web`, or `sync plan`. <!-- item-id: doc.plan.scan-mailboxes-before-sync -->
- Keep progress and Pages wording derived from roadmap/checklist state instead of inventing new project status. <!-- item-id: doc.plan.derived-surfaces-only -->

## Boundaries

- Do not check CI as part of normal `doc` work. <!-- item-id: doc.boundary.no-ci -->
- Do not use tracked source files as personal work logs; update only the per-agent checklist copy. <!-- item-id: doc.boundary.agent-local-copy -->
- Keep issue triage aligned with roadmap/checklist meaning rather than using issues as the planning authority. <!-- item-id: doc.boundary.issue-alignment -->

## Handoff And Finish

- Leave exactly one open same-role mailbox handoff for the current state before ending the work cycle. <!-- item-id: doc.finish.same-role-handoff -->
- Record which planning surfaces changed, or explicitly note that no planning surface changed. <!-- item-id: doc.finish.record-surfaces -->
- End the work cycle with `scripts/agent_work_cycle.py` and resolve any reported unchecked items before considering the cycle complete. <!-- item-id: doc.finish.end-cycle -->
