use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use mycel_core::author::{parse_signing_key_seed, signer_id};
use mycel_core::protocol::parse_json_strict;
use mycel_core::sync::{
    sync_pull_from_peer_store, sync_pull_from_transcript, SyncPeer, SyncPullSummary,
    SyncPullTranscript,
};
use serde::Serialize;

use crate::{emit_error_line, CliError};

#[derive(Args)]
pub(crate) struct SyncCliArgs {
    #[command(subcommand)]
    command: Option<SyncSubcommand>,
}

#[derive(Subcommand)]
enum SyncSubcommand {
    #[command(about = "Replay one wire-session transcript into verify/store sync state")]
    Pull(SyncPullCliArgs),
    #[command(about = "Run the minimal peer-store sync driver into a local store")]
    PeerStore(SyncPeerStoreCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct SyncPullCliArgs {
    #[arg(
        value_name = "TRANSCRIPT",
        help = "Wire-session transcript JSON file to pull from",
        required = true,
        allow_hyphen_values = true
    )]
    transcript: String,
    #[arg(
        long = "into",
        value_name = "STORE_ROOT",
        help = "Store root directory to verify and write into",
        required = true
    )]
    into: String,
    #[arg(long, help = "Emit machine-readable sync-pull output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct SyncPeerStoreCliArgs {
    #[arg(
        long = "from",
        value_name = "REMOTE_STORE_ROOT",
        help = "Remote peer store root to advertise and serve objects from",
        required = true
    )]
    from: String,
    #[arg(
        long = "into",
        value_name = "STORE_ROOT",
        help = "Local store root directory to verify and write into",
        required = true
    )]
    into: String,
    #[arg(
        long = "peer-node-id",
        value_name = "NODE_ID",
        help = "Node ID to advertise for the peer-store source",
        required = true
    )]
    peer_node_id: String,
    #[arg(
        long = "signing-key",
        value_name = "FILE",
        help = "Base64 Ed25519 signing key seed file for the peer-store source",
        required = true
    )]
    signing_key: String,
    #[arg(long, help = "Emit machine-readable peer-store sync output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SyncPullCliSummary {
    source: PathBuf,
    #[serde(flatten)]
    summary: SyncPullSummary,
}

#[derive(Debug, Serialize)]
struct SyncPeerStoreCliSummary {
    source_store: PathBuf,
    signing_key: PathBuf,
    #[serde(flatten)]
    summary: SyncPullSummary,
}

fn unexpected_extra(extra: &[String], context: &str) -> Option<String> {
    extra
        .first()
        .map(|arg| format!("unexpected {context} argument: {arg}"))
}

fn load_sync_transcript(path: &Path) -> Result<SyncPullTranscript, CliError> {
    let rendered = fs::read_to_string(path).map_err(|error| {
        CliError::usage(format!(
            "failed to read sync transcript {}: {error}",
            path.display()
        ))
    })?;
    parse_json_strict(&rendered).map_err(|error| {
        CliError::usage(format!(
            "failed to parse sync transcript {}: {error}",
            path.display()
        ))
    })
}

fn load_signing_key(path: &Path) -> Result<ed25519_dalek::SigningKey, CliError> {
    let rendered = fs::read_to_string(path).map_err(|error| {
        CliError::usage(format!(
            "failed to read signing key file {}: {error}",
            path.display()
        ))
    })?;
    parse_signing_key_seed(&rendered).map_err(CliError::usage)
}

fn cli_summary(source: PathBuf, summary: SyncPullSummary) -> SyncPullCliSummary {
    SyncPullCliSummary { source, summary }
}

fn peer_store_cli_summary(
    source_store: PathBuf,
    signing_key: PathBuf,
    summary: SyncPullSummary,
) -> SyncPeerStoreCliSummary {
    SyncPeerStoreCliSummary {
        source_store,
        signing_key,
        summary,
    }
}

