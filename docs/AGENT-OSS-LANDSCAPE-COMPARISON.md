# Existing OSS Landscape vs Our Stack

Status: draft comparison of existing open-source multi-agent tools against the current Mycel agent-coordination stack

This note compares several existing OSS options with the current Mycel
agent-coordination stack, focusing on the capabilities we care about most:

- registry
- handoff
- workcycle
- Git ownership
- stale-active recovery

The goal is not to rank frameworks in the abstract. The goal is to see where
our stack is differentiated and where existing OSS already covers the problem
space well.

## Scope Of Comparison

Compared systems:

- Mycel current stack
- Microsoft AutoGen
- CrewAI
- LangGraph + LangGraph Supervisor / Swarm pattern
- AWS Agent Squad
- AgentBase

Capability meaning in this table:

- `Registry`
  First-class identity/lifecycle model for multiple concurrent agents.
- `Handoff`
  First-class way to leave resumable context for another agent or later work.
- `Workcycle`
  Explicit per-command or per-session begin/end lifecycle bookkeeping.
- `Git ownership`
  First-class support for avoiding edit collisions in a shared repo.
- `Stale-active recovery`
  First-class support for detecting when an agent still appears active but is actually gone or stale.

Legend:

- `Yes`
  Clearly first-class in the official product or repo surface.
- `Partial`
  Present indirectly, or can be assembled from lower-level primitives, but not as a dedicated built-in coordination feature.
- `No`
  No clear first-class evidence found in the official repo/docs reviewed for this draft.

## Capability Matrix

| Stack | Registry | Handoff | Workcycle | Git ownership | Stale-active recovery | Notes |
|---|---|---|---|---|---|---|
| Mycel current stack | Yes | Yes | Yes | Yes | Yes | Repo-local coordination is a first-class design goal. |
| AutoGen | Partial | Partial | No | No | No | Strong runtime/message-passing model, but not repo-governance-oriented. |
| CrewAI | Partial | Partial | No | No | No | Strong task/agent orchestration, but not a Git-native multi-chat coordination layer. |
| LangGraph + Supervisor/Swarm | Partial | Partial | Partial | No | No | Powerful orchestration primitives; team repo coordination is mostly left to app authors. |
| AWS Agent Squad | Partial | Partial | No | No | No | Strong multi-agent routing and context preservation for conversations, not coding-team workflow governance. |
| AgentBase | Partial | Partial | Partial | Partial | No | Closest coding-team fit among reviewed tools; has parallel agents and isolated edits, but not our registry or stale-active model. |

## Project-By-Project Notes

### Mycel current stack

Current evidence in repo:

- registry: `scripts/agent_registry.py`, [`docs/AGENT-REGISTRY.md`](./AGENT-REGISTRY.md)
- handoff: `scripts/mailbox_handoff.py`, [`docs/AGENT-HANDOFF.md`](./AGENT-HANDOFF.md)
- workcycle: `scripts/agent_work_cycle.py`
- Git ownership/process discipline: [`AGENTS.md`](../AGENTS.md), [`docs/MULTI-AGENT-COORDINATION.md`](./MULTI-AGENT-COORDINATION.md)
- stale-active recovery: `scripts/agent_registry_reconcile.py`

Why it stands out:

- coordination state is repo-local and human-readable
- work is organized around explicit begin/end cycles
- handoff is a first-class artifact, not just chat history
- stale-active cleanup is treated as an operational problem, not ignored

Tradeoff:

- stronger governance than most frameworks
- heavier process burden than orchestration-only libraries

### AutoGen

Official source reviewed:

- GitHub: <https://github.com/microsoft/autogen>

Observed strengths from official repo:

- layered framework with Core API, AgentChat API, Extensions API
- message passing, event-driven agents, local/distributed runtime
- common multi-agent patterns such as two-agent chat and group chat

Evaluation against our capabilities:

- registry: `Partial`
  There is runtime structure and agent identity inside the framework, but not the kind of repo-local agent registry we use for parallel coding chats.
- handoff: `Partial`
  Group-chat and multi-agent workflow patterns exist, but not a Git-native resumable mailbox handoff artifact.
- workcycle: `No`
  I did not find a first-class per-command begin/end cycle concept like Mycel's workcycle.
- Git ownership: `No`
  No first-class repo collision-avoidance or file-ownership model found in the official surface reviewed.
- stale-active recovery: `No`
  No first-class stale-active reconciliation model found in the official surface reviewed.

Bottom line:

- excellent multi-agent runtime
- weak match for repo-governance and resumable coding-team coordination

### CrewAI

Official sources reviewed:

- GitHub: <https://github.com/crewAIInc/crewAI>
- Open-source page: <https://crewai.com/open-source>

Observed strengths from official repo:

- open-source multi-agent orchestration
- explicit agents and tasks configuration
- production-oriented positioning
- human-in-the-loop and external tools/APIs support

Evaluation against our capabilities:

- registry: `Partial`
  Agents and tasks are first-class in CrewAI, but not a local concurrent-chat registry for multiple live coding sessions.
- handoff: `Partial`
  Work can be passed between agents as part of a crew/task model, but I did not find an equivalent to our mailbox handoff artifact for later human or agent continuation.
- workcycle: `No`
  No first-class per-command lifecycle wrapper comparable to Mycel workcycles was evident in the reviewed official surfaces.
- Git ownership: `No`
  I did not find built-in Git edit-ownership or shared-repo anti-collision process features.
