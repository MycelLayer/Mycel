use std::fs;
use std::path::PathBuf;

use base64::Engine;
use serde_json::json;

mod common;

use common::{assert_json_status, assert_success, create_temp_dir, run_mycel};

fn path_arg(path: &PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn write_signing_key_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("signing-key.txt");
    fs::write(
        &path,
        base64::engine::general_purpose::STANDARD.encode([7u8; 32]),
    )
    .expect("signing key should write");
    (dir, path)
}

fn write_ops_file(prefix: &str) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("ops.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:author-smoke-001",
                    "block_type": "paragraph",
                    "content": "Hello author smoke",
                    "attrs": {},
                    "children": []
                }
            }
        ]))
        .expect("ops JSON should serialize"),
    )
    .expect("ops JSON should write");
    (dir, path)
}

#[test]
fn store_authoring_flow_creates_document_patch_and_revision() {
    let store_dir = create_temp_dir("store-author-root");
    let (_key_dir, key_path) = write_signing_key_file("store-author-key");
    let (_ops_dir, ops_path) = write_ops_file("store-author-ops");
    let store_root = path_arg(&store_dir.path().to_path_buf());
    let key_file = path_arg(&key_path);
    let ops_file = path_arg(&ops_path);

    let init = run_mycel(&["store", "init", &store_root, "--json"]);
    assert_success(&init);
    let init_json = assert_json_status(&init, "ok");
    assert_eq!(init_json["store_root"], store_root);

    let document = run_mycel(&[
        "store",
        "create-document",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--title",
        "Author Smoke",
        "--language",
        "en",
        "--signing-key",
        &key_file,
        "--timestamp",
        "10",
        "--json",
    ]);
    assert_success(&document);
    let document_json = assert_json_status(&document, "ok");
    let genesis_revision_id = document_json["genesis_revision_id"]
        .as_str()
        .expect("genesis revision should be string")
        .to_string();
    assert_eq!(document_json["written_object_count"], 2);

    let patch = run_mycel(&[
        "store",
        "create-patch",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--base-revision",
        &genesis_revision_id,
        "--ops",
        &ops_file,
        "--signing-key",
        &key_file,
        "--timestamp",
        "11",
        "--json",
    ]);
    assert_success(&patch);
    let patch_json = assert_json_status(&patch, "ok");
    let patch_id = patch_json["patch_id"]
        .as_str()
        .expect("patch_id should be string")
        .to_string();
    assert_eq!(patch_json["written_object_count"], 1);

    let revision = run_mycel(&[
        "store",
        "commit-revision",
        &store_root,
        "--doc-id",
        "doc:author-smoke",
        "--parent",
        &genesis_revision_id,
        "--patch",
        &patch_id,
        "--signing-key",
        &key_file,
        "--timestamp",
        "12",
        "--json",
    ]);
    assert_success(&revision);
    let revision_json = assert_json_status(&revision, "ok");
    assert_eq!(revision_json["written_object_count"], 1);
    assert!(revision_json["recomputed_state_hash"]
        .as_str()
        .is_some_and(|value| value.starts_with("hash:")));

    let index = run_mycel(&["store", "index", &store_root, "--json"]);
    assert_success(&index);
    let index_json = assert_json_status(&index, "ok");
    assert_eq!(index_json["stored_object_count"], 4);
    assert_eq!(
        index_json["doc_revisions"]["doc:author-smoke"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["document"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["patch"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        index_json["object_ids_by_type"]["revision"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    let rebuild = run_mycel(&["store", "rebuild", &store_root, "--json"]);
    assert_success(&rebuild);
    let rebuild_json = assert_json_status(&rebuild, "ok");
    assert_eq!(rebuild_json["stored_object_count"], 4);
    assert_eq!(rebuild_json["verified_object_count"], 4);
}
