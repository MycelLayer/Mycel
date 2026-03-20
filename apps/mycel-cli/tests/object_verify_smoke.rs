use std::fs;
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use mycel_core::author::signer_id;
use mycel_core::canonical::prefixed_canonical_hash;
use serde_json::{json, Value};

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_json_status, assert_stderr_contains,
    assert_stderr_starts_with, assert_stdout_contains, assert_success, assert_top_level_help,
    create_temp_dir, parse_json_stdout, recompute_test_object_id as recompute_id, run_mycel,
    sign_test_value as sign_value, signed_test_object, stdout_text,
};

struct TempObjectFile {
    _dir: common::TempDir,
    path: PathBuf,
}

fn write_object_file(prefix: &str, name: &str, value: Value) -> TempObjectFile {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join(name);
    let content = serde_json::to_string_pretty(&value).expect("object JSON should serialize");
    fs::write(&path, content).expect("object JSON should be written");
    TempObjectFile { _dir: dir, path }
}

fn write_raw_object_file(prefix: &str, name: &str, content: &str) -> TempObjectFile {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join(name);
    fs::write(&path, content).expect("object JSON should be written");
    TempObjectFile { _dir: dir, path }
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

fn state_hash(value: &Value) -> String {
    prefixed_canonical_hash(value, "hash").expect("state hash should canonicalize")
}

fn signed_object(value: Value, signer_field: &str, id_field: &str, id_prefix: &str) -> Value {
    let signing_key = signing_key();
    signed_test_object(value, &signing_key, signer_field, id_field, id_prefix)
}

#[path = "object_verify_smoke/document_block_general.rs"]
mod document_block_general;
#[path = "object_verify_smoke/patch.rs"]
mod patch;
#[path = "object_verify_smoke/revision.rs"]
mod revision;
#[path = "object_verify_smoke/snapshot.rs"]
mod snapshot;
#[path = "object_verify_smoke/view.rs"]
mod view;
