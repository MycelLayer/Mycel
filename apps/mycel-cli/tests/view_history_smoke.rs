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

fn build_history_store() -> (common::TempDir, String, Value, Value, Value) {
    let store_dir = create_temp_dir("view-history-store");
    let store_root = path_arg(store_dir.path());
    assert_success(&run_mycel(&["store", "init", &store_root, "--json"]));

    let stable_maintainer = SigningKey::from_bytes(&[211; 32]);
    let review_maintainer = SigningKey::from_bytes(&[212; 32]);
    let stable_policy = json!({
        "accept_keys": [signer_id(&stable_maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["stable"]
    });
    let review_policy = json!({
        "accept_keys": [signer_id(&review_maintainer)],
        "merge_rule": "manual-reviewed",
        "preferred_branches": ["review"]
    });
    let early = signed_view(
        &stable_maintainer,
        &stable_policy,
        json!({
            "doc:alpha": "rev:1111111111111111111111111111111111111111111111111111111111111111",
            "doc:beta": "rev:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }),
        10,
    );
    let middle = signed_view(
        &review_maintainer,
        &review_policy,
        json!({
            "doc:alpha": "rev:2222222222222222222222222222222222222222222222222222222222222222",
            "doc:gamma": "rev:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        }),
        20,
    );
    let late = signed_view(
        &stable_maintainer,
        &stable_policy,
        json!({
            "doc:alpha": "rev:3333333333333333333333333333333333333333333333333333333333333333",
            "doc:beta": "rev:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        }),
        30,
    );

    publish_view(&store_root, "view-history-late", &late);
    publish_view(&store_root, "view-history-early", &early);
    publish_view(&store_root, "view-history-middle", &middle);
    (store_dir, store_root, early, middle, late)
}

#[test]
fn view_history_json_reports_chronological_document_transitions() {
    let (_store_dir, store_root, early, middle, late) = build_history_store();
    let output = run_mycel(&[
        "view",
        "history",
        "--store-root",
        &store_root,
        "--doc-id",
        "doc:alpha",
        "--json",
    ]);
    assert_success(&output);
    let summary = parse_json_stdout(&output);
    let result = &summary["result"];

    assert_eq!(summary["status"], "ok");
    assert_eq!(result["doc_id"], "doc:alpha");
    assert_eq!(result["record_count"], 3);
    assert_eq!(result["transition_count"], 2);
    assert_eq!(
        result["records"]
            .as_array()
            .expect("history records should be an array")
            .iter()
            .map(|record| record["timestamp"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![10, 20, 30]
    );
    assert_eq!(result["records"][0]["governance_policy"], early["policy"]);
    assert_eq!(result["records"][1]["governance_policy"], middle["policy"]);
    assert_eq!(result["records"][2]["governance_policy"], late["policy"]);
    assert_eq!(result["transitions"][0]["policy_changed"], true);
    assert_eq!(result["transitions"][1]["policy_changed"], true);
    for transition in result["transitions"]
        .as_array()
        .expect("history transitions should be an array")
    {
        assert_eq!(transition["document_changes"].as_array().unwrap().len(), 1);
        assert_eq!(transition["document_changes"][0]["doc_id"], "doc:alpha");
    }
}

#[test]
fn view_history_profile_and_timestamp_range_are_inclusive() {
    let (_store_dir, store_root, early, _middle, late) = build_history_store();
    let profile_id = mycel_core::protocol::prefixed_canonical_hash(&early["policy"], "hash")
        .expect("profile ID should compute");
    let output = run_mycel(&[
        "view",
        "history",
        "--store-root",
        &store_root,
        "--profile-id",
        &profile_id,
        "--timestamp-min",
        "10",
        "--timestamp-max",
        "30",
        "--json",
    ]);
    assert_success(&output);
    let summary = parse_json_stdout(&output);
    let result = &summary["result"];

    assert_eq!(result["profile_id"], profile_id);
    assert_eq!(result["record_count"], 2);
    assert_eq!(result["transition_count"], 1);
    assert_eq!(result["records"][0]["view_id"], early["view_id"]);
    assert_eq!(result["records"][1]["view_id"], late["view_id"]);
    assert_eq!(result["transitions"][0]["policy_changed"], false);
}

#[test]
fn view_history_text_reports_records_and_rejects_inverted_range() {
    let (_store_dir, store_root, _early, _middle, _late) = build_history_store();
    let output = run_mycel(&[
        "view",
        "history",
        "--store-root",
        &store_root,
        "--doc-id",
        "doc:alpha",
    ]);
    assert_success(&output);
    assert_stdout_contains(&output, "history record count: 3");
    assert_stdout_contains(&output, "history transition count: 2");
    assert_stdout_contains(&output, "history transition: view:");
    assert_stdout_contains(&output, "history document change: doc:alpha");
    assert_stdout_contains(&output, "view history: ok");

    let inverted = run_mycel(&[
        "view",
        "history",
        "--store-root",
        &store_root,
        "--doc-id",
        "doc:alpha",
        "--timestamp-min",
        "31",
        "--timestamp-max",
        "30",
    ]);
    assert_exit_code(&inverted, 2);
    assert_stderr_contains(
        &inverted,
        "view history timestamp-min cannot be greater than timestamp-max",
    );
}
