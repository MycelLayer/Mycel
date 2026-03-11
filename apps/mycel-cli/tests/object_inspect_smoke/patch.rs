use super::*;

#[test]
fn object_inspect_text_warns_for_patch_missing_signature() {
    let object = write_object_file(
        "object-inspect-patch-warning",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path]);

    assert_success(&output);
    assert_empty_stderr(&output);
    assert_stdout_contains(&output, "object type: patch");
    assert_stdout_contains(&output, "signature rule: required");
    assert_stdout_contains(&output, "has signature: no");
    assert_stdout_contains(&output, "inspection: warning");
    assert_stdout_contains(
        &output,
        "note: patch object is missing string signer field 'author'",
    );
    assert_stdout_contains(
        &output,
        "note: patch object is missing top-level 'signature'",
    );
}

#[test]
fn object_inspect_json_warns_for_patch_with_wrong_base_revision_prefix() {
    let object = write_object_file(
        "object-inspect-patch-wrong-base-revision-prefix",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "hash:base",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry
                    .as_str()
                    .is_some_and(|message| message
                        .contains("top-level 'base_revision' must use 'rev:' prefix")))),
        "expected patch base_revision prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_patch_with_wrong_block_reference_prefix() {
    let object = write_object_file(
        "object-inspect-patch-wrong-block-reference-prefix",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "replace_block",
                    "block_id": "paragraph-1",
                    "new_content": "Hello"
                }
            ],
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("top-level 'ops[0]': top-level 'block_id' must use 'blk:' prefix")))),
        "expected patch block reference prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_patch_with_wrong_author_prefix() {
    let object = write_object_file(
        "object-inspect-patch-wrong-author-prefix",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "author:test",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'author' must use 'pk:' prefix")
                })
            })),
        "expected patch author warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_patch_with_unknown_top_level_field() {
    let object = write_object_file(
        "object-inspect-patch-unknown-top-level-field",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "unexpected": true,
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "patch");
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
fn object_inspect_json_warns_for_patch_move_without_destination() {
    let object = write_object_file(
        "object-inspect-patch-move-without-destination",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "move_block",
                    "block_id": "blk:001"
                }
            ],
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes.iter().any(|entry| entry.as_str().is_some_and(
            |message| message.contains("top-level 'ops[0]': move_block requires at least one destination reference")
        ))),
        "expected move_block destination warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_patch_with_empty_metadata_entries() {
    let object = write_object_file(
        "object-inspect-patch-empty-metadata",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "metadata": {}
                }
            ],
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("top-level 'ops[0]': top-level 'metadata' must not be empty")))),
        "expected empty metadata warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_patch_with_mixed_set_metadata_forms() {
    let object = write_object_file(
        "object-inspect-patch-mixed-set-metadata-forms",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:test",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:test",
            "timestamp": 1u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "metadata": {
                        "title": "Hello"
                    },
                    "key": "extra"
                }
            ],
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("top-level 'ops[0]': patch op contains unexpected field 'key'")))),
        "expected mixed set_metadata warning, stdout: {}",
        stdout_text(&output)
    );
}
