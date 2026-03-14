# Enterprise Messenger App Layer

Status: design draft

This note describes how Mycel can carry an enterprise instant messenger app for internal planning, operations, and compliance-heavy collaboration without turning the core protocol into a chat transport, DLP engine, or secret-destruction system.

The main design principle is:

- Mycel carries directory state, conversation policy, membership state, message envelopes, and compliance history
- runtime services handle blob delivery, key custody, timer execution, confidential-content scanning, and bridge integration
- conforming clients render an accepted conversation view under the active profile while preserving auditability

See also:

- [`DESIGN-NOTES.mycel-app-layer.en.md`](./DESIGN-NOTES.mycel-app-layer.en.md): general application-layer split
- [`DESIGN-NOTES.realtime-media-app-layer.en.md`](./DESIGN-NOTES.realtime-media-app-layer.en.md): a related example of keeping transport outside Mycel core

## 0. Goal

Enable Mycel to support an enterprise messenger that is useful for:

- employee roster management
- department and project group management
- policy-bound internal messaging
- confidential message filtering and quarantine
- timer-driven message expiry and destruction workflows
- compliance audit, escalation, and legal hold

Keep in Mycel:

- org and roster definitions
- group and membership state
- conversation metadata
- message envelopes and policy labels
- filter decisions and release history
- retention and destruction policy state
- audit trails and exception approvals

Keep outside Mycel core:

- real-time transport delivery
- push notification infrastructure
- message blob storage
- plaintext secret handling
- enterprise key management
- timer execution
- DLP engines, antivirus scanners, and HRIS / IdP bridge credentials

## 1. Design Rules

This messenger app should follow eight rules.

1. Revision replay MUST remain side-effect free.
2. Plaintext secrets SHOULD NOT be stored as ordinary replicated Mycel message bodies when true destruction is required.
3. Roster changes, membership changes, and permission changes MUST be explicit signed objects.
4. Confidentiality filtering MUST be represented as explicit policy decisions, not silent message rewriting.
5. Timer-driven destruction MUST be modeled as governed state plus runtime receipts, not as an assumption that append-only history can erase itself.
6. Legal hold and destruction override MUST be first-class policy state.
7. Accepted conversation rendering SHOULD come from fixed profile rules rather than ad hoc client trust.
8. External enterprise bridges MAY assist with sync, but their actions MUST write auditable receipts back into Mycel.

The critical implication is:

- if the enterprise needs strong "message gone after expiry" behavior, Mycel should store a signed envelope, retention contract, and content digest, while the revocable ciphertext and keys live in an external sealed store

## 2. Layer Split

### 2.1 Client Layer

The client is the employee-facing or operator-facing surface.

Responsibilities:

- display accepted roster, groups, conversations, and message state
- compose new messages and management intents
- display confidentiality labels and delivery restrictions
- display destruction timers, legal holds, and release states
- display audit-visible filter or quarantine outcomes

Non-responsibilities:

- do not define accepted policy on their own
- do not bypass group or roster policy
- do not silently replace blocked content with edited content
- do not hold enterprise master secrets inside replicated objects

### 2.2 Enterprise Runtime Layer

The runtime is the bridge and policy executor.

Responsibilities:

- synchronize employee roster data from approved upstream systems
- enforce conversation membership and device/session policy
- deliver message blobs through approved enterprise channels
- execute confidential-content scanning or classification
- execute destruction timers, key revocation, and purge workflows
- publish explicit receipts and policy outcomes back into Mycel

Non-responsibilities:

- do not redefine protocol verification
- do not treat unaccepted branch state as policy truth
- do not mutate accepted history silently

### 2.3 Effect Layer

The effect layer is the explicit representation of enterprise-side actions.

Examples:

- `idp.sync-roster`
- `message.blob-upload`
- `message.deliver`
- `dlp.scan`
- `message.quarantine`
- `key.revoke`
- `blob.purge`
- `legal-hold.apply`

Effect objects should remain explicit, replay-safe, and auditable.

## 3. Core Object Families

### 3.1 Enterprise Messenger Manifest

Defines the app itself.

Suggested fields:

- `app_id`
- `app_version`
- `directory_documents`
- `group_documents`
- `conversation_documents`
- `message_documents`
- `compliance_documents`
- `retention_documents`
- `allowed_effect_types`
- `runtime_profile`

Purpose:

- identify the messenger app
- declare participating document families
- declare what effect classes are allowed

### 3.2 Employee Roster Entry

