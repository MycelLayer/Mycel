# Donation App Layer

Status: design draft

This note describes a donation-oriented application layer carried by Mycel while keeping payment execution outside the core protocol.

The main design principle is:

- Mycel carries donation campaigns, donation intents, settlement state, allocation state, and audit history
- a client lets users inspect campaigns, create donation intents, and inspect receipts
- a payment runtime executes external payment actions
- the core protocol remains neutral and purely technical

## 0. Goal

Enable Mycel to carry a durable donation workflow without turning the core protocol into a payment processor.

Keep in Mycel:

- donation app definition
- campaign state
- donation intents
- settlement receipts
- allocation and reporting state
- governance and audit history

Keep outside Mycel core:

- payment execution
- card or bank credential handling
- wallet secret handling
- processor-specific authentication
- irreversible settlement side effects

## 1. Three Layers

### 1.1 Client Layer

The client is the user-facing layer.

Responsibilities:

- display active campaigns and donation targets
- create donation intents or pledges
- display settlement status
- display receipts, allocations, and audit trails
- render campaign-specific UI

Non-responsibilities:

- do not execute payment side effects by default
- do not hold replicated payment secrets
- do not redefine accepted allocation rules

### 1.2 Runtime Layer

The runtime is the payment executor and settlement observer.

Responsibilities:

- watch accepted donation state
- create or reconcile payment sessions
- verify allowed payment methods
- publish settlement receipts
- publish failure or retry receipts

Non-responsibilities:

- do not redefine protocol verification
- do not bypass accepted-head governance
- do not treat unaccepted branches as executable payment instructions

### 1.3 Effect Layer

The effect layer explicitly represents payment-related side effects.

Examples:

- create payment session
- poll settlement status
- verify on-chain confirmation
- send donor notification

Effect objects should remain auditable, replay-safe, and explicit.

## 2. Design Rules

The Donation App Layer should follow five rules.

1. Revision replay MUST remain side-effect free.
2. Payment execution MUST happen outside the core protocol.
3. Settlement evidence and allocation decisions SHOULD be preserved as app-level records.
4. A client SHOULD separate payment completion from allocation acceptance.
5. Local safety policy SHOULD NOT silently rewrite accepted donation state.

## 3. Core Donation Objects

### 3.1 Donation App Manifest

Defines the donation application itself.

Suggested fields:

- `app_id`
- `app_version`
- `campaign_documents`
- `intent_documents`
- `receipt_documents`
- `allocation_documents`
- `allowed_effect_types`
- `runtime_profile`
- `capability_policy`

Purpose:

- identify the donation app
- declare participating document families
- declare payment and runtime expectations

### 3.2 Donation Campaign

Defines one active or historical campaign.

Suggested fields:

- `campaign_id`
- `app_id`
- `title`
- `description`
- `status`
- `target_amount`
- `currency_policy`
- `accepted_payment_methods`
- `allocation_policy_ref`
- `created_at`
- `updated_at`

Typical `status` values:

- `draft`
- `active`
- `paused`
- `closed`
- `archived`

### 3.3 Donation Target

Defines where funds are meant to go.

Suggested fields:

- `target_id`
- `campaign_id`
- `destination_ref`
- `allocation_label`
- `currency_policy`
- `visibility`

Typical `visibility` values:

- `public`
- `restricted`
- `internal`

### 3.4 Donation Intent

Represents a user-side request or pledge to donate.

Suggested fields:

- `intent_id`
- `campaign_id`
- `target_id`
- `donor_ref`
- `intent_kind`
- `amount`
- `currency`
- `payment_method`
- `status`
- `created_at`
- `updated_at`

Typical `intent_kind` values:

- `direct-payment`
- `pledge`
- `recurring-consent`

Typical `status` values:

- `pending`
- `payment-requested`
- `settled`
- `failed`
- `cancelled`

### 3.5 Donation Receipt

Represents settlement or payment confirmation observed by the runtime.

Suggested fields:

- `receipt_id`
- `intent_id`
- `executor`
- `payment_ref`
- `amount_received`
- `currency`
- `status`
- `settled_at`
- `processor_summary`
- `error_summary`

Typical `status` values:

- `confirmed`
- `pending-confirmation`
- `failed`
- `reversed`

### 3.6 Allocation Resolution

Represents how settled donations are allocated or recognized.

Suggested fields:

- `resolution_id`
- `campaign_id`
- `covered_receipts`
- `allocations`
- `accepted_under_profile`
- `decision_trace_ref`
- `updated_at`

Purpose:

- separate payment settlement from allocation governance
- preserve accepted distribution state
- support auditability

## 4. Accepted-State Driven Execution

The runtime should execute payment-related effects only from accepted state.

Recommended rule:

1. read the active accepted head for campaign and intent state
2. identify newly accepted donation intents that require runtime work
3. check payment capability and runtime policy
4. execute allowed payment or settlement checks
5. publish donation receipts
6. let later accepted state incorporate those receipts and any allocation results

This keeps donation execution aligned with Mycel governance and traceability.

## 5. Example Flow

### Client

- shows an active campaign
- lets a donor create a `donation_intent`
- shows pending, settled, or failed state
- shows how settled funds were later allocated

### Runtime

- notices that an accepted donation intent requires a payment session or settlement check
- verifies that the chosen payment method is allowed
- interacts with the external payment system
- writes a `donation_receipt`

### Governance

- maintainers may publish allocation or campaign-closing state
- reader clients show the accepted allocation result under the active profile

## 6. Capability Model

The runtime should never execute arbitrary payment behavior from arbitrary replicated content.

Recommended capability controls:

- payment-method allowlist
- processor allowlist
- destination allowlist
- per-campaign currency restrictions
- minimum and maximum amount checks
- retry limits
- no direct secret material in replicated objects

Capabilities should be declared in app definition and enforced in runtime policy.

## 7. Secrets and Credentials

Secrets should never live in replicated Mycel objects.

Recommended rule:

- Mycel objects may name a credential reference
- only the runtime resolves the actual secret
- receipts should not expose card, bank, or wallet secrets

Examples:

- `credential_ref: vault:stripe-prod`
- `credential_ref: vault:wallet-signer-a`

## 8. Idempotency and Reconciliation

Donation flows must expect retries, delayed settlement, and duplicate notifications.

Recommended rules:

- every payment-side effect should carry an idempotency key
- every receipt should point back to one intent or payment request
- duplicate receipts should be reconciled as state, not treated as protocol corruption
- reversals and failures should be preserved as explicit records

## 9. Privacy and Audit

Donation workflows may need both privacy and accountability.

Recommended design:

- keep donor-visible IDs separate from runtime payment references
- allow deployments to use pseudonymous `donor_ref`
- separate public campaign reporting from restricted donor-identifying data
- preserve receipt and allocation audit links even when donor identity is minimized

## 10. Minimal v0.1 Donation Profile

For a first implementation, I recommend a narrow profile.

- one campaign family
- one donation-intent family
- one receipt family
- optional allocation resolution family
- no secrets in replicated state
- no automatic recurring execution
- one accepted settlement view plus audit-visible alternatives

Tradeoff:

- lower automation
- higher implementation clarity
- easier interoperability

## 11. Open Questions

- Should `donation_target` be a dedicated record family or a field inside campaign state?
- Should first-client settlement support only processor receipts, or also on-chain confirmations?
- Should allocation governance be mandatory for every receipt, or only for campaign-level reporting?
