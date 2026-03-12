# Agent Registry Protocol

Status: active local-registry protocol for multi-agent coordination

Use this file as the tracked specification for the local registry that tells agents how many agents are active, what role each one has, and whether each agent has confirmed that assignment before starting tracked work.

The live registry file is local and gitignored:

- `.agent-local/agents.json`

Recommended startup gate:

- `scripts/agent_registry.py claim <role|auto> [--scope <scope>]`
- `scripts/agent_registry.py start <agent-id>`
- `scripts/agent_registry.py touch <agent-id>`
- `scripts/agent_registry.py finish <agent-id>`
- `scripts/agent_registry.py stop <agent-id> [--status paused|done]`
- `scripts/agent_registry.py cleanup`
- `scripts/agent_registry.py recover <stale-agent-id> [--scope <scope>]`

Recommended status command:

- `scripts/agent_registry.py status [<agent-id>]`
- `scripts/agent_registry.py resume-check <agent-id>`

Recommended startup self-label:

- `<agent-id> | <scope-label>`

Agents should read `.agent-local/agents.json` at the start of work to discover:

- how many agents are currently active
- each agent's `id`
- each agent's `role`
- who assigned that role
- whether the agent has already confirmed that assignment
- each agent's current scope
- whether a peer agent is active, paused, or done

If a new chat receives only a role declaration such as `you are coding` or `you are doc`, the agent should claim a fresh id with `scripts/agent_registry.py claim <role>` before running `scripts/agent_registry.py start <agent-id>`.

If the user does not assign any role in a new chat, the agent should use `scripts/agent_registry.py claim auto` to choose the default role from `.agent-local/agents.json` before starting work:

- if there is no active `coding` agent, take `coding` first
- if active `coding >= 1` and active `doc == 0`, take `doc`
- if active `coding >= 1` and active `doc >= 1`, take `coding`

This default-role rule is only for chats without a user-assigned role. An explicit user role selection still wins.

After claim/start, the agent should begin the chat with one fixed self-label line using the registry id and current scope, for example:

- `coding-2 | forum-design-note-sync`
- `doc-1 | roadmap-sync-for-forum`

## Role Model

The system supports multiple concurrent agents, not just one `coding` and one `doc`.

Each agent entry must declare one role:

- `coding`
  owns issue resolution, feature work, local verification, commit/push flow, and CI checks after each push
- `doc`
  owns design-note sync, roadmap/checklist refresh, explanatory docs, and planning-surface wording; this role does not check CI by default

Any number of agents may share the same role, as long as they do not collide on the same issue or primary file set.

## Registry Shape

The local registry file must be valid JSON and use this top-level shape:

```json
{
  "version": 1,
  "updated_at": "2026-03-11T00:00:00Z",
  "agent_count": 2,
  "agents": [
    {
      "id": "agent-coding-1",
      "role": "coding",
      "assigned_by": "maintainer",
      "assigned_at": "2026-03-11T00:00:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:02:00+0800",
      "last_touched_at": "2026-03-11T00:10:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "#42 accepted-head strictness",
      "files": [
        "crates/mycel-core/src/verify.rs",
        "apps/mycel-cli/tests/object_verify_smoke.rs"
      ],
      "mailbox": ".agent-local/agent-coding-1.md"
    },
    {
      "id": "agent-doc-1",
      "role": "doc",
      "assigned_by": "maintainer",
      "assigned_at": "2026-03-11T00:01:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:03:00+0800",
      "last_touched_at": "2026-03-11T00:11:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "planning sync for #42",
      "files": [
        "ROADMAP.md",
        "IMPLEMENTATION-CHECKLIST.en.md"
      ],
      "mailbox": ".agent-local/agent-doc-1.md"
    }
  ]
}
```

## Required Fields

Top level:

- `version`
- `updated_at`
- `agent_count`
- `agents`

Per agent:

- `id`
- `role`
- `assigned_by`
- `assigned_at`
- `confirmed_by_agent`
- `confirmed_at`
- `last_touched_at`
- `inactive_at`
- `status`
- `scope`
- `files`
- `mailbox`

