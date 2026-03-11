# Project Naming

Status: current outward-facing naming guidance

This document defines how `Mycel` and `MycelLayer` should be used across the repository and public-facing surfaces.

## 1. Short Rule

- Keep `Mycel` as the canonical project, protocol, repository, and implementation name.
- Use `MycelLayer` as the public-facing positioning name when we need to explain what Mycel is to new readers.

The intended first-line explanation is:

> MycelLayer is the public-facing name for Mycel, a protocol layer for verifiable text history, profile-governed accepted reading, and decentralized replication.

## 2. Why This Split Exists

- `Mycel` is already the stable internal name used by the repository, Rust workspace, protocol documents, and code identifiers.
- `MycelLayer` reduces name-collision risk on public surfaces and makes the "protocol layer" positioning easier to understand at first glance.
- This split preserves continuity for implementation and specification work while giving outward messaging a clearer shape.

## 3. Canonical Name vs Public Name

### 3.1 Use `Mycel`

Use `Mycel` for:

- repository name and GitHub URL
- crate names, package names, binary names, and code identifiers
- protocol, wire, profile, fixture, and simulator documents
- issue titles about implementation or specification work
- CLI output and test names

Preferred examples:

- `Mycel is a Rust-based protocol stack...`
- `Mycel object verification`
- `Mycel protocol v0.1`

### 3.2 Use `MycelLayer`

Use `MycelLayer` for:

- homepage hero copy
- README opening positioning when speaking to new external readers
- repo description, social preview text, and short public bios
- grant notes, sponsorship pages, and outreach material
- explainers that focus on the gap Mycel is filling rather than protocol mechanics

Preferred examples:

- `MycelLayer is the public-facing name for Mycel.`
- `MycelLayer is a protocol layer for verifiable text history...`

## 4. Mixed Use Pattern

When both names appear in the same document:

1. Introduce `MycelLayer` first only if the document is outward-facing.
2. Immediately connect it back to `Mycel`.
3. Use `Mycel` for the rest of the technical detail unless the section stays purely public-facing.

Recommended pattern:

- `MycelLayer is the public-facing name for Mycel.`
- `Mycel is a Rust-based protocol stack...`

This avoids accidental drift where readers think `MycelLayer` and `Mycel` are different systems.

## 5. Do Not Do This

- Do not rename the repository, crates, binaries, or protocol document titles from `Mycel` to `MycelLayer`.
- Do not rewrite specification text to say `MycelLayer protocol`, `MycelLayer wire format`, or `MycelLayer object`.
- Do not use `MycelLayer` deep inside protocol, wire, schema, fixture, or design-note wording unless the text is explicitly about outward branding.
- Do not present `MycelLayer` as a separate product line, company, or network distinct from `Mycel`.

## 6. Surface-by-Surface Guidance

### 6.1 README and landing pages

- Title may remain `Mycel` for continuity.
- Hero sentence or first paragraph may use `MycelLayer` if the text is aimed at new public readers.
- The first screen should make the relationship explicit.

Recommended pattern:

- Title: `Mycel`
- Subtitle or first sentence: `MycelLayer is the public-facing name for Mycel...`

### 6.2 Protocol and design documents

- Keep `Mycel` throughout, except for rare notes about public positioning.
- Technical precision has priority over branding language.

### 6.3 Grant and support documents

- Prefer `MycelLayer` in opening sections, summaries, and support asks.
- Reintroduce `Mycel` when pointing to repository artifacts, protocol scope, or implementation status.

### 6.4 Social surfaces

Use `MycelLayer` for:

- GitHub repo description
- social preview headline or subtitle
- public profile bios
- page metadata intended for link previews

## 7. Translation Guidance

- `zh-TW`: keep `MycelLayer` untranslated; explain it as `Mycel 的對外名稱` or `Mycel 的公開定位名稱`.
- `zh-CN`: keep `MycelLayer` untranslated; explain it as `Mycel 的对外名称` or `Mycel 的公开定位名称`.
- Do not invent translated product names such as `菌層`, `菌絲層`, or other localized brand substitutes.

## 8. Rollout Order

Recommended rollout:

1. Establish this naming rule document.
2. Update outward-facing copy such as README openings, homepage hero text, and support pages.
3. Update repo description and social preview text if needed.
4. Leave protocol and implementation identifiers unchanged.

## 9. Current Default

Until outward-facing copy is updated, treat this document as the decision boundary:

- `Mycel` remains the canonical technical name everywhere.
- `MycelLayer` is approved for outward-facing positioning copy.
