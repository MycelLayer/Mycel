mod common;

use std::fs;
use std::path::Path;

use ed25519_dalek::SigningKey;
use mycel_core::author::signer_id;
use serde_json::{json, Value};

use common::{
    assert_exit_code, assert_stderr_contains, assert_stdout_contains, assert_success,
    create_temp_dir, parse_json_stdout, recompute_test_object_id, run_mycel, sign_test_value,
};

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn signed_view(
    signing_key: &SigningKey,
    policy: &Value,
    documents: Value,
    timestamp: u64,
) -> Value {
    let mut value = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": signer_id(signing_key),
        "documents": documents,
        "policy": policy,
        "timestamp": timestamp
    });
    let id = recompute_test_object_id(&value, "view_id", "view");
    value["view_id"] = Value::String(id);
    value["signature"] = Value::String(sign_test_value(signing_key, &value));
    value
}

fn publish_view(store_root: &str, prefix: &str, view: &Value) {
    let source_dir = create_temp_dir(prefix);
    let source_path = source_dir.path().join("view.json");
    fs::write(
        &source_path,
        serde_json::to_string_pretty(view).expect("view should serialize"),
    )
    .expect("view should write");
    let output = run_mycel(&[
        "view",
        "publish",
        &path_arg(&source_path),
        "--into",
        store_root,
        "--json",
    ]);
    assert_success(&output);
}