Allowed `role` values:

- `coding`
- `doc`

Allowed `status` values:

- `active`
- `inactive`
- `paused`
- `blocked`
- `done`

`agent_count` must equal the number of entries in `agents`.

`confirmed_by_agent` must be `true` before the agent starts tracked work.

`confirmed_at` may be `null` only while the entry is still waiting for agent confirmation.

`last_touched_at` may be `null` only before the entry has ever been activated or touched.

`inactive_at` should be a timestamp when `status` is `inactive`, and should be `null` otherwise.

`scripts/agent_registry.py` writes timestamps in `Asia/Taipei (UTC+8)` using the `+0800` offset form.

## Startup Gate

No agent may start tracked work until all of the following are true in `.agent-local/agents.json`:

1. the agent has a matching `id`
2. the entry has a non-empty `role`
3. the entry has `assigned_by` and `assigned_at`
4. the agent has set `confirmed_by_agent` to `true`
5. the entry has a non-null `confirmed_at`
6. the intended scope is present

If any of those checks fail, the agent must stop before editing tracked files and request a corrected assignment.

Recommended enforcement:

1. either a maintainer writes the assignment entry or the agent claims a new entry with `scripts/agent_registry.py claim <role|auto>`
2. the agent runs `scripts/agent_registry.py start <agent-id>`
3. the start script confirms the role, sets `confirmed_by_agent: true`, stamps `confirmed_at`, and creates the mailbox if needed
4. only then may tracked work begin

## Workflow

1. Before starting work, an agent reads `.agent-local/agents.json`.
2. The agent confirms the current agent count and scans the existing scopes and file sets.
3. If no entry exists yet but the role is known, the agent may claim a new id with `scripts/agent_registry.py claim <role>`; if the role is not user-assigned, the agent may use `scripts/agent_registry.py claim auto`.
4. Otherwise, a maintainer or coordinator writes the agent entry with `role`, `assigned_by`, `assigned_at`, `scope`, and `mailbox`.
5. The agent confirms its own assignment by running `scripts/agent_registry.py start <agent-id>`.
6. Only after confirmation may the agent start tracked work.
7. The agent uses its own `mailbox` file for peer coordination and handoff traffic.
8. Before doing work for each user command, the agent should run `scripts/agent_registry.py touch <agent-id>` so the registry marks that role active for the current command cycle.
9. After finishing work for that user command, the agent should run `scripts/agent_registry.py finish <agent-id>` so the registry marks that role inactive.
10. When scope changes, the agent updates its registry entry.
11. When work is finished or paused for longer-lived coordination reasons, the agent updates `status`, preferably with `scripts/agent_registry.py stop <agent-id> [--status paused|done]`.

If two `coding` agents would touch the same primary file or issue, one must pause or choose a narrower scope before proceeding.

## Activity Lease

The registry uses a per-command activity lease, not just a startup confirmation.

Rules:

1. on each new user command, the active agent should `touch` its own entry before starting work
2. when that command's work is complete, the agent should `finish` its own entry so the role becomes `inactive`
3. an entry that remains `inactive` for at least one hour becomes stale and is still resumable during the stale-retention window
4. once an entry has remained stale for at least 24 more hours, `scripts/agent_registry.py` should remove it from `.agent-local/agents.json`
5. `scripts/agent_registry.py cleanup` reports both currently retained stale entries and any entries that were removed because they exceeded the 24-hour stale-retention window
6. a previously inactive confirmed agent may resume by re-checking or touching its own retained entry only before that 24-hour stale-retention window expires

## Standard New Chat Startup

Use this sequence in order. Do not run the registry commands in parallel.

1. read `AGENTS.md`, `AGENTS-LOCAL.md`, and `docs/AGENT-REGISTRY.md`
2. run `git status -sb`
3. check `rg` and `gh`
4. check the latest CI status from the previous push
5. determine the role for this chat:
   - if the user explicitly assigned a role, use that role and run `scripts/agent_registry.py claim <role> [--scope <scope>]`
   - otherwise run `scripts/agent_registry.py claim auto [--scope <scope>]`
