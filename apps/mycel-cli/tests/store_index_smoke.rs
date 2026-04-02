use std::fs;
use std::path::Path;

use ed25519_dalek::SigningKey;
use insta::assert_json_snapshot;
use mycel_core::author::signer_id;
use serde_json::{json, Value};

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_json_status, assert_stderr_contains,
    assert_success, create_temp_dir, parse_json_stdout, prefixed_hash_for_test, run_mycel,
    signed_test_object, stdout_text,
};

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

fn signed_object(value: Value, signer_field: &str, id_field: &str, id_prefix: &str) -> Value {
    let signing_key = signing_key();
    signed_test_object(value, &signing_key, signer_field, id_field, id_prefix)
}

fn profile_id(policy: &Value) -> String {
    prefixed_hash_for_test(policy, "hash")
}

struct StoreFixtureInfo {
    source_dir: common::TempDir,
    store_dir: common::TempDir,
    signer: String,
    revision_id: String,
    view_id: String,
    profile_id: String,
}

struct RelatedGovernanceFixtureInfo {
    store_dir: common::TempDir,
    maintainer: String,
    profile_a_id: String,
    profile_b_id: String,
    view_a1_id: String,
    view_a2_id: String,
    view_b1_id: String,
}

fn build_store_with_view() -> StoreFixtureInfo {
    let source_dir = create_temp_dir("store-index-source");
    let store_dir = create_temp_dir("store-index-root");
    let patch_path = source_dir.path().join("patch.json");
    let revision_path = source_dir.path().join("revision.json");
    let view_path = source_dir.path().join("view.json");

    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:index",
            "base_revision": "rev:genesis-null",
            "timestamp": 1u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    fs::write(
        &patch_path,
        serde_json::to_string_pretty(&patch).expect("patch should serialize"),
    )
    .expect("patch should write");

    let state_hash = prefixed_hash_for_test(&json!({"doc_id": "doc:index", "blocks": []}), "hash");
    let revision = signed_object(
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "doc_id": "doc:index",
            "parents": [],
            "patches": [patch["patch_id"].as_str().expect("patch id should exist")],
            "state_hash": state_hash,
            "timestamp": 2u64
        }),
        "author",
        "revision_id",
        "rev",
    );
    let revision_id = revision["revision_id"]
        .as_str()
        .expect("revision id should exist")
        .to_string();
    fs::write(
        &revision_path,
        serde_json::to_string_pretty(&revision).expect("revision should serialize"),
    )
    .expect("revision should write");

    let policy = json!({
        "accept_keys": [signer_id(&signing_key())],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let profile_id = profile_id(&policy);
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:index": revision["revision_id"].as_str().expect("revision id should exist")
            },
            "policy": policy,
            "timestamp": 3u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let view_id = view["view_id"]
        .as_str()
        .expect("view id should exist")
        .to_string();
    fs::write(
        &view_path,
        serde_json::to_string_pretty(&view).expect("view should serialize"),
    )
    .expect("view should write");

    let ingest = run_mycel(&[
        "store",
        "ingest",
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(store_dir.path()),
    ]);
    assert_success(&ingest);

    StoreFixtureInfo {
        source_dir,
        store_dir,
        signer: signer_id(&signing_key()),
        revision_id,
        view_id,
        profile_id,
    }
}

