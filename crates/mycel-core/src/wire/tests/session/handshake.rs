use super::*;

fn session_ready_for_advertised_object(
    signing_key: &ed25519_dalek::SigningKey,
    capability: &str,
    advertisement: &serde_json::Value,
    object_id: &str,
) -> WireSession {
    let sender_key = sender_public_key(signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let capabilities = json!(["patch-sync", capability]);
    let hello = signed_hello_message_with_capabilities(
        signing_key,
        "node:alpha",
        "node:alpha",
        capabilities.clone(),
    );
    let manifest = signed_manifest_message_with_capabilities(
        signing_key,
        "node:alpha",
        "node:alpha",
        capabilities,
    );
    let want = signed_want_message(signing_key, "node:alpha", &[object_id]);

    for message in [&hello, &manifest, advertisement, &want] {
        session
            .verify_incoming(message)
            .expect("session setup message should verify");
    }

    session
}

#[test]
fn wire_session_verifies_incoming_hello_from_registered_peer() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let value = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

    let envelope = session
        .verify_incoming(&value)
        .expect("registered sender should verify");

    assert_eq!(envelope.from(), "node:alpha");
    assert_eq!(envelope.message_type(), WireMessageType::Hello);
}

#[test]
fn wire_session_rejects_unknown_sender() {
    let signing_key = signing_key();
    let value = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

    let error = WireSession::default().verify_incoming(&value).unwrap_err();

    assert_eq!(error, "unknown wire sender 'node:alpha'");
}

#[test]
fn wire_session_rejects_hello_node_id_mismatch() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::new(WirePeerDirectory::new());
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let value = signed_hello_message(&signing_key, "node:alpha", "node:beta");

    let error = session.verify_incoming(&value).unwrap_err();

    assert_eq!(
        error,
        "wire HELLO payload 'node_id' must equal envelope 'from'"
    );
}

#[test]
fn wire_session_rejects_manifest_before_hello() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let value = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");

    let error = session.verify_incoming(&value).unwrap_err();

    assert_eq!(
        error,
        "wire MANIFEST requires prior HELLO from 'node:alpha'"
    );
}

#[test]
fn wire_session_rejects_snapshot_offer_without_snapshot_capability() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let snapshot_offer =
        signed_snapshot_offer_message(&signing_key, "node:alpha", "snap:test-offer");

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    let error = session.verify_incoming(&snapshot_offer).unwrap_err();

    assert_eq!(
        error,
        "wire SNAPSHOT_OFFER requires advertised capability 'snapshot-sync' from 'node:alpha'"
    );
}

#[test]
fn wire_session_accepts_snapshot_offer_with_matching_snapshot_object() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message_with_capabilities(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!(["patch-sync", "snapshot-sync"]),
    );
    let manifest = signed_manifest_message_with_capabilities(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!(["patch-sync", "snapshot-sync"]),
    );
    let snapshot_object = signed_snapshot_object_message(
        &signing_key,
        "node:alpha",
        "rev:test",
        "hash:snapshot-root",
    );
    let snapshot_id = snapshot_object["payload"]["object_id"]
        .as_str()
        .expect("snapshot OBJECT should include object_id")
        .to_string();
    let snapshot_offer = signed_snapshot_offer_message(&signing_key, "node:alpha", &snapshot_id);
    let want = signed_want_message(&signing_key, "node:alpha", &[&snapshot_id]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&snapshot_offer)
        .expect("SNAPSHOT_OFFER should verify");
    session
        .verify_incoming(&want)
        .expect("snapshot WANT should verify after offer");
    session
        .verify_incoming(&snapshot_object)
        .expect("matching snapshot OBJECT should verify after offer");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state.reachable_object_ids.contains(&snapshot_id));
    assert!(state.pending_object_ids.is_empty());
    assert!(state.snapshot_offers.is_empty());
}

