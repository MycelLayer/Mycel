# Bootstrap Token Analysis

Status: internal note for Codex bootstrap token usage in this repo

## Scope

This note captures a token-usage analysis for one `boot coding` chat in the
Mycel repo.

The estimate is based on:

- `scripts/codex_token_usage_summary.py`
- `scripts/agent_work_cycle.py`
- the rollout JSONL for the matching Codex thread

The ranking below uses `input_tokens` growth as the main signal. It is a strong
estimate for "which work consumed the most context budget", but it is not a
precise per-tool billing breakdown.

## Key Reading

- The Codex UI value such as `44K/258K` reflects the thread's current total
  `input_tokens` against the model context window.
- The work-cycle field such as `+7K this cycle est.` reflects the increase
  during that specific work cycle only.
- Therefore, bootstrap batch 1 can legitimately show `usage 40K/258K | +7K this cycle est.`.
  The `40K` is the thread total at closeout time; the `+7K` is the bootstrap
  cycle's estimated contribution.

## Snapshot Used

For the analyzed bootstrap cycle:

- batch-1 start snapshot: `input_tokens = 33,370`
- batch-1 end snapshot: `input_tokens = 40,458`
- estimated bootstrap-cycle spend: `7,088` input tokens, shown in the UI as `+7K`

For the user's follow-up observation:

- the thread later reached about `44K`
- this matches later rollout rows such as `44,160` and `44,947`

## Ranking To About 44K

The following ranking estimates the largest token consumers from the beginning
of the chat up to the point where the thread was around `44K`.

### 1. Initial chat context load

Estimated cost: about `15.8K`

Main contributors:

- the pasted `AGENTS.md instructions`
- environment and IDE context
- the initial assistant reply and tool-planning frame

### 2. Bootstrap document intake

Estimated cost: about `12.6K`

Main contributors:

- `AGENTS.md`
- `AGENTS-LOCAL.md`
- `.agent-local/dev-setup-status.md`
- `docs/ROLE-CHECKLISTS/README.md`
- `docs/AGENT-REGISTRY.md`
- `.agent-local/agents.json`

This was the largest "work chunk" after the initial chat payload because the
assistant had to absorb the repo bootstrap rules before acting.

### 3. Same-role handoff mailbox review

Estimated cost: about `3.7K`

Main contributors:

- `.agent-local/mailboxes/agt_b2de3eff.md`

That mailbox contained multiple handoff entries, so it added noticeable context
weight even though it was only one file.

### 4. Running bootstrap and absorbing the result

Estimated cost: about `3.4K`

Main contributors:

- `python3 scripts/agent_bootstrap.py coding --model-id gpt-5-codex --scope "boot coding" --concise`
- the claimed role output
- the embedded latest-completed-CI baseline
- the bootstrap next-action summary

### 5. Bootstrap closeout and checklist correction

Estimated cost: about `2.3K`

Main contributors:

- reading the generated work-cycle checklist
- marking the relevant item-id states
- rerunning `scripts/agent_work_cycle.py end`

### 6. Other small bootstrap support steps

Estimated cost: about `7.1K` combined

This bucket includes smaller steps that mattered, but did not dominate on their
own:

- `agent_bootstrap.py --help`
- CI-related repo grep/search
- role-checklist read
- `git status -sb`
- `agent_work_cycle.py end --help`
- locating the latest `coding-10` registry entry
- short progress updates and final bootstrap reply text

## Practical Summary

If this thread is grouped by work type instead of individual steps, the rough
ordering is:

1. reading repo/bootstrap rules and registry state
2. executing bootstrap and absorbing the result
3. closing the cycle cleanly with checklist-driven admin steps

In short, the biggest token driver was not the bootstrap command itself. The
largest cost came from loading and retaining the repo's startup instructions and
coordination state.

## Follow-up Idea

If finer attribution is needed in the future, add a helper that groups rollout
token rows by phase, such as:

- prompt/context load
- docs/rules read
- bootstrap command
- CI lookup
- checklist closeout
- follow-up Q&A
