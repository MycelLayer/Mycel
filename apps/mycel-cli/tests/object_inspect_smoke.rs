use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

mod common;

use common::{
    assert_empty_stderr, assert_exit_code, assert_json_status, assert_stderr_contains,
    assert_stdout_contains, assert_success, create_temp_dir, run_mycel, stdout_text,
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

fn path_arg(path: &PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

#[path = "object_inspect_smoke/document_block_general.rs"]
mod document_block_general;
#[path = "object_inspect_smoke/patch.rs"]
mod patch;
#[path = "object_inspect_smoke/revision.rs"]
mod revision;
#[path = "object_inspect_smoke/snapshot.rs"]
mod snapshot;
#[path = "object_inspect_smoke/view.rs"]
mod view;
