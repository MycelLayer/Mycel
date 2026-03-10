# Client-Non-Discretionary Multi-View

Status: design draft

This note describes one way to keep Mycel multi-view while minimizing how much ordinary clients can influence the active accepted head.

## 0. Goal

Keep:

- multiple signed View objects
- multiple coexisting branches and heads
- deterministic accepted-head selection

Reduce:

- user-controlled view switching
- client-local selector overrides
- UI-driven policy mutation
- ad hoc local promotion of a preferred head

## 1. Core Idea

Mycel remains multi-view at the protocol layer, but conforming reader clients should have almost no discretionary power over the active accepted head.

The client should mostly:

- sync verified objects
- verify hashes and signatures
- compute the selector output
- display the accepted result and its trace

The client should not freely:

- edit selector parameters
- change maintainer weights
- force a preferred head to become active
- build an ad hoc local policy and present it as the active accepted view

## 2. Terminology

- `View object`: a signed governance signal object, not a user preference
- `View profile`: the fixed selector profile used to evaluate View objects
- `accepted head`: the unique selected revision for `(profile_id, doc_id, effective_selection_time)`
- `reader client`: a client that consumes accepted results but does not originate governance signals
- `curator client`: a client that can publish View objects, but still cannot change selector math

## 3. View Objects as Governance Signals

In this model, a View object does not mean "the user's chosen perspective."

It means:

- a maintainer's signed statement about accepted document revisions
- a signed commitment to a policy body
- one input into deterministic accepted-head selection

This keeps `view` as a replicated protocol object while removing it from ordinary end-user preference controls.

## 4. Profile-Locked Selection

Each network profile or document family should define one fixed View profile.

At minimum, the fixed profile includes:

- `policy_hash`
- `epoch_seconds`
- `epoch_zero_timestamp`
- `admission_window_epochs`
- `min_valid_views_for_admission`
- `min_valid_views_per_epoch`
- `weight_cap_per_key`
- tie-break order

Ordinary clients must not modify these values locally for the active accepted-head path.

## 5. Accepted-Head Computation

For each `(profile_id, doc_id, effective_selection_time)`:

1. load verified revisions for the document
2. load verified View objects matching the profile's `policy_hash`
3. compute eligible heads
4. derive maintainer signals
5. compute `selector_score`
6. apply the fixed tie-break order
7. emit one `accepted_head`

The accepted head is protocol-derived, not user-chosen.
It is also not a claim that the whole network has one universally accepted truth; it is the accepted result produced for one fixed profile at one effective selection time.

## 6. Client Roles

### 6.1 Reader Client

A conforming reader client:

- may sync and verify all objects
- may show raw heads and branch graphs
- may show decision traces
- must display the computed accepted head as active
- must not let the user replace the active accepted head with a discretionary local choice

### 6.2 Curator Client

A curator client:

- may create and sign View objects
- may publish governance signals to the network
- must still use the protocol-defined selector profile
- must not rewrite selector math for its own accepted-head output

### 6.3 Governance Update Tooling

If the profile itself changes, that should happen through explicit profile versioning or governance-update workflow, not through silent local client settings.

## 7. UI Rules

Recommended reader UI behavior:

- show one default accepted head
- show the governing `profile_id`
- show `effective_selection_time`
- show a machine-readable or inspectable decision trace
- show other heads only as alternatives, branches, or audit material

Avoid:

- "choose your preferred view" as a primary reader control
- hidden local overrides
- presenting raw branch choice as if it were the protocol-accepted result

## 8. What the Client May Still Influence

Even in this model, clients can still affect outcomes indirectly by:

- failing to sync enough objects
- implementing verification incorrectly
- implementing selector logic incorrectly
- hiding trace output or branch alternatives

So the design goal is not zero influence. The goal is to reduce discretionary influence and move decisions into verifiable protocol data.

## 9. Recommended Conformance Language

Suggested normative direction for a future spec revision:

- View objects are governance signals, not end-user preference objects.
- A conforming reader client MUST derive the active accepted head from verified objects and the fixed protocol-defined View profile only.
- A conforming reader client MUST NOT expose discretionary local policy controls that alter the active accepted head.
- A conforming reader client MAY expose raw heads, branch graphs, and rejected alternatives, but MUST NOT present them as the active accepted head unless another valid View profile governs that result.

## 10. Tradeoffs

Benefits:

- keeps Mycel multi-view
- reduces client-side divergence
- improves auditability
- lowers the chance that UI settings silently reshape acceptance

Costs:

- profile design becomes more important
- governance updates need a clearer versioning path
- reader and curator tooling should be separated more cleanly

## 11. Open Questions

- Should one network permit multiple fixed profiles, or only one active profile per document family?
- Should reader clients be allowed to inspect other valid profiles, or only the network default?
- Should `VIEW_ANNOUNCE` remain optional, or become effectively mandatory for governed multi-view networks?
- Should the implementation checklist split reader-client and curator-client requirements explicitly?
