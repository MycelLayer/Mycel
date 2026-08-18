use super::*;

#[test]
fn wire_session_accepts_heads_before_manifest_and_unlocks_want() {
    let signing_key = signing_key();
    let mut session = registered_session(&signing_key, "node:alpha");
    let graph = patch_revision_graph(&signing_key, "node:alpha", "rev:genesis-null");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": [graph.revision_id.clone()]
        }),
        true,
    );
    let want = signed_want_message(&signing_key, "node:alpha", &[graph.revision_id.as_str()]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&heads)
        .expect("HEADS should verify before MANIFEST");
    session
        .verify_incoming(&want)
        .expect("WANT should verify after HEADS establishes sync roots");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state
        .advertised_document_heads
        .get("doc:test")
        .is_some_and(|revisions| revisions.contains(&graph.revision_id)));
    assert!(state.pending_object_ids.contains(&graph.revision_id));
}

#[test]
fn wire_session_records_manifest_heads() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert_eq!(
        state
            .advertised_document_heads
            .get("doc:test")
            .map(|revisions| revisions.len()),
        Some(1)
    );
    assert!(state
        .advertised_document_heads
        .get("doc:test")
        .is_some_and(|revisions| revisions.contains("rev:test")));
}

#[test]
fn wire_session_merges_incremental_heads_updates() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": ["rev:next"],
            "doc:extra": ["rev:extra"]
        }),
        false,
    );

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&heads)
        .expect("HEADS should verify");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state
        .advertised_document_heads
        .get("doc:test")
        .is_some_and(|revisions| {
            revisions.contains("rev:test") && revisions.contains("rev:next")
        }));
    assert!(state
        .advertised_document_heads
        .get("doc:extra")
        .is_some_and(|revisions| revisions.contains("rev:extra")));
}

#[test]
fn wire_session_replaces_only_listed_heads_when_replace_is_true() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message_with_heads(
        &signing_key,
        "node:alpha",
        "node:alpha",
        json!({
            "doc:test": ["rev:test"],
            "doc:preserved": ["rev:preserved"]
        }),
    );
    let heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": ["rev:replacement"]
        }),
        true,
    );
    let want = signed_want_message(
        &signing_key,
        "node:alpha",
        &["rev:replacement", "rev:preserved"],
    );

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    session
        .verify_incoming(&heads)
        .expect("HEADS should verify");
    session
        .verify_incoming(&want)
        .expect("replacement and preserved roots should remain eligible for WANT");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(state
        .advertised_document_heads
        .get("doc:test")
        .is_some_and(|revisions| revisions.contains("rev:replacement")));
    assert!(state
        .advertised_document_heads
        .get("doc:test")
        .is_some_and(|revisions| !revisions.contains("rev:test")));
    assert!(state
        .advertised_document_heads
        .get("doc:preserved")
        .is_some_and(|revisions| revisions.contains("rev:preserved")));
    assert!(state.pending_object_ids.contains("rev:replacement"));
    assert!(state.pending_object_ids.contains("rev:preserved"));
}

#[test]
fn wire_session_preserves_pending_root_for_unlisted_document_after_heads_replace() {
    let signing_key = signing_key();
    let mut session = registered_session(&signing_key, "node:alpha");
    let replaced_graph = patch_revision_graph(&signing_key, "node:alpha", "rev:replaced-base");
    let preserved_graph = patch_revision_graph(&signing_key, "node:alpha", "rev:preserved-base");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let initial_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:replaced": [replaced_graph.revision_id.clone()],
            "doc:preserved": [preserved_graph.revision_id.clone()]
        }),
        true,
    );
    let request_roots = signed_want_message(
        &signing_key,
        "node:alpha",
        &[
            replaced_graph.revision_id.as_str(),
            preserved_graph.revision_id.as_str(),
        ],
    );
    let replacement_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:replaced": ["rev:replacement"]
        }),
        true,
    );

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&initial_heads)
        .expect("initial HEADS should verify");
    session
        .verify_incoming(&request_roots)
        .expect("root WANT should verify");
    session
        .verify_incoming(&replacement_heads)
        .expect("replacement HEADS should verify");

    let state = session
        .peer_session("node:alpha")
        .expect("peer session should exist");
    assert!(!state
        .pending_object_ids
        .contains(&replaced_graph.revision_id));
    assert!(state
        .pending_object_ids
        .contains(&preserved_graph.revision_id));

    session
        .verify_incoming(&preserved_graph.revision_object)
        .expect("unlisted document's pending root OBJECT should remain valid");
    let error = session
        .verify_incoming(&replaced_graph.revision_object)
        .unwrap_err();
    assert_eq!(
        error,
        format!(
            "wire OBJECT '{}' was not requested from 'node:alpha'",
            replaced_graph.revision_id
        )
    );
}