6. run `scripts/agent_registry.py start <agent-id>`
7. run `scripts/agent_registry.py status <agent-id>`
8. when the first concrete user task arrives, run `scripts/agent_registry.py touch <agent-id>` before starting work
9. begin the chat with the startup self-label: `<agent-id> | <scope-label>`
10. only after that, report repo status and wait for the concrete task

Recommended startup output:

```text
coding-1 | pending-user-task

Please read AGENTS.md and operate as the coding agent.

已完成 coding agent 啟動流程，接下來我會照這套規則執行。

目前狀態：
- repo 乾淨：## main...origin/main
- 已讀取並套用 AGENTS.md、AGENTS-LOCAL.md、docs/AGENT-REGISTRY.md
- 已確認本地 agent registry：這個 chat 是 coding-1，狀態 active，scope 是 pending-user-task
- 前一次已完成的 CI 正常：latest completed workflow success
- 後續 commit 會用 `gpt-5:coding-1` 作為 agent identity

把具體任務丟給我，我就直接開始做。
```

Keep this startup output narrow:

- do not claim file-specific context before the user gives a concrete task
- do not run `claim`, `start`, and `status` in parallel
- do not omit the startup self-label line
- keep the CI line about the latest completed workflow, not a possibly in-progress run
- mark the agent `inactive` with `scripts/agent_registry.py finish <agent-id>` after the command-level work is done

## Interrupted Chat Recovery

Treat the local registry and mailbox files as the source of truth if a chat stops unexpectedly because of an OpenAI or Codespaces issue.

Recovery rules:

1. do not assume an `active` agent is still reachable just because the registry says `active`
2. read `.agent-local/agents.json` and the relevant mailbox file first
3. preserve the old agent entry for auditability; do not overwrite its `id`
4. if the old chat is clearly gone, mark that agent `paused` with `scripts/agent_registry.py stop <agent-id>`
5. claim a new id for the replacement chat and continue from the mailbox handoff
6. if a previously forgotten chat is reopened later, that chat must run `scripts/agent_registry.py resume-check <its-agent-id>` before doing tracked work again
7. if the reopened chat is no longer `active`, it must stop and must not resume tracked work under the old id

Recommended recovery sequence:

1. run `scripts/agent_registry.py status`
2. identify the stale `active` agent
3. read `.agent-local/<agent-id>.md`
4. either run `scripts/agent_registry.py stop <old-agent-id>` then `scripts/agent_registry.py claim <role>` plus `scripts/agent_registry.py start <new-agent-id>`, or use `scripts/agent_registry.py recover <old-agent-id>`
5. read the stale mailbox before resuming tracked work

Recommended scripted shortcut:

- `scripts/agent_registry.py recover <old-agent-id>`

The recovery helper pauses the stale agent, creates a fresh id for the same role, starts the replacement entry immediately, and appends the default takeover note to the new mailbox.

Recommended takeover note:

- `taking over from coding-2 after interrupted chat`

Recommended reopened chat startup:

```text
<new-agent-id> | <scope-label>

Please read AGENTS.md and operate as the <role> agent.

已完成 interrupted-chat recovery，接下來我會接手前一個中斷 chat 的工作。

目前狀態：
- repo 乾淨：## main...origin/main
- 已讀取並套用 AGENTS.md、AGENTS-LOCAL.md、docs/AGENT-REGISTRY.md
- 已執行 `scripts/agent_registry.py status` 並確認舊 agent `<old-agent-id>` 需要接手
- 已執行 `scripts/agent_registry.py recover <old-agent-id>`，目前這個 chat 是 `<new-agent-id>`，狀態 active
- 已讀取舊 mailbox `.agent-local/<old-agent-id>.md` 與新 mailbox `.agent-local/<new-agent-id>.md`
- 前一次已完成的 CI 正常：latest completed workflow success
- 後續 commit 會用 `gpt-5:<new-agent-id>` 作為 agent identity

把接續的任務丟給我，我就直接開始做。
```