fn build_store_with_related_views() -> RelatedGovernanceFixtureInfo {
    let source_dir = create_temp_dir("store-index-related-source");
    let store_dir = create_temp_dir("store-index-related-root");

    let policy_a = json!({
        "accept_keys": [signer_id(&signing_key())],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let policy_b = json!({
        "accept_keys": [signer_id(&SigningKey::from_bytes(&[8u8; 32]))],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });

    let view_a1 = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:alpha": "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "doc:beta": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            },
            "policy": policy_a,
            "timestamp": 10u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let view_a2 = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:alpha": "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            },
            "policy": policy_a,
            "timestamp": 11u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let view_b1 = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:beta": "rev:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            },
            "policy": policy_b,
            "timestamp": 12u64
        }),
        "maintainer",
        "view_id",
        "view",
    );

    let view_a1_id = view_a1["view_id"]
        .as_str()
        .expect("view a1 id should exist")
        .to_string();
    let view_a2_id = view_a2["view_id"]
        .as_str()
        .expect("view a2 id should exist")
        .to_string();
    let view_b1_id = view_b1["view_id"]
        .as_str()
        .expect("view b1 id should exist")
        .to_string();
    let profile_a_id = profile_id(&policy_a);
    let profile_b_id = profile_id(&policy_b);

    fs::write(
        source_dir.path().join("view-a1.json"),
        serde_json::to_string_pretty(&view_a1).expect("view a1 should serialize"),
    )
    .expect("view a1 should write");
    fs::write(
        source_dir.path().join("view-a2.json"),
        serde_json::to_string_pretty(&view_a2).expect("view a2 should serialize"),
    )
    .expect("view a2 should write");
    fs::write(
        source_dir.path().join("view-b1.json"),
        serde_json::to_string_pretty(&view_b1).expect("view b1 should serialize"),
    )
    .expect("view b1 should write");

    let ingest = run_mycel(&[
        "store",
        "ingest",
        &path_arg(source_dir.path()),
        "--into",
        &path_arg(store_dir.path()),
    ]);
    assert_success(&ingest);

    RelatedGovernanceFixtureInfo {
        store_dir,
        maintainer: signer_id(&signing_key()),
        profile_a_id,
        profile_b_id,
        view_a1_id,
        view_a2_id,
        view_b1_id,
    }
}

