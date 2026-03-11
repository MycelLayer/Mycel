# Text-worker Editor Implementation Sketch

Status: design draft

This note describes a plausible implementation plan for a future Mycel text-worker editor.

It is intentionally narrower than a full product specification.
The goal is to define a buildable technical shape that matches Mycel's protocol discipline.

See also:

- [`DESIGN-NOTES.text-worker-editor.en.md`](./DESIGN-NOTES.text-worker-editor.en.md) for the product and interaction design goals
- [`DESIGN-NOTES.first-client-scope-v0.1.en.md`](./DESIGN-NOTES.first-client-scope-v0.1.en.md) for why rich authoring remains outside the first client
- [`DESIGN-NOTES.mycel-app-layer.en.md`](./DESIGN-NOTES.mycel-app-layer.en.md) for client/runtime/effect layering
- [`DESIGN-NOTES.commentary-citation-schema.en.md`](./DESIGN-NOTES.commentary-citation-schema.en.md) for commentary-oriented record shapes

## 0. Goal

Define how we would implement a serious text-worker editor without collapsing Mycel into:

- one opaque document blob
- one browser-only state model
- one platform-controlled reading state

The implementation goal is a word-processor-style authoring surface backed by explicit Mycel objects, replayable revision history, and profile-governed accepted reading.

## 1. Recommended Stack

The most plausible implementation shape today is:

- React for the application shell
- ProseMirror or TipTap for the editing surface
- Rust core logic compiled for native use and selectively compiled to WASM for browser use
- IndexedDB for local browser draft persistence
- SQLite for desktop or heavier local clients

This is not the only possible stack.
It is the most defensible one if the editor must handle:

- block-aware authoring
- dense annotation
- revision comparison
- explicit range and block identity
- future extension hooks

### 1.1 Why ProseMirror-style Editing

The editor surface needs more than rich text.
It needs a constrained structural model.

That favors ProseMirror-style systems because they provide:

- schema-controlled documents
- transaction-based edits
- decoration and annotation support
- plugin hooks
- strong control over block and inline structure

Lexical is a viable alternative for a more product-polished first pass.
A custom editor core is not recommended at the start because it would consume too much effort before Mycel-specific flows are validated.

## 2. Architectural Split

The implementation should be split into five cooperating layers:

1. editor UI layer
2. document projection layer
3. local draft store
4. validation and rebuild layer
5. publish and sync layer

These layers should stay explicit even if they run inside one client.

### 2.1 Editor UI Layer

Responsibilities:

- visible text editing
- selection handling
- block transforms
- inline formatting controls
- commentary sidebars
- revision comparison views

This layer is responsible for interaction quality, not for canonical truth.

### 2.2 Document Projection Layer

This is the most important Mycel-specific layer.

Responsibilities:

- map editor blocks to stable internal identities
- map editor transactions to draft revisions or commentary actions
- convert UI selections into durable anchors
- distinguish local authoring state from publishable candidate state
- produce inspectable deltas rather than opaque snapshots

The projection layer prevents the editor framework from becoming the storage model.

### 2.3 Local Draft Store

Responsibilities:

- persist unpublished drafts
- persist branch context
- persist commentary drafts
- persist local undo/redo boundaries where appropriate
- recover after browser or client restart

The draft store should be local-first and explicit about publication state.

### 2.4 Validation and Rebuild Layer

Responsibilities:

- canonicalization checks
- object validation
- accepted-reading rebuild
- branch comparison
- projection sanity checks

This layer should be implemented in Rust and reused across environments.

### 2.5 Publish and Sync Layer

Responsibilities:

- submit candidate revisions
- attach authorship and signing context
- surface verification failures
- reconcile local drafts with remote state after reconnect

This layer should not pretend that local edits are already accepted network state.

## 3. Editor-facing Document Model

The editor should not treat a document as one mutable string.

The internal authoring model should assume:

- ordered blocks
- stable block identity
- inline content within blocks
- explicit marks with bounded meaning
- separate linked structures for commentary and citations where needed

Recommended visible nodes:

- title
- section heading
- paragraph
- quote
- list item
- note reference
- citation anchor

The user may experience a continuous page.
The system should still preserve structured units underneath.

## 4. Projection Rules

The projection layer should enforce a small number of non-negotiable rules:

1. UI state is not canonical state.
2. Selections are temporary; durable anchors must be reconstructed into stable references.
3. Formatting actions should map to constrained semantics where possible.
4. Draft transforms must become inspectable edits, not hidden mutations.
5. Accepted reading must always be rebuilt from protocol-valid state, not trusted from the editor cache.

