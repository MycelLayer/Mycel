use super::*;

#[test]
fn object_verify_text_reports_ok_for_document_without_signature() {
    let object = write_object_file(
        "object-verify-document",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "en",
            "content_model": "block-tree",
            "created_at": 1777777777u64,
            "created_by": "pk:authorA",
            "genesis_revision": "rev:genesis"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path]);

    assert_success(&output);
    assert_empty_stderr(&output);
    assert_stdout_contains(&output, "object type: document");
    assert_stdout_contains(&output, "signature rule: forbidden");
    assert_stdout_contains(&output, "verification: ok");
}

#[test]
fn object_verify_json_fails_for_document_missing_doc_id() {
    let object = write_object_file(
        "object-verify-document-missing-doc-id",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "title": "Plain document"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(
                |errors| errors
                    .iter()
                    .any(|entry| entry.as_str().is_some_and(|message| message
                        .contains("document object is missing string field 'doc_id'")))
            ),
        "expected missing doc_id error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_non_string_doc_id() {
    let object = write_object_file(
        "object-verify-document-non-string-doc-id",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": 7,
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'doc_id' must be a string"))
            })),
        "expected doc_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_wrong_doc_id_prefix() {
    let object = write_object_file(
        "object-verify-document-wrong-doc-id-prefix",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "revision:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(
                |errors| errors
                    .iter()
                    .any(|entry| entry.as_str().is_some_and(|message| {
                        message.contains("top-level 'doc_id' must use 'doc:' prefix")
                    }))
            ),
        "expected wrong doc_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_unknown_top_level_field() {
    let object = write_object_file(
        "object-verify-document-unknown-field",
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
            "genesis_revision": "rev:test",
            "unexpected": true
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level contains unexpected field 'unexpected'")
                })
            })),
        "expected unknown-field validation error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_wrong_content_model() {
    let object = write_object_file(
        "object-verify-document-wrong-content-model",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "markdown",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'content_model' must equal 'block-tree'")
                })
            })),
        "expected content_model validation error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_wrong_created_by_prefix() {
    let object = write_object_file(
        "object-verify-document-wrong-created-by-prefix",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "sig:bad",
            "genesis_revision": "rev:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'created_by' must use 'pk:' prefix")
                })
            })),
        "expected created_by prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_wrong_genesis_revision_prefix() {
    let object = write_object_file(
        "object-verify-document-wrong-genesis-revision-prefix",
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
            "genesis_revision": "hash:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'genesis_revision' must use 'rev:' prefix")
                })
            })),
        "expected genesis_revision prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_duplicate_object_keys() {
    let object = write_raw_object_file(
        "object-verify-duplicate-keys",
        "document.json",
        r#"{
  "type": "document",
  "version": "mycel/0.1",
  "doc_id": "doc:first",
  "doc_id": "doc:second",
  "title": "Duplicate key object"
}"#,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("failed to parse JSON: duplicate object key 'doc_id'")
                })
            })),
        "expected duplicate-key parse error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_missing_title() {
    let object = write_object_file(
        "object-verify-document-missing-title",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "language": "zh-Hant",
            "content_model": "block-tree",
            "created_at": 1u64,
            "created_by": "pk:ed25519:test",
            "genesis_revision": "rev:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing string field 'title'"))
            })),
        "expected missing title error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_missing_created_at() {
    let document = json!({
        "type": "document",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "title": "Test Document",
        "language": "en",
        "content_model": "block-tree",
        "created_by": signer_id(&signing_key()),
        "genesis_revision": "rev:genesis-test"
    });
    let object = write_object_file(
        "object-verify-document-missing-created-at",
        "document.json",
        document,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing integer field 'created_at'"))
            })),
        "expected missing created_at error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_document_with_negative_created_at() {
    let document = json!({
        "type": "document",
        "version": "mycel/0.1",
        "doc_id": "doc:test",
        "title": "Test Document",
        "language": "en",
        "content_model": "block-tree",
        "created_at": -1,
        "created_by": signer_id(&signing_key()),
        "genesis_revision": "rev:genesis-test"
    });
    let object = write_object_file(
        "object-verify-document-negative-created-at",
        "document.json",
        document,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'created_at' must be a non-negative integer")
                })
            })),
        "expected created_at integer error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_missing_block_id() {
    let object = write_object_file(
        "object-verify-block-missing-block-id",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(
                |errors| errors
                    .iter()
                    .any(|entry| entry.as_str().is_some_and(|message| message
                        .contains("block object is missing string field 'block_id'")))
            ),
        "expected missing block_id error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_non_string_block_id() {
    let object = write_object_file(
        "object-verify-block-non-string-block-id",
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
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'block_id' must be a string")
                })
            })),
        "expected block_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_missing_attrs() {
    let object = write_object_file(
        "object-verify-block-missing-attrs",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing object field 'attrs'"))
            })),
        "expected missing attrs error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_non_string_content() {
    let object = write_object_file(
        "object-verify-block-non-string-content",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": 7,
            "attrs": {},
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'content' must be a string"))
            })),
        "expected content type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_non_object_attrs() {
    let object = write_object_file(
        "object-verify-block-non-object-attrs",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": [],
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'attrs' must be an object"))
            })),
        "expected attrs object-shape error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_empty_attr_key() {
    let object = write_object_file(
        "object-verify-block-empty-attr-key",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {
                "": "value"
            },
            "children": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'attrs' keys must not be empty strings")
                })
            })),
        "expected empty attrs-key error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_non_array_children() {
    let object = write_object_file(
        "object-verify-block-non-array-children",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": {}
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'children' must be an array")
                })
            })),
        "expected children array-shape error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_wrong_block_id_prefix() {
    let object = write_object_file(
        "object-verify-block-wrong-block-id-prefix",
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
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'block_id' must use 'blk:' prefix")
                })
            })),
        "expected block_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_unknown_top_level_field() {
    let object = write_object_file(
        "object-verify-block-unknown-top-level-field",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [],
            "unexpected": true
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level contains unexpected field 'unexpected'")
                })
            })),
        "expected unknown-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_block_with_unknown_nested_child_field() {
    let object = write_object_file(
        "object-verify-block-unknown-nested-child-field",
        "block.json",
        json!({
            "type": "block",
            "version": "mycel/0.1",
            "block_id": "blk:001",
            "block_type": "paragraph",
            "content": "Hello",
            "attrs": {},
            "children": [
                {
                    "block_id": "blk:002",
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
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "block");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message
                        .contains("top-level 'children[0]' contains unexpected field 'unexpected'")
                })
            })),
        "expected nested child unknown-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_floating_point_values() {
    let object = write_raw_object_file(
        "object-verify-float-value",
        "document.json",
        r#"{
  "type": "document",
  "version": "mycel/0.1",
  "doc_id": "doc:test",
  "priority": 1.5
}"#,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(
                |errors| errors
                    .iter()
                    .any(|entry| entry.as_str().is_some_and(|message| {
                        message.contains("$.priority: floating-point numbers are not allowed")
                    }))
            ),
        "expected floating-point validation error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_when_document_has_forbidden_signature() {
    let object = write_object_file(
        "object-verify-document-signature",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": "Plain document",
            "signature": "sig:not-allowed"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["object_type"], "document");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("must not include top-level 'signature'")
                })
            })),
        "expected forbidden signature error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_null_values() {
    let object = write_object_file(
        "object-verify-null-values",
        "document.json",
        json!({
            "type": "document",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "title": Value::Null
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("$.title: null is not allowed"))
            })),
        "expected null validation error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_missing_target_fails_cleanly() {
    let output = run_mycel(&["object", "verify"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "required arguments were not provided");
    assert_stderr_contains(&output, "<PATH>");
}

#[test]
fn object_verify_unknown_subcommand_fails_cleanly() {
    let output = run_mycel(&["object", "bogus"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unknown object subcommand: bogus");
    assert_top_level_help(&stdout_text(&output));
}
