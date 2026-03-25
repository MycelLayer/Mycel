# Mycel Wire Session XState Draft

Status: draft discussion document for protocol-state modeling

This document sketches how we could model a minimal Mycel wire session as an
XState machine. The goal is not to replace the current Rust implementation.
The goal is to make session states, message ordering, guards, and rejection
paths easier to discuss and test.

## Why Model This

The Mycel wire protocol already behaves like a state machine:

- some messages are only valid after earlier messages
- some messages require capability gates
- some faults should reject immediately
- some paths are warning-only or terminal

That makes XState / Stately a useful design aid for:

1. checking whether state boundaries are clear
2. finding missing or contradictory rejection paths
3. turning protocol rules into a cleaner transition matrix
4. deriving simulator or transcript tests from explicit transitions

## Scope

This first draft intentionally models only the narrow session progression around:

- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`
- `BYE`
- `ERROR`

It does not try to encode every protocol detail yet.

## Minimal Machine

```ts
import { setup, assign } from 'xstate';

type Msg =
  | { type: 'RECV_HELLO'; senderId: string; claimedSenderId: string }
  | { type: 'RECV_MANIFEST' }
  | { type: 'RECV_HEADS'; replace: boolean }
  | { type: 'RECV_WANT'; advertised: boolean; reachable: boolean }
  | { type: 'RECV_OBJECT'; requested: boolean }
  | { type: 'RECV_SNAPSHOT_OFFER'; capabilityPresent: boolean }
  | { type: 'RECV_VIEW_ANNOUNCE'; capabilityPresent: boolean }
  | { type: 'RECV_BYE' }
  | { type: 'RECV_ERROR' };

type Ctx = {
  senderEstablished: boolean;
  syncRootsEstablished: boolean;
  byeSeen: boolean;
  lastRejectReason: string | null;
};

