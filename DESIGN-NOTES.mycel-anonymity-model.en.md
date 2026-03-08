# Mycel Anonymity Model

Status: design draft

This note describes how a Mycel-based system should reason about anonymity as a multi-layer property rather than a single transport setting.

The main design principle is:

- transport anonymity is necessary but not sufficient
- protocol metadata must be minimized intentionally
- client and runtime behavior can still deanonymize users
- anonymity and auditability must be balanced explicitly

## 0. Goal

Enable Mycel deployments to reduce network-identity leakage, metadata correlation, and local attribution risk without pretending that any single tool provides complete anonymity.

This note does not define one mandatory anonymity mode for all deployments.

Instead, it defines:

- anonymity layers
- threat surfaces
- recommended controls
- realistic limits

## 1. Anonymity Is Multi-layer

For Mycel, anonymity must be considered across at least four layers.

### 1.1 Transport Layer

This layer concerns how packets move between peers.

Examples:

- Tor
- onion services
- transport relays
- VPNs or other network indirection

Transport anonymity helps hide source IPs, but it does not by itself remove identity leakage from content, timing, or long-lived identifiers.

### 1.2 Protocol Metadata Layer

This layer concerns what Mycel messages and objects reveal.

Examples:

- sender identifiers
- stable node IDs
- timestamps
- document references
- signer references
- profile references

If protocol metadata is highly linkable, transport anonymity alone is weak.

### 1.3 Client and Runtime Layer

This layer concerns how local software behaves.

Examples:

- caching
- local account binding
- logging
- device identifiers
- runtime receipts
- effect execution patterns

Even when transport is anonymized, client and runtime behavior may reveal long-lived identity.

### 1.4 Replication and Governance Layer

This layer concerns how replicated history and accepted state can be correlated over time.

Examples:

- repeated signatures from the same key
- stable maintainer identities
- fixed signer-set membership
- recurring synchronization timing

Mycel's verifiable history is valuable, but it also creates long-term correlation risk.

## 2. Threat Model

An anonymity-aware Mycel deployment should consider at least these risks:

- network observers correlating source IP and timing
- peers correlating stable sender identifiers over time
- clients leaking identity through logs, caches, or secrets
- runtimes linking accepted events to external payment or effect systems
- repeated signatures linking a user or signer to long-lived activity
- traffic analysis based on synchronization frequency or object-fetch patterns

## 3. Tor Is Helpful but Not Sufficient

Tor should be treated as a transport-layer anonymity tool, not as a complete anonymity model.

Tor can help with:

- hiding direct IP-level source information
- reducing direct peer-to-peer address exposure
- making traffic origin harder to observe

Tor does not by itself solve:

- stable protocol identifiers
- linkable governance keys
- timing correlation
- local logging or runtime leakage
- application-level account binding

For Mycel, the correct model is:

- Tor may be one transport option
- anonymity still depends on metadata discipline and local behavior

## 4. Metadata Minimization

A deployment that wants stronger anonymity should reduce unnecessary metadata exposure.

Recommended controls:

- avoid exposing long-lived node identifiers unless required
- avoid exposing sender fields when not required by the active profile
- reduce timestamp precision when full precision is not needed
- avoid embedding real-world account references in replicated objects
- separate public object references from local session identifiers

The principle is:

- only replicate metadata that is needed for verification, routing, or governance

## 5. Role Separation

A deployment should not assume that one identity should play every role.

Recommended separations:

- reader identity separate from maintainer identity
- maintainer identity separate from signer identity
- signer identity separate from effect runtime identity
- anonymous reading separate from public governance participation

This reduces cross-layer linkage.

Tradeoff:

- stronger separation improves anonymity
- stronger separation makes operations more complex

## 6. Client Hardening

Client behavior can easily undermine network anonymity.

Recommended controls:

- limit persistent local logs
- separate local profiles for different identities
- avoid embedding wallet, payment, or sensor identifiers into replicated records
- avoid sending application telemetry outside the accepted deployment profile
- support local cache cleanup or compartmentalized storage

If a client mixes anonymous reading with authenticated payment or governance operations in one local identity context, anonymity is reduced.

## 7. Runtime Hardening

Runtimes are a major anonymity risk because they interact with external systems.

Recommended controls:

- keep external effect execution separate from reader identities
- avoid publishing unnecessary executor details
- minimize runtime-specific identifiers in receipts
- separate payment, sensor, and governance runtimes where possible
- avoid correlating external service account IDs with Mycel identities

The runtime should expose only the minimum execution evidence needed for audit.

## 8. Replication Strategy

Replication improves durability but can also increase visibility.

Recommended controls:

- allow reader nodes to fetch only the object families they need
- support role-specific replication policies
- avoid forcing all peers to mirror all sensitive app-layer records
- preserve accepted-state verification without requiring universal visibility of all local artifacts

A deployment may choose:

- wider replication for audit
- narrower replication for anonymity

This tradeoff should be explicit.

## 9. Governance Tradeoffs

Mycel's governance and audit model naturally creates attribution pressure.

Examples:

- signed governance signals
- fixed signer sets
- accepted-head traces
- disbursement receipts

A deployment must decide which of the following it prefers more strongly:

- public accountability
- operational anonymity

These are not perfectly compatible.

Recommended approach:

- define anonymity expectations per role and per profile
- do not pretend that governance-signing identities are anonymous by default

## 10. Deployment Tiers

I recommend three practical anonymity tiers.

### 10.1 Basic Anonymous Transport

Characteristics:

- traffic uses Tor or equivalent transport indirection
- little or no protocol-level metadata reduction

Tradeoff:

- easy to deploy
- weak against metadata correlation

### 10.2 Metadata-aware Anonymous Deployment

Characteristics:

- anonymous transport
- minimized sender metadata
- reduced timestamp precision
- stronger role separation

Tradeoff:

- better anonymity
- more deployment complexity

### 10.3 Hardened Anonymous Deployment

Characteristics:

- anonymous transport
- metadata minimization
- compartmentalized clients
- separated runtimes
- role-specific replication
- explicit local-hardening rules

Tradeoff:

- strongest practical anonymity
- highest operational cost

## 11. Minimal First-client Rules

For a first anonymity-aware client, I recommend:

- support proxyable transport such as Tor
- keep anonymous reading separate from signer or payment operations
- avoid unnecessary long-lived local identifiers
- avoid publishing executor details by default
- document clearly which actions break anonymity assumptions

## 12. Non-goals

This note does not claim:

- perfect anonymity
- immunity to global traffic analysis
- anonymity for public governance signers by default
- anonymity preservation when users mix anonymous and identified workflows

## 13. Open Questions

- Should Mycel define one explicit anonymous deployment profile, or leave anonymity policy to deployments?
- Which metadata fields are truly required across all interoperable profiles?
- How much anonymity should be traded away for audit visibility in fund and governance workflows?
