# Dynamic Module Loading for Mycel Apps

Status: design draft

This note proposes a conservative way for a Mycel-based app to load executable extension logic at runtime and download missing modules on demand.

The core recommendation is:

- treat executable code as a separately modeled, signed artifact
- prefer sandboxed `WASM` modules over native binaries
- verify module identity and policy before execution
- grant only explicit capabilities rather than ambient host access

Related notes:

- `DESIGN-NOTES.app-signing-model.*` for the distinction between object signing, release signing, and execution-evidence signing
- `DESIGN-NOTES.mycel-app-layer.*` for where app-layer behaviors should sit above protocol core
- `DESIGN-NOTES.mycel-full-stack-map.*` for the broader layering model

## 0. Goal

Allow a Mycel app to:

- discover that a required runtime module is missing
- fetch the missing module in real time
- verify it as a signed, content-addressed artifact
- execute it inside a constrained runtime

Preserve:

- object-level verifiability
- reproducible module identity
- explicit trust and capability boundaries
- auditability of which code ran

Avoid by default:

- arbitrary native code execution
- implicit host access
- execution of unauthenticated script fragments

## 1. Why This Needs a Separate Model

Downloading ordinary Mycel content is not the same as downloading executable logic.

Executable logic changes the trust model because it can:

- affect local confidentiality
- affect local integrity
- shape how application state is interpreted
- create non-deterministic side effects

The system should therefore model code as a first-class, signed artifact with stricter admission rules than normal content objects.

## 2. Recommended Execution Direction

There are three broad implementation choices.

### 2.1 Sandboxed `WASM` Modules

Recommended default.

Benefits:

- portable across platforms
- easier to sandbox than native code
- compatible with capability-based host APIs
- clearer verification and cache story

Tradeoff:

- requires a defined host API and runtime embedding layer

### 2.2 Restricted Script Runtime

Possible short-term option.

Examples:

- Lua
- Rhai
- another embedded DSL

Benefits:

- smaller implementation to start with

Tradeoff:

- weaker portability guarantees
- easier to drift into host-specific behavior
- lower ceiling for richer modules

### 2.3 Native Binary or Dynamic Library Loading

Not recommended as the default.

Reason:

- strongest supply-chain risk
- hardest sandboxing problem
- worst cross-platform story
- easiest path to ambient host compromise

If native modules ever exist, they should be treated as a separate, higher-risk deployment mode rather than the baseline Mycel app model.

## 3. Proposed Artifact Split

Do not treat executable code as an unstructured attachment inside ordinary content objects.

Use two explicit artifacts instead.

### 3.1 `module` Metadata Object

This object describes:

- module identity
- version
- runtime target
- entry points
- required capabilities
- expected code hash
- fetch hints
- author or release signature

### 3.2 `module_blob` Artifact

This artifact contains:

- the actual `WASM` payload or other approved bytecode form

The app should verify that:

- the blob hash matches the metadata object
- the metadata object signature is valid
- the runtime target is supported locally

This split makes it easier to:

- mirror blobs
- cache them by content hash
- reuse identical code across multiple metadata objects
- audit what was approved versus what was executed

## 4. Suggested Metadata Shape

Illustrative fields:

- `module_id`
- `name`
- `version`
- `runtime`
- `entry`
- `code_hash`
- `capabilities`
- `resource_limits`
- `fetch_uris`
- `signature`

Possible resource-limit fields:

- `max_memory_mb`
- `max_fuel`
- `max_output_bytes`
- `network_policy`
- `filesystem_policy`

The exact schema does not need to be part of protocol core at first.

It can begin as an app-layer schema carried by signed Mycel objects.

## 5. Runtime Fetch and Load Flow

Suggested runtime sequence:

1. a document, view, profile, or app manifest references a `module_id`
2. the app checks whether the required module blob already exists in local cache
3. if missing, the app fetches the module metadata object and blob from approved Mycel or external locations
4. the app verifies metadata signature, code hash, runtime compatibility, and local policy admission
5. the app stores the verified blob in module cache
6. the app instantiates the module inside a sandbox runtime
7. the app records execution metadata for later audit

Important rule:

- the app should download missing modules in real time only as complete, signed module artifacts
- it should not execute partial fragments before full integrity verification completes

## 6. Segment and Chunk Fetching

If large blobs ever need chunked transfer, chunking should remain a transport detail rather than an execution identity.

Recommended rule:

- chunks may be fetched incrementally
- execution identity remains the fully verified module blob
- the app may not execute the module until the final hash and signature checks pass

This avoids turning "code segment download" into an under-specified partial-execution model.

## 7. Capability Model

Modules should not receive unrestricted host access.

Instead, the host should expose a narrow capability API.

Examples:

- `read_document`
- `read_revision_history`
- `read_view_state`
- `write_render_output`
- `write_local_cache`
- `emit_diagnostics`
- `request_network_fetch`

Recommended default-deny behavior:

- no arbitrary filesystem access
- no arbitrary subprocess execution
- no arbitrary outbound network access
- no dynamic loading of further native libraries

## 8. Trust and Governance Boundary

Mycel verifiability does not by itself mean a module should be trusted to run.

The app should therefore apply two separate checks:

1. integrity check
   - is the module object and blob authentic and untampered?
2. execution-admission check
   - is this signer, module family, and capability request allowed under local or profile policy?

Possible admission models:

- local signer allowlist
- profile-bound signer policy
- governance recommendation plus local final approval

Recommended short-term baseline:

- local allowlist plus explicit capability grant

## 9. Caching and Versioning

The local app should maintain:

- `module_id -> metadata`
- `code_hash -> blob path`
- runtime-compiled cache keyed by module hash and runtime version

Recommended version rules:

- pin exact versions in references
- do not auto-upgrade major versions
- keep old cached blobs while referenced by active state

The cache should be content-addressed, not name-addressed only.

## 10. Audit and Replay

If a module influences rendering, transformation, or external behavior, the app should record at least:

- `module_id`
- `version`
- `code_hash`
- runtime identity
- granted capabilities
- relevant input object IDs
- output artifact hash if applicable

This does not guarantee perfect deterministic replay.

It does make later review and incident analysis materially stronger.

## 11. Recommended Non-goals for the First Version

The first version should avoid:

- arbitrary shell script execution
- automatic execution of downloaded native binaries
- module-to-module recursive dependency loading without explicit policy
- partial execution before full blob verification
- governance-driven auto-execution without local admission

## 12. Suggested MVP

A practical first implementation could limit itself to:

1. one sandbox runtime, preferably `WASM`
2. one signed module metadata object type
3. one content-addressed blob type
4. a small capability surface
5. local module cache
6. audit logging for module execution

This would be enough to support app-level renderers, transformers, or policy helpers without committing Mycel to native plugin loading.

## 13. Open Questions

- Should module metadata live entirely in app-layer schema, or eventually gain protocol-level conventions?
- Should profile policy be able to recommend modules, or only permit signer classes?
- How much determinism should Mycel require from modules that affect accepted-state derivation?
- Should external network fetch ever be allowed after module start, or only through host-mediated fetch requests?
