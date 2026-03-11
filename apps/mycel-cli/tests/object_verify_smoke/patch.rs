use super::*;

#[test]
fn object_verify_json_reports_ok_for_valid_patch() {
    let object = write_object_file(
        "object-verify-patch",
        "patch.json",
        signed_object(
            json!({
                "type": "patch",
                "version": "mycel/0.1",
                "doc_id": "doc:test",
                "base_revision": "rev:genesis-null",
                "timestamp": 1777778888u64,
                "ops": []
            }),
            "author",
            "patch_id",
            "patch",
        ),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["object_type"], "patch");
    assert_eq!(json["signature_rule"], "required");
    assert_eq!(json["signer_field"], "author");
    assert_eq!(json["signature_verification"], "verified");
    assert_eq!(json["signer"], signer_id(&signing_key()));
    assert_eq!(json["declared_id"], json["recomputed_id"]);
    assert_eq!(json["notes"], Value::Array(Vec::new()));
}

#[test]
fn object_verify_json_fails_for_invalid_patch_signature() {
    let mut patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    patch["signature"] = Value::String(
        "sig:ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
            .to_string(),
    );
    let object = write_object_file("object-verify-patch-bad-signature", "patch.json", patch);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["signature_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("Ed25519 signature verification failed")))),
        "expected signature failure, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_wrong_signature_format() {
    let object = write_object_file(
        "object-verify-patch-wrong-signature-format",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": signer_id(&signing_key()),
            "timestamp": 1777778888u64,
            "ops": [],
            "signature": "sig:bad"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"].as_array().is_some_and(|errors| errors
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("signature field must use format 'sig:ed25519:<base64>'")))),
        "expected signature format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_malformed_signature_bytes() {
    let object = write_object_file(
        "object-verify-patch-malformed-signature",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": signer_id(&signing_key()),
            "timestamp": 1777778888u64,
            "ops": [],
            "signature": "sig:ed25519:not-base64"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("failed to decode Ed25519 signature"))
            })),
        "expected malformed signature decode error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_invalid_signature_bytes() {
    let object = write_object_file(
        "object-verify-patch-invalid-signature-bytes",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": signer_id(&signing_key()),
            "timestamp": 1777778888u64,
            "ops": [],
            "signature": "sig:ed25519:AA=="
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("invalid Ed25519 signature bytes"))
            })),
        "expected invalid signature bytes error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_op_unknown_field() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "delete_block",
                    "block_id": "blk:001",
                    "new_content": "unexpected"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file("object-verify-patch-op-unknown-field", "patch.json", patch);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("patch op contains unexpected field 'new_content'")
                })
            })),
        "expected patch-op unknown-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_nested_block_shape_with_path() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:001",
                        "block_type": "paragraph",
                        "content": "Hello"
                    }
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-nested-block-shape",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'ops[0]': top-level 'new_block'")
                        && message.contains("missing object field 'attrs'")
                })
            })),
        "expected nested block shape error with path, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_wrong_base_revision_prefix() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "hash:base",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-wrong-base-revision-prefix",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'base_revision' must use 'rev:' prefix")
                })
            })),
        "expected base_revision prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_wrong_block_reference_prefix() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "replace_block",
                    "block_id": "paragraph-1",
                    "new_content": "Hello"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-wrong-block-reference-prefix",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message
                        .contains("top-level 'ops[0]': top-level 'block_id' must use 'blk:' prefix")
                })
            })),
        "expected block reference prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_wrong_author_prefix() {
    let mut patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    patch["author"] = Value::String("author:test".to_string());
    let object = write_object_file(
        "object-verify-patch-wrong-author-prefix",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("signer field must use format 'pk:ed25519:<base64>'")
                })
            })),
        "expected signer-format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_malformed_author_key() {
    let mut patch = json!({
        "type": "patch",
        "version": "mycel/0.1",
        "patch_id": "patch:placeholder",
        "doc_id": "doc:test",
        "base_revision": "rev:genesis-null",
        "author": "pk:ed25519:not-base64",
        "timestamp": 1777778888u64,
        "ops": []
    });
    patch["signature"] = Value::String(sign_value(&signing_key(), &patch));
    let object = write_object_file(
        "object-verify-patch-malformed-author-key",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("failed to decode Ed25519 public key"))
            })),
        "expected malformed public-key error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_invalid_author_key_bytes() {
    let object = write_object_file(
        "object-verify-patch-invalid-author-bytes",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:AA==",
            "timestamp": 1777778888u64,
            "ops": [],
            "signature": "sig:ed25519:AA=="
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("Ed25519 public key must decode to 32 bytes")
                })
            })),
        "expected invalid public-key length error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_missing_author() {
    let mut patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    patch
        .as_object_mut()
        .expect("patch should be an object")
        .remove("author");
    let object = write_object_file("object-verify-patch-missing-author", "patch.json", patch);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("patch object is missing string signer field 'author'")
                })
            })),
        "expected missing author signer-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_missing_signature() {
    let object = write_object_file(
        "object-verify-patch-missing-signature",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": signer_id(&signing_key()),
            "timestamp": 1777778888u64,
            "ops": []
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("patch object is missing required top-level 'signature'")
                })
            })),
        "expected missing signature error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_missing_timestamp() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file("object-verify-patch-missing-timestamp", "patch.json", patch);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing integer field 'timestamp'"))
            })),
        "expected missing timestamp error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_negative_timestamp() {
    let mut patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    patch["timestamp"] = json!(-1);
    patch["signature"] = Value::String(sign_value(&signing_key(), &patch));
    let object = write_object_file(
        "object-verify-patch-negative-timestamp",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'timestamp' must be a non-negative integer")
                })
            })),
        "expected timestamp integer error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_non_string_signature() {
    let object = write_object_file(
        "object-verify-patch-non-string-signature",
        "patch.json",
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": signer_id(&signing_key()),
            "timestamp": 1777778888u64,
            "ops": [],
            "signature": 7
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'signature' must be a string")
                })
            })),
        "expected signature type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_wrong_patch_id_prefix() {
    let mut patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    patch["patch_id"] = Value::String(
        patch["patch_id"]
            .as_str()
            .expect("patch_id should exist")
            .replacen("patch:", "rev:", 1),
    );
    patch["signature"] = Value::String(sign_value(&signing_key(), &patch));
    let object = write_object_file(
        "object-verify-patch-wrong-derived-id-prefix",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'patch_id' must use 'patch:' prefix")
                })
            })),
        "expected patch_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_non_string_patch_id() {
    let mut patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": []
        }),
        "author",
        "patch_id",
        "patch",
    );
    patch["patch_id"] = json!(7);
    let object = write_object_file(
        "object-verify-patch-non-string-derived-id",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'patch_id' must be a string")
                })
            })),
        "expected patch_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_unknown_top_level_field() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [],
            "unexpected": true
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-unknown-top-level-field",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level contains unexpected field 'unexpected'")
                })
            })),
        "expected unknown top-level field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_missing_ops() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file("object-verify-patch-missing-ops", "patch.json", patch);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing array field 'ops'"))
            })),
        "expected missing ops error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_non_array_ops() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": {}
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file("object-verify-patch-non-array-ops", "patch.json", patch);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'ops' must be an array"))
            })),
        "expected ops array-shape error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_move_without_destination() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "move_block",
                    "block_id": "blk:001"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-move-without-destination",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'ops[0]': move_block requires at least one destination reference",
                    )
                })
            })),
        "expected move_block destination error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_mixed_set_metadata_forms() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "metadata": {
                        "title": "Hello"
                    },
                    "key": "extra"
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-mixed-set-metadata-forms",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'ops[0]': patch op contains unexpected field 'key'")
                })
            })),
        "expected mixed set_metadata forms error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_patch_with_empty_metadata_entries() {
    let patch = signed_object(
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "timestamp": 1777778888u64,
            "ops": [
                {
                    "op": "set_metadata",
                    "metadata": {}
                }
            ]
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object = write_object_file(
        "object-verify-patch-empty-metadata-entries",
        "patch.json",
        patch,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "patch");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'ops[0]': top-level 'metadata' must not be empty")
                })
            })),
        "expected empty metadata error, stdout: {}",
        stdout_text(&output)
    );
}
