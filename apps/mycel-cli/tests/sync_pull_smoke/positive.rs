use super::*;

struct RevisionStep {
    patch_object: Value,
    patch_id: String,
    revision_object: Value,
    revision_id: String,
}

fn make_revision_step(
    signing_key: &SigningKey,
    sender: &str,
    parent_revision_ids: &[&str],
    base_revision: &str,
) -> RevisionStep {
    let patch_object = signed_patch_object_message(signing_key, sender, base_revision);
    let patch_id = patch_object["payload"]["object_id"]
        .as_str()
        .expect("patch object id should exist")
        .to_string();
    let revision_object =
        signed_revision_object_message(signing_key, sender, parent_revision_ids, &[&patch_id]);
    let revision_id = revision_object["payload"]["object_id"]
        .as_str()
        .expect("revision object id should exist")
        .to_string();

    RevisionStep {
        patch_object,
        patch_id,
        revision_object,
        revision_id,
    }
}

fn write_existing_objects(store_root: &Path, objects: &[&Value]) {
    for body in objects {
        write_object_value_to_store(store_root, body)
            .expect("existing object should write to store");
    }
}

fn run_sync_pull_json(transcript_path: &Path, store_root: &Path) -> std::process::Output {
    run_mycel(&[
        "sync",
        "pull",
        &path_arg(transcript_path),
        "--into",
        &path_arg(store_root),
        "--json",
    ])
}

fn load_manifest(store_root: &Path) -> Value {
    let manifest_path = store_root.join("indexes").join("manifest.json");
    serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest should read"))
        .expect("manifest should parse")
}

fn assert_doc_revisions_include(manifest: &Value, expected_revision_ids: &[&str]) {
    let revisions = manifest["doc_revisions"]["doc:test"]
        .as_array()
        .expect("manifest[\"doc_revisions\"][\"doc:test\"] should be an array of revision IDs");
    assert_eq!(revisions.len(), expected_revision_ids.len());
    for expected_revision_id in expected_revision_ids {
        assert!(revisions
            .iter()
            .any(|value| value.as_str() == Some(*expected_revision_id)));
    }
}

#[test]
fn sync_pull_json_replays_incremental_transcript_into_existing_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";

    let base_step = make_revision_step(&signing_key, sender, &[], "rev:genesis-null");
    let follow_step = make_revision_step(
        &signing_key,
        sender,
        &[&base_step.revision_id],
        &base_step.revision_id,
    );

    let transcript_dir = create_temp_dir("sync-pull-incremental-source");
    let transcript_path = transcript_dir
        .path()
        .join("pull-incremental-transcript.json");
    let store_root = create_temp_dir("sync-pull-incremental-store");
    write_existing_objects(
        store_root.path(),
        &[
            &base_step.patch_object["payload"]["body"],
            &base_step.revision_object["payload"]["body"],
        ],
    );

    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &follow_step.revision_id),
                signed_want_message(&signing_key, sender, &[&follow_step.revision_id]),
                follow_step.revision_object.clone(),
                signed_want_message(&signing_key, sender, &[&follow_step.patch_id]),
                follow_step.patch_object.clone(),
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_sync_pull_json(&transcript_path, store_root.path());

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

    let manifest = load_manifest(store_root.path());
    assert_eq!(manifest["stored_object_count"], 4);
    assert_doc_revisions_include(
        &manifest,
        &[&base_step.revision_id, &follow_step.revision_id],
    );
}

#[test]
fn sync_pull_json_replays_depth_3_catchup_transcript_into_existing_store() {
    let signing_key = signing_key();
    let sender = "node:alpha";

    let genesis_step = make_revision_step(&signing_key, sender, &[], "rev:genesis-null");
    let middle_step = make_revision_step(
        &signing_key,
        sender,
        &[&genesis_step.revision_id],
        &genesis_step.revision_id,
    );
    let follow_step = make_revision_step(
        &signing_key,
        sender,
        &[&middle_step.revision_id],
        &middle_step.revision_id,
    );

    let transcript_dir = create_temp_dir("sync-pull-depth-3-source");
    let transcript_path = transcript_dir.path().join("pull-depth-3-transcript.json");
    let store_root = create_temp_dir("sync-pull-depth-3-store");
    write_existing_objects(
        store_root.path(),
        &[
            &genesis_step.patch_object["payload"]["body"],
            &genesis_step.revision_object["payload"]["body"],
            &middle_step.patch_object["payload"]["body"],
            &middle_step.revision_object["payload"]["body"],
        ],
    );

    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, sender),
                signed_manifest_message(&signing_key, sender, &follow_step.revision_id),
                signed_want_message(&signing_key, sender, &[&follow_step.revision_id]),
                follow_step.revision_object.clone(),
                signed_want_message(&signing_key, sender, &[&follow_step.patch_id]),
                follow_step.patch_object.clone(),
                signed_bye_message(&signing_key, sender)
            ]
        }),
    );

    let output = run_sync_pull_json(&transcript_path, store_root.path());

    assert_success(&output);
    let json = assert_json_status(&output, "ok");
    assert_eq!(json["peer_node_id"], sender);
    assert_eq!(json["message_count"], 7);
    assert_eq!(json["verified_message_count"], 7);
    assert_eq!(json["object_message_count"], 2);
    assert_eq!(json["verified_object_count"], 2);
    assert_eq!(json["written_object_count"], 2);
    assert_eq!(json["existing_object_count"], 0);

    let manifest = load_manifest(store_root.path());
    assert_eq!(manifest["stored_object_count"], 6);
    assert_doc_revisions_include(
        &manifest,
        &[
            &genesis_step.revision_id,
            &middle_step.revision_id,
            &follow_step.revision_id,
        ],
    );
}
