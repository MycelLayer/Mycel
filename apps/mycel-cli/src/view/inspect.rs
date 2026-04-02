use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mycel_core::store::{
    inspect_current_governance, inspect_governance_view, load_store_index_manifest,
    GovernanceCurrentSummarySource,
};
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::ViewInspectCliArgs;

#[derive(Debug, Clone, Serialize)]
struct ViewInspectSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    view_id: String,
    maintainer: Option<String>,
    profile_id: Option<String>,
    timestamp: Option<u64>,
    accepted_editor_keys: Vec<String>,
    maintainer_is_admitted_editor: bool,
    admitted_editor_only_keys: Vec<String>,
    current_profile_view_id: Option<String>,
    current_profile_document_view_ids: BTreeMap<String, String>,
    current_profile_source: Option<String>,
    current_profile_maintainer: Option<String>,
    current_profile_timestamp: Option<u64>,
    is_current_profile_view: Option<bool>,
    is_current_document_view_ids: BTreeMap<String, bool>,
    documents: BTreeMap<String, String>,
    profile_heads: BTreeMap<String, Vec<String>>,
    maintainer_view_ids: Vec<String>,
    profile_view_ids: Vec<String>,
    document_view_ids: BTreeMap<String, Vec<String>>,
    notes: Vec<String>,
    errors: Vec<String>,
}

