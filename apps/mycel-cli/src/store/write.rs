use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use mycel_core::author::{
    commit_revision_to_store, create_document_in_store, create_patch_in_store,
    parse_signing_key_seed, DocumentCreateParams, DocumentCreateSummary, PatchCreateParams,
    PatchCreateSummary, RevisionCommitParams, RevisionCommitSummary,
};
use mycel_core::protocol::parse_json_value_strict;
use mycel_core::store::{initialize_store_root, StoreInitSummary};
use serde::Serialize;
use serde_json::Value;

use crate::CliError;

use super::{StoreCommitRevisionCliArgs, StoreCreateDocumentCliArgs, StoreCreatePatchCliArgs};

pub(super) fn store_init(store_root: PathBuf, json: bool) -> Result<i32, CliError> {
    match initialize_store_root(&store_root) {
        Ok(summary) => {
            if json {
                print_json(&summary, "store init summary")
            } else {
                Ok(print_store_init_text(&summary))
            }
        }
        Err(error) => Err(CliError::usage(error.to_string())),
    }
}

pub(super) fn store_create_document(args: StoreCreateDocumentCliArgs) -> Result<i32, CliError> {
    let signing_key = load_signing_key(&args.signing_key)?;
    let params = DocumentCreateParams {
        doc_id: args.doc_id,
        title: args.title,
        language: args.language,
        timestamp: resolve_timestamp(args.timestamp)?,
    };

    match create_document_in_store(Path::new(&args.store_root), &signing_key, &params) {
        Ok(summary) => {
            if args.json {
                print_json(&summary, "document create summary")
            } else {
                Ok(print_document_create_text(&summary))
            }
        }
        Err(error) => Err(CliError::usage(error.to_string())),
    }
}

pub(super) fn store_create_patch(args: StoreCreatePatchCliArgs) -> Result<i32, CliError> {
    let signing_key = load_signing_key(&args.signing_key)?;
    let ops = load_ops_value(&args.ops)?;
    if !ops.is_array() {
        return Err(CliError::usage(
            "patch ops file must contain a top-level JSON array",
        ));
    }

    let params = PatchCreateParams {
        doc_id: args.doc_id,
        base_revision: args.base_revision,
        timestamp: resolve_timestamp(args.timestamp)?,
        ops,
    };

    match create_patch_in_store(Path::new(&args.store_root), &signing_key, &params) {
        Ok(summary) => {
            if args.json {
                print_json(&summary, "patch create summary")
            } else {
                Ok(print_patch_create_text(&summary))
            }
        }
        Err(error) => Err(CliError::usage(error.to_string())),
    }
}

pub(super) fn store_commit_revision(args: StoreCommitRevisionCliArgs) -> Result<i32, CliError> {
    let signing_key = load_signing_key(&args.signing_key)?;
    let params = RevisionCommitParams {
        doc_id: args.doc_id,
        parents: args.parents,
        patches: args.patches,
        merge_strategy: args.merge_strategy,
        timestamp: resolve_timestamp(args.timestamp)?,
    };

    match commit_revision_to_store(Path::new(&args.store_root), &signing_key, &params) {
        Ok(summary) => {
            if args.json {
                print_json(&summary, "revision commit summary")
            } else {
                Ok(print_revision_commit_text(&summary))
            }
        }
        Err(error) => Err(CliError::usage(error.to_string())),
    }
}

fn load_signing_key(path: &str) -> Result<ed25519_dalek::SigningKey, CliError> {
    let content = fs::read_to_string(path).map_err(|error| {
        CliError::usage(format!("failed to read signing key file {path}: {error}"))
    })?;
    parse_signing_key_seed(&content).map_err(CliError::usage)
}

fn load_ops_value(path: &str) -> Result<Value, CliError> {
    let content = fs::read_to_string(path).map_err(|error| {
        CliError::usage(format!("failed to read patch ops file {path}: {error}"))
    })?;
    parse_json_value_strict(&content)
        .map_err(|error| CliError::usage(format!("failed to parse patch ops file {path}: {error}")))
}

fn resolve_timestamp(timestamp: Option<u64>) -> Result<u64, CliError> {
    match timestamp {
        Some(timestamp) => Ok(timestamp),
        None => SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .map_err(|error| {
                CliError::usage(format!("failed to resolve current timestamp: {error}"))
            }),
    }
}

fn print_store_init_text(summary: &StoreInitSummary) -> i32 {
    println!("store init: {}", summary.status);
    println!("store root: {}", summary.store_root.display());
    println!("index manifest: {}", summary.index_manifest_path.display());
    0
}

fn print_document_create_text(summary: &DocumentCreateSummary) -> i32 {
    println!("document create: {}", summary.status);
    println!("store root: {}", summary.store_root.display());
    println!("document: {}", summary.document_object_id);
    println!("genesis revision: {}", summary.genesis_revision_id);
    println!("written objects: {}", summary.written_object_count);
    println!("existing objects: {}", summary.existing_object_count);
    if let Some(path) = &summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }
    0
}

fn print_patch_create_text(summary: &PatchCreateSummary) -> i32 {
    println!("patch create: {}", summary.status);
    println!("store root: {}", summary.store_root.display());
    println!("doc_id: {}", summary.doc_id);
    println!("patch: {}", summary.patch_id);
    println!("base revision: {}", summary.base_revision);
    println!("written objects: {}", summary.written_object_count);
    println!("existing objects: {}", summary.existing_object_count);
    if let Some(path) = &summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }
    0
}

fn print_revision_commit_text(summary: &RevisionCommitSummary) -> i32 {
    println!("revision commit: {}", summary.status);
    println!("store root: {}", summary.store_root.display());
    println!("doc_id: {}", summary.doc_id);
    println!("revision: {}", summary.revision_id);
    println!("state_hash: {}", summary.recomputed_state_hash);
    println!("written objects: {}", summary.written_object_count);
    println!("existing objects: {}", summary.existing_object_count);
    if let Some(path) = &summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }
    0
}

fn print_json<T: Serialize>(value: &T, context: &'static str) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(value) {
        Ok(json) => {
            println!("{json}");
            Ok(0)
        }
        Err(source) => Err(CliError::serialization(context, source)),
    }
}