#[test]
fn wire_session_preserves_shared_dependency_provenance_after_partial_heads_replace() {
    let signing_key = signing_key();
    let mut session = registered_session(&signing_key, "node:alpha");
    let shared_patch = signed_patch_object_message(&signing_key, "node:alpha", "rev:genesis-null");
    let shared_patch_id = shared_patch["payload"]["object_id"]
        .as_str()
        .expect("shared patch object ID should exist")
        .to_owned();
    let shared_revision = signed_revision_object_message(
        &signing_key,
        "node:alpha",
        &[],
        &[shared_patch_id.as_str()],
    );
    let shared_revision_id = shared_revision["payload"]["object_id"]
        .as_str()
        .expect("shared revision object ID should exist")
        .to_owned();
    let replaced_revision = signed_revision_object_message(
        &signing_key,
        "node:alpha",
        &[shared_revision_id.as_str()],
        &[],
    );
    let replaced_revision_id = replaced_revision["payload"]["object_id"]
        .as_str()
        .expect("replaced revision object ID should exist")
        .to_owned();
    let preserved_side_patch =
        signed_patch_object_message(&signing_key, "node:alpha", "rev:preserved-side-base");
    let preserved_side_patch_id = preserved_side_patch["payload"]["object_id"]
        .as_str()
        .expect("preserved side patch object ID should exist")
        .to_owned();
    let preserved_revision = signed_revision_object_message(
        &signing_key,
        "node:alpha",
        &[shared_revision_id.as_str()],
        &[preserved_side_patch_id.as_str()],
    );
    let preserved_revision_id = preserved_revision["payload"]["object_id"]
        .as_str()
        .expect("preserved revision object ID should exist")
        .to_owned();
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let initial_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:replaced": [replaced_revision_id.clone()],
            "doc:preserved": [preserved_revision_id.clone()]
        }),
        true,
    );
    let request_roots = signed_want_message(
        &signing_key,
        "node:alpha",
        &[
            replaced_revision_id.as_str(),
            preserved_revision_id.as_str(),
        ],
    );
    let request_shared_revision =
        signed_want_message(&signing_key, "node:alpha", &[shared_revision_id.as_str()]);
    let request_patch =
        signed_want_message(&signing_key, "node:alpha", &[shared_patch_id.as_str()]);
    let replacement_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:replaced": ["rev:replacement"]
        }),
        true,
    );

    for message in [
        &hello,
        &initial_heads,
        &request_roots,
        &replaced_revision,
        &request_shared_revision,
        &shared_revision,
        &preserved_revision,
        &request_patch,
        &replacement_heads,
    ] {
        session
            .verify_incoming(message)
            .expect("shared-dependency setup message should verify");
    }

    session
        .verify_incoming(&shared_patch)
        .expect("dependency shared with an unlisted document should remain valid");
}