Keep this recovery startup output narrow:

- identify the stale agent id explicitly
- confirm that the old mailbox was read before resumed work
- use the new replacement id in the self-label and agent identity line
- do not claim new file-level context until the user gives the next concrete task

Forgotten-chat note:

- a reopened old chat is not trusted just because the window still exists
- it must re-check its own registry status before resuming work, preferably with `scripts/agent_registry.py resume-check <agent-id>`
- if the old id is merely `inactive` and still retained, it may resume its own entry during the first 24 hours of stale retention
- if the old entry has already been cleaned out after 24 hours of staleness, the reopened chat must not continue under that old id and should treat itself as a new chat or explicit recovery case
- if another chat already recovered the scope and the old id is now `paused`, the reopened old chat must stop and yield to the replacement id

Role note:

- `coding` should keep the CI line because that role owns CI checks after pushes
- `doc` can omit the CI line unless the maintainer explicitly asked that chat to monitor CI

## Mailbox Rule

The registry tells agents who exists. Mailboxes carry the actual messages.

Recommended local mailbox pattern:

- `.agent-local/<agent-id>.md`

If a simpler shared mailbox flow is preferred, agents may still use:

- `.agent-local/coding-to-doc.md`
- `.agent-local/doc-to-coding.md`

The registry remains the source for current agent count and role assignment.

## Minimal Example

For one `coding` agent and one `doc` agent:

```json
{
  "version": 1,
  "updated_at": "2026-03-11T00:00:00Z",
  "agent_count": 2,
  "agents": [
    {
      "id": "coding-1",
      "role": "coding",
      "assigned_by": "maintainer",
      "assigned_at": "2026-03-11T00:00:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:02:00+0800",
      "last_touched_at": "2026-03-11T00:10:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "#17 store refactor",
      "files": [
        "apps/mycel-cli/src/store.rs",
        "apps/mycel-cli/src/store/index.rs"
      ],
      "mailbox": ".agent-local/coding-1.md"
    },
    {
      "id": "doc-1",
      "role": "doc",
      "assigned_by": "maintainer",
      "assigned_at": "2026-03-11T00:01:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:03:00+0800",
      "last_touched_at": "2026-03-11T00:11:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "planning sync for #17",
      "files": [
        "ROADMAP.md",
        "IMPLEMENTATION-CHECKLIST.en.md"
      ],
      "mailbox": ".agent-local/doc-1.md"
    }
  ]
}
```

## Future Identity-Split Design

Status: design draft

This section proposes a future registry redesign that splits the current single agent identifier into:

- `agent_uid`: the true agent identity, never reused
- `display_id`: the human-readable short id such as `coding-1`, which may be recycled

The goal is to solve two problems at once:

- keep short display ids converged instead of unbounded growth
- prevent old and new chats from colliding just because both would otherwise appear as `coding-1`

This is a future design draft. It does not replace the active protocol above until the repo explicitly migrates to it.

### 0. Problem

Today the local registry uses a single field for all of these concerns:

- the agent's true identity
- the CLI write key
- the human-readable short label

That creates two competing pressures:

1. if ids are never reused, resume safety and auditability are strong, but names like `coding-17` grow forever
2. if ids are reused, display names converge, but old and new chats can be confused as the same agent

The `agent_uid + recyclable display_id` model separates those concerns into different fields.

### 1. Goals

Keep:

- a stable identity per chat
- clear identification when an old chat returns
- a complete audit trail across assignment, resume, recover, and takeover

Add:

- the ability to recycle short ids such as `coding-1` and `doc-1`
- a safe resume decision for old chats
- predictable short-slot allocation for multiple chats sharing the same role

Do not optimize for:

- cryptographic-strength chat identity verification
- cross-machine registry synchronization
- backward compatibility with the old schema by default

### 2. Terms

#### 2.1 `agent_uid`

The true primary identity key for an agent.

