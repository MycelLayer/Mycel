# Mycel Characteristics

Status: working explanation

This document explains several characteristic terms commonly used to describe
Mycel, what each term means in Mycel, and what it should not be mistaken for.

This is not a normative source. If a term here conflicts with the actual data
model, validation rules, or accepted-head derivation, follow
[`PROTOCOL.en.md`](../PROTOCOL.en.md), [`ROADMAP.md`](../ROADMAP.md), and the
relevant implementation instead.

## 1. `content-addressed`

In Mycel, `content-addressed` means the identity of certain core objects is
derived from canonical content rather than assigned by a central server.

What this means in Mycel:

- Objects such as `patch`, `revision`, `view`, and `snapshot` can derive a
  canonical object ID from content.
- Whether two objects are the same is determined first by whether their
  canonical content is the same, not by who uploaded them first or where they
  were stored.
- Replication, validation, and reconstruction can be built on
  content-addressed objects instead of depending on local database primary keys
  from one storage node.

What this does not mean:

- Every ID is a content hash.
- Mycel has no logical IDs. References such as `doc_id` and `block_id` are
  still logical references and are not the same thing as content-addressed IDs.
- Having a hash automatically gives full governance or accepted-head semantics.

## 2. `append-only`

In Mycel, `append-only` means verifiable history is primarily extended by adding
new objects rather than overwriting existing historical objects in place.

What this means in Mycel:

- New `patch`, `revision`, `view`, and `snapshot` objects extend history
  instead of directly modifying old objects.
- History needs to be preserved because it can be replayed or rebuilt, rather
  than reduced to only the latest state.
- Audit, validation, and accepted-state derivation are built on accumulated
  history.

What this does not mean:

- Every local index or cache is forbidden from being updated.
- There can be no garbage collection, compaction, or presentation-layer
  summaries.
- Every app layer must make all of its own derived data append-only too.

## 3. `replayable`

In Mycel, `replayable` means the currently visible state can be derived again
from verified historical objects, rather than depending only on some
non-reconstructable live database state.

What this means in Mycel:

- `revision` state can be validated by replaying patch and revision history.
- Parts of the store index or accepted reading can be rebuilt from existing
  objects instead of acting as the only source of truth.
- "Rebuild should produce the same result" is an important engineering goal.

What this does not mean:

- Every result comes from replay alone without any policy, profile, or
  governance signal.
- Replay is always cheap or immediate.
- Every app-layer side effect must be fully replayable.

## 4. `policy-driven`

In Mycel, `policy-driven` means some visible outcomes are not one hard-coded
answer, but are derived according to profiles, policy bundles, or other explicit
rules.

What this means in Mycel:

- Accepted head is not determined only by "latest timestamp" or "last write
  wins."
- Whether a head is accepted, how governance signals are handled, and which
  readings are treated as the default can all depend on fixed rules.
- App-layer or profile-layer logic can carry higher-level decisions instead of
  forcing every worldview into the protocol core.

What this does not mean:

- A client can arbitrarily choose whichever result it likes.
- Policy is just a matter of user interface preference.
- Every node will necessarily produce the same accepted head. Results still
  depend on the shared rules and the visible object set.

## 5. `head-selected`

In Mycel, `head-selected` means the system allows multiple valid heads to exist,
but derives the default adopted head through explicit selector rules.

What this means in Mycel:

- Mycel does not require the data model to permit only one head by design.
- Multiple candidate heads can coexist, and a selector can derive the accepted
  head within a fixed context.
- The selector is an important layer between the data model and the governance
  model, not just a UI sort order.

What this does not mean:

- Mycel is merely a generic multi-head version graph.
- Any multi-head situation is automatically merged into one truth.
- `accepted head` is the same thing as global consensus.

## 6. `governance-aware`

In Mycel, `governance-aware` means the system does not only preserve content
history, but also treats governance signals, roles, and rules as formal inputs
that affect accepted state.

What this means in Mycel:

- `view` is not merely an annotation. It is part of the governance signal set.
- Accepted reading may depend on view maintainers, profiles, and governance
  rules, not just on content itself.
- Relationship summaries, governance indexes, and inspect/list/publish surfaces
  are all part of reader-facing interpretation.

What this does not mean:

- The protocol core already bakes in every governance system.
- Governance necessarily implies centralized authority.
- Every app-layer governance rule must move into the protocol core.

## 7. How These Terms Fit Together in Mycel

Taken together, these six terms point to a description closer to this:

- Mycel is built on content-addressed objects.
- It preserves append-only verifiable history.
- It treats replayable and rebuildable state derivation as important.
- It allows valid heads to coexist, then derives the default adopted result
  through head selection.
- It treats policy and governance as first-class semantics rather than as a
  pure UI add-on layer.

If this needs to be compressed into one shorter English description:

- Mycel is a system built on content-addressed objects, preserving replayable
  history, and deriving default reading outcomes through policy-driven head
  selection and governance-aware interpretation.

## 8. Misreadings to Avoid

The following phrasings flatten these terms too much and are better avoided:

- "content-addressed = every ID is a hash"
- "append-only = nothing can ever be updated"
- "replayable = no local index or cache is needed"
- "policy-driven = users can pick whatever result they want"
- "head-selected = there is only one real head"
- "governance-aware = it must be centralized governance"
