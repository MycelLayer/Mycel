use super::*;

#[test]
fn sync_pull_json_replays_incremental_transcript_into_existing_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";

    let base_patch_object = signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let base_patch_id = base_patch_object["payload"]["object_id"]
        .as_str()
        .expect("base patch object id should exist")
        .to_string();
    let base_revision_object =
        signed_revision_object_message(&signing_key, sender, &[], &[&base_patch_id]);
    let base_revision_id = base_revision_object["payload"]["object_id"]
        .as_str()
        .expect("base revision object id should exist")
        .to_string();

    let follow_patch_object = signed_patch_object_message(&signing_key, sender, &base_revision_id);
    let follow_patch_id = follow_patch_object["payload"]["object_id"]
        .as_str()
        .expect("follow patch object id should exist")
        .to_string();
    let follow_revision_object = signed_revision_object_message(
        &signing_key,
        sender,
        &[&base_revision_id],
        &[&follow_patch_id],
    );
    let follow_revision_id = follow_revision_object["payload"]["object_id"]
        .as_str()
        .expect("follow revision object id should exist")
        .to_string();

    let transcript_dir = create_temp_dir("sync-pull-incremental-source");
    let transcript_path = transcript_dir
        .path()
        .join("pull-incremental-transcript.json");
    let store_root = create_temp_dir("sync-pull-incremental-store");
    write_object_value_to_store(store_root.path(), &base_patch_object["payload"]["body"])
        .expect("base patch should write to store");
    write_object_value_to_store(store_root.path(), &base_revision_object["payload"]["body"])
        .expect("base revision should write to store");

    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &follow_revision_id),
                signed_want_message(&signing_key, sender, &[&follow_revision_id]),
                follow_revision_object,
                signed_want_message(&signing_key, sender, &[&follow_patch_id]),
                follow_patch_object,
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

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 7);
    assert_eq!(json["verified_message_count"], 7);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["verified_object_count"], 2);
    assert_eq!(json["written_object_count"], 2);
    assert_eq!(json["existing_object_count"], 0);
    assert!(
        json["notes"]
            .as_array()
            .is_some_and(|notes| notes.is_empty()),
        "expected no incremental sync warnings, stdout: {}",
        stdout_text(&output)
    );

    let manifest_path = store_root.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 4);
    let revisions = manifest["doc_revisions"]["doc:test"]
        .as_array()
        .expect("expected synced revision index array");
    assert_eq!(revisions.len(), 2);
    assert!(revisions
        .iter()
        .any(|value| value.as_str() == Some(base_revision_id.as_str())));
    assert!(revisions
        .iter()
        .any(|value| value.as_str() == Some(follow_revision_id.as_str())));
}

#[test]
fn sync_pull_json_replays_depth_3_catchup_transcript_into_existing_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";

    let genesis_patch_object =
        signed_patch_object_message(&signing_key, sender, "rev:genesis-null");
    let genesis_patch_id = genesis_patch_object["payload"]["object_id"]
        .as_str()
        .expect("genesis patch object id should exist")
        .to_string();
    let genesis_revision_object =
        signed_revision_object_message(&signing_key, sender, &[], &[&genesis_patch_id]);
    let genesis_revision_id = genesis_revision_object["payload"]["object_id"]
        .as_str()
        .expect("genesis revision object id should exist")
        .to_string();

    let middle_patch_object =
        signed_patch_object_message(&signing_key, sender, &genesis_revision_id);
    let middle_patch_id = middle_patch_object["payload"]["object_id"]
        .as_str()
        .expect("middle patch object id should exist")
        .to_string();
    let middle_revision_object = signed_revision_object_message(
        &signing_key,
        sender,
        &[&genesis_revision_id],
        &[&middle_patch_id],
    );
    let middle_revision_id = middle_revision_object["payload"]["object_id"]
        .as_str()
        .expect("middle revision object id should exist")
        .to_string();

    let follow_patch_object =
        signed_patch_object_message(&signing_key, sender, &middle_revision_id);
    let follow_patch_id = follow_patch_object["payload"]["object_id"]
        .as_str()
        .expect("follow patch object id should exist")
        .to_string();
    let follow_revision_object = signed_revision_object_message(
        &signing_key,
        sender,
        &[&middle_revision_id],
        &[&follow_patch_id],
    );
    let follow_revision_id = follow_revision_object["payload"]["object_id"]
        .as_str()
        .expect("follow revision object id should exist")
        .to_string();

    let transcript_dir = create_temp_dir("sync-pull-depth-3-source");
    let transcript_path = transcript_dir.path().join("pull-depth-3-transcript.json");
    let store_root = create_temp_dir("sync-pull-depth-3-store");
    for body in [
        &genesis_patch_object["payload"]["body"],
        &genesis_revision_object["payload"]["body"],
        &middle_patch_object["payload"]["body"],
        &middle_revision_object["payload"]["body"],
    ] {
        write_object_value_to_store(store_root.path(), body)
            .expect("existing object should write to store");
    }

    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &follow_revision_id),
                signed_want_message(&signing_key, sender, &[&follow_revision_id]),
                follow_revision_object,
                signed_want_message(&signing_key, sender, &[&follow_patch_id]),
                follow_patch_object,
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

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 7);
    assert_eq!(json["verified_message_count"], 7);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["verified_object_count"], 2);
    assert_eq!(json["written_object_count"], 2);
    assert_eq!(json["existing_object_count"], 0);

    let manifest_path = store_root.path().join("indexes").join("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
            .expect("manifest should parse");
    assert_eq!(manifest["stored_object_count"], 6);
    let revisions = manifest["doc_revisions"]["doc:test"]
        .as_array()
        .expect("expected synced revision index array");
    assert_eq!(revisions.len(), 3);
    assert!(revisions
        .iter()
        .any(|value| value.as_str() == Some(genesis_revision_id.as_str())));
    assert!(revisions
        .iter()
        .any(|value| value.as_str() == Some(middle_revision_id.as_str())));
    assert!(revisions
        .iter()
        .any(|value| value.as_str() == Some(follow_revision_id.as_str())));
}
