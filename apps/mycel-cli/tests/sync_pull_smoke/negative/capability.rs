use super::*;

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
