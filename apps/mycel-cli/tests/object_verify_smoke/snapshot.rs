use super::*;

#[test]
fn object_verify_json_fails_for_mismatched_snapshot_id() {
    let mut snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    snapshot["snapshot_id"] = Value::String("snap:wrong".to_string());
    snapshot["signature"] = Value::String(sign_value(&signing_key(), &snapshot));
    let object = write_object_file("object-verify-snapshot-mismatch", "snapshot.json", snapshot);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("declared snapshot_id does not match")))),
        "expected snapshot derived ID mismatch error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_wrong_snapshot_id_prefix() {
    let mut snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    snapshot["snapshot_id"] = Value::String(
        snapshot["snapshot_id"]
            .as_str()
            .expect("snapshot_id should exist")
            .replacen("snap:", "view:", 1),
    );
    snapshot["signature"] = Value::String(sign_value(&signing_key(), &snapshot));
    let object = write_object_file(
        "object-verify-snapshot-wrong-derived-id-prefix",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'snapshot_id' must use 'snap:' prefix")
                })
            })),
        "expected snapshot_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_duplicate_snapshot_included_objects() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "rev:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-duplicate-included-objects",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'included_objects[1]' duplicates 'included_objects[0]'",
                    )
                })
            })),
        "expected duplicate included_objects error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_empty_included_object_entry() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", ""],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-empty-included-object-entry",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'included_objects[1]' must not be an empty string")
                })
            })),
        "expected empty included_objects entry error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_non_canonical_included_object_id() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["doc:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-non-canonical-included-object-id",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'included_objects[0]' must use a canonical object ID prefix",
                    )
                })
            })),
        "expected canonical included_objects ID error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_missing_documents() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-missing-documents",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing object field 'documents'"))
            })),
        "expected missing documents error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_non_object_documents() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": [],
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-non-object-documents",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents' must be an object")
                })
            })),
        "expected documents object-shape error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_missing_included_objects() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-missing-included-objects",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("missing array field 'included_objects'")
                })
            })),
        "expected missing included_objects error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_missing_root_hash() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test"],
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-missing-root-hash",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing string field 'root_hash'"))
            })),
        "expected missing root_hash error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_empty_documents() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {},
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-empty-documents",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents' must not be empty")
                })
            })),
        "expected empty documents error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_missing_declared_revision_in_included_objects() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-missing-declared-revision",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'included_objects' must include revision 'rev:test' declared by 'documents.doc:test'",
                    )
                })
            })),
        "expected missing declared revision error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_wrong_root_hash_prefix() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "rev:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-wrong-root-hash-prefix",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'root_hash' must use 'hash:' prefix")
                })
            })),
        "expected root_hash prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_wrong_document_value_prefix() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "patch:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-wrong-document-value-prefix",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents.doc:test' must use 'rev:' prefix")
                })
            })),
        "expected snapshot document revision-prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_wrong_document_key_prefix() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "patch:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-wrong-document-key-prefix",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents.patch:test key' must use 'doc:' prefix")
                })
            })),
        "expected snapshot document key-prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_wrong_created_by_prefix() {
    let mut snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    snapshot["created_by"] = Value::String("sig:bad".to_string());
    let object = write_object_file(
        "object-verify-snapshot-wrong-created-by-prefix",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("signer field must use format 'pk:ed25519:<base64>'")
                })
            })),
        "expected created_by signer-format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_malformed_created_by_key() {
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": "pk:ed25519:not-base64",
        "timestamp": 1777778890u64
    });
    snapshot["signature"] = Value::String(sign_value(&signing_key(), &snapshot));
    let object = write_object_file(
        "object-verify-snapshot-malformed-created-by-key",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
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
fn object_verify_json_fails_for_snapshot_missing_created_by() {
    let mut snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    snapshot
        .as_object_mut()
        .expect("snapshot should be an object")
        .remove("created_by");
    let object = write_object_file(
        "object-verify-snapshot-missing-created-by",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("snapshot object is missing string signer field 'created_by'")
                })
            })),
        "expected missing created_by signer-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_invalid_created_by_key_bytes() {
    let object = write_object_file(
        "object-verify-snapshot-invalid-created-by-bytes",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:placeholder",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:AA==",
            "timestamp": 1777778890u64,
            "signature": "sig:ed25519:AA=="
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
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
fn object_verify_json_fails_for_snapshot_missing_signature() {
    let object = write_object_file(
        "object-verify-snapshot-missing-signature",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:placeholder",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": signer_id(&signing_key()),
            "timestamp": 1777778890u64
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("snapshot object is missing required top-level 'signature'")
                })
            })),
        "expected missing signature error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_missing_timestamp() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test"
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-missing-timestamp",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
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
fn object_verify_json_fails_for_snapshot_with_negative_timestamp() {
    let mut snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    snapshot["timestamp"] = json!(-1);
    snapshot["signature"] = Value::String(sign_value(&signing_key(), &snapshot));
    let object = write_object_file(
        "object-verify-snapshot-negative-timestamp",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
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
fn object_verify_json_fails_for_invalid_snapshot_signature() {
    let mut snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 1777778890u64
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    snapshot["signature"] = Value::String(
        "sig:ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
            .to_string(),
    );
    let object = write_object_file(
        "object-verify-snapshot-bad-signature",
        "snapshot.json",
        snapshot,
    );
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
fn object_verify_json_fails_for_snapshot_with_wrong_signature_format() {
    let object = write_object_file(
        "object-verify-snapshot-wrong-signature-format",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:placeholder",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": signer_id(&signing_key()),
            "timestamp": 1777778890u64,
            "signature": "sig:bad"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
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
fn object_verify_json_fails_for_snapshot_with_malformed_signature_bytes() {
    let object = write_object_file(
        "object-verify-snapshot-malformed-signature",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:placeholder",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": signer_id(&signing_key()),
            "timestamp": 1777778890u64,
            "signature": "sig:ed25519:not-base64"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
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
fn object_verify_json_fails_for_snapshot_with_invalid_signature_bytes() {
    let object = write_object_file(
        "object-verify-snapshot-invalid-signature-bytes",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:placeholder",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": signer_id(&signing_key()),
            "timestamp": 1777778890u64,
            "signature": "sig:ed25519:AA=="
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
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
fn object_verify_json_fails_for_snapshot_with_non_string_signature() {
    let object = write_object_file(
        "object-verify-snapshot-non-string-signature",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:placeholder",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": signer_id(&signing_key()),
            "timestamp": 1777778890u64,
            "signature": 7
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
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
fn object_verify_json_fails_for_snapshot_with_non_string_snapshot_id() {
    let mut snapshot = json!({
        "type": "snapshot",
        "version": "mycel/0.1",
        "documents": {
            "doc:test": "rev:test"
        },
        "included_objects": ["rev:test", "patch:test"],
        "root_hash": "hash:test",
        "created_by": signer_id(&signing_key()),
        "timestamp": 9u64,
        "snapshot_id": 7
    });
    snapshot["signature"] = Value::String(sign_value(&signing_key(), &snapshot));
    let object = write_object_file(
        "object-verify-snapshot-non-string-id",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'snapshot_id' must be a string")
                })
            })),
        "expected snapshot_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_snapshot_with_unknown_top_level_field() {
    let snapshot = signed_object(
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "timestamp": 9u64,
            "unexpected": true
        }),
        "created_by",
        "snapshot_id",
        "snap",
    );
    let object = write_object_file(
        "object-verify-snapshot-unknown-top-level-field",
        "snapshot.json",
        snapshot,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "snapshot");
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
