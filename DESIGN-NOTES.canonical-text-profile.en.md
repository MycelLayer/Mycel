# Canonical Text Profile

Status: design draft

This note describes a neutral text profile for carrying structured, cited, and annotated canonical corpora on top of Mycel.

The main design principle is:

- the profile should model stable reference texts without assuming one content domain
- text structure and citation anchors should be explicit
- commentary, translation, and reference layers should remain separate from root text
- multiple witnesses and accepted reading profiles should coexist without forcing a single universal edition

## 0. Goal

Enable Mycel to carry long-lived reference texts that need:

- stable citation
- section-aware reading
- multiple witnesses or editions
- translation layers
- commentary layers
- accepted reading profiles

This note does not define one mandatory canon model for every deployment.

Instead, it defines a neutral baseline that can be specialized by domain-specific extensions.

## 1. Scope

This profile is intended for corpora such as:

- scriptural collections
- legal codes
- philosophical canons
- historical primary texts
- other reference-oriented literature

This profile is not optimized for:

- chat-like content
- rapidly edited short-form documents
- generic wiki pages without stable citation requirements

## 2. Core Design Rule

The root text should remain distinct from all secondary layers.

This means:

- root text is not silently overwritten by commentary
- translation is not treated as the same object as source text
- explanatory notes remain separately attributable
- accepted reading state may vary by profile without mutating the underlying witnesses

## 3. Text Families

A canonical corpus should be modeled as a family of related document types.

### 3.1 Work Record

Defines the conceptual work.

Suggested fields:

- `work_id`
- `title`
- `language_family`
- `corpus_id`
- `reference_scheme`
- `root_witnesses`

Purpose:

- identify the work independent of any one edition
- provide the stable top-level reference target

### 3.2 Witness Document

Represents a concrete textual witness.

Examples:

- source-language witness
- translation witness
- critical edition witness
- curated reading witness

Suggested fields:

- `witness_id`
- `work_id`
- `witness_kind`
- `language`
- `source_description`
- `text_document_id`
- `lineage_ref`

### 3.3 Text Document

Carries the actual structured text.

This should be composed of blocks aligned to explicit citation anchors and section hierarchy.

### 3.4 Commentary Document

Carries secondary interpretation or explanation.

It should reference root text anchors instead of replacing root text.

### 3.5 Alignment Document

Carries mappings between two or more witnesses.

Examples:

- source-to-translation mapping
- edition-to-edition mapping
- segment-level correspondence

### 3.6 Citation Set

Carries stable references used by commentary, Q&A, or other higher layers.

## 4. Structural Model

The text profile should expose hierarchy explicitly.

Recommended layers:

- `corpus`
- `work`
- `section`
- `subsection`
- `text_unit`
- optional `line_unit`

These names are intentionally neutral.

Deployments may map them to domain-specific names through extensions.

Examples:

- `section` might map to a chapter-like division
- `text_unit` might map to a verse-like or paragraph-like unit
- `line_unit` might be used only where fine-grained line citation is needed

## 5. Citation Anchors

Stable citation is one of the main reasons to use this profile.

Each text document should support explicit anchors.

Suggested fields for an anchor record:

- `anchor_id`
- `work_id`
- `witness_id`
- `anchor_path`
- `anchor_kind`
- `block_ref`

Recommended properties:

- anchors should survive minor editorial cleanup when possible
- anchors should be explicit rather than inferred only from visual formatting
- anchor paths should be readable enough for citation and machine use

## 6. Witness and Edition Model

The profile should not assume that one text witness is globally definitive.

Instead:

- multiple witnesses may coexist for one work
- one deployment may prefer one accepted reading
- another deployment may prefer another accepted reading

This fits Mycel's multi-branch and accepted-head model better than forcing one universal edition.

## 7. Alignment Model

Canonical corpora often require cross-witness comparison.

An alignment document should support:

- one-to-one mapping
- one-to-many mapping
- partial overlap
- unmatched segments

Suggested alignment fields:

- `alignment_id`
- `source_witness_id`
- `target_witness_id`
- `alignment_units`
- `alignment_method`
- `confidence`

The profile should assume imperfect alignment is normal.

## 8. Secondary Layers

The following layers should remain distinct from root text:

- commentary
- translation
- glossary
- index
- Q&A
- teaching or explanatory summaries

Each secondary layer should reference root anchors or alignment units rather than duplicating authority implicitly.

## 9. Accepted Reading Model

Mycel's accepted-head model should apply to canonical text reading as follows:

- the root witness set remains historically preserved
- deployments may publish accepted reading profiles
- reader clients should display one accepted reading by default
- alternative witnesses and branches should remain visible

This avoids pretending that all readers must share one universal reading state.

## 10. Reader Expectations

A client implementing this profile should support at least:

- reading the accepted text
- viewing anchor-aware structure
- viewing alternative witnesses
- opening commentary tied to anchors
- opening citations tied to anchors
- seeing which profile selected the current accepted reading

## 11. Governance Boundaries

This profile should distinguish:

- root text maintenance
- witness publication
- commentary publication
- accepted reading selection

These roles may overlap, but they should not be silently collapsed.

## 12. Extension Strategy

The core profile should stay neutral.

Tradition-specific or domain-specific detail should be expressed through extensions such as:

- additional hierarchy labels
- domain-specific citation conventions
- specialized alignment metadata
- domain-specific commentary classes

The extension should not redefine the neutral baseline unless interoperability clearly requires it.

## 13. Minimal First Version

A minimal first deployment of this profile should require:

- one `work record`
- one or more `witness documents`
- one structured `text document` per witness
- explicit citation anchors
- optional commentary documents
- optional alignment documents
- one accepted reading profile

It should not require:

- perfect cross-witness alignment
- automatic semantic merge of commentary
- universal agreement on one edition

## 14. Recommended Next Step

The next practical step after this note is to define:

- a minimal canonical text schema
- an anchor syntax
- one example corpus with two witnesses and one commentary layer

This would make the profile concrete enough for a first reader client.
