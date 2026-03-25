use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mycel_core::store::{
    inspect_current_maintainer_governance, load_store_index_manifest,
    GovernanceMaintainerSummarySource,
};
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::ViewMaintainerCliArgs;

#[derive(Debug, Clone, Serialize)]
struct ViewMaintainerCurrentDocumentSummary {
    doc_id: String,
    current_view_id: String,
    current_revision_id: String,
    maintainer: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
struct ViewMaintainerProfileSummary {
    profile_id: String,
    current_view_id: String,
    timestamp: u64,
    documents: BTreeMap<String, String>,
    current_documents: Vec<ViewMaintainerCurrentDocumentSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct ViewMaintainerDocumentProfileSummary {
    profile_id: String,
    current_view_id: String,
    current_revision_id: String,
    maintainer: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
struct ViewMaintainerDocumentSummary {
    doc_id: String,
    profiles: Vec<ViewMaintainerDocumentProfileSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct ViewMaintainerSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    maintainer: String,
    profile_id: Option<String>,
    doc_id: Option<String>,
    source: Option<GovernanceMaintainerSummarySource>,
    current_profiles: Vec<ViewMaintainerProfileSummary>,
    current_documents: Vec<ViewMaintainerDocumentSummary>,
    notes: Vec<String>,
    errors: Vec<String>,
}

impl ViewMaintainerSummary {
    fn new(
        store_root: &Path,
        maintainer: &str,
        profile_id: Option<String>,
        doc_id: Option<String>,
    ) -> Self {
        Self {
            store_root: store_root.to_path_buf(),
            manifest_path: store_root.join("indexes").join("manifest.json"),
            status: "ok".to_string(),
            maintainer: maintainer.to_string(),
            profile_id,
            doc_id,
            source: None,
            current_profiles: Vec::new(),
            current_documents: Vec::new(),
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

fn print_view_maintainer_text(summary: &ViewMaintainerSummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    println!("maintainer: {}", summary.maintainer);
    if let Some(profile_id) = &summary.profile_id {
        println!("profile id: {profile_id}");
    }
    if let Some(doc_id) = &summary.doc_id {
        println!("doc id: {doc_id}");
    }
    if let Some(source) = summary.source {
        println!(
            "source: {}",
            match source {
                GovernanceMaintainerSummarySource::Persisted => "persisted",
                GovernanceMaintainerSummarySource::Synthesized => "synthesized",
            }
        );
    }
    println!("current profile count: {}", summary.current_profiles.len());
    for profile in &summary.current_profiles {
        println!(
            "current profile: {} view={} timestamp={}",
            profile.profile_id, profile.current_view_id, profile.timestamp
        );
        for (doc_id, revision_id) in &profile.documents {
            println!("profile document: {} -> {}", doc_id, revision_id);
        }
        for current in &profile.current_documents {
            println!(
                "profile current document: {} view={} revision={} maintainer={} timestamp={}",
                current.doc_id,
                current.current_view_id,
                current.current_revision_id,
                current.maintainer,
                current.timestamp
            );
        }
    }
    println!(
        "current document count: {}",
        summary.current_documents.len()
    );
    for document in &summary.current_documents {
        println!("current document: {}", document.doc_id);
        for profile in &document.profiles {
            println!(
                "document profile: {} view={} revision={} maintainer={} timestamp={}",
                profile.profile_id,
                profile.current_view_id,
                profile.current_revision_id,
                profile.maintainer,
                profile.timestamp
            );
        }
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("view maintainer: ok");
        0
    } else {
        println!("view maintainer: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_view_maintainer_json(summary: &ViewMaintainerSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("view maintainer summary", source)),
    }
}

pub(super) fn handle(args: ViewMaintainerCliArgs) -> Result<i32, CliError> {
    let ViewMaintainerCliArgs {
        store_root,
        maintainer,
        profile_id,
        doc_id,
        json,
        extra: _,
    } = args;
    let store_root = PathBuf::from(store_root);

    let mut summary =
        ViewMaintainerSummary::new(&store_root, &maintainer, profile_id.clone(), doc_id.clone());
    let manifest = match load_store_index_manifest(&store_root) {
        Ok(manifest) => manifest,
        Err(error) => {
            summary.push_error(format!("failed to read store index manifest: {error}"));
            return if json {
                print_view_maintainer_json(&summary)
            } else {
                Ok(print_view_maintainer_text(&summary))
            };
        }
    };

    match inspect_current_maintainer_governance(
        &manifest,
        &maintainer,
        profile_id.as_deref(),
        doc_id.as_deref(),
    ) {
        Ok(maintainer_summary) => {
            summary.source = Some(maintainer_summary.source);
            match maintainer_summary.source {
                GovernanceMaintainerSummarySource::Persisted => summary.notes.push(
                    "current maintainer governance is read from persisted maintainer-governance summaries instead of rebuilding maintainer coverage at query time"
                        .to_string(),
                ),
                GovernanceMaintainerSummarySource::Synthesized => summary.notes.push(
                    "current maintainer governance was synthesized from persisted profile/document governance state because the persisted maintainer-governance summary was unavailable"
                        .to_string(),
                ),
            }
            summary.current_profiles = maintainer_summary
                .current_profiles
                .into_iter()
                .map(|profile| ViewMaintainerProfileSummary {
                    profile_id: profile.profile_id,
                    current_view_id: profile.current_view_id,
                    timestamp: profile.timestamp,
                    documents: profile.documents,
                    current_documents: profile
                        .current_documents
                        .into_iter()
                        .map(|current| ViewMaintainerCurrentDocumentSummary {
                            doc_id: current.doc_id,
                            current_view_id: current.current_view_id,
                            current_revision_id: current.current_revision_id,
                            maintainer: current.maintainer,
                            timestamp: current.timestamp,
                        })
                        .collect(),
                })
                .collect();
            summary.current_documents = maintainer_summary
                .current_documents
                .into_iter()
                .map(|document| ViewMaintainerDocumentSummary {
                    doc_id: document.doc_id,
                    profiles: document
                        .profiles
                        .into_iter()
                        .map(|profile| ViewMaintainerDocumentProfileSummary {
                            profile_id: profile.profile_id,
                            current_view_id: profile.current_view_id,
                            current_revision_id: profile.current_revision_id,
                            maintainer: profile.maintainer,
                            timestamp: profile.timestamp,
                        })
                        .collect(),
                })
                .collect();
        }
        Err(error) => {
            summary.push_error(error.to_string());
        }
    }

    summary.notes.push(
        "maintainer-centric governance inspection complements profile-centric view current and document-centric view document output"
            .to_string(),
    );

    if json {
        print_view_maintainer_json(&summary)
    } else {
        Ok(print_view_maintainer_text(&summary))
    }
}
