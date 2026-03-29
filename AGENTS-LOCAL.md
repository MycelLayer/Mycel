# AGENTS-LOCAL.md

Status: active repo-local overlay for communication defaults

This file augments `AGENTS.md` with repo-local or user-local overlays only.
Keep shared workflow rules in `AGENTS.md`.

## Communication

- Respond to the user in Traditional Chinese (`zh-TW`) by default unless the user explicitly asks for another language.

## GitHub

- For any GitHub-related action in this workspace, agents should use the `Mycel-agent` path backed by the agent `GH_TOKEN` exported from `~/.bashrc`.
- The host-provided GitHub connector should be treated as read-only in this workspace because its authenticated identity may differ from `Mycel-agent`; do not use it for issue comments, pull-request comments, reviews, labels, reactions, branch mutations, or any other GitHub-side write action.
- All GitHub write actions, including agent commit/push workflow for GitHub-facing work, must go through the local `gh` / git path with the `Mycel-agent` identity and the agent `GH_TOKEN`, not the user's personal GitHub identity, the Codespaces default `GITHUB_TOKEN`, or a differently authenticated connector session, unless the user explicitly asks otherwise.