fn print_sync_pull_text(summary: &SyncPullCliSummary) -> i32 {
    println!("source: {}", summary.source.display());
    println!("peer: {}", summary.summary.peer_node_id);
    println!("store root: {}", summary.summary.store_root.display());
    println!("status: {}", summary.summary.status);
    println!("messages: {}", summary.summary.message_count);
    println!(
        "verified messages: {}",
        summary.summary.verified_message_count
    );
    println!("object messages: {}", summary.summary.object_message_count);
    println!(
        "verified objects: {}",
        summary.summary.verified_object_count
    );
    println!("written objects: {}", summary.summary.written_object_count);
    println!(
        "existing objects: {}",
        summary.summary.existing_object_count
    );
    if let Some(path) = &summary.summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }

    for note in &summary.summary.notes {
        println!("note: {note}");
    }

    if summary.summary.is_ok() {
        println!("sync pull: {}", summary.summary.status);
        0
    } else {
        println!("sync pull: failed");
        for error in &summary.summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_sync_pull_json(summary: &SyncPullCliSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("sync pull summary", source)),
    }
}

fn sync_pull(args: SyncPullCliArgs) -> Result<i32, CliError> {
    let source = PathBuf::from(args.transcript);
    let store_root = PathBuf::from(args.into);
    let transcript = load_sync_transcript(&source)?;
    let summary = sync_pull_from_transcript(&transcript, &store_root)
        .map_err(|error| CliError::usage(error.to_string()))?;
    let summary = cli_summary(source, summary);

    if args.json {
        print_sync_pull_json(&summary)
    } else {
        Ok(print_sync_pull_text(&summary))
    }
}

fn print_sync_peer_store_text(summary: &SyncPeerStoreCliSummary) -> i32 {
    println!("source store: {}", summary.source_store.display());
    println!("signing key: {}", summary.signing_key.display());
    println!("peer: {}", summary.summary.peer_node_id);
    println!("store root: {}", summary.summary.store_root.display());
    println!("status: {}", summary.summary.status);
    println!("messages: {}", summary.summary.message_count);
    println!(
        "verified messages: {}",
        summary.summary.verified_message_count
    );
    println!("object messages: {}", summary.summary.object_message_count);
    println!(
        "verified objects: {}",
        summary.summary.verified_object_count
    );
    println!("written objects: {}", summary.summary.written_object_count);
    println!(
        "existing objects: {}",
        summary.summary.existing_object_count
    );
    if let Some(path) = &summary.summary.index_manifest_path {
        println!("index manifest: {}", path.display());
    }

    for note in &summary.summary.notes {
        println!("note: {note}");
    }

    if summary.summary.is_ok() {
        println!("sync peer-store: {}", summary.summary.status);
        0
    } else {
        println!("sync peer-store: failed");
        for error in &summary.summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_sync_peer_store_json(summary: &SyncPeerStoreCliSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("peer-store sync summary", source)),
    }
}

fn sync_peer_store(args: SyncPeerStoreCliArgs) -> Result<i32, CliError> {
    let source_store = PathBuf::from(args.from);
    let store_root = PathBuf::from(args.into);
    let signing_key_path = PathBuf::from(args.signing_key);
    let signing_key = load_signing_key(&signing_key_path)?;
    let peer = SyncPeer {
        node_id: args.peer_node_id,
        public_key: signer_id(&signing_key),
    };
    let summary = sync_pull_from_peer_store(&peer, &signing_key, &source_store, &store_root)
        .map_err(|error| CliError::usage(error.to_string()))?;
    let summary = peer_store_cli_summary(source_store, signing_key_path, summary);

    if args.json {
        print_sync_peer_store_json(&summary)
    } else {
        Ok(print_sync_peer_store_text(&summary))
    }
}

pub(crate) fn handle_sync_command(command: SyncCliArgs) -> Result<i32, CliError> {
    match command.command {
        Some(SyncSubcommand::Pull(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "sync pull") {
                return Err(CliError::usage(message));
            }

            sync_pull(args)
        }
        Some(SyncSubcommand::PeerStore(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "sync peer-store") {
                return Err(CliError::usage(message));
            }

            sync_peer_store(args)
        }
        Some(SyncSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!("unknown sync subcommand: {other}")))
        }
        None => Err(CliError::usage("missing sync subcommand")),
    }
}