Properties:

- created at claim time
- never reused
- all state-changing CLI operations should key off it
- mailbox paths and audit history should attach to it

Suggested format:

- `agt_` prefix plus a random suffix, for example `agt_7b2e9f4c`

#### 2.2 `display_id`

The short id shown to humans.

Properties:

- keeps the familiar `coding-1` and `doc-1` format
- is bound to at most one current agent at a time
- may be reassigned after release
- must not be treated as the true identity key

#### 2.3 Display Slot

A numbered slot within a role, such as `coding-1` or `coding-2`.

Slots are reusable.
`display_id` tells us which slot an agent currently occupies, not which identity it permanently is across time.

### 3. Proposed Data Model

#### 3.1 Registry v2 Top-Level Shape

```json
{
  "version": 2,
  "updated_at": "2026-03-12T12:00:00+0800",
  "agent_count": 2,
  "agents": [
    {
      "agent_uid": "agt_a1b2c3d4",
      "role": "coding",
      "current_display_id": "coding-1",
      "display_history": [
        {
          "display_id": "coding-1",
          "assigned_at": "2026-03-12T11:00:00+0800",
          "released_at": null,
          "released_reason": null
        }
      ],
      "assigned_by": "user",
      "assigned_at": "2026-03-12T11:00:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-12T11:01:00+0800",
      "last_touched_at": "2026-03-12T11:10:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "forum inbox sync",
      "files": [],
      "mailbox": ".agent-local/mailboxes/agt_a1b2c3d4.md",
      "recovery_of": null,
      "superseded_by": null
    },
    {
      "agent_uid": "agt_e5f6g7h8",
      "role": "doc",
      "current_display_id": "doc-1",
      "display_history": [
        {
          "display_id": "doc-1",
          "assigned_at": "2026-03-12T11:05:00+0800",
          "released_at": null,
          "released_reason": null
        }
      ],
      "assigned_by": "user",
      "assigned_at": "2026-03-12T11:05:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-12T11:06:00+0800",
      "last_touched_at": "2026-03-12T11:15:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "registry design note",
      "files": [],
      "mailbox": ".agent-local/mailboxes/agt_e5f6g7h8.md",
      "recovery_of": null,
      "superseded_by": null
    }
  ]
}
```

#### 3.2 Required Fields

Top level:

- `version`
- `updated_at`
- `agent_count`
- `agents`

Per agent:

- `agent_uid`
- `role`
- `current_display_id`
- `display_history`
- `assigned_by`
- `assigned_at`
- `confirmed_by_agent`
- `confirmed_at`
- `last_touched_at`
- `inactive_at`
- `status`
- `scope`
- `files`
- `mailbox`
- `recovery_of`
- `superseded_by`

#### 3.3 Field Rules

- `agent_uid` is the registry primary key
- `current_display_id` may be `null`
- if `current_display_id != null`, that value must be unique across all agents
- `display_history` must be time ordered, and the last record represents the most recent slot assignment
- any `display_history` record with `released_at == null` must correspond to `current_display_id`
- `mailbox` should be keyed by `agent_uid`, not by `display_id`
- `recovery_of` marks that this agent took over from an older stale agent
- `superseded_by` marks that this agent has already been superseded by another agent

#### 3.4 Why `current_display_id` May Be `null`

This is what allows slot recycling without identity loss.

When an agent entry still needs to remain in the registry for audit purposes, but its short slot has already been released to someone else:

- the agent entry remains
- `agent_uid` stays the same
- `current_display_id` becomes `null`
- the last `display_history` record gains a `released_at`

### 4. Slot Recycling Rules

#### 4.1 Stale and Slot Release

This future design keeps the same command-level lease idea:

- `finish` moves the agent to `inactive`
- after one hour of inactivity, the agent becomes stale

Under the future identity split, a stale agent no longer keeps its old short slot forever.
The system may release its display slot during `cleanup` or before the next registry write:

- set `current_display_id = null`
- update the last `display_history.released_at`
- set `released_reason = "stale-recycled"`