#[test]
fn wire_session_rejects_stale_dependency_want_after_heads_replace() {
    let signing_key = signing_key();
    let mut session = registered_session(&signing_key, "node:alpha");
    let graph = patch_revision_graph(&signing_key, "node:alpha", "rev:genesis-null");

    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let initial_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": [graph.revision_id.clone()]
        }),
        true,
    );
    let request_revision =
        signed_want_message(&signing_key, "node:alpha", &[graph.revision_id.as_str()]);
    let replacement_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": ["rev:replacement"]
        }),
        true,
    );
    let request_stale_patch =
        signed_want_message(&signing_key, "node:alpha", &[graph.patch_id.as_str()]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&initial_heads)
        .expect("initial HEADS should verify");
    session
        .verify_incoming(&request_revision)
        .expect("root revision WANT should verify");
    session
        .verify_incoming(&graph.revision_object)
        .expect("root revision OBJECT should verify");
    session
        .verify_incoming(&replacement_heads)
        .expect("replacement HEADS should verify");
    let error = session.verify_incoming(&request_stale_patch).unwrap_err();

    assert_eq!(
        error,
        format!(
            "wire WANT object '{}' is not reachable from accepted sync roots for 'node:alpha'",
            graph.patch_id
        )
    );
}

#[test]
fn wire_session_rejects_stale_root_revision_want_after_heads_replace() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");

    let revision_id = "rev:stale-root";
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let initial_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": [revision_id]
        }),
        true,
    );
    let replacement_heads = signed_heads_message(
        &signing_key,
        "node:alpha",
        json!({
            "doc:test": ["rev:replacement"]
        }),
        true,
    );
    let request_stale_revision = signed_want_message(&signing_key, "node:alpha", &[revision_id]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&initial_heads)
        .expect("initial HEADS should verify");
    session
        .verify_incoming(&replacement_heads)
        .expect("replacement HEADS should verify");
    let error = session
        .verify_incoming(&request_stale_revision)
        .unwrap_err();

    assert_eq!(
        error,
        "wire WANT revision 'rev:stale-root' is not reachable from accepted sync roots for 'node:alpha'"
    );
}

#[test]
fn wire_session_snapshot_offer_before_manifest_still_requires_head_context_for_want() {
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
    let snapshot_offer =
        signed_snapshot_offer_message(&signing_key, "node:alpha", "snap:test-offer");
    let want = signed_want_message(&signing_key, "node:alpha", &["snap:test-offer"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&snapshot_offer)
        .expect("SNAPSHOT_OFFER should verify before MANIFEST");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT requires prior MANIFEST or HEADS from 'node:alpha'"
    );
    assert!(session
        .peer_session("node:alpha")
        .is_some_and(|state| state.reachable_object_ids.contains("snap:test-offer")));
}

#[test]
fn wire_session_view_announce_before_manifest_still_requires_head_context_for_want() {
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
    let view_announce =
        signed_view_announce_message(&signing_key, "node:alpha", "view:test-announce");
    let want = signed_want_message(&signing_key, "node:alpha", &["view:test-announce"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&view_announce)
        .expect("VIEW_ANNOUNCE should verify before MANIFEST");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT requires prior MANIFEST or HEADS from 'node:alpha'"
    );
    assert!(session
        .peer_session("node:alpha")
        .is_some_and(|state| state.reachable_object_ids.contains("view:test-announce")));
}

#[test]
fn wire_session_rejects_want_before_head_context() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT requires prior MANIFEST or HEADS from 'node:alpha'"
    );
}

#[test]
fn wire_session_rejects_unadvertised_revision_want() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["rev:missing"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT revision 'rev:missing' is not reachable from accepted sync roots for 'node:alpha'"
    );
}

#[test]
fn wire_session_rejects_non_revision_want_without_sync_root() {
    let signing_key = signing_key();
    let sender_key = sender_public_key(&signing_key);
    let mut session = WireSession::default();
    session
        .register_known_peer("node:alpha", &sender_key)
        .expect("known peer should register");
    let hello = signed_hello_message(&signing_key, "node:alpha", "node:alpha");
    let manifest = signed_manifest_message(&signing_key, "node:alpha", "node:alpha");
    let want = signed_want_message(&signing_key, "node:alpha", &["patch:test"]);

    session
        .verify_incoming(&hello)
        .expect("HELLO should verify");
    session
        .verify_incoming(&manifest)
        .expect("MANIFEST should verify");
    let error = session.verify_incoming(&want).unwrap_err();

    assert_eq!(
        error,
        "wire WANT object 'patch:test' is not reachable from accepted sync roots for 'node:alpha'"
    );
}
