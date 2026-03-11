use super::*;

#[test]
fn object_inspect_json_warns_for_snapshot_with_wrong_document_value_prefix() {
    let object = write_object_file(
        "object-inspect-snapshot-wrong-document-value-prefix",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "patch:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(|message| message
                    .contains("top-level 'documents.doc:test' must use 'rev:' prefix")))),
        "expected snapshot documents value-prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_missing_declared_revision() {
    let object = write_object_file(
        "object-inspect-snapshot-missing-declared-revision",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes.iter().any(|entry| {
            entry.as_str().is_some_and(|message| {
                message.contains(
                    "top-level 'included_objects' must include revision 'rev:test' declared by 'documents.doc:test'",
                )
            })
        })),
        "expected missing declared revision warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_wrong_document_key_prefix() {
    let object = write_object_file(
        "object-inspect-snapshot-wrong-document-key-prefix",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "patch:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"].as_array().is_some_and(|notes| notes
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("top-level 'documents.patch:test key' must use 'doc:' prefix")))),
        "expected document key prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_missing_documents() {
    let object = write_object_file(
        "object-inspect-snapshot-missing-documents",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "included_objects": ["rev:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing object field 'documents'"))
            })),
        "expected missing documents warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_unknown_top_level_field() {
    let object = write_object_file(
        "object-inspect-snapshot-unknown-top-level-field",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "unexpected": true,
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
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
fn object_inspect_json_warns_for_snapshot_with_duplicate_included_objects() {
    let object = write_object_file(
        "object-inspect-snapshot-duplicate-included-objects",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "rev:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'included_objects[1]' duplicates 'included_objects[0]'",
                    )
                })
            })),
        "expected duplicate included_objects warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_empty_included_object_entry() {
    let object = write_object_file(
        "object-inspect-snapshot-empty-included-object-entry",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", ""],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'included_objects[1]' must not be an empty string")
                })
            })),
        "expected empty included_objects entry warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_non_canonical_included_object_id() {
    let object = write_object_file(
        "object-inspect-snapshot-non-canonical-included-object-id",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["doc:test"],
            "root_hash": "hash:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains(
                        "top-level 'included_objects[0]' must use a canonical object ID prefix",
                    )
                })
            })),
        "expected canonical included_objects warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_wrong_root_hash_prefix() {
    let object = write_object_file(
        "object-inspect-snapshot-wrong-root-hash-prefix",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "rev:test",
            "created_by": "pk:ed25519:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'root_hash' must use 'hash:' prefix")
                ))),
        "expected root_hash prefix warning, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_inspect_json_warns_for_snapshot_with_wrong_created_by_prefix() {
    let object = write_object_file(
        "object-inspect-snapshot-wrong-created-by-prefix",
        "snapshot.json",
        json!({
            "type": "snapshot",
            "version": "mycel/0.1",
            "snapshot_id": "snap:test",
            "documents": {
                "doc:test": "rev:test"
            },
            "included_objects": ["rev:test", "patch:test"],
            "root_hash": "hash:test",
            "created_by": "creator:test",
            "timestamp": 9u64,
            "signature": "sig:ed25519:test"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "inspect", &path, "--json"]);

    assert_success(&output);
    let json = assert_json_status(&output, "warning");
    assert_eq!(json["object_type"], "snapshot");
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|entry| entry.as_str().is_some_and(
                    |message| message.contains("top-level 'created_by' must use 'pk:' prefix")
                ))),
        "expected created_by prefix warning, stdout: {}",
        stdout_text(&output)
    );
}
