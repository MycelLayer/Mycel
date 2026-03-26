use std::fs;
use std::path::PathBuf;

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use proptest::prelude::*;
use serde_json::{json, Value};

use crate::canonical::{signed_payload_bytes, wire_envelope_signed_payload_bytes};
use crate::protocol::{recompute_declared_object_identity, recompute_object_id};
use crate::replay::{compute_state_hash, DocumentState};

pub(super) fn hello_envelope_with(timestamp: &str) -> Value {
    json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-proptest-001",
        "timestamp": timestamp,
        "from": "node:alpha",
        "payload": {
            "node_id": "node:alpha",
            "capabilities": ["patch-sync"],
            "nonce": "n:test"
        },
        "sig": "sig:placeholder"
    })
}

fn signed_patch_body_for_wire_tests() -> Value {
    sign_object_value(
        &signing_key(),
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": "rev:genesis-null",
            "author": "pk:ed25519:placeholder",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:placeholder"
        }),
        "author",
        "patch_id",
        "patch",
    )
}

pub(super) fn valid_object_payload_for_proptests() -> Value {
    let body = signed_patch_body_for_wire_tests();
    let identity = recompute_declared_object_identity(&body)
        .expect("wire proptest patch body identity should recompute");
    json!({
        "object_id": identity.object_id,
        "object_type": "patch",
        "encoding": "json",
        "hash_alg": "sha256",
        "hash": identity.hash,
        "body": body
    })
}

pub(super) fn valid_wire_timestamp_strategy() -> impl Strategy<Value = String> {
    (
        0u16..=9999,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        any::<bool>(),
        prop_oneof![Just('+'), Just('-')],
        0u8..=99,
        0u8..=99,
    )
        .prop_map(
            |(year, month, day, hour, minute, second, use_z, offset_sign, offset_hour, offset_minute)| {
                if use_z {
                    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
                } else {
                    format!(
                        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{offset_sign}{offset_hour:02}:{offset_minute:02}"
                    )
                }
            },
        )
}

pub(super) fn invalid_wire_timestamp_strategy() -> impl Strategy<Value = String> {
    (
        0u16..=9999,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        0u8..=99,
        prop_oneof![Just('+'), Just('-')],
        0u8..=99,
        0u8..=99,
    )
        .prop_flat_map(
            |(year, month, day, hour, minute, second, offset_sign, offset_hour, offset_minute)| {
                let no_t = format!(
                    "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}Z"
                );
                let slash_date = format!(
                    "{year:04}/{month:02}/{day:02}T{hour:02}:{minute:02}:{second:02}Z"
                );
                let no_offset_colon = format!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{offset_sign}{offset_hour:02}{offset_minute:02}"
                );
                let missing_offset =
                    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
                let short_time = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}Z");
                prop_oneof![
                    Just(no_t),
                    Just(slash_date),
                    Just(no_offset_colon),
                    Just(missing_offset),
                    Just(short_time),
                ]
            },
        )
}

pub(super) fn invalid_object_type_strategy() -> impl Strategy<Value = String> {
    ".*".prop_filter("object_type must be unsupported", |value| {
        !matches!(
            value.as_str(),
            "document" | "block" | "patch" | "revision" | "view" | "snapshot"
        )
    })
}

pub(super) fn invalid_canonical_object_id_strategy() -> impl Strategy<Value = String> {
    ".*".prop_filter("object_id must violate canonical prefix rules", |value| {
        !["patch:", "rev:", "view:", "snap:"]
            .iter()
            .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len())
    })
}

pub(super) fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[9u8; 32])
}