#[rstest::rstest]
#[case::snapshot_root_hash("snapshot", "root_hash")]
#[case::view_maintainer("view", "maintainer")]
#[case::view_documents("view", "documents")]
fn wire_session_rejects_object_with_mismatched_advertisement(
    #[case] object_type: &str,
    #[case] mismatched_field: &str,
) {
    let signing_key = signing_key();
    let (capability, object, advertisement, advertisement_type) =
        match (object_type, mismatched_field) {
            ("snapshot", "root_hash") => {
                let object = signed_snapshot_object_message(
                    &signing_key,
                    "node:alpha",
                    "rev:test",
                    "hash:unexpected-root",
                );
                let object_id = object["payload"]["object_id"]
                    .as_str()
                    .expect("snapshot OBJECT should include object_id");
                let advertisement =
                    signed_snapshot_offer_message(&signing_key, "node:alpha", object_id);
                ("snapshot-sync", object, advertisement, "SNAPSHOT_OFFER")
            }
            ("view", "maintainer") => {
                let object = signed_view_object_message(&signing_key, "node:alpha", "rev:test");
                let object_id = object["payload"]["object_id"]
                    .as_str()
                    .expect("view OBJECT should include object_id");
                let advertisement = signed_view_announce_message_with_metadata(
                    &signing_key,
                    "node:alpha",
                    object_id,
                    "pk:announced-other",
                    json!({"doc:test": "rev:test"}),
                );
                ("view-sync", object, advertisement, "VIEW_ANNOUNCE")
            }
            ("view", "documents") => {
                let object =
                    signed_view_object_message(&signing_key, "node:alpha", "rev:unexpected");
                let object_id = object["payload"]["object_id"]
                    .as_str()
                    .expect("view OBJECT should include object_id");
                let advertisement =
                    signed_view_announce_message(&signing_key, "node:alpha", object_id);
                ("view-sync", object, advertisement, "VIEW_ANNOUNCE")
            }
            _ => unreachable!("unsupported advertisement mismatch test case"),
        };
    let object_id = object["payload"]["object_id"]
        .as_str()
        .expect("OBJECT should include object_id")
        .to_string();
    let mut session =
        session_ready_for_advertised_object(&signing_key, capability, &advertisement, &object_id);

    let error = session.verify_incoming(&object).unwrap_err();
    let mismatch_verb = if mismatched_field == "documents" {
        "do not match"
    } else {
        "does not match"
    };

    assert_eq!(
        error,
        format!(
            "wire {object_type} OBJECT '{object_id}' {mismatched_field} {mismatch_verb} prior {advertisement_type} from 'node:alpha'"
        )
    );
    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state.pending_object_ids.contains(&object_id));
    if object_type == "snapshot" {
        assert!(state.snapshot_offers.contains_key(&object_id));
    } else {
        assert!(state.view_announcements.contains_key(&object_id));
    }
}

#[test]
fn wire_session_rejects_view_announce_without_view_capability() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let view_announce =
        signed_view_announce_message(&signing_key, "node:alpha", "view:test-announce");

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    let error = session.verify_incoming(&view_announce).unwrap_err();

    assert_eq!(
        error,
        "wire VIEW_ANNOUNCE requires advertised capability 'view-sync' from 'node:alpha'"
    );
}

#[test]
fn wire_session_accepts_view_announce_with_matching_view_object() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message_with_capabilities(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!(["patch-sync", "view-sync"]),
    );
    let manifest = signed_manifest_message_with_capabilities(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!(["patch-sync", "view-sync"]),
    );
    let view_object = signed_view_object_message(&signing_key, "node:alpha", "rev:test");
    let view_id = view_object["payload"]["object_id"]
        .as_str()
        .expect("view OBJECT should include object_id")
        .to_string();
    let view_announce = signed_view_announce_message(&signing_key, "node:alpha", &view_id);
    let want = signed_want_message(&signing_key, "node:alpha", &[&view_id]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&view_announce)
        .expect("VIEW_ANNOUNCE should verify");
    session
        .verify_incoming(&want)
        .expect("view WANT should verify after announcement");
    session
        .verify_incoming(&view_object)
        .expect("matching view OBJECT should verify after announcement");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state.reachable_object_ids.contains(&view_id));
    assert!(state.pending_object_ids.is_empty());
    assert!(state.view_announcements.is_empty());
}

#[test]
fn wire_session_rejects_messages_after_bye() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let bye = signed_bye_message(&signing_key, "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session.verify_incoming(&bye).expect("BYE should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(error, "wire session for 'node:alpha' is already closed");
}

#[test]
fn wire_session_rejects_duplicate_hello() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");

    session
        .verify_incoming(&hello)
        .expect("first HELLO should verify");
    let error = session.verify_incoming(&hello).unwrap_err();

    assert_eq!(
        error,
        "wire session already received HELLO from 'node:alpha'"
    );
}

#[test]
fn wire_session_accepts_error_before_hello() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let error_msg = signed_error_message(&signing_key, "node:alpha", "msg:some-prior-msg");

    session
        .verify_incoming(&error_msg)
        .expect("ERROR should be accepted before HELLO");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(
        !state.hello_received(),
        "hello_received must remain false after an ERROR-only exchange"
    );
}
