use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mycel_core::store::{
    inspect_current_governance, list_current_governance, load_store_index_manifest,
    GovernanceCurrentSummarySource,
};
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::ViewCurrentCliArgs;

#[derive(Debug, Clone, Serialize)]
pub(super) struct ViewCurrentDocumentSummary {
    doc_id: String,
    current_view_id: String,
    current_revision_id: String,
    maintainer: String,
    timestamp: u64,
    accepted_editor_keys: Vec<String>,
    maintainer_is_admitted_editor: bool,
    admitted_editor_only_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ViewCurrentProfileSummary {
    profile_id: String,
    source: Option<GovernanceCurrentSummarySource>,
    current_view_id: Option<String>,
    profile_current_view_id: Option<String>,
    maintainer: Option<String>,
    timestamp: Option<u64>,
    current_document_revision_id: Option<String>,
    accepted_editor_keys: Vec<String>,
    maintainer_is_admitted_editor: bool,
    admitted_editor_only_keys: Vec<String>,
    documents: BTreeMap<String, String>,
    current_profile_document_view_ids: BTreeMap<String, String>,
    current_documents: Vec<ViewCurrentDocumentSummary>,
    profile_heads: BTreeMap<String, Vec<String>>,
}

impl ViewCurrentProfileSummary {
    fn new(profile_id: &str) -> Self {
        Self {
            profile_id: profile_id.to_string(),
            source: None,
            current_view_id: None,
            profile_current_view_id: None,
            maintainer: None,
            timestamp: None,
            current_document_revision_id: None,
            accepted_editor_keys: Vec::new(),
            maintainer_is_admitted_editor: false,
            admitted_editor_only_keys: Vec::new(),
            documents: BTreeMap::new(),
            current_profile_document_view_ids: BTreeMap::new(),
            current_documents: Vec::new(),
            profile_heads: BTreeMap::new(),
        }
    }

