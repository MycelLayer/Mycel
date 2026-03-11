use super::*;

#[test]
fn object_inspect_json_warns_for_revision_with_wrong_state_hash_prefix() {
    let object = write_object_file(
        "object-inspect-revision-wrong-state-hash-prefix",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "rev:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry
                    .as_str()
                    .is_some_and(|message| message
                        .contains("top-level 'state_hash' must use 'hash:' prefix")))),
        "expected revision state_hash prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_genesis_revision_with_merge_strategy() {
    let object = write_object_file(
        "object-inspect-revision-genesis-merge-strategy",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": [],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("top-level 'merge_strategy' is not allowed when 'parents' is empty")))),
        "expected genesis merge_strategy warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_revision_with_duplicate_parent_ids() {
    let object = write_object_file(
        "object-inspect-revision-duplicate-parents",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry
                    .as_str()
                    .is_some_and(|message| message
                        .contains("top-level 'parents[1]' duplicates 'parents[0]'")))),
        "expected duplicate parent warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_revision_with_unknown_top_level_field() {
    let object = write_object_file(
        "object-inspect-revision-unknown-top-level-field",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "unexpected": true,
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
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

#[test]
fn object_inspect_json_warns_for_revision_with_wrong_parent_prefix() {
    let object = write_object_file(
        "object-inspect-revision-wrong-parent-prefix",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["hash:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'parents[0]' must use 'rev:' prefix")
                ))),
        "expected parent prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_revision_with_duplicate_patch_ids() {
    let object = write_object_file(
        "object-inspect-revision-duplicate-patches",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": ["patch:test", "patch:test"],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry
                    .as_str()
                    .is_some_and(|message| message
                        .contains("top-level 'patches[1]' duplicates 'patches[0]'")))),
        "expected duplicate patch warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_revision_with_wrong_author_prefix() {
    let object = write_object_file(
        "object-inspect-revision-wrong-author-prefix",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "author:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'author' must use 'pk:' prefix")
                ))),
        "expected revision author prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_revision_with_merge_strategy_on_single_parent() {
    let object = write_object_file(
        "object-inspect-revision-merge-strategy-single-parent",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base"],
            "patches": [],
            "merge_strategy": "semantic-block-merge",
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(|message| message
                    .contains("top-level 'merge_strategy' requires multiple parents")))),
        "expected merge_strategy warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_multi_parent_revision_without_merge_strategy() {
    let object = write_object_file(
        "object-inspect-revision-missing-merge-strategy",
        "revision.json",
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:test",
            "doc_id": "doc:test",
            "parents": ["rev:base", "rev:side"],
            "patches": [],
            "state_hash": "hash:test",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "revision");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes.iter().any(|entry| entry.as_str().is_some_and(
            |message| message.contains("top-level 'merge_strategy' is required when 'parents' has multiple entries")
        ))),
        "expected missing merge_strategy warning, stdout: {}",
        stdout_text(&output)
    );
}
