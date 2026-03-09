# Peer Simulator v0

Status: design draft

This note defines a narrow multi-peer simulator for early Mycel implementation testing.

The main design principle is:

- prove core sync and verification behavior before building a full client
- keep the simulator deterministic and easy to reset
- prefer bounded local topologies over broad discovery
- test wire behavior and object verification separately from rich product features

## 0. Goal

Provide a practical first test harness that can:

- run multiple Mycel peer identities locally
- exchange the minimum v0.1 wire messages
- verify canonical objects and replay state
- expose sync and acceptance bugs early

This simulator is for implementation testing.

It is not a production deployment model.

## 1. Recommended Form

I recommend two phases.

### 1.1 Phase A: Single-process Multi-peer Simulator

Run multiple peer states inside one process.

Benefits:

- fastest to build
- deterministic scheduling
- easy fixture injection
- easy trace capture

Limitations:

- does not test real socket behavior
- does not test process-level isolation
- does not test restart behavior realistically

### 1.2 Phase B: Localhost Multi-process Peer Harness

Run the same node implementation multiple times on `127.0.0.1`.

Benefits:

- tests actual session setup and transport framing
- tests per-peer storage isolation
- tests bootstrap configuration and restart behavior

Limitations:

- slower than Phase A
- requires more harness code and process control

The recommended path is:

1. build Phase A first
2. move the same peer logic into Phase B
3. keep both harnesses if possible

## 2. In-scope Capabilities

The simulator should cover only the narrowest interoperable path.

Required:

- multiple peer identities
- one local object store per peer
- deterministic fixture loading
- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `BYE`
- `ERROR`
- object ID verification
- object signature verification
- replay-based `state_hash` verification

Optional for simulator v0:

- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`
- accepted-head comparison across peers

Deferred:

- rich reader UI
- editor workflows
- public discovery
- Tor transport
- signer or runtime roles

## 3. Peer Roles

The first simulator should use a very small role set.

### 3.1 Seed Peer

The seed peer starts with known fixture objects.

Responsibilities:

- serve initial `MANIFEST` and `HEADS`
- answer `WANT` requests
- remain the most stable peer in the test topology

### 3.2 Reader Peer

The reader peer starts empty or partially populated.

Responsibilities:

- bootstrap from a bounded peer list
- sync missing objects
- verify all received objects before indexing
- compute local accepted state if that layer is enabled

### 3.3 Fault Peer

The fault peer is optional but highly useful.

Responsibilities:

- send malformed or inconsistent messages
- advertise missing objects
- send wrong hashes, wrong signatures, or duplicate announcements

This role is useful for negative tests without corrupting the normal seed peer.

## 4. Minimal Topologies

The simulator should start with only three topologies.

### 4.1 Two-peer Sync

- `peer-seed`
- `peer-reader-a`

Use this for the first end-to-end sync test.

### 4.2 Three-peer Consistency

- `peer-seed`
- `peer-reader-a`
- `peer-reader-b`

Use this to compare:

- received object sets
- replay results
- accepted-head results

### 4.3 Fault-injection Topology

- `peer-seed`
- `peer-reader-a`
- `peer-fault`

Use this for rejection and recovery behavior.

## 5. Per-peer State

Each simulated peer should have isolated state.

Minimum state:

- `node_id`
- transport endpoint or logical bus address
- peer keypair
- peer capabilities
- bootstrap peer list
- object store
- derived indexes
- sync session history
- local transport policy

Recommended extra state:

- event log
- decision trace log
- fault-injection flags

Even in a single-process simulator, state must remain logically separate.

## 6. Transport Model

The transport layer should be swappable.

### 6.1 Phase A Transport

Use an in-memory message bus.

Requirements:

- preserve sender and receiver identity
- preserve message order within one session
- allow deterministic delay or drop injection
- capture full wire envelopes for inspection

### 6.2 Phase B Transport

Use real localhost sockets.

Requirements:

- one listen address per peer
- configurable bootstrap peers
- session lifecycle events
- clean shutdown and restart support

The peer logic should not depend on whether transport is in-memory or socket-based.

## 7. Fixture Strategy

The simulator should use explicit fixtures rather than ad hoc generated content at first.

Recommended fixture sets:

1. one valid document with one revision chain
2. one document with two valid heads
3. one invalid object set with hash mismatch
4. one invalid object set with signature mismatch
5. one partial store that requires `WANT` recovery

Fixtures should be:

- deterministic
- version-controlled
- loadable into any peer role

## 8. Minimal Flow

The baseline simulator flow should be:

1. initialize peer identities and stores
2. load fixture objects into `peer-seed`
3. start peer sessions
4. exchange `HELLO`
5. exchange `MANIFEST` or `HEADS`
6. compute missing canonical object IDs
7. request them with `WANT`
8. return them with `OBJECT`
9. verify object ID, hash, signature, and replayed state
10. index only verified objects
11. optionally compute accepted head
12. emit a session result report

## 9. Test Cases

Simulator v0 should support at least these tests.

### 9.1 Positive Cases

- first sync from empty reader
- incremental sync after new heads appear
- replay rebuild from stored objects only
- two readers produce the same accepted result from the same verified object set

### 9.2 Negative Cases

- reject `OBJECT` with mismatched derived ID
- reject `OBJECT` with mismatched body hash
- reject object with invalid signature
- reject wire envelope with invalid signature
- reject invalid parent ordering or invalid replay result

### 9.3 Recovery Cases

- retry after partial object delivery
- reconnect after peer restart
- continue sync after a faulty peer is ignored

## 10. Harness Outputs

The simulator should produce outputs that are easy to diff.

Recommended outputs:

- per-peer received object IDs
- per-peer verification results
- per-peer final heads by `doc_id`
- accepted-head result when enabled
- wire trace log
- failure summary

These outputs should be machine-readable.

Human-readable summaries may be added on top.

## 11. Non-goals

Simulator v0 should explicitly avoid:

- proving full protocol completeness
- simulating public-mesh scale
- building the final reader UI
- modeling fund execution flows
- modeling signer consent flows
- modeling anonymous deployment behavior
- treating peer-discovery drafts as mandatory runtime behavior

The simulator is a build aid, not the whole platform.

## 12. Recommended Repository Shape

One practical repo shape would be:

- `fixtures/`
- `sim/`
- `sim/peers/`
- `sim/topologies/`
- `sim/tests/`
- `sim/reports/`

Suggested top-level simulator components:

- peer state model
- transport adapter
- fixture loader
- wire session driver
- verification engine wrapper
- report generator

## 13. Success Criteria

Peer Simulator v0 is successful if we can:

1. launch at least three isolated peer identities locally
2. load deterministic fixtures into one peer
3. sync those fixtures to one or more other peers
4. reject malformed or inconsistent objects correctly
5. rebuild local state from canonical objects only
6. compare outputs across peers and confirm deterministic agreement where expected

If those six things work, the simulator is already useful.

## 14. Recommended Next Step

Once simulator v0 works, the next expansion should be one of:

- add accepted-head comparison as a first-class report
- add localhost multi-process mode
- add snapshot-assisted catch-up tests

Do not expand all three at once.