    fn from_core(current: mycel_core::store::GovernanceCurrentSummary) -> Self {
        Self {
            profile_id: current.profile_id,
            source: Some(current.source),
            current_view_id: Some(current.current_view_id),
            profile_current_view_id: Some(current.profile_current_view_id),
            maintainer: Some(current.maintainer),
            timestamp: Some(current.timestamp),
            current_document_revision_id: current.current_document_revision_id,
            accepted_editor_keys: current.accepted_editor_keys,
            maintainer_is_admitted_editor: current.maintainer_is_admitted_editor,
            admitted_editor_only_keys: current.admitted_editor_only_keys,
            documents: current.documents,
            current_profile_document_view_ids: current.current_profile_document_view_ids,
            current_documents: current
                .current_documents
                .into_iter()
                .map(|current_document| ViewCurrentDocumentSummary {
                    doc_id: current_document.doc_id,
                    current_view_id: current_document.current_view_id,
                    current_revision_id: current_document.current_revision_id,
                    maintainer: current_document.maintainer,
                    timestamp: current_document.timestamp,
                    accepted_editor_keys: current_document.accepted_editor_keys,
                    maintainer_is_admitted_editor: current_document.maintainer_is_admitted_editor,
                    admitted_editor_only_keys: current_document.admitted_editor_only_keys,
                })
                .collect(),
            profile_heads: current.profile_heads,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ViewCurrentSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    doc_id: Option<String>,
    #[serde(flatten)]
    profile: ViewCurrentProfileSummary,
    notes: Vec<String>,
    errors: Vec<String>,
}

impl ViewCurrentSummary {
    fn new(store_root: &Path, profile_id: &str, doc_id: Option<String>) -> Self {
        Self {
            store_root: store_root.to_path_buf(),
            manifest_path: store_root.join("indexes").join("manifest.json"),
            status: "ok".to_string(),
            doc_id,
            profile: ViewCurrentProfileSummary::new(profile_id),
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

    fn apply_current(&mut self, current: mycel_core::store::GovernanceCurrentSummary) {
        self.profile = ViewCurrentProfileSummary::from_core(current);
    }
}

#[derive(Debug, Clone, Serialize)]
struct ViewCurrentListSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    doc_id: Option<String>,
    profile_count: usize,
    profiles: Vec<ViewCurrentProfileSummary>,
    notes: Vec<String>,
    errors: Vec<String>,
}

impl ViewCurrentListSummary {
    fn new(store_root: &Path, doc_id: Option<String>) -> Self {
        Self {
            store_root: store_root.to_path_buf(),
            manifest_path: store_root.join("indexes").join("manifest.json"),
            status: "ok".to_string(),
            doc_id,
            profile_count: 0,
            profiles: Vec::new(),
            notes: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn set_profiles(&mut self, profiles: Vec<ViewCurrentProfileSummary>) {
        self.profile_count = profiles.len();
        self.profiles = profiles;
    }

    fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.status = "failed".to_string();
        self.errors.push(message.into());
    }
}

fn print_profile_text(summary: &ViewCurrentProfileSummary) {
    println!("profile id: {}", summary.profile_id);
    if let Some(source) = summary.source {
        println!(
            "source: {}",
            match source {
                GovernanceCurrentSummarySource::Persisted => "persisted",
                GovernanceCurrentSummarySource::Synthesized => "synthesized",
            }
        );
    }
    if let Some(current_view_id) = &summary.current_view_id {
        println!("current view id: {current_view_id}");
    }
    if let Some(profile_current_view_id) = &summary.profile_current_view_id {
        println!("profile current view id: {profile_current_view_id}");
    }
    if let Some(maintainer) = &summary.maintainer {
        println!("maintainer: {maintainer}");
    }
    if let Some(timestamp) = summary.timestamp {
        println!("timestamp: {timestamp}");
    }
    if let Some(current_document_revision_id) = &summary.current_document_revision_id {
        println!("current document revision id: {current_document_revision_id}");
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
    println!(
        "current profile document view count: {}",
        summary.current_profile_document_view_ids.len()
    );
    for (doc_id, current_view_id) in &summary.current_profile_document_view_ids {
        println!("current profile document view: {doc_id} -> {current_view_id}");
    }
    println!(
        "current document summary count: {}",
        summary.current_documents.len()
    );
    for current in &summary.current_documents {
        println!(
            "current document: {} view={} revision={} maintainer={} timestamp={} admitted_editors={} maintainer_is_admitted_editor={} editor_only_keys={}",
            current.doc_id,
            current.current_view_id,
            current.current_revision_id,
            current.maintainer,
            current.timestamp,
            current.accepted_editor_keys.join(", "),
            current.maintainer_is_admitted_editor,
            current.admitted_editor_only_keys.join(", "),
        );
    }
    println!("profile head doc count: {}", summary.profile_heads.len());
    for (doc_id, revision_ids) in &summary.profile_heads {
        println!("profile heads: {doc_id} -> {}", revision_ids.join(", "));
    }
    println!("document count: {}", summary.documents.len());
    for (doc_id, revision_id) in &summary.documents {
        println!("document: {doc_id} -> {revision_id}");
    }
}

fn print_view_current_text(summary: &ViewCurrentSummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    if let Some(doc_id) = &summary.doc_id {
        println!("doc id: {doc_id}");
    }
    print_profile_text(&summary.profile);
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("view current: ok");
        0
    } else {
        println!("view current: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_view_current_list_text(summary: &ViewCurrentListSummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    if let Some(doc_id) = &summary.doc_id {
        println!("doc id: {doc_id}");
    }
    println!(
        "current governance profile count: {}",
        summary.profile_count
    );
    for profile in &summary.profiles {
        print_profile_text(profile);
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.is_ok() {
        println!("view current: ok");
        0
    } else {
        println!("view current: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_view_current_json(summary: &ViewCurrentSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("view current summary", source)),
    }
}

fn print_view_current_list_json(summary: &ViewCurrentListSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("view current summary", source)),
    }
}

fn push_current_source_note(notes: &mut Vec<String>, source: GovernanceCurrentSummarySource) {
    match source {
        GovernanceCurrentSummarySource::Persisted => notes.push(
            "current governance state is read from persisted governance summaries instead of replaying all stored views"
                .to_string(),
        ),
        GovernanceCurrentSummarySource::Synthesized => notes.push(
            "current governance state was synthesized from persisted latest profile/document indexes because the persisted current-governance summary was unavailable"
                .to_string(),
        ),
    }
}

fn push_current_list_source_note(notes: &mut Vec<String>, profiles: &[ViewCurrentProfileSummary]) {
    let has_persisted = profiles.iter().any(|profile| {
        matches!(
            profile.source,
            Some(GovernanceCurrentSummarySource::Persisted)
        )
    });
    let has_synthesized = profiles.iter().any(|profile| {
        matches!(
            profile.source,
            Some(GovernanceCurrentSummarySource::Synthesized)
        )
    });

    match (has_persisted, has_synthesized) {
        (true, false) => push_current_source_note(notes, GovernanceCurrentSummarySource::Persisted),
        (false, true) => {
            push_current_source_note(notes, GovernanceCurrentSummarySource::Synthesized)
        }
        (true, true) => notes.push(
            "current governance profiles mix persisted summaries with synthesized fallback when persisted current-governance entries are unavailable"
                .to_string(),
        ),
        (false, false) => {}
    }
}

pub(super) fn handle(args: ViewCurrentCliArgs) -> Result<i32, CliError> {
    let ViewCurrentCliArgs {
        store_root,
        profile_id,
        all_profiles,
        doc_id,
        json,
        extra: _,
    } = args;
    let store_root = PathBuf::from(store_root);

    let manifest = match load_store_index_manifest(&store_root) {
        Ok(manifest) => manifest,
        Err(error) => {
            let message = format!("failed to read store index manifest: {error}");
            return if all_profiles {
                let mut summary = ViewCurrentListSummary::new(&store_root, doc_id.clone());
                summary.push_error(message);
                if json {
                    print_view_current_list_json(&summary)
                } else {
                    Ok(print_view_current_list_text(&summary))
                }
            } else {
                let fallback_profile_id = profile_id
                    .as_deref()
                    .expect("clap should require --profile-id unless --all-profiles is present");
                let mut summary =
                    ViewCurrentSummary::new(&store_root, fallback_profile_id, doc_id.clone());
                summary.push_error(message);
                if json {
                    print_view_current_json(&summary)
                } else {
                    Ok(print_view_current_text(&summary))
                }
            };
        }
    };

    if all_profiles {
        let mut summary = ViewCurrentListSummary::new(&store_root, doc_id.clone());
        match list_current_governance(&manifest, doc_id.as_deref()) {
            Ok(profiles) => {
                summary.set_profiles(
                    profiles
                        .into_iter()
                        .map(ViewCurrentProfileSummary::from_core)
                        .collect(),
                );
            }
            Err(error) => {
                summary.push_error(error.to_string());
            }
        }
        push_current_list_source_note(&mut summary.notes, &summary.profiles);
        summary.notes.push(
            "profiles are emitted in deterministic profile-id order so tooling can diff repeated runs"
                .to_string(),
        );
        if doc_id.is_some() {
            summary.notes.push(
                "doc-scoped current governance only includes profiles whose persisted current state mentions the selected document"
                    .to_string(),
            );
        }
        summary.notes.push(
            "accepted editor keys come from persisted governance summaries so editor-maintainer and view-maintainer assignments stay visible without re-reading stored views"
                .to_string(),
        );
        return if json {
            print_view_current_list_json(&summary)
        } else {
            Ok(print_view_current_list_text(&summary))
        };
    }

    let profile_id =
        profile_id.expect("clap should require --profile-id unless --all-profiles is present");
    let mut summary = ViewCurrentSummary::new(&store_root, &profile_id, doc_id.clone());

    match inspect_current_governance(&manifest, &profile_id, doc_id.as_deref()) {
        Ok(current) => {
            summary.apply_current(current);
        }
        Err(error) => {
            summary.push_error(error.to_string());
        }
    }
    if let Some(source) = summary.profile.source {
        push_current_source_note(&mut summary.notes, source);
    }
    summary.notes.push(
        "profile head IDs come from persisted governance head indexes for the selected profile"
            .to_string(),
    );
    if doc_id.is_some() {
        summary.notes.push(
            "doc-scoped current governance may differ from the latest profile-wide view when a newer view does not mention that document"
                .to_string(),
        );
    }
    summary.notes.push(
        "accepted editor keys come from persisted governance summaries so editor-maintainer and view-maintainer assignments stay visible without re-reading stored views"
            .to_string(),
    );

    if json {
        print_view_current_json(&summary)
    } else {
        Ok(print_view_current_text(&summary))
    }
}
