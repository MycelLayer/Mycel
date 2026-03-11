use std::path::{Path, PathBuf};

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use serde::Serialize;
use serde_json::{json, Value};

use crate::protocol::{
    recompute_object_id, signed_payload_bytes, RevisionObject, CORE_PROTOCOL_VERSION,
};
use crate::replay::{compute_state_hash, replay_revision, DocumentState, GENESIS_BASE_REVISION};
use crate::store::{
    load_store_object_index, load_stored_object_value, write_object_value_to_store,
    StoreRebuildError, StoredObjectRecord,
};

#[derive(Debug, Clone)]
pub struct DocumentCreateParams {
    pub doc_id: String,
    pub title: String,
    pub language: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct PatchCreateParams {
    pub doc_id: String,
    pub base_revision: String,
    pub timestamp: u64,
    pub ops: Value,
}

#[derive(Debug, Clone)]
pub struct RevisionCommitParams {
    pub doc_id: String,
    pub parents: Vec<String>,
    pub patches: Vec<String>,
    pub merge_strategy: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentCreateSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub document_object_id: String,
    pub genesis_revision_id: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_objects: Vec<StoredObjectRecord>,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchCreateSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub patch_id: String,
    pub base_revision: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_object: StoredObjectRecord,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RevisionCommitSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub revision_id: String,
    pub parent_revision_ids: Vec<String>,
    pub patch_ids: Vec<String>,
    pub recomputed_state_hash: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_object: StoredObjectRecord,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

pub fn parse_signing_key_seed(seed: &str) -> Result<SigningKey, String> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(seed.trim())
        .map_err(|error| format!("failed to decode base64 signing key seed: {error}"))?;
    let bytes: [u8; 32] = decoded
        .try_into()
        .map_err(|_| "signing key seed must decode to exactly 32 bytes".to_string())?;
    Ok(SigningKey::from_bytes(&bytes))
}

pub fn signer_id(signing_key: &SigningKey) -> String {
    format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    )
}

pub fn create_document_in_store(
    store_root: &Path,
    signing_key: &SigningKey,
    params: &DocumentCreateParams,
) -> Result<DocumentCreateSummary, StoreRebuildError> {
    if params.doc_id.is_empty() {
        return Err(StoreRebuildError::new("document doc_id must not be empty"));
    }
    if params.title.is_empty() {
        return Err(StoreRebuildError::new("document title must not be empty"));
    }
    if params.language.is_empty() {
        return Err(StoreRebuildError::new(
            "document language must not be empty",
        ));
    }
    if load_stored_object_value(store_root, &params.doc_id).is_ok() {
        return Err(StoreRebuildError::new(format!(
            "document '{}' already exists in the store",
            params.doc_id
        )));
    }

    let created_by = signer_id(signing_key);
    let empty_state = DocumentState {
        doc_id: params.doc_id.clone(),
        blocks: Vec::new(),
        metadata: serde_json::Map::new(),
    };
    let state_hash = compute_state_hash(&empty_state).map_err(|error| {
        StoreRebuildError::new(format!("failed to compute genesis state_hash: {error}"))
    })?;
    let mut genesis_revision = json!({
        "type": "revision",
        "version": CORE_PROTOCOL_VERSION,
        "doc_id": params.doc_id,
        "parents": [],
        "patches": [],
        "state_hash": state_hash,
        "author": created_by,
        "timestamp": params.timestamp
    });
    let genesis_revision_id = recompute_object_id(&genesis_revision, "revision_id", "rev")
        .map_err(StoreRebuildError::new)?;
    genesis_revision["revision_id"] = Value::String(genesis_revision_id.clone());
    genesis_revision["signature"] =
        Value::String(sign_object_value(signing_key, &genesis_revision)?);

    let document = json!({
        "type": "document",
        "version": CORE_PROTOCOL_VERSION,
        "doc_id": params.doc_id,
        "title": params.title,
        "language": params.language,
        "content_model": "block-tree",
        "created_at": params.timestamp,
        "created_by": signer_id(signing_key),
        "genesis_revision": genesis_revision_id
    });

    let revision_write = write_object_value_to_store(store_root, &genesis_revision)?;
    let document_write = write_object_value_to_store(store_root, &document)?;
    let written_object_count =
        usize::from(revision_write.created) + usize::from(document_write.created);
    let existing_object_count =
        usize::from(!revision_write.created) + usize::from(!document_write.created);
    let index_manifest_path = document_write
        .index_manifest_path
        .or(revision_write.index_manifest_path);

    Ok(DocumentCreateSummary {
        store_root: store_root.to_path_buf(),
        status: "ok".to_string(),
        doc_id: params.doc_id.clone(),
        document_object_id: params.doc_id.clone(),
        genesis_revision_id: genesis_revision["revision_id"]
            .as_str()
            .expect("generated revision_id should be string")
            .to_string(),
        written_object_count,
        existing_object_count,
        stored_objects: vec![document_write.record, revision_write.record],
        index_manifest_path,
        notes: Vec::new(),
        errors: Vec::new(),
    })
}