Represents one enterprise identity.

Suggested fields:

- `employee_id`
- `person_ref`
- `employment_state`
- `department_id`
- `manager_ref`
- `job_title`
- `policy_tier`
- `contact_points`
- `effective_at`
- `supersedes_roster_entry`

Suggested `employment_state` values:

- `active`
- `leave`
- `contractor`
- `suspended`
- `terminated`

Purpose:

- define who can participate
- link org policy to a stable identity
- preserve explicit join, leave, and suspension history

### 3.3 Group Document

Represents a managed department, project space, or control room.

Suggested fields:

- `group_id`
- `group_kind`
- `display_name`
- `owner_refs`
- `default_message_policy`
- `membership_policy`
- `retention_policy_ref`
- `classification_policy_ref`
- `created_at`
- `updated_at`

Suggested `group_kind` values:

- `department`
- `project`
- `incident`
- `leadership`
- `vendor-bridge`

### 3.4 Membership Grant Document

Represents who is allowed into a group or conversation.

Suggested fields:

- `membership_id`
- `subject_ref`
- `target_group_id`
- `role`
- `grant_state`
- `granted_by`
- `granted_at`
- `expires_at`
- `supersedes_membership`

Suggested `role` values:

- `member`
- `group-admin`
- `compliance-reviewer`
- `incident-commander`
- `guest`

Suggested `grant_state` values:

- `pending`
- `active`
- `suspended`
- `revoked`
- `expired`

### 3.5 Conversation Document

Represents one chat room, direct thread, or announcement channel.

Suggested fields:

- `conversation_id`
- `conversation_kind`
- `group_id`
- `participant_refs`
- `posting_policy`
- `visibility_policy`
- `classification_policy_ref`
- `retention_policy_ref`
- `destruction_mode`
- `status`
- `created_at`

Suggested `conversation_kind` values:

- `direct`
- `team-room`
- `announcement`
- `war-room`
- `case-room`

Suggested `destruction_mode` values:

- `none`
- `timer-hide`
- `timer-revoke`
- `timer-purge`

### 3.6 Message Envelope Document

Represents a message without assuming the plaintext itself is permanently replicated.

Suggested fields:

- `message_id`
- `conversation_id`
- `sender_ref`
- `sent_at`
- `message_kind`
- `content_digest`
- `blob_ref`
- `key_ref`
- `classification_label`
- `delivery_scope`
- `expiry_policy_ref`
- `reply_to`
- `supersedes_message`

Suggested `message_kind` values:

- `text`
- `file`
- `announcement`
- `task-card`
- `approval-request`

Purpose:

- identify the message
- bind it to policy and retention controls
- support verification without requiring plaintext replication

### 3.7 Compliance Decision Document

Represents classification, filter, quarantine, or release outcomes.

Suggested fields:

- `decision_id`
- `target_message_id`
- `decision_kind`
- `decision_state`
- `classifier_ref`
- `reason_code`
- `reason_summary`
- `issued_at`
- `release_scope`
- `supersedes_decision`

Suggested `decision_kind` values:

- `classify`
- `quarantine`
- `redact-view`
- `release`
- `escalate`
- `false-positive-clear`

Suggested `decision_state` values:

- `pending-review`
- `blocked`
- `restricted`
- `released`
- `held`

This object is what prevents "confidential filtering" from becoming an invisible rewrite.

### 3.8 Retention Contract Document

Represents the governing retention and destruction terms for one message set or conversation.

Suggested fields:

- `retention_contract_id`
- `target_ref`
- `retention_class`
- `destroy_after`
- `destroy_mode`
- `legal_hold_state`
- `hold_reason`
- `approved_by`
- `effective_at`

Suggested `retention_class` values:

- `standard`
- `confidential`
- `restricted`
- `regulated`

Suggested `destroy_mode` values:

- `hide-only`
- `key-revoke`
- `blob-purge`
- `key-revoke-and-purge`

### 3.9 Destruction Receipt

Represents what the runtime actually destroyed or revoked.

Suggested fields:

- `destruction_receipt_id`
- `retention_contract_id`
- `target_message_id`
- `executor`
- `action_taken`
- `started_at`
- `finished_at`
- `result_state`
- `artifact_digest`
- `error_summary`

Purpose:

- prove destruction workflow execution
- record whether a timer succeeded, was blocked by legal hold, or partially failed
- support compliance audit without pretending immutable history vanished

## 4. Enterprise Workflows

### 4.1 Employee Roster Management