impl ViewInspectSummary {
    fn new(store_root: &Path, view_id: &str) -> Self {
        Self {
            store_root: store_root.to_path_buf(),
            manifest_path: store_root.join("indexes").join("manifest.json"),
            status: "ok".to_string(),
            view_id: view_id.to_string(),
            maintainer: None,
            profile_id: None,
            timestamp: None,
            accepted_editor_keys: Vec::new(),
            maintainer_is_admitted_editor: false,
            admitted_editor_only_keys: Vec::new(),
            current_profile_view_id: None,
            current_profile_document_view_ids: BTreeMap::new(),
            current_profile_source: None,
            current_profile_maintainer: None,
            current_profile_timestamp: None,
            is_current_profile_view: None,
            is_current_document_view_ids: BTreeMap::new(),
            documents: BTreeMap::new(),
            profile_heads: BTreeMap::new(),
            maintainer_view_ids: Vec::new(),
            profile_view_ids: Vec::new(),
            document_view_ids: BTreeMap::new(),
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

fn print_view_inspect_text(summary: &ViewInspectSummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    println!("view id: {}", summary.view_id);
    if let Some(maintainer) = &summary.maintainer {
        println!("maintainer: {maintainer}");
    }
    if let Some(profile_id) = &summary.profile_id {
        println!("profile id: {profile_id}");
    }
    if let Some(timestamp) = summary.timestamp {
        println!("timestamp: {timestamp}");
    }
    println!(
        "accepted editor key count: {}",
        summary.accepted_editor_keys.len()
    );
    if !summary.accepted_editor_keys.is_empty() {
        println!(
            "accepted editor keys: {}",
            summary.accepted_editor_keys.join(", ")
        );
    }
    println!(
        "maintainer is admitted editor: {}",
        summary.maintainer_is_admitted_editor
    );
    println!(
        "admitted editor-only key count: {}",
        summary.admitted_editor_only_keys.len()
    );
    if !summary.admitted_editor_only_keys.is_empty() {
        println!(
            "admitted editor-only keys: {}",
            summary.admitted_editor_only_keys.join(", ")
        );
    }
    if let Some(current_profile_view_id) = &summary.current_profile_view_id {
        println!("current profile view id: {current_profile_view_id}");
    }
    if let Some(current_profile_source) = &summary.current_profile_source {
        println!("current profile source: {current_profile_source}");
    }
    if let Some(current_profile_maintainer) = &summary.current_profile_maintainer {
        println!("current profile maintainer: {current_profile_maintainer}");
    }
    if let Some(current_profile_timestamp) = summary.current_profile_timestamp {
        println!("current profile timestamp: {current_profile_timestamp}");
    }
    if let Some(is_current_profile_view) = summary.is_current_profile_view {
        println!("is current profile view: {is_current_profile_view}");
    }
    println!("document count: {}", summary.documents.len());
    for (doc_id, revision_id) in &summary.documents {
        println!("document: {doc_id} -> {revision_id}");
    }
    println!(
        "current profile document view count: {}",
        summary.current_profile_document_view_ids.len()
    );
    for (doc_id, current_view_id) in &summary.current_profile_document_view_ids {
        println!("current profile document view: {doc_id} -> {current_view_id}");
    }
    if !summary.is_current_document_view_ids.is_empty() {
        println!(
            "is current document view count: {}",
            summary.is_current_document_view_ids.len()
        );
        for (doc_id, is_current) in &summary.is_current_document_view_ids {
            println!("is current document view: {doc_id} -> {is_current}");
        }
    }
    println!("profile head doc count: {}", summary.profile_heads.len());
    for (doc_id, revision_ids) in &summary.profile_heads {
        println!("profile heads: {doc_id} -> {}", revision_ids.join(", "));
    }
    println!(
        "maintainer related view count: {}",
        summary.maintainer_view_ids.len()
    );
    if !summary.maintainer_view_ids.is_empty() {
        println!(
            "maintainer related views: {}",
            summary.maintainer_view_ids.join(", ")
        );
    }
    println!(
        "profile related view count: {}",
        summary.profile_view_ids.len()
    );
    if !summary.profile_view_ids.is_empty() {
        println!(
            "profile related views: {}",
            summary.profile_view_ids.join(", ")
        );
    }
    println!(
        "document related view doc count: {}",
        summary.document_view_ids.len()
    );
    for (doc_id, view_ids) in &summary.document_view_ids {
        println!(
            "document related views: {doc_id} -> {}",
            view_ids.join(", ")
        );
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("view inspection: ok");
        0
    } else {
        println!("view inspection: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_view_inspect_json(summary: &ViewInspectSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("view inspection summary", source)),
    }
}

fn current_source_label(source: GovernanceCurrentSummarySource) -> &'static str {
    match source {
        GovernanceCurrentSummarySource::Persisted => "persisted",
        GovernanceCurrentSummarySource::Synthesized => "synthesized",
    }
}

pub(super) fn handle(args: ViewInspectCliArgs) -> Result<i32, CliError> {
    let ViewInspectCliArgs {
        view_id,
        store_root,
        json,
        extra: _,
    } = args;
    let store_root = PathBuf::from(store_root);

    let mut summary = ViewInspectSummary::new(&store_root, &view_id);
    let manifest = match load_store_index_manifest(&store_root) {
        Ok(manifest) => manifest,
        Err(error) => {
            summary.push_error(format!("failed to read store index manifest: {error}"));
            return if json {
                print_view_inspect_json(&summary)
            } else {
                Ok(print_view_inspect_text(&summary))
            };
        }
    };
    match inspect_governance_view(&manifest, &view_id) {
        Ok(inspection) => {
            summary.accepted_editor_keys = inspection.accepted_editor_keys;
            summary.maintainer_is_admitted_editor = inspection.maintainer_is_admitted_editor;
            summary.admitted_editor_only_keys = inspection.admitted_editor_only_keys;
            summary.maintainer = Some(inspection.maintainer);
            summary.profile_id = Some(inspection.profile_id);
            summary.timestamp = Some(inspection.timestamp);
            summary.current_profile_view_id = inspection.current_profile_view_id;
            summary.current_profile_document_view_ids =
                inspection.current_profile_document_view_ids;
            summary.is_current_profile_view = summary
                .current_profile_view_id
                .as_ref()
                .map(|current_view_id| current_view_id == &summary.view_id);
            summary.is_current_document_view_ids = inspection
                .documents
                .keys()
                .map(|doc_id| {
                    let is_current = summary
                        .current_profile_document_view_ids
                        .get(doc_id)
                        .is_some_and(|current_view_id| current_view_id == &summary.view_id);
                    (doc_id.clone(), is_current)
                })
                .collect();
            summary.documents = inspection.documents;
            summary.profile_heads = inspection.profile_heads;
            summary.maintainer_view_ids = inspection.maintainer_view_ids;
            summary.profile_view_ids = inspection.profile_view_ids;
            summary.document_view_ids = inspection.document_view_ids;
            match inspect_current_governance(
                &manifest,
                summary
                    .profile_id
                    .as_deref()
                    .expect("profile id should exist after successful view inspection"),
                None,
            ) {
                Ok(current) => {
                    summary.current_profile_source =
                        Some(current_source_label(current.source).to_string());
                    summary.current_profile_maintainer = Some(current.maintainer);
                    summary.current_profile_timestamp = Some(current.timestamp);
                }
                Err(error) => summary.notes.push(format!(
                    "current profile governance summary was unavailable while inspecting this view: {error}"
                )),
            }
        }
        Err(error) => {
            summary.push_error(error.to_string());
        }
    }
    summary.notes.push(
        "governance inspection is separate from reader-facing accepted-head workflows".to_string(),
    );
    summary.notes.push(
        "related maintainer/profile/document view IDs come from persisted governance indexes"
            .to_string(),
    );
    summary.notes.push(
        "current profile governance state comes from persisted governance summaries and latest-view indexes"
            .to_string(),
    );
    summary.notes.push(
        "accepted editor keys come from persisted governance view summaries so mixed-role and shared-key assignments stay inspectable without re-reading the stored view body"
            .to_string(),
    );

    if json {
        print_view_inspect_json(&summary)
    } else {
        Ok(print_view_inspect_text(&summary))
    }
}
