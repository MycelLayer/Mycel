use super::*;

#[test]
fn sync_pull_json_rejects_unknown_sender() {
    let signing_key = signing_key();
    let registered_sender = "node:alpha";
    let actual_sender = "node:beta";
    let transcript_dir = create_temp_dir("sync-pull-unknown-sender");
    let transcript_path = transcript_dir.path().join("unknown-sender-transcript.json");
    let store_root = create_temp_dir("sync-pull-unknown-sender-store");
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": registered_sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                signed_hello_message(&signing_key, actual_sender)
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
fn sync_pull_json_rejects_hello_node_id_mismatch() {
    let signing_key = signing_key();
    let sender = "node:alpha";
    let transcript_dir = create_temp_dir("sync-pull-hello-node-id-mismatch");
    let transcript_path = transcript_dir
        .path()
        .join("hello-node-id-mismatch-transcript.json");
    let store_root = create_temp_dir("sync-pull-hello-node-id-mismatch-store");
    let mut hello = signed_hello_message(&signing_key, sender);
    hello["payload"]["node_id"] = Value::String("node:beta".to_string());
    hello["sig"] = Value::String(sign_wire_value(&signing_key, &hello));
    write_transcript(
        &transcript_path,
        &json!({
            "peer": {
                "node_id": sender,
                "public_key": sender_public_key(&signing_key)
            },
            "messages": [
                hello
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
                    message.contains("wire HELLO payload 'node_id' must equal envelope 'from'")
                })
            })),
        "expected HELLO node_id mismatch error, stdout: {}",
        stdout_text(&output)
    );
    assert!(!store_root
        .path()
        .join("indexes")
        .join("manifest.json")
        .exists());
}