#[test]
fn store_index_json_reads_persisted_manifest() {
    let fixture = build_store_with_view();
    let signer = fixture.signer.clone();
    let profile_id = fixture.profile_id.clone();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["stored_object_count"], 3);
    assert!(
        json["object_ids_by_type"]["patch"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected patch index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["doc_revisions"]["doc:index"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected doc revision index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["author_patches"][signer.as_str()]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected author patch index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["maintainer_views"][signer.as_str()]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected maintainer view index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["profile_views"][profile_id.as_str()]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected profile view index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["document_views"]["doc:index"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected document view index, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["profile_heads"][profile_id.as_str()]["doc:index"]
            .as_array()
            .is_some_and(|values| values.len() == 1),
        "expected profile head index, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn store_index_json_filters_common_indexes() {
    let fixture = build_store_with_view();
    let signer = fixture.signer.clone();
    let profile_id = fixture.profile_id.clone();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-id",
        "doc:index",
        "--author",
        &signer,
        "--maintainer",
        &signer,
        "--profile-id",
        &profile_id,
        "--object-type",
        "patch",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_json_snapshot!(
        "store_index_json_filters_common_indexes",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_json_filters_by_revision_and_view() {
    let fixture = build_store_with_view();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--revision-id",
        &fixture.revision_id,
        "--view-id",
        &fixture.view_id,
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_json_snapshot!(
        "store_index_json_filters_by_revision_and_view",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_text_reports_summary() {
    let fixture = build_store_with_view();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-id",
        "doc:index",
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("document revision indexes: 1"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("filter doc_id: doc:index"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("store index: ok"), "stdout: {stdout}");
}

#[test]
fn store_index_path_only_prints_manifest_path() {
    let fixture = build_store_with_view();

    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--path-only",
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    assert_eq!(
        stdout_text(&output).trim(),
        fixture
            .store_dir
            .path()
            .join("indexes")
            .join("manifest.json")
            .to_string_lossy()
    );
    let _ = fixture.source_dir.path();
}

#[test]
fn store_index_path_only_rejects_json() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--path-only",
        "--json",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(
        &output,
        "store index --path-only cannot be used with --json",
    );
}

#[test]
fn store_index_filters_only_json_emits_query_metadata() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-id",
        "doc:index",
        "--head-only",
        "--filters-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    let object = json
        .as_object()
        .expect("filters-only output should be a JSON object");
    assert!(
        !object.contains_key("doc_revisions"),
        "filters-only output should omit full indexes, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        !object.contains_key("profile_heads"),
        "filters-only output should omit profile heads, stdout: {}",
        stdout_text(&output)
    );
    assert_json_snapshot!(
        "store_index_filters_only_json_emits_query_metadata",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_counts_only_json_emits_section_counts() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--counts-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    let object = json
        .as_object()
        .expect("counts-only output should be a JSON object");
    assert!(
        !object.contains_key("object_ids_by_type"),
        "counts-only output should omit full indexes, stdout: {}",
        stdout_text(&output)
    );
    assert_json_snapshot!(
        "store_index_counts_only_json_emits_section_counts",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_manifest_only_json_emits_manifest_metadata() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--manifest-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    let object = json
        .as_object()
        .expect("manifest-only output should be a JSON object");
    assert!(
        !object.contains_key("filters"),
        "manifest-only output should omit query filters, stdout: {}",
        stdout_text(&output)
    );
    assert_json_snapshot!(
        "store_index_manifest_only_json_emits_manifest_metadata",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_doc_only_json_prunes_other_sections() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_json_snapshot!(
        "store_index_doc_only_json_prunes_other_sections",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_governance_only_json_prunes_non_governance_sections() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--governance-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_json_snapshot!(
        "store_index_governance_only_json_prunes_non_governance_sections",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_governance_only_json_embeds_related_view_context_per_record() {
    let fixture = build_store_with_related_views();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--governance-only",
        "--view-id",
        &fixture.view_a1_id,
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(
        json["view_governance"][0]["current_profile_source"],
        json!("persisted")
    );
    assert_eq!(
        json["view_governance"][0]["current_profile_maintainer"],
        json!(fixture.maintainer)
    );
    assert_eq!(
        json["view_governance"][0]["current_profile_timestamp"],
        json!(11)
    );
    assert_eq!(
        json["view_governance"][0]["is_current_profile_view"],
        json!(false)
    );
    assert_eq!(
        json["view_governance"][0]["is_current_document_view_ids"]["doc:alpha"],
        json!(false)
    );
    assert_eq!(
        json["view_governance"][0]["is_current_document_view_ids"]["doc:beta"],
        json!(true)
    );
    assert_json_snapshot!(
        "store_index_governance_only_json_embeds_related_view_context_per_record",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_top_level_maintainer_counts_match_view_maintainer_output() {
    let fixture = build_store_with_related_views();
    let store_root = path_arg(fixture.store_dir.path());
    let store_index = run_mycel(&["store", "index", &store_root, "--governance-only", "--json"]);
    assert_success(&store_index);
    let store_index_json = assert_json_status(&store_index, "ok");

    let view_maintainer = run_mycel(&[
        "view",
        "maintainer",
        "--store-root",
        &store_root,
        "--maintainer",
        &fixture.maintainer,
        "--json",
    ]);
    assert_success(&view_maintainer);
    let view_maintainer_json = parse_json_stdout(&view_maintainer);

    let maintainer_summary =
        &store_index_json["current_maintainer_governance"][fixture.maintainer.as_str()];
    let maintainer_profiles = maintainer_summary["current_profiles"]
        .as_object()
        .expect("store index maintainer profiles should be an object");
    let maintainer_documents = maintainer_summary["current_documents"]
        .as_object()
        .expect("store index maintainer documents should be an object");
    let current_profiles = view_maintainer_json["current_profiles"]
        .as_array()
        .expect("view maintainer current_profiles should be an array");
    let current_documents = view_maintainer_json["current_documents"]
        .as_array()
        .expect("view maintainer current_documents should be an array");

    assert_eq!(
        store_index_json["current_maintainer_governance"]
            .as_object()
            .map(|values| values.len()),
        Some(1)
    );
    assert_eq!(
        store_index_json["current_maintainer_governance_profile_count"],
        json!(current_profiles.len())
    );
    assert_eq!(
        store_index_json["current_maintainer_governance_document_count"],
        json!(current_documents.len())
    );
    assert_eq!(maintainer_profiles.len(), current_profiles.len());
    assert_eq!(maintainer_documents.len(), current_documents.len());

    for document in current_documents {
        let doc_id = document["doc_id"]
            .as_str()
            .expect("view maintainer current document should have doc_id");
        let view_profiles = document["profiles"]
            .as_array()
            .expect("view maintainer current document profiles should be an array");
        let store_profiles = maintainer_documents[doc_id]["profiles"]
            .as_object()
            .expect("store index current document profiles should be an object");
        assert_eq!(
            store_profiles.len(),
            view_profiles.len(),
            "profile coverage mismatch for document {doc_id}"
        );
        for profile in view_profiles {
            let profile_id = profile["profile_id"]
                .as_str()
                .expect("view maintainer current document profile should have profile_id");
            assert_eq!(
                store_profiles[profile_id]["accepted_editor_keys"], profile["accepted_editor_keys"],
                "accepted editors mismatch for maintainer document {doc_id} profile {profile_id}"
            );
            assert_eq!(
                store_profiles[profile_id]["maintainer_is_admitted_editor"],
                profile["maintainer_is_admitted_editor"],
                "maintainer admission mismatch for maintainer document {doc_id} profile {profile_id}"
            );
        }
    }
}

#[test]
fn store_index_top_level_profile_counts_match_view_current_output() {
    let fixture = build_store_with_related_views();
    let store_root = path_arg(fixture.store_dir.path());
    let store_index = run_mycel(&["store", "index", &store_root, "--governance-only", "--json"]);
    assert_success(&store_index);
    let store_index_json = assert_json_status(&store_index, "ok");

    let view_current_a = run_mycel(&[
        "view",
        "current",
        "--store-root",
        &store_root,
        "--profile-id",
        &fixture.profile_a_id,
        "--json",
    ]);
    assert_success(&view_current_a);
    let view_current_a_json = parse_json_stdout(&view_current_a);

    let view_current_b = run_mycel(&[
        "view",
        "current",
        "--store-root",
        &store_root,
        "--profile-id",
        &fixture.profile_b_id,
        "--json",
    ]);
    assert_success(&view_current_b);
    let view_current_b_json = parse_json_stdout(&view_current_b);

    let profile_a_summary = &store_index_json["current_governance"][fixture.profile_a_id.as_str()];
    let profile_b_summary = &store_index_json["current_governance"][fixture.profile_b_id.as_str()];
    let profile_a_documents = profile_a_summary["current_documents"]
        .as_object()
        .expect("store index profile a current_documents should be an object");
    let profile_b_documents = profile_b_summary["current_documents"]
        .as_object()
        .expect("store index profile b current_documents should be an object");
    let view_current_a_documents = view_current_a_json["current_documents"]
        .as_array()
        .expect("view current profile a current_documents should be an array");
    let view_current_b_documents = view_current_b_json["current_documents"]
        .as_array()
        .expect("view current profile b current_documents should be an array");

    assert_eq!(
        store_index_json["current_governance"]
            .as_object()
            .map(|values| values.len()),
        Some(2)
    );
    assert_eq!(
        store_index_json["current_governance_document_count"],
        json!(view_current_a_documents.len() + view_current_b_documents.len())
    );
    assert_eq!(profile_a_documents.len(), view_current_a_documents.len());
    assert_eq!(profile_b_documents.len(), view_current_b_documents.len());
    assert_eq!(
        profile_a_summary["accepted_editor_keys"],
        view_current_a_json["accepted_editor_keys"]
    );
    assert_eq!(
        profile_a_summary["maintainer_is_admitted_editor"],
        view_current_a_json["maintainer_is_admitted_editor"]
    );
    assert_eq!(
        profile_a_summary["admitted_editor_only_keys"],
        view_current_a_json["admitted_editor_only_keys"]
    );
    assert_eq!(
        profile_b_summary["accepted_editor_keys"],
        view_current_b_json["accepted_editor_keys"]
    );
    assert_eq!(
        profile_b_summary["maintainer_is_admitted_editor"],
        view_current_b_json["maintainer_is_admitted_editor"]
    );
    assert_eq!(
        profile_b_summary["admitted_editor_only_keys"],
        view_current_b_json["admitted_editor_only_keys"]
    );

    for current_document in view_current_a_documents {
        let doc_id = current_document["doc_id"]
            .as_str()
            .expect("view current profile a document should have doc_id");
        assert_eq!(
            profile_a_documents[doc_id]["view_id"], current_document["current_view_id"],
            "profile a current view mismatch for document {doc_id}"
        );
        assert_eq!(
            profile_a_documents[doc_id]["revision_id"], current_document["current_revision_id"],
            "profile a current revision mismatch for document {doc_id}"
        );
        assert_eq!(
            profile_a_documents[doc_id]["accepted_editor_keys"],
            current_document["accepted_editor_keys"],
            "profile a accepted editors mismatch for document {doc_id}"
        );
        assert_eq!(
            profile_a_documents[doc_id]["maintainer_is_admitted_editor"],
            current_document["maintainer_is_admitted_editor"],
            "profile a maintainer admission mismatch for document {doc_id}"
        );
    }

    for current_document in view_current_b_documents {
        let doc_id = current_document["doc_id"]
            .as_str()
            .expect("view current profile b document should have doc_id");
        assert_eq!(
            profile_b_documents[doc_id]["view_id"], current_document["current_view_id"],
            "profile b current view mismatch for document {doc_id}"
        );
        assert_eq!(
            profile_b_documents[doc_id]["revision_id"], current_document["current_revision_id"],
            "profile b current revision mismatch for document {doc_id}"
        );
        assert_eq!(
            profile_b_documents[doc_id]["accepted_editor_keys"],
            current_document["accepted_editor_keys"],
            "profile b accepted editors mismatch for document {doc_id}"
        );
        assert_eq!(
            profile_b_documents[doc_id]["maintainer_is_admitted_editor"],
            current_document["maintainer_is_admitted_editor"],
            "profile b maintainer admission mismatch for document {doc_id}"
        );
    }
}

#[test]
fn store_index_top_level_document_counts_match_view_document_output() {
    let fixture = build_store_with_related_views();
    let store_root = path_arg(fixture.store_dir.path());
    let store_index = run_mycel(&["store", "index", &store_root, "--governance-only", "--json"]);
    assert_success(&store_index);
    let store_index_json = assert_json_status(&store_index, "ok");

    let view_document = run_mycel(&[
        "view",
        "document",
        "--store-root",
        &store_root,
        "--doc-id",
        "doc:beta",
        "--json",
    ]);
    assert_success(&view_document);
    let view_document_json = parse_json_stdout(&view_document);

    let document_summary = &store_index_json["current_document_governance"]["doc:beta"];
    let document_profiles = document_summary["profiles"]
        .as_object()
        .expect("store index document profiles should be an object");
    let view_profiles = view_document_json["profiles"]
        .as_array()
        .expect("view document profiles should be an array");

    assert_eq!(
        store_index_json["current_document_governance"]
            .as_object()
            .map(|values| values.len()),
        Some(2)
    );
    assert_eq!(
        store_index_json["current_document_governance_profile_count"],
        json!(3)
    );
    assert_eq!(document_profiles.len(), view_profiles.len());

    for profile in view_profiles {
        let profile_id = profile["profile_id"]
            .as_str()
            .expect("view document profile should have profile_id");
        assert_eq!(
            document_profiles[profile_id]["view_id"], profile["current_view_id"],
            "current view mismatch for profile {profile_id}"
        );
        assert_eq!(
            document_profiles[profile_id]["revision_id"], profile["current_revision_id"],
            "current revision mismatch for profile {profile_id}"
        );
        assert_eq!(
            document_profiles[profile_id]["maintainer"], profile["maintainer"],
            "maintainer mismatch for profile {profile_id}"
        );
        assert_eq!(
            document_profiles[profile_id]["accepted_editor_keys"], profile["accepted_editor_keys"],
            "accepted editors mismatch for profile {profile_id}"
        );
        assert_eq!(
            document_profiles[profile_id]["maintainer_is_admitted_editor"],
            profile["maintainer_is_admitted_editor"],
            "maintainer admission mismatch for profile {profile_id}"
        );
    }
}

#[test]
fn store_index_governance_only_text_reports_related_view_context() {
    let fixture = build_store_with_related_views();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--governance-only",
        "--view-id",
        &fixture.view_a1_id,
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains(&format!("view governance record: {}", fixture.view_a1_id)),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "  maintainer related views: {}, {}, {}",
            fixture.view_a1_id, fixture.view_b1_id, fixture.view_a2_id
        )),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "  document related views: doc:beta -> {}, {}",
            fixture.view_a1_id, fixture.view_b1_id
        )),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "  current profile view id: {}",
            fixture.view_a2_id
        )),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("  current profile source: persisted"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("  current profile timestamp: 11"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("  is current profile view: false"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "  current profile document view: doc:alpha -> {}",
            fixture.view_a2_id
        )),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("  is current document view: doc:alpha -> false"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("  is current document view: doc:beta -> true"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("current governance profiles: 1"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("current document governance summaries: 2"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("current document governance profiles: 2"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("current maintainer governance summaries: 1"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("current maintainer governance profiles: 1"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("current maintainer governance documents: 2"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(&format!("  current view id: {}", fixture.view_a2_id)),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("current maintainer governance:"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "  current profile: {} view={}",
            fixture.profile_a_id, fixture.view_a2_id
        )),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("  current maintainer document: doc:beta"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("current document governance: doc:beta"),
        "stdout: {stdout}"
    );
}

#[test]
fn store_index_head_only_json_prunes_non_head_sections() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--head-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_json_snapshot!(
        "store_index_head_only_json_prunes_non_head_sections",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_patches_only_json_prunes_non_patch_sections() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--patches-only",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_json_snapshot!(
        "store_index_patches_only_json_prunes_non_patch_sections",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_parents_only_text_reports_projection() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--parents-only",
    ]);

    assert_success(&output);
    assert_empty_stderr(&output);
    let stdout = stdout_text(&output);
    assert!(
        stdout.contains("projection: parents-only"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("revision parent indexes: 1"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("document revision indexes: 0"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("view governance records: 0"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("maintainer view indexes: 0"),
        "stdout: {stdout}"
    );
}

#[test]
fn store_index_empty_query_fails_without_empty_ok() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-id",
        "doc:missing",
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "empty");
    assert_json_snapshot!(
        "store_index_empty_query_fails_without_empty_ok",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_empty_query_succeeds_with_empty_ok() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-id",
        "doc:missing",
        "--empty-ok",
        "--json",
    ]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_json_snapshot!(
        "store_index_empty_query_succeeds_with_empty_ok",
        json,
        {
            ".manifest_path" => "[manifest_path]",
            ".store_root" => "[store_root]",
        }
    );
}

#[test]
fn store_index_rejects_multiple_projection_flags() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--doc-only",
        "--head-only",
        "--governance-only",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(
        &output,
        "store index projection flags are mutually exclusive",
    );
}

#[test]
fn store_index_rejects_multiple_output_modes() {
    let fixture = build_store_with_view();
    let output = run_mycel(&[
        "store",
        "index",
        &path_arg(fixture.store_dir.path()),
        "--filters-only",
        "--counts-only",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(
        &output,
        "store index output mode flags are mutually exclusive",
    );
}

#[test]
fn store_index_missing_manifest_fails_cleanly() {
    let store_dir = create_temp_dir("store-index-missing");
    let output = run_mycel(&["store", "index", &path_arg(store_dir.path())]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "failed to read store index manifest");
}
