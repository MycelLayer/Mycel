use std::path::{Path, PathBuf};

use mycel_core::store::{
    inspect_document_governance, load_store_index_manifest, GovernanceDocumentSummarySource,
};
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::ViewDocumentCliArgs;

#[derive(Debug, Clone, Serialize)]
struct ViewDocumentProfileSummary {
    profile_id: String,
    current_view_id: String,
    current_revision_id: String,
    maintainer: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
struct ViewDocumentSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    doc_id: String,
    profile_id: Option<String>,
    profiles: Vec<ViewDocumentProfileSummary>,
    notes: Vec<String>,
    errors: Vec<String>,
}

impl ViewDocumentSummary {
    fn new(store_root: &Path, doc_id: &str, profile_id: Option<String>) -> Self {
        Self {
            store_root: store_root.to_path_buf(),
            manifest_path: store_root.join("indexes").join("manifest.json"),
            status: "ok".to_string(),
            doc_id: doc_id.to_string(),
            profile_id,
            profiles: Vec::new(),
            notes: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.status = "failed".to_string();
        self.errors.push(message.into());
    }
}

fn print_view_document_text(summary: &ViewDocumentSummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    println!("doc id: {}", summary.doc_id);
    if let Some(profile_id) = &summary.profile_id {
        println!("profile id: {profile_id}");
    }
    println!("profile count: {}", summary.profiles.len());
    for profile in &summary.profiles {
        println!(
            "profile current document: {} view={} revision={} maintainer={} timestamp={}",
            profile.profile_id,
            profile.current_view_id,
            profile.current_revision_id,
            profile.maintainer,
            profile.timestamp
        );
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("view document: ok");
        0
    } else {
        println!("view document: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_view_document_json(summary: &ViewDocumentSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("view document summary", source)),
    }
}

pub(super) fn handle(args: ViewDocumentCliArgs) -> Result<i32, CliError> {
    let ViewDocumentCliArgs {
        store_root,
        doc_id,
        profile_id,
        json,
        extra: _,
    } = args;
    let store_root = PathBuf::from(store_root);

    let mut summary = ViewDocumentSummary::new(&store_root, &doc_id, profile_id.clone());
    let manifest = match load_store_index_manifest(&store_root) {
        Ok(manifest) => manifest,
        Err(error) => {
            summary.push_error(format!("failed to read store index manifest: {error}"));
            return if json {
                print_view_document_json(&summary)
            } else {
                Ok(print_view_document_text(&summary))
            };
        }
    };

    match inspect_document_governance(&manifest, &doc_id, profile_id.as_deref()) {
        Ok(document) => {
            match document.source {
                GovernanceDocumentSummarySource::Persisted => summary.notes.push(
                    "current document governance is read from persisted document-governance summaries instead of scanning every profile at query time"
                        .to_string(),
                ),
                GovernanceDocumentSummarySource::Synthesized => summary.notes.push(
                    "current document governance was synthesized from persisted latest profile/document indexes because the persisted document-governance summary was unavailable"
                        .to_string(),
                ),
            }
            summary.profiles = document
                .profiles
                .into_iter()
                .map(|profile| ViewDocumentProfileSummary {
                    profile_id: profile.profile_id,
                    current_view_id: profile.current_view_id,
                    current_revision_id: profile.current_revision_id,
                    maintainer: profile.maintainer,
                    timestamp: profile.timestamp,
                })
                .collect();
        }
        Err(error) => {
            summary.push_error(error.to_string());
        }
    }

    summary.notes.push(
        "document-centric governance inspection complements profile-centric view current output"
            .to_string(),
    );

    if json {
        print_view_document_json(&summary)
    } else {
        Ok(print_view_document_text(&summary))
    }
}
