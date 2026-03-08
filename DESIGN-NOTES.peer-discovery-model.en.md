# Peer Discovery Model

Status: design draft

This note describes how a Mycel-based system should discover, classify, and retain peers without assuming one universal global discovery mode.

The main design principle is:

- peer discovery should be bounded and policy-aware
- transport policy should constrain which peers are usable
- discovery sources should be explicit and enumerable
- different deployment modes should support different discovery breadth

## 0. Goal

Enable Mycel nodes to find and maintain useful peers while keeping discovery behavior compatible with transport, anonymity, governance, and deployment constraints.

This note defines:

- peer roles in discovery
- address classes
- discovery sources
- admission and retention rules
- deployment modes
- common failure cases

## 1. Discovery Roles

Not every node needs the same discovery behavior.

### 1.1 Reader Nodes

Reader nodes usually need bounded discovery only.

Typical needs:

- find one or more sync-capable peers
- refresh accepted state
- replace stale peers when needed

### 1.2 Governance-maintainer Nodes

Governance-maintainer nodes need more stable discovery.

Typical needs:

- fetch current document heads
- publish governance-related state
- maintain reliable peer relationships

### 1.3 Signer Nodes

Signer nodes should use narrow, explicit discovery.

Typical needs:

- maintain a known peer set
- avoid opportunistic broad-network discovery
- preserve operational stability and policy alignment

### 1.4 Runtime Nodes

Runtime nodes should usually follow the strictest discovery policy available in the deployment.

Typical needs:

- consume accepted state from known peers
- avoid broad or noisy network exploration

## 2. Peer Address Classes

Peer discovery should classify addresses explicitly.

Recommended address classes:

- `clearnet:host-port`
- `tor:onion-v3`
- `restricted:bootstrap-alias`
- `local:manual-peer`
- `relay:transport-ref`

Address classes should not be mixed silently.

A deployment profile should decide which classes are preferred, allowed, or forbidden.

## 3. Discovery Sources

Peer discovery should rely on explicit sources.

### 3.1 Local Bootstrap List

A static local list of peers or aliases.

Properties:

- predictable
- easy to reason about
- useful for restricted or Tor-oriented deployments

### 3.2 Peer-provided Manifest Discovery

Peers may advertise additional peers or served topics through manifests or local peer catalogs.

Properties:

- more dynamic
- useful for mesh expansion
- requires stronger filtering

### 3.3 Out-of-band Trusted Introduction

A user or operator obtains peer information outside the running network.

Examples:

- QR code
- signed document
- operator-managed config

Properties:

- high trust
- low automation

### 3.4 Federation List

A deployment may maintain one explicit membership list.

Properties:

- stable
- governance-friendly
- low flexibility

## 4. Admission Rules

A discovered peer should not become active automatically without checks.

Recommended admission checks:

1. address class is allowed by the active deployment profile
2. transport policy permits connection to that address class
3. peer responds with a valid wire-compatible session
4. capabilities match the local role's needs
5. the peer is not on a local denylist

Optional checks:

- topic compatibility
- profile compatibility
- minimum reliability history

## 5. Ranking and Retention

Discovered peers should be ranked and retained intentionally.

Suggested ranking inputs:

- successful session rate
- recent object availability
- transport compatibility
- role compatibility
- last successful sync time

Suggested retention states:

- `candidate`
- `active`
- `degraded`
- `quarantined`
- `expired`

A node should not keep every discovered peer forever.

## 6. Deployment Modes

I recommend three practical discovery modes.

### 6.1 Public Mesh Discovery

Characteristics:

- broader peer set
- more dynamic discovery sources
- more tolerance for peer churn

Allowed sources:

- local bootstrap
- peer-provided discovery
- optional public seed infrastructure

Tradeoff:

- more connectivity
- higher attack and metadata surface

### 6.2 Restricted Federation Discovery

Characteristics:

- peer set is mostly known in advance
- federation or operator list is primary
- little or no opportunistic expansion

Allowed sources:

- local bootstrap
- federation list
- trusted introduction

Tradeoff:

- stable and governable
- less flexible growth

### 6.3 Tor-oriented Bounded Discovery

Characteristics:

- onion-first addressing
- narrow bootstrap set
- explicit transport constraints
- no silent clearnet fallback

Allowed sources:

- local Tor-aware bootstrap list
- Tor-routed manifest discovery
- trusted introduction

Tradeoff:

- better anonymity posture
- narrower peer graph and slower recovery

## 7. Discovery Flow

The minimal bounded discovery flow is:

1. load configured bootstrap sources
2. classify candidate peer addresses
3. filter by active deployment profile
4. attempt transport-compatible session establishment
5. validate wire compatibility and required capabilities
6. mark the peer as `candidate` or `active`
7. refresh ranking and retention state over time

## 8. Refresh and Rotation

Peer discovery is ongoing, not one-time.

Recommended refresh behavior:

- periodically retry degraded peers
- expire peers that remain unavailable beyond policy limits
- replace failed peers from allowed discovery sources
- preserve a small stable active set for role-critical nodes

## 9. Failure Cases

### 9.1 Stale Bootstrap Entry

- mark as degraded or expired
- do not loop indefinitely without backoff

### 9.2 Transport Mismatch

- reject the peer for the active profile
- preserve a local reason code

### 9.3 Capability Mismatch

- do not promote the peer to active for that role
- keep it as candidate only if useful for another role

### 9.4 Manifest-expansion Abuse

- do not trust all advertised peers automatically
- apply admission checks to discovered peers from manifests

## 10. Minimal First-client Rules

For a first interoperable client, I recommend:

- one explicit bootstrap list
- explicit address-class filtering
- no universal peer promotion from peer advertisements
- bounded active peer set
- visible peer state (`candidate`, `active`, `degraded`, `expired`)

## 11. Open Questions

- Should peer-provided discovery become a normative wire feature later, or remain deployment-local?
- Should topic compatibility be mandatory in discovery admission for v0.1 clients?
- Should Tor-oriented discovery and restricted federation discovery remain separate profiles or share one narrower base model?
