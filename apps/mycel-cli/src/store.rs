use std::path::PathBuf;

use crate::CliError;
use clap::{Args, Subcommand};

#[derive(Args)]
pub(crate) struct StoreCliArgs {
    #[command(subcommand)]
    command: Option<StoreSubcommand>,
}

#[derive(Subcommand)]
enum StoreSubcommand {
    #[command(about = "Initialize an empty local object store")]
    Init(StoreInitCliArgs),
    #[command(about = "Create a document object and its genesis revision in the store")]
    CreateDocument(StoreCreateDocumentCliArgs),
    #[command(about = "Create a signed patch object in the store")]
    CreatePatch(StoreCreatePatchCliArgs),
    #[command(about = "Commit a signed revision object in the store")]
    CommitRevision(StoreCommitRevisionCliArgs),
    #[command(about = "Verify and ingest objects into a local object store")]
    Ingest(StoreIngestCliArgs),
    #[command(about = "Query persisted local object-store indexes")]
    Index(index::StoreIndexCliArgs),
    #[command(about = "Rebuild local object-store indexes from stored objects")]
    Rebuild(StoreRebuildCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct StoreRebuildCliArgs {
    #[arg(
        value_name = "PATH",
        help = "Object-store directory or one object file to rebuild from",
        required = true,
        allow_hyphen_values = true
    )]
    target: String,
    #[arg(long, help = "Emit machine-readable store-rebuild output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct StoreIngestCliArgs {
    #[arg(
        value_name = "SOURCE",
        help = "Object file or directory to ingest from",
        required = true,
        allow_hyphen_values = true
    )]
    source: String,
    #[arg(
        long = "into",
        value_name = "STORE_ROOT",
        help = "Store root directory to write into",
        required = true
    )]
    into: String,
    #[arg(long, help = "Emit machine-readable store-ingest output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct StoreInitCliArgs {
    #[arg(
        value_name = "STORE_ROOT",
        help = "Store root directory to initialize",
        required = true,
        allow_hyphen_values = true
    )]
    store_root: String,
    #[arg(long, help = "Emit machine-readable store-init output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct StoreCreateDocumentCliArgs {
    #[arg(
        value_name = "STORE_ROOT",
        help = "Store root directory to write into",
        required = true,
        allow_hyphen_values = true
    )]
    store_root: String,
    #[arg(long, value_name = "DOC_ID", required = true)]
    doc_id: String,
    #[arg(long, value_name = "TITLE", required = true)]
    title: String,
    #[arg(long, value_name = "LANGUAGE", default_value = "und")]
    language: String,
    #[arg(long, value_name = "KEY_FILE", required = true)]
    signing_key: String,
    #[arg(long, value_name = "UNIX_SECS")]
    timestamp: Option<u64>,
    #[arg(long, help = "Emit machine-readable document-create output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct StoreCreatePatchCliArgs {
    #[arg(
        value_name = "STORE_ROOT",
        help = "Store root directory to write into",
        required = true,
        allow_hyphen_values = true
    )]
    store_root: String,
    #[arg(long, value_name = "DOC_ID", required = true)]
    doc_id: String,
    #[arg(long, value_name = "REVISION_ID", required = true)]
    base_revision: String,
    #[arg(long, value_name = "OPS_FILE", required = true)]
    ops: String,
    #[arg(long, value_name = "KEY_FILE", required = true)]
    signing_key: String,
    #[arg(long, value_name = "UNIX_SECS")]
    timestamp: Option<u64>,
    #[arg(long, help = "Emit machine-readable patch-create output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct StoreCommitRevisionCliArgs {
    #[arg(
        value_name = "STORE_ROOT",
        help = "Store root directory to write into",
        required = true,
        allow_hyphen_values = true
    )]
    store_root: String,
    #[arg(long, value_name = "DOC_ID", required = true)]
    doc_id: String,
    #[arg(long = "parent", value_name = "REVISION_ID")]
    parents: Vec<String>,
    #[arg(long = "patch", value_name = "PATCH_ID", required = true)]
    patches: Vec<String>,
    #[arg(long, value_name = "MERGE_STRATEGY")]
    merge_strategy: Option<String>,
    #[arg(long, value_name = "KEY_FILE", required = true)]
    signing_key: String,
    #[arg(long, value_name = "UNIX_SECS")]
    timestamp: Option<u64>,
    #[arg(long, help = "Emit machine-readable revision-commit output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[path = "store/index.rs"]
mod index;
#[path = "store/write.rs"]
mod write;

fn unexpected_extra(extra: &[String], context: &str) -> Option<String> {
    extra
        .first()
        .map(|arg| format!("unexpected {context} argument: {arg}"))
}

pub(crate) fn handle_store_command(command: StoreCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(StoreSubcommand::Init(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store init") {
                return Err(CliError::usage(message));
            }

            write::store_init(PathBuf::from(args.store_root), args.json)
        }
        Some(StoreSubcommand::CreateDocument(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store create-document") {
                return Err(CliError::usage(message));
            }

            write::store_create_document(args)
        }
        Some(StoreSubcommand::CreatePatch(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store create-patch") {
                return Err(CliError::usage(message));
            }

            write::store_create_patch(args)
        }
        Some(StoreSubcommand::CommitRevision(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store commit-revision") {
                return Err(CliError::usage(message));
            }

            write::store_commit_revision(args)
        }
        Some(StoreSubcommand::Ingest(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store ingest") {
                return Err(CliError::usage(message));
            }

            index::store_ingest(
                PathBuf::from(args.source),
                PathBuf::from(args.into),
                args.json,
            )
        }
        Some(StoreSubcommand::Index(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store index") {
                return Err(CliError::usage(message));
            }

            index::store_index(args)
        }
        Some(StoreSubcommand::Rebuild(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "store rebuild") {
                return Err(CliError::usage(message));
            }

            index::store_rebuild(PathBuf::from(args.target), args.json)
        }
        Some(StoreSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!(
                "unknown store subcommand: {other}"
            )))
        }
        None => Err(CliError::usage("missing store subcommand")),
    }
}