The agent entry itself remains, so auditability and resume decisions are still possible.

#### 4.2 New Claim Slot Allocation

A new short slot should use the smallest available positive integer suffix for that role, not the historical max plus one.

Example:

- currently used slots are `coding-2` and `coding-4`
- available slots are `coding-1` and `coding-3`
- the next claim should take `coding-1`

That is how short ids converge again.

### 5. Lifecycle

#### 5.1 New Chat Claim

1. create a new `agent_uid`
2. choose the smallest available `display_id` for the role
3. create the mailbox at `.agent-local/mailboxes/<agent_uid>.md`
4. write the new agent entry
5. return both `agent_uid` and `display_id`

#### 5.2 Start

`start <agent_uid>` only confirms that specific agent entry:

- `confirmed_by_agent = true`
- `confirmed_at = now`
- `status = active`
- `inactive_at = null`

The human-facing label remains the `display_id`, but writes key off `agent_uid`.

#### 5.3 Each User Command

1. `touch <agent_uid>`
2. do the work
3. `finish <agent_uid>`

This matches the current lease flow except that the CLI primary key becomes `agent_uid`.

#### 5.4 Old Chat Returns Before Slot Release

If agent A still has `current_display_id = coding-1`, then:

- `resume-check <agent_uid=A>` should return `safe_to_resume = true`
- A may directly `touch A`
- A still presents itself as `coding-1`

#### 5.5 Old Chat Returns After Slot Release and Reassignment

If:

- A still exists in the registry
- A has `current_display_id = null`
- B has already claimed `coding-1`

Then:

- `resume-check <agent_uid=A>` must not permit a direct resume
- it should return `must_recover = true`
- A may not directly act as `coding-1`
- A must go through `recover <agent_uid=A>`

After recovery, A receives a new short slot such as `coding-2`.

#### 5.6 Takeover by a Different Chat

If the original chat A is not the one returning, and some new chat needs to take over A's scope, then the new chat must not reuse A's `agent_uid`.

Instead, takeover should work like this:

1. the new chat claims a fresh `agent_uid`
2. the new chat receives a fresh `display_id`
3. old A records `superseded_by = <new-agent-uid>`
4. the new agent records `recovery_of = <old-agent-uid>`
5. the new mailbox appends a takeover note

This keeps the distinction clear:

- self-resume keeps the same `agent_uid`
- takeover creates a new `agent_uid`

### 6. Proposed CLI Behavior

#### 6.1 Primary Keys

From this version onward, the design proposes:

- write commands should accept only `agent_uid` by default
- read commands may accept either `agent_uid` or `display_id`

Reason:

- `display_id` may be reused
- `agent_uid` will not be reused

#### 6.2 `claim`

```text
scripts/agent_registry.py claim <role|auto> [--scope <scope>] [--json]
```

Output:

- `agent_uid`
- `display_id`
- `role`
- `scope`
- `mailbox`

Claim should create the `agent_uid` immediately and allocate the smallest available `display_id`.

#### 6.3 `start`

```text
scripts/agent_registry.py start <agent_uid> [--json]
```

Behavior:

- verify that the `agent_uid` exists
- if `current_display_id` is `null`, reject `start` and require `recover` first
- on success, return both `agent_uid` and `display_id`

#### 6.4 `status`

```text
scripts/agent_registry.py status [--agent-uid <agent_uid> | --display-id <display_id>] [--json]
```

By default it should list all agents, and each row should display:

- `agent_uid`
- `current_display_id`
- `status`
- `scope`
- `last_touched_at`
- `inactive_at`

If queried by `display_id`, it should return only the current holder of that slot, not historical holders.

#### 6.5 `touch`

```text
scripts/agent_registry.py touch <agent_uid> [--json]
```

Behavior:

- if `current_display_id` is `null`, reject `touch` and require `recover` first
- otherwise update `last_touched_at` and move the agent to `active`

#### 6.6 `finish`

```text
scripts/agent_registry.py finish <agent_uid> [--json]
```