- stale-active recovery: `No`
  I did not find a stale-active or abandoned-session recovery model.

Bottom line:

- stronger than Mycel in task orchestration as a generic framework
- weaker than Mycel in local coding-session governance

### LangGraph + LangGraph Supervisor / Swarm

Official sources reviewed:

- LangGraph GitHub: <https://github.com/langchain-ai/langgraph>
- Supervisor: <https://github.com/langchain-ai/langgraph-supervisor-py>
- Swarm: <https://github.com/langchain-ai/langgraph-swarm-py>

Observed strengths from official repos:

- long-running, stateful agents
- graph-based orchestration
- supervisor pattern
- explicit tool-based handoff in supervisor
- shared or customized message-state management in swarm

Evaluation against our capabilities:

- registry: `Partial`
  Graph state and agent topology exist, but not a durable repo-local multi-chat registry equivalent.
- handoff: `Partial`
  Supervisor handoff is first-class inside the graph runtime, but it is not the same as our external mailbox handoff for resumable repo work.
- workcycle: `Partial`
  LangGraph is stateful and long-running, which is closer to workcycle thinking than most frameworks, but I did not find an explicit begin/end per command model like ours.
- Git ownership: `No`
  No first-class shared-repo edit-ownership feature found in the official sources reviewed.
- stale-active recovery: `No`
  No first-class stale-active session reconciliation feature found in the official sources reviewed.

Bottom line:

- best-in-class orchestration primitive set
- not a coding-team process layer by default

### AWS Agent Squad

Official source reviewed:

- GitHub: <https://github.com/awslabs/agent-squad>

Observed strengths from official repo:

- multi-agent routing across specialized agents
- preserves context across multi-turn conversations
- human-in-the-loop support in examples
- Python and TypeScript support

Evaluation against our capabilities:

- registry: `Partial`
  There is implicit agent routing/selection, but not a durable local registry like ours.
- handoff: `Partial`
  Context switching between specialized agents exists, but not a resumable file-backed handoff artifact for repo work.
- workcycle: `No`
  No explicit per-command workcycle model found in the official repo/docs surface reviewed.
- Git ownership: `No`
  No first-class coding-repo ownership or edit-isolation governance feature found.
- stale-active recovery: `No`
  No first-class stale-active reconciliation model found.

Bottom line:

- strong conversation/router framework
- weak fit for multi-chat coding coordination in one repo

### AgentBase

Official sources reviewed:

- GitHub: <https://github.com/AgentOrchestrator/AgentBase>
- Docs: <https://docs.agentbase.sh/primitives/essentials/multi-agents>

Observed strengths from official repo:

- visual canvas for running agents
- parallel agents with shared context
- isolated edits to prevent overwrite collisions
- progress tracking
- command center for approvals
- explicit focus on AI coding assistant conversations

Evaluation against our capabilities:

- registry: `Partial`
  It clearly tracks multiple running agents in a visual/orchestration sense, but I did not find a Mycel-style durable repo-local registry protocol.
- handoff: `Partial`
  Shared context and conversation visibility are strong, but I did not find a file-backed mailbox handoff pattern like ours.
- workcycle: `Partial`
  It has progress tracking and orchestration over running sessions, which is closer to workcycle than most tools, but not an explicit begin/end workcycle contract.
- Git ownership: `Partial`
  `Isolated Edits` is the strongest Git/worktree-adjacent feature in any reviewed tool and is the closest existing match to our anti-collision goals.
- stale-active recovery: `No`
  I did not find a first-class stale-active detection and downgrade model.

Bottom line:

- closest reviewed OSS to our coding-team use case
- still missing our lifecycle/registry/handoff/reconcile stack

## What Seems Differentiated In Our Stack

After this comparison, the clearest differentiators are:

1. repo-local registry as a first-class artifact
2. explicit per-command workcycle semantics
3. mailbox handoff as an operational surface, not just chat history
4. Git/process governance as part of the toolchain rather than team folklore
5. stale-active reconciliation from persisted local evidence

This suggests that if we ever spin out an OSS project, the strongest position is
not "another multi-agent framework."

The stronger position is closer to:

- Git-native coordination layer for multi-agent coding teams

## Recommendations

Practical implications from this comparison:

1. Do not compete head-on with AutoGen, CrewAI, or LangGraph on generic agent orchestration.
2. Treat AgentBase as the most relevant adjacent benchmark for coding-team ergonomics.
3. If we spin out a project, emphasize:
   - registry
   - handoff
   - workcycle
   - repo-local coordination
   - stale-active recovery
4. Keep Mycel role policy and planning cadence out of that first OSS product boundary.

## Sources

- AutoGen: <https://github.com/microsoft/autogen>
- CrewAI GitHub: <https://github.com/crewAIInc/crewAI>
- CrewAI open-source page: <https://crewai.com/open-source>
- LangGraph: <https://github.com/langchain-ai/langgraph>
- LangGraph Supervisor: <https://github.com/langchain-ai/langgraph-supervisor-py>
- LangGraph Swarm: <https://github.com/langchain-ai/langgraph-swarm-py>
- AWS Agent Squad: <https://github.com/awslabs/agent-squad>
- AgentBase: <https://github.com/AgentOrchestrator/AgentBase>
- AgentBase multi-agent docs: <https://docs.agentbase.sh/primitives/essentials/multi-agents>