export const wireSessionMachine = setup({
  types: {
    context: {} as Ctx,
    events: {} as Msg,
  },
  guards: {
    senderMatches: ({ event }) =>
      event.type !== 'RECV_HELLO' || event.senderId === event.claimedSenderId,

    capabilityPresent: ({ event }) =>
      !('capabilityPresent' in event) || event.capabilityPresent,

    advertisedWant: ({ event }) =>
      event.type !== 'RECV_WANT' || event.advertised,

    reachableWant: ({ event }) =>
      event.type !== 'RECV_WANT' || event.reachable,

    requestedObject: ({ event }) =>
      event.type !== 'RECV_OBJECT' || event.requested,
  },
  actions: {
    markSenderEstablished: assign({
      senderEstablished: true,
      lastRejectReason: null,
    }),
    markSyncRootsEstablished: assign({
      syncRootsEstablished: true,
      lastRejectReason: null,
    }),
    markByeSeen: assign({
      byeSeen: true,
    }),
    reject: assign({
      lastRejectReason: ({ event }) => event.type,
    }),
  },
}).createMachine({
  id: 'mycelWireSession',
  initial: 'preHello',
  context: {
    senderEstablished: false,
    syncRootsEstablished: false,
    byeSeen: false,
    lastRejectReason: null,
  },
  states: {
    preHello: {
      on: {
        RECV_HELLO: [
          {
            guard: 'senderMatches',
            target: 'helloAccepted',
            actions: 'markSenderEstablished',
          },
          {
            target: 'protocolRejected',
            actions: 'reject',
          },
        ],
        RECV_ERROR: {
          target: 'errorOnlyAccepted',
        },
        RECV_MANIFEST: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_HEADS: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_WANT: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_OBJECT: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_BYE: {
          target: 'protocolRejected',
          actions: 'reject',
        },
      },
    },

    helloAccepted: {
      on: {
        RECV_MANIFEST: {
          target: 'manifestAccepted',
        },
        RECV_SNAPSHOT_OFFER: [
          {
            guard: 'capabilityPresent',
            target: 'helloAccepted',
          },
          {
            target: 'protocolRejected',
            actions: 'reject',
          },
        ],
        RECV_VIEW_ANNOUNCE: [
          {
            guard: 'capabilityPresent',
            target: 'helloAccepted',
          },
          {
            target: 'protocolRejected',
            actions: 'reject',
          },
        ],
        RECV_BYE: {
          target: 'byeSeen',
          actions: 'markByeSeen',
        },
      },
    },

    manifestAccepted: {
      on: {
        RECV_HEADS: {
          target: 'syncRootsAccepted',
          actions: 'markSyncRootsEstablished',
        },
        RECV_WANT: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_OBJECT: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_BYE: {
          target: 'byeSeen',
          actions: 'markByeSeen',
        },
      },
    },

    syncRootsAccepted: {
      on: {
        RECV_WANT: [
          {
            guard: ({ context, event }) =>
              context.syncRootsEstablished &&
              event.type === 'RECV_WANT' &&
              event.advertised &&
              event.reachable,
            target: 'syncRootsAccepted',
          },
          {
            target: 'protocolRejected',
            actions: 'reject',
          },
        ],
        RECV_OBJECT: [
          {
            guard: 'requestedObject',
            target: 'syncRootsAccepted',
          },
          {
            target: 'protocolRejected',
            actions: 'reject',
          },
        ],
        RECV_BYE: {
          target: 'byeSeen',
          actions: 'markByeSeen',
        },
      },
    },

    byeSeen: {
      on: {
        RECV_HELLO: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_MANIFEST: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_HEADS: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_WANT: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_OBJECT: {
          target: 'protocolRejected',
          actions: 'reject',
        },
        RECV_BYE: {
          target: 'protocolRejected',
          actions: 'reject',
        },
      },
    },

    errorOnlyAccepted: {
      type: 'final',
    },

    protocolRejected: {
      type: 'final',
    },
  },
});
```

## State Intent

| State | Meaning | Discussion focus |
|---|---|---|
| `preHello` | No valid `HELLO` accepted yet. | Which messages are always illegal before session establishment? |
| `helloAccepted` | Peer identity is accepted, but sync setup is not complete. | Which optional messages may already appear here? |
| `manifestAccepted` | We have enough session context to accept `HEADS`, but not yet sync roots. | Should any `WANT` or `OBJECT` ever be legal here? |
| `syncRootsAccepted` | We can now evaluate `WANT` and `OBJECT` against accepted roots / requests. | This is where most interop fault-path work lives. |
| `byeSeen` | Session is logically over. | Why are later messages permanently rejected? |
| `errorOnlyAccepted` | An explicit `ERROR`-only path is accepted as terminal. | Matches the special-case protocol allowance. |
| `protocolRejected` | The session hit a hard rejection path. | Useful as a discussion sink for invalid transitions. |

## Transition Table Outline

| Current State | Event | Guard / Condition | Next State | Side Effects | Reject? | Notes |
|---|---|---|---|---|---|---|
| `preHello` | `RECV_HELLO` | sender matches claimed identity | `helloAccepted` | record sender / establish peer identity | no | Normal session start. |
| `preHello` | `RECV_ERROR` | none | `errorOnlyAccepted` | none or record terminal error | no | Mirrors the explicit `ERROR`-before-`HELLO` allowance. |
| `preHello` | `RECV_MANIFEST` / `HEADS` / `WANT` / `OBJECT` / `BYE` | none | `protocolRejected` | reject reason | yes | Pre-`HELLO` rejection family. |
| `helloAccepted` | `RECV_MANIFEST` | valid payload | `manifestAccepted` | record manifest context | no | Normal progression. |
| `helloAccepted` | `RECV_SNAPSHOT_OFFER` / `RECV_VIEW_ANNOUNCE` | capability present | `helloAccepted` | optional capability-gated handling | no | Models capability gates. |
| `helloAccepted` | `RECV_SNAPSHOT_OFFER` / `RECV_VIEW_ANNOUNCE` | capability missing | `protocolRejected` | reject reason | yes | Missing capability fault. |
| `manifestAccepted` | `RECV_HEADS` | valid root setup | `syncRootsAccepted` | establish sync roots | no | Opens fetch phase. |
| `manifestAccepted` | `RECV_WANT` / `RECV_OBJECT` | none | `protocolRejected` | reject reason | yes | Pre-root / pre-sync-root fault. |
| `syncRootsAccepted` | `RECV_WANT` | advertised and reachable | `syncRootsAccepted` | enqueue fetch | no | Main fetch path. |
| `syncRootsAccepted` | `RECV_WANT` | not advertised or unreachable | `protocolRejected` | reject reason | yes | Covers unadvertised / unreachable `WANT`. |
| `syncRootsAccepted` | `RECV_OBJECT` | object was requested | `syncRootsAccepted` | accept object | no | Main object-ingest path. |
| `syncRootsAccepted` | `RECV_OBJECT` | object was not requested | `protocolRejected` | reject reason | yes | Covers unrequested `OBJECT`. |
| `*` | `RECV_BYE` | valid timing | `byeSeen` | mark session closed | no | Session close path. |
| `byeSeen` | any later protocol message | none | `protocolRejected` | reject reason | yes | Permanent messages-after-`BYE` rejection. |

## Why This Is Useful

This model is most useful when we want to discuss:

1. where message legality changes between session phases
2. which rejections are capability-based versus ordering-based
3. which fault paths are already covered in simulator tests
4. which transitions still lack deterministic proof

## How To Use It

Recommended workflow:

1. keep this draft small and explicit
2. review one state boundary at a time
3. compare each transition against current simulator / transcript coverage
4. only after the model feels right, decide whether to encode more detail or
   derive tests from it

## Open Questions

1. Should `ERROR` be modeled as a terminal sibling state or as an orthogonal outcome on top of other states?
2. Should `SNAPSHOT_OFFER` and `VIEW_ANNOUNCE` stay as self-transitions on `helloAccepted`, or should they have their own substate structure?
3. Should `HEADS replace=true` and stale-root/stale-snapshot/stale-object rejection become explicit substates instead of guard-only logic?
4. Should warning-only outcomes, such as missing `BYE`, be represented as terminal states distinct from hard rejection?
5. Do we want a future machine-readable transition source for simulator test generation?
