use super::*;

#[test]
fn object_verify_text_fails_when_signed_object_is_missing_signature() {
    let object = write_object_file(
        "object-verify-view-missing-signature",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": signer_id(&signing_key()),
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "accept_keys": ["pk:maintainerA"],
                "merge_rule": "manual-reviewed",
                "preferred_branches": ["main"]
            },
            "timestamp": 1777778891u64,
            "view_id": "view:placeholder"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path]);

    assert_exit_code(&output, 1);
    assert_stdout_contains(&output, "verification: failed");
    assert_stderr_starts_with(&output, "error: ");
    assert_stderr_contains(
        &output,
        "view object is missing required top-level 'signature'",
    );
}

#[test]
fn object_verify_json_fails_for_view_missing_signature() {
    let object = write_object_file(
        "object-verify-view-missing-signature-json",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": signer_id(&signing_key()),
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64,
            "view_id": "view:placeholder"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("view object is missing required top-level 'signature'")
                })
            })),
        "expected missing signature error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_invalid_view_signature() {
    let mut view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    view["signature"] = Value::String(
        "sig:ed25519:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
            .to_string(),
    );
    let object = write_object_file("object-verify-view-bad-signature", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert_eq!(json["signature_verification"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| entry
                .as_str()
                .is_some_and(|message| message.contains("Ed25519 signature verification failed")))),
        "expected signature failure, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_missing_timestamp() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            }
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file("object-verify-view-missing-timestamp", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing integer field 'timestamp'"))
            })),
        "expected missing timestamp error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_negative_timestamp() {
    let mut view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    view["timestamp"] = json!(-1);
    view["signature"] = Value::String(sign_value(&signing_key(), &view));
    let object = write_object_file("object-verify-view-negative-timestamp", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'timestamp' must be a non-negative integer")
                })
            })),
        "expected timestamp integer error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_invalid_policy_accept_key_prefix() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "accept_keys": ["sig:test"],
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file(
        "object-verify-view-bad-policy-accept-key",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'policy.accept_keys[0]' must use 'pk:' prefix")
                })
            })),
        "expected policy accept_keys prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_non_string_signature() {
    let object = write_object_file(
        "object-verify-view-non-string-signature",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": signer_id(&signing_key()),
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64,
            "view_id": "view:placeholder",
            "signature": 7
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'signature' must be a string")
                })
            })),
        "expected signature type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_wrong_signature_format() {
    let object = write_object_file(
        "object-verify-view-wrong-signature-format",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": signer_id(&signing_key()),
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64,
            "view_id": "view:placeholder",
            "signature": "sig:bad"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"].as_array().is_some_and(|errors| errors
            .iter()
            .any(|entry| entry.as_str().is_some_and(|message| message
                .contains("signature field must use format 'sig:ed25519:<base64>'")))),
        "expected signature format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_malformed_signature_bytes() {
    let object = write_object_file(
        "object-verify-view-malformed-signature",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": signer_id(&signing_key()),
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64,
            "view_id": "view:placeholder",
            "signature": "sig:ed25519:not-base64"
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("failed to decode Ed25519 signature"))
            })),
        "expected malformed signature decode error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_invalid_signature_bytes() {
    let object = write_object_file(
        "object-verify-view-invalid-signature-bytes",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": signer_id(&signing_key()),
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64,
            "view_id": "view:placeholder",
            "signature": "sig:ed25519:AA=="
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("invalid Ed25519 signature bytes"))
            })),
        "expected invalid signature bytes error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_wrong_maintainer_prefix() {
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": "sig:bad",
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 1777778891u64
    });
    let view_id = recompute_id(&view, "view_id", "view");
    view["view_id"] = Value::String(view_id);
    view["signature"] = Value::String(sign_value(&signing_key(), &view));
    let object = write_object_file(
        "object-verify-view-wrong-maintainer-prefix",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("signer field must use format 'pk:ed25519:<base64>'")
                })
            })),
        "expected maintainer signer-format error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_malformed_maintainer_key() {
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": "pk:ed25519:not-base64",
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 1777778891u64
    });
    let view_id = recompute_id(&view, "view_id", "view");
    view["view_id"] = Value::String(view_id);
    view["signature"] = Value::String(sign_value(&signing_key(), &view));
    let object = write_object_file(
        "object-verify-view-malformed-maintainer-key",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("failed to decode Ed25519 public key"))
            })),
        "expected malformed public-key error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_invalid_maintainer_key_bytes() {
    let object = write_object_file(
        "object-verify-view-invalid-maintainer-bytes",
        "view.json",
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "maintainer": "pk:ed25519:AA==",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64,
            "view_id": "view:placeholder",
            "signature": "sig:ed25519:AA=="
        }),
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("Ed25519 public key must decode to 32 bytes")
                })
            })),
        "expected invalid public-key length error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_missing_maintainer() {
    let mut view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    view.as_object_mut()
        .expect("view should be an object")
        .remove("maintainer");
    let object = write_object_file("object-verify-view-missing-maintainer", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("view object is missing string signer field 'maintainer'")
                })
            })),
        "expected missing maintainer signer-field error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_wrong_view_id_prefix() {
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": signer_id(&signing_key()),
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 1777778891u64
    });
    let view_id = recompute_id(&view, "view_id", "view");
    view["view_id"] = Value::String(view_id.replacen("view:", "snap:", 1));
    view["signature"] = Value::String(sign_value(&signing_key(), &view));
    let object = write_object_file(
        "object-verify-view-wrong-derived-id-prefix",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'view_id' must use 'view:' prefix")
                })
            })),
        "expected view_id prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_non_string_view_id() {
    let mut view = json!({
        "type": "view",
        "version": "mycel/0.1",
        "maintainer": signer_id(&signing_key()),
        "documents": {
            "doc:test": "rev:test"
        },
        "policy": {
            "merge_rule": "manual-reviewed"
        },
        "timestamp": 1777778891u64
    });
    view["view_id"] = Value::String(recompute_id(&view, "view_id", "view"));
    view["signature"] = Value::String(sign_value(&signing_key(), &view));
    view["view_id"] = json!(7);
    let object = write_object_file(
        "object-verify-view-non-string-derived-id",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'view_id' must be a string"))
            })),
        "expected view_id type error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_non_object_policy() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": "manual-reviewed",
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file("object-verify-view-non-object-policy", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("top-level 'policy' must be an object"))
            })),
        "expected non-object policy error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_non_object_documents() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": [],
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file("object-verify-view-non-object-documents", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents' must be an object")
                })
            })),
        "expected documents object-shape error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_missing_policy() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file("object-verify-view-missing-policy", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing object field 'policy'"))
            })),
        "expected missing policy error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_missing_documents() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file("object-verify-view-missing-documents", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("missing object field 'documents'"))
            })),
        "expected missing documents error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_empty_documents() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {},
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file("object-verify-view-empty-documents", "view.json", view);
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents' must not be empty")
                })
            })),
        "expected empty documents error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_wrong_document_value_prefix() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "patch:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file(
        "object-verify-view-wrong-document-value-prefix",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents.doc:test' must use 'rev:' prefix")
                })
            })),
        "expected view document revision-prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_wrong_document_key_prefix() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "patch:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file(
        "object-verify-view-wrong-document-key-prefix",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level 'documents.patch:test key' must use 'doc:' prefix")
                })
            })),
        "expected view document key-prefix error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn object_verify_json_fails_for_view_with_unknown_top_level_field() {
    let view = signed_object(
        json!({
            "type": "view",
            "version": "mycel/0.1",
            "documents": {
                "doc:test": "rev:test"
            },
            "policy": {
                "merge_rule": "manual-reviewed"
            },
            "timestamp": 1777778891u64,
            "unexpected": true
        }),
        "maintainer",
        "view_id",
        "view",
    );
    let object = write_object_file(
        "object-verify-view-unknown-top-level-field",
        "view.json",
        view,
    );
    let path = path_arg(&object.path);
    let output = run_mycel(&["object", "verify", &path, "--json"]);

    assert_exit_code(&output, 1);
    let json = assert_json_status(&output, "failed");
    assert_eq!(json["object_type"], "view");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry.as_str().is_some_and(|message| {
                    message.contains("top-level contains unexpected field 'unexpected'")
                })
            })),
        "expected unknown-field error, stdout: {}",
        stdout_text(&output)
    );
}
