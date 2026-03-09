# Peers

This directory defines peer roles and per-peer configuration expectations.

Recommended first peer roles:

- `peer-seed`
- `peer-reader-a`
- `peer-reader-b`
- `peer-fault`

Minimum per-peer state:

- `node_id`
- key material reference
- transport endpoint or logical bus address
- capabilities
- bootstrap peers
- object store path or logical store name

Suggested future additions:

- peer role schema
- sample peer configs
- fault-injection toggles

## Schema

- `peer.schema.json` is the formal contract for standalone peer config files in this directory.
- `peer.example.json` should validate against this schema.
- `topology.schema.json` reuses the same peer contract instead of redefining peer fields separately.
