use super::*;

#[test]
fn object_inspect_json_reports_ok_for_document() {
    let object = write_object_file(
        "object-inspect-document",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["object_type"], "document");
    assert_eq!(json["version"], "mycel/0.1");
    assert_eq!(json["signature_rule"], "forbidden");
    assert_eq!(json["has_signature"], false);
    assert_eq!(
        json["top_level_keys"],
        json!([
            "content_model",
            "created_at",
            "created_by",
            "doc_id",
            "genesis_revision",
            "language",
            "title",
            "type",
            "version"
        ])
    );
}

#[test]
fn object_inspect_json_warns_for_unsupported_type() {
    let object = write_object_file(
        "object-inspect-unsupported",
        "custom.json",
        json!({
            "type": "custom-object",
            "version": "mycel/0.1",
            "title": "Experimental object"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "custom-object");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("unsupported object type 'custom-object'")
                ))),
        "expected unsupported-type note, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_document_with_non_string_doc_id() {
    let object = write_object_file(
        "object-inspect-document-wrong-id-type",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": 7,
            "title": "Hello",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("top-level 'doc_id' should be a string")))),
        "expected doc_id warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_document_with_wrong_content_model() {
    let object = write_object_file(
        "object-inspect-document-wrong-content-model",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "content_model": "rich-text",
            "title": "Hello",
            "language": "zh-Hant",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'content_model' must equal 'block-tree'")
                }))),
        "expected content_model warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_document_with_wrong_created_by_prefix() {
    let object = write_object_file(
        "object-inspect-document-wrong-created-by-prefix",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "created_by": "author:test",
            "title": "Hello",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "genesis_revision": "rev:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'created_by' must use 'pk:' prefix")
                ))),
        "expected created_by warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_document_with_wrong_genesis_revision_prefix() {
    let object = write_object_file(
        "object-inspect-document-wrong-genesis-revision-prefix",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "genesis_revision": "hash:test",
            "title": "Hello",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'genesis_revision' must use 'rev:' prefix")
                }))),
        "expected genesis_revision warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_block_with_non_string_block_id() {
    let object = write_object_file(
        "object-inspect-block-wrong-id-type",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": 7,
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'block_id' should be a string")
                ))),
        "expected block_id warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_block_with_wrong_block_id_prefix() {
    let object = write_object_file(
        "object-inspect-block-wrong-id-prefix",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "paragraph-1",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'block_id' must use 'blk:' prefix")
                ))),
        "expected block_id prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_block_with_unknown_top_level_field() {
    let object = write_object_file(
        "object-inspect-block-unknown-top-level-field",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:test",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [],
            "unexpected": true
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(|message| {
                    message.contains("top-level contains unexpected field 'unexpected'")
                }))),
        "expected unexpected-field warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_block_with_unknown_nested_child_field() {
    let object = write_object_file(
        "object-inspect-block-unknown-nested-child-field",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:test",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [
                {
                    "block_id": "blk:child",
                    "block_type": "paragraph",
                    "content": "Child",
                    "attrs": {},
                    "children": [],
                    "unexpected": true
                }
            ]
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(|message| {
                    message
                        .contains("top-level 'children[0]' contains unexpected field 'unexpected'")
                }))),
        "expected nested child warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_fails_for_non_object_top_level_value() {
    let object = write_raw_object_file("object-inspect-non-object", "array.json", "[1,2,3]");
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level JSON value must be an object")
                })
            })),
        "expected top-level object error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_missing_target_fails_cleanly() {
    let output = run_mycel(&["object", "inspect"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "required arguments were not provided");
    assert_stderr_contains(&output, "<PATH>");
}
