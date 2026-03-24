use super::*;

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
fn sync_pull_json_rejects_stale_object_want_after_heads_replace() {
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
    let transcript_dir = create_temp_dir("sync-pull-stale-object-want-after-heads-replace");
    let transcript_path = transcript_dir
        .path()
        .join("stale-object-want-after-heads-replace-transcript.json");
    let store_root = create_temp_dir("sync-pull-stale-object-want-after-heads-replace-store");
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
                signed_heads_message(&signing_key, sender, "rev:replacement", true),
                signed_want_message(&signing_key, sender, &[&patch_id]),
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
            .is_some_and(|count| count >= 5),
        "expected replacement HEADS to verify before stale object WANT rejection, stdout: {}",
        stdout_text(&output)
    );
    assert_eq!(json["object_message_count"], 1);
    assert_eq!(json["written_object_count"], 0);
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|error| {
                error.as_str().is_some_and(|message| {
                    message.contains("wire WANT object")
                        && message.contains(&patch_id)
                        && message.contains("is not reachable from accepted sync roots")
                })
            })),
        "expected stale object WANT error after HEADS replace, stdout: {}",
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