pub fn create_patch_in_store(
    store_root: &Path,
    signing_key: &SigningKey,
    params: &PatchCreateParams,
) -> Result<PatchCreateSummary, StoreRebuildError> {
    ensure_document_exists(store_root, &params.doc_id)?;
    if params.base_revision != GENESIS_BASE_REVISION {
        ensure_object_exists(store_root, &params.base_revision, "base revision")?;
    }

    let mut patch = json!({
        "type": "patch",
        "version": CORE_PROTOCOL_VERSION,
        "doc_id": params.doc_id,
        "base_revision": params.base_revision,
        "author": signer_id(signing_key),
        "timestamp": params.timestamp,
        "ops": params.ops
    });
    let patch_id =
        recompute_object_id(&patch, "patch_id", "patch").map_err(StoreRebuildError::new)?;
    patch["patch_id"] = Value::String(patch_id.clone());
    patch["signature"] = Value::String(sign_object_value(signing_key, &patch)?);

    let write = write_object_value_to_store(store_root, &patch)?;

    Ok(PatchCreateSummary {
        store_root: store_root.to_path_buf(),
        status: "ok".to_string(),
        doc_id: params.doc_id.clone(),
        patch_id,
        base_revision: params.base_revision.clone(),
        written_object_count: usize::from(write.created),
        existing_object_count: usize::from(!write.created),
        stored_object: write.record,
        index_manifest_path: write.index_manifest_path,
        notes: Vec::new(),
        errors: Vec::new(),
    })
}

pub fn commit_revision_to_store(
    store_root: &Path,
    signing_key: &SigningKey,
    params: &RevisionCommitParams,
) -> Result<RevisionCommitSummary, StoreRebuildError> {
    ensure_document_exists(store_root, &params.doc_id)?;
    for parent_id in &params.parents {
        ensure_object_exists(store_root, parent_id, "parent revision")?;
    }
    for patch_id in &params.patches {
        ensure_object_exists(store_root, patch_id, "patch")?;
    }

    let object_index = load_store_object_index(store_root)?;
    let author = signer_id(signing_key);
    let replay_revision_object = RevisionObject {
        revision_id: "rev:pending".to_string(),
        doc_id: params.doc_id.clone(),
        parents: params.parents.clone(),
        patches: params.patches.clone(),
        merge_strategy: params.merge_strategy.clone(),
        state_hash: "hash:pending".to_string(),
        author: author.clone(),
        timestamp: params.timestamp,
    };
    let state = replay_revision(&replay_revision_object, &object_index).map_err(|error| {
        StoreRebuildError::new(format!("failed to replay committed revision: {error}"))
    })?;
    let recomputed_state_hash = compute_state_hash(&state).map_err(|error| {
        StoreRebuildError::new(format!("failed to compute revision state_hash: {error}"))
    })?;

    let mut revision = json!({
        "type": "revision",
        "version": CORE_PROTOCOL_VERSION,
        "doc_id": params.doc_id,
        "parents": params.parents,
        "patches": params.patches,
        "state_hash": recomputed_state_hash,
        "author": author,
        "timestamp": params.timestamp
    });
    if let Some(merge_strategy) = &params.merge_strategy {
        revision["merge_strategy"] = Value::String(merge_strategy.clone());
    }
    let revision_id =
        recompute_object_id(&revision, "revision_id", "rev").map_err(StoreRebuildError::new)?;
    revision["revision_id"] = Value::String(revision_id.clone());
    revision["signature"] = Value::String(sign_object_value(signing_key, &revision)?);

    let write = write_object_value_to_store(store_root, &revision)?;

    Ok(RevisionCommitSummary {
        store_root: store_root.to_path_buf(),
        status: "ok".to_string(),
        doc_id: params.doc_id.clone(),
        revision_id,
        parent_revision_ids: params.parents.clone(),
        patch_ids: params.patches.clone(),
        recomputed_state_hash,
        written_object_count: usize::from(write.created),
        existing_object_count: usize::from(!write.created),
        stored_object: write.record,
        index_manifest_path: write.index_manifest_path,
        notes: Vec::new(),
        errors: Vec::new(),
    })
}