Behavior:

- set `status = inactive`
- update `inactive_at`
- do not release the display slot yet

Slot release happens during stale recycling or explicit recover logic, not immediately at `finish`.

#### 6.7 `resume-check`

```text
scripts/agent_registry.py resume-check <agent_uid> [--json]
```

Output should include at least:

- `agent_uid`
- `current_display_id`
- `safe_to_resume`
- `must_recover`
- `recommended_action`
- `reason`

Decision rules:

1. if the agent is already `paused`, `done`, or `blocked`, return stop
2. if `current_display_id != null`, return direct resume
3. if `current_display_id == null`, return `must_recover = true`

#### 6.8 `recover`

```text
scripts/agent_registry.py recover <agent_uid> [--scope <scope>] [--json]
```

Here `recover` means the same agent identity resumes work but needs a new short slot.

Behavior:

1. verify that the `agent_uid` exists
2. verify that the agent is not `done`
3. verify that `current_display_id == null`
4. assign a new smallest available `display_id`
5. append a new record to `display_history`
6. if `--scope` is present, update the scope
7. return the new `display_id`

After `recover`:

- `agent_uid` stays the same
- `display_id` changes
- mailbox stays the same

#### 6.9 `takeover`

```text
scripts/agent_registry.py takeover <stale-agent-uid> [--scope <scope>] [--json]
```

This should be a distinct command, not just another flavor of `recover`.

Use it when:

- the original agent is not the one coming back
- a different chat must take over the stale agent's work

Behavior:

1. create a new `agent_uid`
2. assign a new `display_id`
3. set `superseded_by = <new-agent-uid>` on the old agent
4. set `recovery_of = <old-agent-uid>` on the new agent
5. create a new mailbox

### 7. Edge Case Walkthrough

Scenario:

- A originally was `agent_uid=A`, `display_id=coding-1`
- A finished work and became `inactive`
- after one hour A became stale and the slot was released
- new chat B then claimed `agent_uid=B`, `display_id=coding-1`
- later the user returned to old chat A

The correct behavior should be:

1. A runs `resume-check A`
2. the system sees `current_display_id == null` for A
3. the system returns `must_recover = true`
4. A must not directly `touch A` and continue as `coding-1`
5. A runs `recover A`
6. the system assigns the next available slot, for example `coding-2`
7. A continues under `display_id=coding-2`

Result:

- B remains `coding-1`
- A safely becomes `coding-2`
- the two chats never collide as the same identity

### 8. Migration Guidance

#### 8.1 Schema Migration

For each old v1 agent entry:

- move `id` into `current_display_id`
- add a new `agent_uid`
- add `display_history`

If the old mailbox path used the display id, such as `.agent-local/coding-1.md`, then after migration the recommended shape is:

- keep the old file during migration if needed
- move or copy it to `.agent-local/mailboxes/<agent_uid>.md`
- store only the uid-based mailbox path in the registry

#### 8.2 CLI Migration

Recommended in two phases:

1. transitional phase
   - `touch/start/finish/stop/resume-check/recover` accept both `agent_uid` and `display_id`
   - if a write command receives a `display_id`, emit a deprecation warning
2. stable phase
   - write commands accept only `agent_uid`
   - `display_id` remains only for display and lookup

### 9. Why This Is Better Than Never Reusing Ids

Compared with the current approach, this model:

- keeps a stable identity for old chats
- allows short ids to converge
- no longer relies on keeping every historical id blocked forever
- makes self-resume and takeover distinct concepts

Tradeoffs:

- the schema is more complex
- the CLI has to move from display-id-first to uid-first
- documents and tests would need a full rewrite

### 10. Open Questions

1. should we also add a `session_token` so another chat cannot impersonate the same `agent_uid`
2. should `takeover` remain a separate command or be folded into `recover`
3. after how much stale time should a display slot be released, and should that use the same threshold as the current one-hour stale TTL
4. should `status` abbreviate `agent_uid` by default so the human-readable output does not become too noisy