fn build_diff_store() -> (common::TempDir, String, Value, Value) {
    let store_dir = create_temp_dir("view-diff-store");
    let store_root = path_arg(store_dir.path());
    assert_success(&run_mycel(&["store", "init", &store_root, "--json"]));

    let base_maintainer = SigningKey::from_bytes(&[201; 32]);
    let target_maintainer = SigningKey::from_bytes(&[202; 32]);
    let base_policy = json!({
        "accept_keys": [signer_id(&base_maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["main"]
    });
    let target_policy = json!({
        "accept_keys": [signer_id(&target_maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["stable"]
    });
    let base = signed_view(
        &base_maintainer,
        &base_policy,
        json!({
            "doc:alpha": "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "doc:beta": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "doc:shared": "rev:1111111111111111111111111111111111111111111111111111111111111111"
        }),
        10,
    );
    let target = signed_view(
        &target_maintainer,
        &target_policy,
        json!({
            "doc:alpha": "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "doc:gamma": "rev:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "doc:shared": "rev:1111111111111111111111111111111111111111111111111111111111111111"
        }),
        20,
    );
    publish_view(&store_root, "view-diff-base", &base);
    publish_view(&store_root, "view-diff-target", &target);
    (store_dir, store_root, base, target)
}

fn remove_policy_catalog(store_root: &str) {
    let manifest_path = Path::new(store_root).join("indexes").join("manifest.json");
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    manifest
        .as_object_mut()
        .expect("manifest should be an object")
        .remove("governance_profiles");
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
    )
    .expect("manifest should write");
}

#[test]
fn view_diff_json_reports_deterministic_governance_changes() {
    let (_store_dir, store_root, base, target) = build_diff_store();
    let output = run_mycel(&[
        "view",
        "diff",
        base["view_id"].as_str().expect("base View ID should exist"),
        target["view_id"]
            .as_str()
            .expect("target View ID should exist"),
        "--store-root",
        &store_root,
        "--json",
    ]);
    assert_success(&output);
    let summary = parse_json_stdout(&output);
    let result = &summary["result"];

    assert_eq!(summary["status"], "ok");
    assert_eq!(result["comparison"], "different");
    assert_eq!(result["difference_count"], 6);
    assert_eq!(result["maintainer_changed"], true);
    assert_eq!(result["policy_changed"], true);
    assert_eq!(result["timestamp_changed"], true);
    assert_eq!(result["unchanged_document_count"], 1);
    assert_eq!(result["base_governance_policy"], base["policy"]);
    assert_eq!(result["target_governance_policy"], target["policy"]);
    assert_eq!(
        result["document_changes"],
        json!([
            {
                "doc_id": "doc:alpha",
                "kind": "changed",
                "base_revision_id": "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "target_revision_id": "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            },
            {
                "doc_id": "doc:beta",
                "kind": "removed",
                "base_revision_id": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "target_revision_id": null
            },
            {
                "doc_id": "doc:gamma",
                "kind": "added",
                "base_revision_id": null,
                "target_revision_id": "rev:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            }
        ])
    );
}

#[test]
fn view_diff_same_view_matches_and_fail_on_diff_is_automation_friendly() {
    let (_store_dir, store_root, base, target) = build_diff_store();
    let base_id = base["view_id"].as_str().expect("base View ID should exist");
    let target_id = target["view_id"]
        .as_str()
        .expect("target View ID should exist");

    let matching = run_mycel(&[
        "view",
        "diff",
        base_id,
        base_id,
        "--store-root",
        &store_root,
        "--fail-on-diff",
        "--json",
    ]);
    assert_success(&matching);
    let matching_json = parse_json_stdout(&matching);
    assert_eq!(matching_json["result"]["comparison"], "match");
    assert_eq!(matching_json["result"]["difference_count"], 0);

    let different = run_mycel(&[
        "view",
        "diff",
        base_id,
        target_id,
        "--store-root",
        &store_root,
        "--fail-on-diff",
        "--json",
    ]);
    assert_exit_code(&different, 1);
    let different_json = parse_json_stdout(&different);
    assert_eq!(different_json["status"], "ok");
    assert_eq!(different_json["result"]["comparison"], "different");
}

#[test]
fn view_diff_text_reports_governance_identity_and_timestamp() {
    let (_store_dir, store_root, base, target) = build_diff_store();
    let output = run_mycel(&[
        "view",
        "diff",
        base["view_id"].as_str().expect("base View ID should exist"),
        target["view_id"]
            .as_str()
            .expect("target View ID should exist"),
        "--store-root",
        &store_root,
    ]);

    assert_success(&output);
    assert_stdout_contains(
        &output,
        &format!("base maintainer: {}", base["maintainer"].as_str().unwrap()),
    );
    assert_stdout_contains(
        &output,
        &format!(
            "target maintainer: {}",
            target["maintainer"].as_str().unwrap()
        ),
    );
    assert_stdout_contains(&output, "base profile id: hash:");
    assert_stdout_contains(&output, "target profile id: hash:");
    assert_stdout_contains(&output, "base timestamp: 10");
    assert_stdout_contains(&output, "target timestamp: 20");
}

#[test]
fn view_diff_reports_missing_view_without_panicking() {
    let (_store_dir, store_root, base, _target) = build_diff_store();
    let output = run_mycel(&[
        "view",
        "diff",
        base["view_id"].as_str().expect("base View ID should exist"),
        "view:missing",
        "--store-root",
        &store_root,
    ]);

    assert_exit_code(&output, 1);
    assert_stderr_contains(&output, "view 'view:missing' was not found");
}

#[test]
fn view_diff_legacy_manifest_keeps_hash_comparison_without_policy_values() {
    let (_store_dir, store_root, base, target) = build_diff_store();
    remove_policy_catalog(&store_root);

    let output = run_mycel(&[
        "view",
        "diff",
        base["view_id"].as_str().expect("base View ID should exist"),
        target["view_id"]
            .as_str()
            .expect("target View ID should exist"),
        "--store-root",
        &store_root,
        "--json",
    ]);
    assert_success(&output);
    let summary = parse_json_stdout(&output);

    assert_eq!(summary["result"]["policy_changed"], true);
    assert_eq!(summary["result"]["base_governance_policy"], Value::Null);
    assert_eq!(summary["result"]["target_governance_policy"], Value::Null);
}