pub(super) fn temp_dir(prefix: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("mycel-wire-{prefix}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

pub(super) fn sender_public_key(signing_key: &SigningKey) -> String {
    format!(
        "pk:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signing_key.verifying_key().as_bytes())
    )
}

fn sign_wire_value(signing_key: &SigningKey, value: &Value) -> String {
    let payload =
        wire_envelope_signed_payload_bytes(value).expect("wire payload should canonicalize");
    let signature = signing_key.sign(&payload);
    format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    )
}

pub(super) fn sign_object_value(
    signing_key: &SigningKey,
    mut value: Value,
    signer_field: &str,
    id_field: &str,
    prefix: &str,
) -> Value {
    value[signer_field] = Value::String(sender_public_key(signing_key));
    let object_id =
        recompute_object_id(&value, id_field, prefix).expect("test object ID should recompute");
    value[id_field] = Value::String(object_id);
    let payload = signed_payload_bytes(&value).expect("object payload should canonicalize");
    let signature = signing_key.sign(&payload);
    value["signature"] = Value::String(format!(
        "sig:ed25519:{}",
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
    ));
    value
}

pub(super) fn empty_state_hash(doc_id: &str) -> String {
    compute_state_hash(&DocumentState {
        doc_id: doc_id.to_string(),
        blocks: Vec::new(),
        metadata: serde_json::Map::new(),
    })
    .expect("empty state hash should compute")
}

pub(super) fn signed_hello_message(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
) -> Value {
    signed_hello_message_with_capabilities(
        signing_key,
        sender,
        payload_node_id,
        json!(["patch-sync"]),
    )
}

pub(super) fn signed_hello_message_with_capabilities(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
    capabilities: Value,
) -> Value {
    let mut value = json!({
        "type": "HELLO",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:hello-signed-001",
        "timestamp": "2026-03-08T20:00:00+08:00",
        "from": sender,
        "payload": {
            "node_id": payload_node_id,
            "capabilities": capabilities,
            "nonce": "n:test"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(super) fn signed_manifest_message(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
) -> Value {
    signed_manifest_message_with_capabilities(
        signing_key,
        sender,
        payload_node_id,
        json!(["patch-sync"]),
    )
}

pub(super) fn signed_manifest_message_with_capabilities(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
    capabilities: Value,
) -> Value {
    let mut value = json!({
        "type": "MANIFEST",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:manifest-signed-001",
        "timestamp": "2026-03-08T20:00:10+08:00",
        "from": sender,
        "payload": {
            "node_id": payload_node_id,
            "capabilities": capabilities,
            "heads": {
                "doc:test": ["rev:test"]
            }
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(super) fn signed_snapshot_offer_message(
    signing_key: &SigningKey,
    sender: &str,
    snapshot_id: &str,
) -> Value {
    let mut value = json!({
        "type": "SNAPSHOT_OFFER",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:snapshot-offer-signed-001",
        "timestamp": "2026-03-08T20:00:40+08:00",
        "from": sender,
        "payload": {
            "snapshot_id": snapshot_id,
            "root_hash": "hash:snapshot-root",
            "documents": ["doc:test"]
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(super) fn signed_view_announce_message(
    signing_key: &SigningKey,
    sender: &str,
    view_id: &str,
) -> Value {
    let mut value = json!({
        "type": "VIEW_ANNOUNCE",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:view-announce-signed-001",
        "timestamp": "2026-03-08T20:00:45+08:00",
        "from": sender,
        "payload": {
            "view_id": view_id,
            "maintainer": sender_public_key(signing_key),
            "documents": {
                "doc:test": "rev:test"
            }
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(super) fn signed_manifest_message_with_heads(
    signing_key: &SigningKey,
    sender: &str,
    payload_node_id: &str,
    heads: Value,
) -> Value {
    let mut value = json!({
        "type": "MANIFEST",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:manifest-signed-001",
        "timestamp": "2026-03-08T20:00:10+08:00",
        "from": sender,
        "payload": {
            "node_id": payload_node_id,
            "capabilities": ["patch-sync"],
            "heads": heads
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(super) fn signed_want_message(
    signing_key: &SigningKey,
    sender: &str,
    object_ids: &[&str],
) -> Value {
    let mut value = json!({
        "type": "WANT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:want-signed-001",
        "timestamp": "2026-03-08T20:01:00+08:00",
        "from": sender,
        "payload": {
            "objects": object_ids
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(super) fn signed_heads_message(
    signing_key: &SigningKey,
    sender: &str,
    documents: Value,
    replace: bool,
) -> Value {
    let mut value = json!({
        "type": "HEADS",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:heads-signed-001",
        "timestamp": "2026-03-08T20:00:30+08:00",
        "from": sender,
        "payload": {
            "documents": documents,
            "replace": replace
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(super) fn signed_object_message(signing_key: &SigningKey, sender: &str) -> Value {
    signed_patch_object_message(signing_key, sender, "rev:genesis-null")
}

pub(super) fn signed_patch_object_message(
    signing_key: &SigningKey,
    sender: &str,
    base_revision: &str,
) -> Value {
    let body = sign_object_value(
        signing_key,
        json!({
            "type": "patch",
            "version": "mycel/0.1",
            "patch_id": "patch:placeholder",
            "doc_id": "doc:test",
            "base_revision": base_revision,
            "author": "pk:ed25519:placeholder",
            "timestamp": 1u64,
            "ops": [],
            "signature": "sig:placeholder"
        }),
        "author",
        "patch_id",
        "patch",
    );
    let object_id = body["patch_id"]
        .as_str()
        .expect("signed patch body should include patch_id")
        .to_owned();
    let object_hash = object_id
        .split_once(':')
        .map(|(_, hash)| hash.to_string())
        .expect("wire object ID should contain hash");

    let mut value = json!({
        "type": "OBJECT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:object-signed-001",
        "timestamp": "2026-03-08T20:01:02+08:00",
        "from": sender,
        "payload": {
            "object_id": object_id,
            "object_type": "patch",
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": format!("hash:{object_hash}"),
            "body": body
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(super) fn signed_revision_object_message(
    signing_key: &SigningKey,
    sender: &str,
    parents: &[&str],
    patches: &[&str],
) -> Value {
    let body = sign_object_value(
        signing_key,
        json!({
            "type": "revision",
            "version": "mycel/0.1",
            "revision_id": "rev:placeholder",
            "doc_id": "doc:test",
            "parents": parents,
            "patches": patches,
            "state_hash": empty_state_hash("doc:test"),
            "author": "pk:ed25519:placeholder",
            "timestamp": 1u64,
            "signature": "sig:placeholder"
        }),
        "author",
        "revision_id",
        "rev",
    );
    let object_id = body["revision_id"]
        .as_str()
        .expect("signed revision body should include revision_id")
        .to_owned();
    let object_hash = object_id
        .split_once(':')
        .map(|(_, hash)| hash.to_string())
        .expect("wire revision ID should contain hash");

    let mut value = json!({
        "type": "OBJECT",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:revision-object-signed-001",
        "timestamp": "2026-03-08T20:01:02+08:00",
        "from": sender,
        "payload": {
            "object_id": object_id,
            "object_type": "revision",
            "encoding": "json",
            "hash_alg": "sha256",
            "hash": format!("hash:{object_hash}"),
            "body": body
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(super) fn signed_error_message(
    signing_key: &SigningKey,
    sender: &str,
    in_reply_to: &str,
) -> Value {
    let mut value = json!({
        "type": "ERROR",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:error-signed-001",
        "timestamp": "2026-03-08T20:02:00+08:00",
        "from": sender,
        "payload": {
            "in_reply_to": in_reply_to,
            "code": "ERR_UNKNOWN",
            "detail": "test error"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}

pub(super) fn signed_bye_message(signing_key: &SigningKey, sender: &str) -> Value {
    let mut value = json!({
        "type": "BYE",
        "version": "mycel-wire/0.1",
        "msg_id": "msg:bye-signed-001",
        "timestamp": "2026-03-08T20:02:00+08:00",
        "from": sender,
        "payload": {
            "reason": "done"
        },
        "sig": "sig:placeholder"
    });
    value["sig"] = Value::String(sign_wire_value(signing_key, &value));
    value
}