Recommended flow:

1. HRIS or IdP bridge publishes roster-sync intents.
2. Runtime validates the source system and writes roster-entry updates.
3. Group and conversation membership policies consume accepted roster state.
4. Suspended or terminated staff lose delivery and key-access eligibility through follow-on policy objects and receipts.

Important rule:

- roster state should control who may receive new content, but past audit visibility should still be governed explicitly rather than disappearing silently

### 4.2 Group Management

Recommended model:

- keep department groups, project groups, and incident rooms as separate group objects
- let conversations inherit default policy from groups while still allowing stricter local overrides
- record owner changes, guest access, and emergency access as explicit membership documents

This supports:

- employee directory-backed group creation
- temporary project rooms
- executive channels
- incident command rooms with elevated retention and access policy

### 4.3 Confidential Message Filtering

Filtering should be modeled as a policy pipeline, not a hidden content mutation.

Recommended stages:

1. A sender publishes a message envelope that references sealed content.
2. Delivery runtime or review runtime requests a `dlp.scan` effect.
3. The scanning system returns labels or risk codes.
4. A compliance decision document records one of: release, restrict, quarantine, or escalate.
5. The accepted conversation view shows the allowed result for the active profile.

Good enterprise behavior is:

- employees see that a message was held or restricted
- compliance reviewers can inspect reason codes and release history
- the original content is not silently replaced by a "clean" version without trace

### 4.4 Destruction Timer and Legal Hold

Timer-based deletion must separate user experience from cryptographic reality.

Recommended model:

- `timer-hide`: readers stop rendering the content after expiry, but no strong destruction claim is made
- `timer-revoke`: runtime revokes message keys after expiry, making ciphertext unreadable to ordinary clients
- `timer-purge`: runtime revokes keys and requests blob purge in the sealed store

Legal hold behavior:

- legal hold should freeze `timer-revoke` or `timer-purge`
- the hold state must be visible in retention contracts and destruction receipts
- hold release should be another explicit policy event

This is essential for enterprise planning because "self-destruct" and "records retention" usually conflict unless the system models both directly.

## 5. Suggested Policy Profiles

The messenger should support at least three policy profiles.

### 5.1 Standard Team Chat

Use for everyday internal coordination.

Characteristics:

- broad employee-group membership
- ordinary DLP classification
- `timer-hide` optional
- standard audit retention

Tradeoff:

- easiest to use, but weakest destruction guarantee

### 5.2 Restricted Project Room

Use for finance, HR, procurement, or sensitive partnership work.

Characteristics:

- explicit membership grants
- mandatory classification labels
- quarantine-before-release for attachments
- `timer-revoke` or `timer-purge`

Tradeoff:

- stronger control, but more review friction and runtime dependency

### 5.3 Sealed Executive / Incident Room

Use for crisis handling, security response, or board-level coordination.

Characteristics:

- narrow roster eligibility
- out-of-band approval for membership changes
- stronger device and export restrictions
- default legal-hold awareness
- aggressive key revocation on expiry

Tradeoff:

- strongest control posture, but highest operational cost and support burden

## 6. Feature Map

A typical enterprise IM usually needs to be planned across at least twelve capability groups.

### 6.1 Identity and Account Governance

Common capabilities:

- `SSO`
- automated joiner / mover / leaver provisioning
- `SCIM` or HRIS / IdP roster sync
- guest, contractor, and temporary accounts
- suspension, reinstatement, and forced session invalidation

This group determines who can enter the system, when access ends, and whether the directory stays aligned with the real enterprise org.

### 6.2 Group and Conversation Space Management

Common capabilities:

- department groups
- project groups
- announcement channels
- incident / war rooms
- direct messages
- guest rooms

This group determines the communication boundary inside the enterprise and which rooms require stricter membership and retention policy.

### 6.3 Messaging and Interaction

Common capabilities:

- threads
- `@mention`
- quoted reply
- pinning
- drafts
- scheduled send
- message edit and recall
- read receipts
- presence / status messages

This group is closer to day-to-day user experience, but still has to stay compatible with compliance and retention policy.

### 6.4 Collaboration Attachments and Workflow Objects

Common capabilities:

- file sharing
- attachment preview
- task cards
- approval requests
- polls
- calendar invites
- meeting links
- knowledge-base or ticket links

This group is what moves the product from "chat tool" toward "work coordination entry point."

### 6.5 Search, Organization, and Knowledge Retention

Common capabilities:

