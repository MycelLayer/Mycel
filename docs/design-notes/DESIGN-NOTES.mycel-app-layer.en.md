# Mycel App Layer

Status: design draft

This note describes an application layer carried by Mycel while keeping side-effect execution outside the core protocol.

The main design principle is:

- Mycel carries app definition, app state, intents, and effect history
- a runtime executes external side effects
- a client reads and edits app-facing state but does not directly define transport or execution semantics

## 0. Goal

Enable Mycel to carry application behavior without turning the core protocol into a side-effect engine.

Keep in Mycel:

- app definition
- app state
- app governance
- app intents
- effect requests
- effect receipts

Keep outside Mycel core:

- HTTP execution
- timers
- secret handling
- network credentials
- OS-level or cloud-level side effects

## 1. Three Layers

The App Layer should be modeled as three distinct layers.

### 1.1 Client Layer

The client is the human-facing or local-user-facing layer.

Responsibilities:

- display accepted app state
- create intents or actions
- inspect effect status
- inspect receipts and audit trails
- render app-specific UI

Non-responsibilities:

- do not directly execute privileged effects by default
- do not redefine accepted-head rules
- do not embed local secrets into replicated Mycel objects

### 1.2 Runtime Layer

The runtime is the effect executor.

Responsibilities:

- watch accepted heads
- read app manifests and app state
- resolve pending effect requests
- enforce capability policy
- execute allowed side effects
- publish effect receipts back into Mycel

Non-responsibilities:

- do not redefine protocol verification
- do not bypass governance rules
- do not treat any branch as executable unless it is accepted under the active profile

### 1.3 Effect Layer

The effect layer is the explicit representation of side effects.

Responsibilities:

- describe what external action is requested
- record what external action was actually performed
- preserve a verifiable execution trail

Effect objects should be auditable, replay-safe, and explicit.

## 2. Design Rule

Revision replay MUST remain side-effect free.

This means:

- replaying Mycel history reconstructs state
- replaying Mycel history does not re-trigger HTTP calls
- effect execution is driven by runtime logic observing accepted state transitions

This rule is essential to keep verification deterministic.

## 3. Core App Objects

An App Layer can be expressed with a small set of object families.

### 3.1 App Manifest

Defines the application itself.

Suggested fields:

- `app_id`
- `app_version`
- `entry_documents`
- `state_documents`
- `intent_documents`
- `allowed_effect_types`
- `runtime_profile`
- `capability_policy`

Purpose:

- identify the app
- declare what documents belong to it
- declare what kinds of effects may exist

### 3.2 App State Document

Stores accepted application state.

Examples:

- workflow state
- queue state
- approval state
- resource summary
- last successful effect status

### 3.3 Intent or Action Document

Stores requested business actions.

Examples:

- submit task
- request publication
- request sync
- request API fetch

An intent is not itself a side effect.
It is a stateful request that may later produce an effect request.

### 3.4 Effect Request

Represents a request for external execution.

Suggested fields:

- `effect_request_id`
- `app_id`
- `effect_type`
- `trigger_revision`
- `requested_by`
- `request_payload`
- `idempotency_key`
- `requested_at`

Examples of `effect_type`:

- `http.fetch`
- `http.post`
- `webhook.deliver`
- `notification.send`

### 3.5 Effect Receipt

Represents what the runtime actually executed.

Suggested fields:

- `effect_receipt_id`
- `effect_request_id`
- `executor`
- `status`
- `started_at`
- `finished_at`
- `response_digest`
- `response_summary`
- `error_summary`

Purpose:

- prove that execution happened
- record success or failure
- support audit and retry logic

## 4. Accepted-Head Driven Execution

The runtime should execute effects only from accepted state.

Recommended rule:

1. read the active accepted head for the app state documents
2. identify newly accepted effect requests
3. check capability and runtime policy
4. execute allowed effects
5. publish effect receipts
6. let later accepted state incorporate those receipts

This keeps app execution aligned with Mycel governance.

## 5. HTTP Example

For an HTTP-capable app:

### Client

- shows a form or task UI
- writes an intent requesting an outbound call
- displays pending or completed execution state

### Runtime

- notices that an accepted effect request exists
- verifies that `http.post` is allowed
- checks URL allowlist and runtime policy
- performs the HTTP request
- writes a receipt summarizing the response

### Effect Objects

- `effect_request`: "POST this payload to this approved endpoint"
- `effect_receipt`: "runtime X executed it at time T and got status 200"

## 6. Capability Model

The runtime should never execute arbitrary effects from arbitrary replicated content.

Recommended capability controls:

- effect type allowlist
- domain allowlist for HTTP
- method allowlist
- timeout limits
- payload size limits
- response size limits
- no direct local-network access unless explicitly allowed

Capabilities should be declared in app definition and enforced in runtime policy.

## 7. Secrets and Credentials

Secrets should not live in replicated Mycel objects.

Recommended rule:

- Mycel objects may name a credential reference
- only the runtime resolves the actual secret
- receipts should not expose secrets

Examples:

- `credential_ref: vault:mailgun-prod`
- not `api_key: ...`

## 8. Idempotency and Duplicate Execution

Because multiple runtimes may observe the same accepted state, duplicate execution must be expected.

Recommended controls:

- every effect request has an `idempotency_key`
- runtimes persist seen request IDs
- receipts point back to the exact request ID
- app logic treats duplicate receipts as reconcilable state, not as protocol corruption

## 9. Governance Integration

The App Layer should respect Mycel governance rather than bypassing it.

Recommended integration:

- only accepted heads may enter the executable effect queue
- view-maintainers indirectly control what becomes executable by controlling accepted heads
- editor-maintainers may create app changes or requests, but those are not executable until accepted

This matches the current profile-governed accepted-head model.

## 10. Minimal Lifecycle

A minimal app lifecycle looks like this:

1. define `app_manifest`
2. create app state documents
3. submit intent
4. accept intent into the active accepted head
5. materialize or derive an effect request
6. runtime executes the effect
7. runtime writes effect receipt
8. accepted state reflects the result

## 11. Practical First Apps

Good first App Layer examples:

- webhook workflow app
- approval app
- content publish app
- notification app
- external sync app

These are easier than trying to build a fully general remote-execution platform first.

## 12. Non-Goals

This App Layer should not try to be:

- a smart-contract VM
- an unrestricted function-as-a-service platform
- a deterministic replay engine for external HTTP
- a secret storage system

## 13. Suggested Future Spec Direction

If this design matures, future spec work could define:

- app manifest schema
- effect request schema
- effect receipt schema
- runtime profile schema
- capability policy schema
- client/runtime conformance split

## 14. Open Questions

- Should effect requests be first-class protocol objects or app-level document content?
- Should receipts always be signed by runtime keys?
- Should runtimes be admitted by governance in the same way as view-maintainers?
- Should the first App Layer profile support only HTTP, or a smaller subset such as webhook delivery only?
