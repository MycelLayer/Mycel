use super::*;

#[test]
fn object_inspect_json_warns_for_view_with_wrong_document_key_prefix() {
    let object = write_object_file(
        "object-inspect-view-wrong-document-key-prefix",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "patch:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("top-level 'documents.patch:test key' must use 'doc:' prefix")))),
        "expected view documents key-prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_non_object_policy() {
    let object = write_object_file(
        "object-inspect-view-non-object-policy",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": "manual-reviewed",
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("top-level 'policy' must be an object")))),
        "expected non-object policy warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_non_string_policy_merge_rule() {
    let object = write_object_file(
        "object-inspect-view-non-string-policy-merge-rule",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": 7
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'policy.merge_rule' must be a string")
                })
            })),
        "expected merge_rule type warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_empty_policy_merge_rule() {
    let object = write_object_file(
        "object-inspect-view-empty-policy-merge-rule",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": ""
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'policy.merge_rule' must not be an empty string")
                })
            })),
        "expected empty merge_rule warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_non_array_policy_accept_keys() {
    let object = write_object_file(
        "object-inspect-view-non-array-policy-accept-keys",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed",
                "accept_keys": "pk:ed25519:test"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'policy.accept_keys' must be an array")
                })
            })),
        "expected accept_keys array-shape warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_duplicate_policy_preferred_branches() {
    let object = write_object_file(
        "object-inspect-view-duplicate-policy-preferred-branches",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed",
                "preferred_branches": ["main", "main"]
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains(
                    "top-level 'policy.preferred_branches[1]' duplicates 'policy.preferred_branches[0]'"
                )))),
        "expected duplicate preferred_branches warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_non_string_policy_preferred_branch() {
    let object = write_object_file(
        "object-inspect-view-non-string-policy-preferred-branch",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed",
                "preferred_branches": [7]
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'policy.preferred_branches[0]' must be a string")
                })
            })),
        "expected preferred_branches type warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_duplicate_policy_accept_keys() {
    let object = write_object_file(
        "object-inspect-view-duplicate-policy-accept-keys",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed",
                "accept_keys": ["pk:ed25519:test", "pk:ed25519:test"]
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'policy.accept_keys[1]' duplicates 'policy.accept_keys[0]'",
                    )
                })
            })),
        "expected duplicate accept_keys warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_empty_policy_preferred_branch() {
    let object = write_object_file(
        "object-inspect-view-empty-policy-preferred-branch",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed",
                "preferred_branches": [""]
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'policy.preferred_branches[0]' must not be an empty string",
                    )
                })
            })),
        "expected empty preferred_branches warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_wrong_maintainer_prefix() {
    let object = write_object_file(
        "object-inspect-view-wrong-maintainer-prefix",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "maintainer:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'maintainer' must use 'pk:' prefix")
                ))),
        "expected maintainer prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_empty_documents() {
    let object = write_object_file(
        "object-inspect-view-empty-documents",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {},
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'documents' must not be empty")
                ))),
        "expected empty documents warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_wrong_document_value_prefix() {
    let object = write_object_file(
        "object-inspect-view-wrong-document-value-prefix",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "patch:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(|message| message
                    .contains("top-level 'documents.doc:test' must use 'rev:' prefix")))),
        "expected document value prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_view_with_unknown_top_level_field() {
    let object = write_object_file(
        "object-inspect-view-unknown-top-level-field",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "view_id": "view:test",
            "maintainer": "pk:ed25519:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "unexpected": true,
            "timestamp": 12u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry
                    .as_str()
                    .is_some_and(|message| message
                        .contains("top-level contains unexpected field 'unexpected'")))),
        "expected unknown top-level field warning, stdout: {}",
        stdout_text(&output)
    );
}