- full-text search
- filtering by person / group / label / date
- favorites
- pinned messages
- archiving
- summaries
- history review

This group determines whether messages become durable work knowledge rather than one-time chat noise.

### 6.6 Compliance, Audit, and Legal Support

Common capabilities:

- audit logs
- export
- eDiscovery
- legal hold
- retention policy
- review workflows
- incident escalation
- admin audit views

This is one of the clearest boundaries between enterprise IM and ordinary consumer chat.

### 6.7 Security and Data Protection

Common capabilities:

- DLP
- sensitive-content classification
- attachment quarantine
- malware scanning
- forwarding / copy / download restrictions
- device binding
- key revocation
- encrypted blob purge

This group maps directly to the `Compliance Decision`, `Retention Contract`, and `Destruction Receipt` objects in this design.

### 6.8 Device, Session, and Endpoint Management

Common capabilities:

- multi-device login
- session management
- remote sign-out
- device trust tiers
- `BYOD` policy
- `MDM` / `MAM` integration
- offline-message policy

This group determines how much control the enterprise has over endpoints and whether "who can see" can be governed separately from "which device can see."

### 6.9 Notifications, On-Call, and Operational Coordination

Common capabilities:

- broadcast announcements
- shift handoff
- incident escalation
- reply SLA reminders
- cross-timezone notification control
- do-not-disturb policy

This group is often what makes enterprise IM operationally indispensable for support, SRE, or security teams.

### 6.10 Integrations, Automation, and Bots

Common capabilities:

- bots
- webhooks
- workflow automation
- ticketing integration
- `CRM` / `ERP` / `HR` integrations
- approval and reminder flows

This group lets enterprise IM connect to existing internal systems rather than becoming an isolated island.

### 6.11 Deployment, Tenancy, and Data Sovereignty

Common capabilities:

- multi-tenant isolation
- data partitioning
- data residency
- backup and disaster recovery
- `BYOK`
- `KMS` / `HSM` integration

This group determines whether the product can enter enterprises or public-sector environments with stricter sovereignty and compliance requirements.

### 6.12 User Experience and Accessibility

Common capabilities:

- consistent desktop and mobile experience
- multilingual support
- translation
- speech-to-text
- accessibility support
- noisy-channel mute and summarization

This group looks more product-facing, but it often decides adoption success.

## 7. Phased Planning

If this enterprise messenger is treated as a product plan, it should be staged in at least three phases.

### 7.1 MVP

Start with the capabilities that prove the enterprise governance model:

1. roster entries and roster sync
2. managed groups and membership grants
3. conversation metadata
4. message envelopes with sealed blob references
5. classification and quarantine decisions
6. retention contracts and destruction receipts
7. basic audit logging and admin review surfaces

Tradeoff:

- this proves the core enterprise value early, but intentionally keeps user experience and integration depth narrow

### 7.2 Phase 2

Add the capabilities most often required for real enterprise adoption:

- `SSO` / `SCIM`
- guest access
- announcement channels
- search / archive / pin / favorite
- read receipts / presence
- `BYOD` / device policy
- webhook / bot / approval-request workflow
- eDiscovery and legal-hold operator views

Tradeoff:

- adoption friction drops a lot, but runtime, policy, and admin-surface complexity rise

### 7.3 Phase 3

Expand toward high-control and high-operations environments:

- incident / war-room package
- stronger export restrictions
- finer-grained data residency and tenant partitioning
- deeper `BYOK` / `KMS` / `HSM` integration
- advanced workflow orchestration and cross-system automation
- calls, meetings, speech-to-text, and cross-system federation

Tradeoff:

- this opens higher-requirement markets, but both product and operational cost increase meaningfully

## 8. Why Mycel Fits This Layer

Mycel is a good fit for this messenger layer because it can preserve:

- explicit roster and membership history
- accepted conversation policy under fixed governance
- audit-visible compliance and release decisions
- retention and destruction state with signed history
- app-specific views without turning the protocol core into a proprietary enterprise server

Mycel is not trying to replace:

- transport protocols
- enterprise KMS or HSM systems
- content-scanning engines
- notification gateways
- archival blob stores

## 9. Minimal Planning Slice

If the team wants a narrow first version, start with:

1. roster entries
2. managed groups
3. conversation metadata
4. message envelopes with sealed blob references
5. classification and quarantine decisions
6. retention contracts and destruction receipts

That slice is enough to prove the enterprise planning model before expanding to calls, bots, workflow tasks, or external federation.
