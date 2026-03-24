use super::*;

#[path = "negative/verification.rs"]
mod verification;

#[path = "negative/session.rs"]
mod session;

#[path = "negative/heads_replace.rs"]
mod heads_replace;

#[path = "negative/capability.rs"]
mod capability;

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
