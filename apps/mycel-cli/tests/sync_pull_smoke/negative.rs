use super::*;

#[path = "negative/verification.rs"]
mod verification;

#[path = "negative/session.rs"]
mod session;

#[test]
fn sync_pull_json_rejects_unknown_sender_hello() {
    let expected_peer = "node:alpha";
    let unexpected_sender = "node:beta";
    let expected_signing_key = signing_key();
    let unexpected_signing_key = SigningKey::from_bytes(&[19u8; 32]);
    let transcript_dir = create_temp_dir("sync-pull-unknown-sender-hello");
    let transcript_path = transcript_dir
        .path()
        .join("unknown-sender-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-unknown-sender-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": expected_peer,
                "public_key": sender_public_key(&expected_signing_key)
            },
            "messages": [
                signed_hello_message(&unexpected_signing_key, unexpected_sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["peer_node_id"], expected_peer);
    assert_eq!(json["message_count"], 1);
    assert_eq!(json["verified_message_count"], 0);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("unknown wire sender 'node:beta'"))
            })),
        "expected unknown-sender error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_hello_sender_identity_mismatch() {
    let sender = "node:alpha";
    let mismatched_node_id = "node:beta";
    let signing_key = signing_key();
    let transcript_dir = create_temp_dir("sync-pull-hello-node-id-mismatch");
    let transcript_path = transcript_dir
        .path()
        .join("hello-node-id-mismatch-transcript.json");
    let store_root = create_temp_dir("sync-pull-hello-node-id-mismatch-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message_with_node_id(&signing_key, sender, mismatched_node_id)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 1);
    assert_eq!(json["verified_message_count"], 0);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire HELLO payload 'node_id' must equal envelope 'from'")
                })
            })),
        "expected HELLO sender-identity mismatch error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_want_before_manifest_or_heads() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-want-before-manifest");
    let transcript_path = transcript_dir
        .path()
        .join("want-before-manifest-transcript.json");
    let store_root = create_temp_dir("sync-pull-want-before-manifest-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_want_message(&signing_key, sender, &["patch:test"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 1);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT requires prior MANIFEST or HEADS")
                })
            })),
        "expected WANT-before-head-context error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unreachable_want_revision() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-unreachable-want-revision");
    let transcript_path = transcript_dir
        .path()
        .join("unreachable-want-revision-transcript.json");
    let store_root = create_temp_dir("sync-pull-unreachable-want-revision-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:test"),
                signed_want_message(&signing_key, sender, &["rev:missing"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(
                        "wire WANT revision 'rev:missing' is not reachable from accepted sync roots for 'node:alpha'",
                    )
                })
            })),
        "expected unreachable-WANT-revision error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unreachable_want_object() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-unreachable-want-object");
    let transcript_path = transcript_dir
        .path()
        .join("unreachable-want-object-transcript.json");
    let store_root = create_temp_dir("sync-pull-unreachable-want-object-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:test"),
                signed_want_message(&signing_key, sender, &["patch:test"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(
                        "wire WANT object 'patch:test' is not reachable from accepted sync roots for 'node:alpha'",
                    )
                })
            })),
        "expected unreachable-WANT-object error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unrequested_object_before_manifest_or_heads() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let object_id = object["payload"]["object_id"]
        .as_str()
        .expect("signed OBJECT payload should include object_id")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-unrequested-object-before-manifest");
    let transcript_path = transcript_dir
        .path()
        .join("unrequested-object-before-manifest-transcript.json");
    let store_root = create_temp_dir("sync-pull-unrequested-object-before-manifest-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                object
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 1);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(&format!(
                        "wire OBJECT '{object_id}' was not requested from '{sender}'"
                    ))
                })
            })),
        "expected unrequested-object error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_allows_error_before_hello_but_still_requires_sync_messages() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-error-before-hello-then-hello");
    let transcript_path = transcript_dir
        .path()
        .join("error-before-hello-then-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-error-before-hello-then-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_error_message(&signing_key, sender, "msg:missing-hello"),
                signed_hello_message(&signing_key, sender),
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 3);
    assert_eq!(json["verified_message_count"], 3);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("did not include MANIFEST or HEADS"))
            })),
        "expected missing MANIFEST/HEADS error, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("did not include any OBJECT messages"))
            })),
        "expected missing OBJECT error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_reports_explicit_error_only_transcript_as_failed_sync() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-error-only");
    let transcript_path = transcript_dir.path().join("error-only-transcript.json");
    let store_root = create_temp_dir("sync-pull-error-only-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_error_message(&signing_key, sender, "msg:missing-hello")
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 1);
    assert_eq!(json["verified_message_count"], 1);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("did not include HELLO from 'node:alpha'")
                })
            })),
        "expected missing-HELLO error after ERROR-only transcript, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("did not include MANIFEST or HEADS"))
            })),
        "expected missing-MANIFEST/HEADS error after ERROR-only transcript, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error
                    .as_str()
                    .is_some_and(|message| message.contains("did not include any OBJECT messages"))
            })),
        "expected missing-OBJECT error after ERROR-only transcript, stdout: {}",
        stdout_text(&output)
    );
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes
                .iter()
                .any(|note| note.as_str().is_some_and(|message| message
                    .contains("sync transcript ended without BYE from 'node:alpha'")))),
        "expected missing-BYE note after ERROR-only transcript, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_unrequested_object_message() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-unrequested-object");
    let transcript_path = transcript_dir
        .path()
        .join("unrequested-object-transcript.json");
    let store_root = create_temp_dir("sync-pull-unrequested-object-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:test"),
                patch_object
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(&format!(
                        "wire OBJECT '{patch_id}' was not requested from '{sender}'"
                    ))
                })
            })),
        "expected unrequested-object error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_stale_dependency_object_after_heads_replace() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-stale-dependency-object-after-heads-replace");
    let transcript_path = transcript_dir
        .path()
        .join("stale-dependency-object-after-heads-replace-transcript.json");
    let store_root = create_temp_dir("sync-pull-stale-dependency-object-after-heads-replace-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                revision_object,
                signed_want_message(&signing_key, sender, &[&patch_id]),
                signed_heads_message(&signing_key, sender, "rev:replacement", true),
                patch_object,
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert!(
        json["verified_message_count"]
            .as_u64()
            .is_some_and(|count| count >= 6),
        "expected replacement HEADS to verify before stale dependency OBJECT rejection, stdout: {}",
        stdout_text(&output)
    );
    assert_eq!(json["object_message_count"], 1);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(&format!(
                        "wire OBJECT '{patch_id}' was not requested from '{sender}'"
                    ))
                })
            })),
        "expected stale dependency OBJECT error after HEADS replace, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_stale_root_want_after_heads_replace() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-stale-root-want-after-heads-replace");
    let transcript_path = transcript_dir
        .path()
        .join("stale-root-want-after-heads-replace-transcript.json");
    let store_root = create_temp_dir("sync-pull-stale-root-want-after-heads-replace-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                signed_heads_message(&signing_key, sender, "rev:replacement", true),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert!(
        json["verified_message_count"]
            .as_u64()
            .is_some_and(|count| count >= 4),
        "expected replacement HEADS to verify before stale root WANT rejection, stdout: {}",
        stdout_text(&output)
    );
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT revision")
                        && message.contains(&revision_id)
                        && message.contains("is not reachable from accepted sync roots")
                })
            })),
        "expected stale root WANT error after HEADS replace, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_stale_root_object_after_heads_replace() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-stale-root-object-after-heads-replace");
    let transcript_path = transcript_dir
        .path()
        .join("stale-root-object-after-heads-replace-transcript.json");
    let store_root = create_temp_dir("sync-pull-stale-root-object-after-heads-replace-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                signed_heads_message(&signing_key, sender, "rev:replacement", true),
                revision_object,
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert!(
        json["verified_message_count"]
            .as_u64()
            .is_some_and(|count| count >= 4),
        "expected replacement HEADS to verify before stale root OBJECT rejection, stdout: {}",
        stdout_text(&output)
    );
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(&format!(
                        "wire OBJECT '{revision_id}' was not requested from '{sender}'"
                    ))
                })
            })),
        "expected stale root OBJECT error after HEADS replace, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_stale_view_want_after_heads_replace() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-stale-view-want-after-heads-replace");
    let transcript_path = transcript_dir
        .path()
        .join("stale-view-want-after-heads-replace-transcript.json");
    let store_root = create_temp_dir("sync-pull-stale-view-want-after-heads-replace-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "view-sync"])
                ),
                signed_manifest_message_with_capabilities(
                    &signing_key,
                    sender,
                    "rev:test",
                    json!(["patch-sync", "view-sync"])
                ),
                signed_view_announce_message(&signing_key, sender, "view:test-announce"),
                signed_heads_message(&signing_key, sender, "rev:replacement", true),
                signed_want_message(&signing_key, sender, &["view:test-announce"]),
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert!(
        json["verified_message_count"]
            .as_u64()
            .is_some_and(|count| count >= 4),
        "expected replacement HEADS to verify before stale view WANT rejection, stdout: {}",
        stdout_text(&output)
    );
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT object")
                        && message.contains("view:test-announce")
                        && message.contains("is not reachable from accepted sync roots")
                })
            })),
        "expected stale view WANT error after HEADS replace, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_stale_snapshot_want_after_heads_replace() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-stale-snapshot-want-after-heads-replace");
    let transcript_path = transcript_dir
        .path()
        .join("stale-snapshot-want-after-heads-replace-transcript.json");
    let store_root = create_temp_dir("sync-pull-stale-snapshot-want-after-heads-replace-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "snapshot-sync"])
                ),
                signed_manifest_message_with_capabilities(
                    &signing_key,
                    sender,
                    "rev:test",
                    json!(["patch-sync", "snapshot-sync"])
                ),
                signed_snapshot_offer_message(&signing_key, sender, "snap:test-offer"),
                signed_heads_message(&signing_key, sender, "rev:replacement", true),
                signed_want_message(&signing_key, sender, &["snap:test-offer"]),
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert!(
        json["verified_message_count"]
            .as_u64()
            .is_some_and(|count| count >= 4),
        "expected replacement HEADS to verify before stale snapshot WANT rejection, stdout: {}",
        stdout_text(&output)
    );
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT object")
                        && message.contains("snap:test-offer")
                        && message.contains("is not reachable from accepted sync roots")
                })
            })),
        "expected stale snapshot WANT error after HEADS replace, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_snapshot_offer_without_advertised_capability() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-snapshot-offer-without-capability");
    let transcript_path = transcript_dir
        .path()
        .join("snapshot-offer-without-capability-transcript.json");
    let store_root = create_temp_dir("sync-pull-snapshot-offer-without-capability-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:test"),
                signed_snapshot_offer_message(&signing_key, sender, "snap:test-offer")
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains(
                        "wire SNAPSHOT_OFFER requires advertised capability 'snapshot-sync'",
                    )
                })
            })),
        "expected snapshot capability error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_snapshot_offer_before_hello() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-snapshot-offer-before-hello");
    let transcript_path = transcript_dir
        .path()
        .join("snapshot-offer-before-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-snapshot-offer-before-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_snapshot_offer_message(&signing_key, sender, "snap:test-offer"),
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "snapshot-sync"])
                )
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 0);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire SNAPSHOT_OFFER requires prior HELLO from 'node:alpha'")
                })
            })),
        "expected prior-HELLO SNAPSHOT_OFFER error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_view_announce_without_advertised_capability() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-view-announce-without-capability");
    let transcript_path = transcript_dir
        .path()
        .join("view-announce-without-capability-transcript.json");
    let store_root = create_temp_dir("sync-pull-view-announce-without-capability-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, "rev:test"),
                signed_view_announce_message(&signing_key, sender, "view:test-announce")
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message
                        .contains("wire VIEW_ANNOUNCE requires advertised capability 'view-sync'")
                })
            })),
        "expected view capability error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_rejects_view_announce_before_hello() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-view-announce-before-hello");
    let transcript_path = transcript_dir
        .path()
        .join("view-announce-before-hello-transcript.json");
    let store_root = create_temp_dir("sync-pull-view-announce-before-hello-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_view_announce_message(&signing_key, sender, "view:test-announce"),
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "view-sync"])
                )
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 0);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire VIEW_ANNOUNCE requires prior HELLO from 'node:alpha'")
                })
            })),
        "expected prior-HELLO VIEW_ANNOUNCE error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_snapshot_offer_before_manifest_does_not_unlock_want() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-snapshot-offer-before-manifest");
    let transcript_path = transcript_dir
        .path()
        .join("snapshot-offer-before-manifest-transcript.json");
    let store_root = create_temp_dir("sync-pull-snapshot-offer-before-manifest-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "snapshot-sync"])
                ),
                signed_snapshot_offer_message(&signing_key, sender, "snap:test-offer"),
                signed_want_message(&signing_key, sender, &["snap:test-offer"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT requires prior MANIFEST or HEADS")
                })
            })),
        "expected WANT-before-manifest error after SNAPSHOT_OFFER, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_json_view_announce_before_manifest_does_not_unlock_want() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-view-announce-before-manifest");
    let transcript_path = transcript_dir
        .path()
        .join("view-announce-before-manifest-transcript.json");
    let store_root = create_temp_dir("sync-pull-view-announce-before-manifest-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message_with_capabilities(
                    &signing_key,
                    sender,
                    json!(["patch-sync", "view-sync"])
                ),
                signed_view_announce_message(&signing_key, sender, "view:test-announce"),
                signed_want_message(&signing_key, sender, &["view:test-announce"])
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
        "--json",
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["verified_message_count"], 2);
    assert_eq!(json["object_message_count"], 0);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT requires prior MANIFEST or HEADS")
                })
            })),
        "expected WANT-before-manifest error after VIEW_ANNOUNCE, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}

#[test]
fn sync_pull_text_reports_pending_requested_object_failure() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object = signed_revision_object_message(&signing_key, sender, &[], &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();
    let transcript_dir = create_temp_dir("sync-pull-pending");
    let transcript_path = transcript_dir.path().join("pending-transcript.json");
    let store_root = create_temp_dir("sync-pull-pending-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &revision_id),
                signed_want_message(&signing_key, sender, &[&revision_id]),
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_mycel(&[
        "sync",
        "pull",
        &path_arg(&transcript_path),
        "--into",
        &path_arg(store_root.path()),
    ]);

    assert!(
        !output.status.success(),
        "expected failure, stdout: {}, stderr: {}",
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_stderr_contains(
        &output,
        "sync transcript did not include any OBJECT messages",
    );
    assert_stderr_contains(
        &output,
        "sync transcript ended with 1 pending requested object(s)",
    );
    let stdout = stdout_text(&output);
    assert!(stdout.contains("sync pull: failed"), "stdout: {stdout}");
    assert!(stdout.contains("verified messages: 4"), "stdout: {stdout}");
    assert!(stdout.contains("object messages: 0"), "stdout: {stdout}");
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}