This is the main place where Mycel differs from ordinary document apps.

## 5. Local State Model

The client should maintain at least four distinct local states:

1. current editable draft
2. last known accepted reading
3. selected comparison target, if any
4. unsent commentary or action output

These should not be flattened into one editor buffer.

Recommended browser persistence:

- draft document projection in IndexedDB
- lightweight UI state in local storage or IndexedDB
- explicit revision checkpoints for recoverability

Recommended desktop persistence:

- SQLite for draft objects, checkpoints, and action logs

## 6. Rust and WASM Boundary

WASM means WebAssembly.
In this design, it is the mechanism for reusing Rust logic inside a browser client without rewriting the same rules in JavaScript.

Good candidates for Rust or Rust-to-WASM reuse:

- canonicalization
- object validation
- rebuild of accepted reading
- deterministic diffing
- anchor normalization
- revision projection checks

Poor candidates for Rust or WASM:

- text selection UX
- DOM rendering
- toolbar behavior
- editor plugin UI

The browser UI should stay in the web stack.
The deterministic rule engine should stay in Rust where possible.

## 7. Selection-driven Actions

A serious text-worker editor will need selection-driven actions.

The baseline flow should be:

1. the user selects a range or block
2. the user invokes a command
3. the editor resolves the selection into a stable local anchor
4. the command produces one of:
   - a draft text transform
   - a commentary draft
   - a citation insertion
   - a candidate revision fragment
5. the result appears as inspectable local state
6. publication remains explicit

This keeps automation useful without allowing hidden mutation of canonical state.

## 8. Macro-style Actions and Future Add-ins

The implementation should arrive in two stages:

1. built-in and local macro-style actions
2. capability-based add-in extensions

Early action API responsibilities:

- receive a normalized selection payload
- return structured action output
- declare whether the output is a draft edit, commentary, citation, or publish candidate

Later add-in API responsibilities:

- permissioned access to projections
- access to commentary and citation structures
- explicit publish-intent hooks
- isolated failure boundaries

The implementation should not begin with a marketplace model.

## 9. Revision and Diff Engine

The editor must treat revision comparison as a first-class capability.

Minimum diff engine responsibilities:

- compare current draft against accepted reading
- compare one candidate revision against another
- distinguish textual edits from structural edits
- identify block moves separately from replacements
- attribute changes when signer or author metadata is available

The first diff UX can be simple.
The underlying comparison model cannot be vague.

## 10. Commentary and Citation Plumbing

Commentary and citation support should not be faked as plain visual markup.

Implementation requirements:

- attach commentary to stable range or block anchors
- preserve citation target information independently of visual rendering
- distinguish editor notes from publishable commentary where policy requires it
- allow side-panel and inline views to represent the same underlying structures

This is necessary if annotation is meant to survive export, replay, and governance review.

## 11. Publish Flow

The publish path should be explicit and inspectable.

Recommended sequence:

1. author edits local draft
2. client computes draft delta
3. client validates projection locally
4. author reviews diff against accepted reading
5. client prepares candidate revision package
6. signing or authorship checks run
7. candidate is submitted
8. accepted state is later rebuilt from protocol-valid data

The editor should never imply that pressing Enter already changed accepted text.

## 12. Failure Modes to Design For

The implementation must explicitly handle:

- invalid local projection
- stale accepted-reading cache
- anchor drift after heavy edits
- failed publication or signature checks
- action output that cannot be normalized into valid structures
- reconnect reconciliation after offline work

These failures should be visible to the user.
Silent recovery is acceptable only when it does not obscure the state transition.

## 13. Recommended Build Sequence

Implementation should proceed in this order:

1. accepted-reading inspector with clean reconstruction
2. narrow structured draft editor
3. local draft persistence and recovery
4. draft-versus-accepted diff
5. explicit publish path
6. commentary and citation plumbing
7. macro-style action hooks
8. later add-in extension boundaries

This order keeps the Mycel-specific invariants ahead of surface polish.

## 14. Explicit Non-goals

The first implementation should not try to provide:

- complete desktop word processor parity
- arbitrary layout engines
- unconstrained embedded objects
- real-time multiplayer presence as a first requirement
- a public add-in marketplace
- direct editor writes to canonical accepted state

## 15. Open Questions

- What is the smallest editor schema that still supports real legal, scriptural, and historical text work?
- How should stable anchors be represented so that commentary survives revision-heavy editing?
- Which validation and rebuild functions are worth shipping in WASM first?
- How much local action scripting is safe before a full capability model exists?
- When should desktop-specific persistence or packaging become a separate client track?
