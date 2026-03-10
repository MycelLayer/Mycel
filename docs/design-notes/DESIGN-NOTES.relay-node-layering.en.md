# Relay Node Layering

Status: design note

This note defines where a `relay node` belongs in Mycel's layer model.

It is intentionally narrow. It does not standardize a full relay protocol, relay discovery mechanism, or relay incentive model.

## 1. Working Definition

A `relay node` is a node role optimized for transport availability and object/session forwarding.

It may help with:

- stable reachability
- lower-latency cross-region transport
- NAT/firewall bridging
- bounded object forwarding
- optional short-lived transport-side caching

It should not be assumed to define social truth, accepted-head selection, or domain semantics.

## 2. Primary Layer Placement

The relay-node concept belongs primarily in the `wire / transport layer`.

Reason:

- the role is about connection behavior and data forwarding
- it affects session interoperability, not core signed-object meaning
- it should remain separable from governance and app semantics

If Mycel later formalizes relay capabilities, the first normative home should therefore be the wire protocol surface.

## 3. Secondary Layer Placement

### 3.1 Profile Layer

Deployment-specific relay policy belongs in profiles.

Examples:

- whether relays are allowed or required
- whether Tor-only transport relays are required
- which relay classes are trusted for bootstrap or forwarding
- relay preference, fallback, or locality policy

These are not universal protocol truths and should remain profile-specific unless cross-implementation interoperability requires more.

### 3.2 Local Tooling

Operational relay support belongs in local tooling.

Examples:

- relay health checks
- relay latency ranking
- relay configuration helpers
- relay bootstrap lists
- local relay observability

These improve operations but should not be forced into protocol objects.

## 4. What Does Not Belong in Core

The relay-node concept should not primarily live in `core protocol`.

The core should not define relay nodes as:

- governance authorities
- accepted-state selectors
- origin-truth sources
- app-domain actors

Core signed objects may reference transport information where needed, but relay role semantics should not become part of signed social truth by default.

## 5. Why Relay Nodes Can Speed Things Up

Relay nodes do not need stronger hardware to be useful.

They can improve performance because they are:

- deployed in better network locations
- kept online continuously
- optimized for forwarding rather than full local workflows
- better connected across regions
- useful as stable intermediate transport points

Their main advantage is network topology and availability, not raw compute power.

## 6. Recommended Mycel Position

For now:

- keep `relay node` as a design-note concept
- treat its main normative home as future wire-protocol work
- keep deployment-specific relay rules in profiles
- keep operational relay behavior in tooling

This preserves scope discipline until Mycel reaches the stage where `M4` wire and peer interoperability need a more formal transport-role model.