fn ensure_document_exists(store_root: &Path, doc_id: &str) -> Result<(), StoreRebuildError> {
    ensure_object_exists(store_root, doc_id, "document")
}

fn ensure_object_exists(
    store_root: &Path,
    object_id: &str,
    label: &str,
) -> Result<(), StoreRebuildError> {
    load_stored_object_value(store_root, object_id)
        .map(|_| ())
        .map_err(|_| {
            StoreRebuildError::new(format!(
                "{label} '{}' was not found in the store",
                object_id
            ))
        })
}

fn sign_object_value(signing_key: &SigningKey, value: &Value) -> Result<String, StoreRebuildError> {
    let payload = signed_payload_bytes(value).map_err(|error| {
        StoreRebuildError::new(format!("failed to compute signed payload: {error}"))
    })?;
    let signature = signing_key.sign(&payload);
    Ok(format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    ))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use base64::Engine;
    use ed25519_dalek::SigningKey;
    use serde_json::json;

    use super::{
        commit_revision_to_store, create_document_in_store, create_patch_in_store,
        parse_signing_key_seed, signer_id, DocumentCreateParams, PatchCreateParams,
        RevisionCommitParams,
    };
    use crate::replay::replay_revision_from_index;
    use crate::store::{load_store_index_manifest, load_stored_object_value};

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mycel-author-{prefix}-{unique}"));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    fn signing_key() -> SigningKey {
        parse_signing_key_seed(&base64::engine::general_purpose::STANDARD.encode([7u8; 32]))
            .expect("signing key seed should parse")
    }

    #[test]
    fn authoring_flow_creates_document_patch_and_revision_in_store() {
        let store_root = temp_dir("flow");
        let signing_key = signing_key();
        let document = create_document_in_store(
            &store_root,
            &signing_key,
            &DocumentCreateParams {
                doc_id: "doc:author-flow".to_string(),
                title: "Author Flow".to_string(),
                language: "en".to_string(),
                timestamp: 10,
            },
        )
        .expect("document should be created");
        assert_eq!(document.written_object_count, 2);

        let patch = create_patch_in_store(
            &store_root,
            &signing_key,
            &PatchCreateParams {
                doc_id: "doc:author-flow".to_string(),
                base_revision: document.genesis_revision_id.clone(),
                timestamp: 11,
                ops: json!([
                    {
                        "op": "insert_block",
                        "new_block": {
                            "block_id": "blk:001",
                            "block_type": "paragraph",
                            "content": "Hello authoring",
                            "attrs": {},
                            "children": []
                        }
                    }
                ]),
            },
        )
        .expect("patch should be created");
        assert_eq!(patch.written_object_count, 1);

        let revision = commit_revision_to_store(
            &store_root,
            &signing_key,
            &RevisionCommitParams {
                doc_id: "doc:author-flow".to_string(),
                parents: vec![document.genesis_revision_id.clone()],
                patches: vec![patch.patch_id.clone()],
                merge_strategy: None,
                timestamp: 12,
            },
        )
        .expect("revision should be committed");
        assert_eq!(revision.written_object_count, 1);

        let manifest = load_store_index_manifest(&store_root).expect("manifest should load");
        assert_eq!(
            manifest.doc_revisions.get("doc:author-flow").map(Vec::len),
            Some(2)
        );
        assert_eq!(
            manifest
                .author_patches
                .get(&signer_id(&signing_key))
                .map(Vec::len),
            Some(1)
        );

        let mut object_index =
            crate::store::load_store_object_index(&store_root).expect("object index should load");
        object_index.insert(
            "doc:author-flow".to_string(),
            load_stored_object_value(&store_root, "doc:author-flow").expect("document should load"),
        );
        let replay = replay_revision_from_index(
            &load_stored_object_value(&store_root, &revision.revision_id)
                .expect("revision should load"),
            &object_index,
        )
        .expect("revision replay should succeed");
        assert_eq!(replay.revision_id, revision.revision_id);
        assert_eq!(replay.state.doc_id, "doc:author-flow");
        assert_eq!(replay.state.blocks.len(), 1);

        let _ = fs::remove_dir_all(store_root);
    }
}
