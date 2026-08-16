use std::path::{Path, PathBuf};

use mycel_core::store::{
    diff_governance_views, load_store_index_manifest, GovernanceDocumentChangeKind,
    GovernanceViewDiffComparison, GovernanceViewDiffSummary,
};
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::ViewDiffCliArgs;

#[derive(Debug, Clone, Serialize)]
struct ViewDiffCliSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    fail_on_diff: bool,
    result: Option<GovernanceViewDiffSummary>,
    notes: Vec<String>,
    errors: Vec<String>,
}

impl ViewDiffCliSummary {
    fn new(store_root: &Path, fail_on_diff: bool) -> Self {
        Self {
            store_root: store_root.to_path_buf(),
            manifest_path: store_root.join("indexes").join("manifest.json"),
            status: "ok".to_string(),
            fail_on_diff,
            result: None,
            notes: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.status = "failed".to_string();
        self.errors.push(message.into());
    }

    fn exit_code(&self) -> i32 {
        if !self.errors.is_empty()
            || (self.fail_on_diff
                && self
                    .result
                    .as_ref()
                    .is_some_and(GovernanceViewDiffSummary::is_different))
        {
            1
        } else {
            0
        }
    }
}

fn change_kind_label(kind: GovernanceDocumentChangeKind) -> &'static str {
    match kind {
        GovernanceDocumentChangeKind::Added => "added",
        GovernanceDocumentChangeKind::Removed => "removed",
        GovernanceDocumentChangeKind::Changed => "changed",
    }
}

fn comparison_label(comparison: GovernanceViewDiffComparison) -> &'static str {
    match comparison {
        GovernanceViewDiffComparison::Match => "match",
        GovernanceViewDiffComparison::Different => "different",
    }
}

fn print_text(summary: &ViewDiffCliSummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    println!("fail on diff: {}", summary.fail_on_diff);
    if let Some(result) = &summary.result {
        println!("comparison: {}", comparison_label(result.comparison));
        println!("difference count: {}", result.difference_count);
        println!("base view id: {}", result.base_view_id);
        println!("target view id: {}", result.target_view_id);
        println!("base maintainer: {}", result.base_maintainer);
        println!("target maintainer: {}", result.target_maintainer);
        println!("maintainer changed: {}", result.maintainer_changed);
        println!("base profile id: {}", result.base_profile_id);
        println!("target profile id: {}", result.target_profile_id);
        println!("policy changed: {}", result.policy_changed);
        println!("base timestamp: {}", result.base_timestamp);
        println!("target timestamp: {}", result.target_timestamp);
        println!("timestamp changed: {}", result.timestamp_changed);
        println!(
            "unchanged document count: {}",
            result.unchanged_document_count
        );
        println!("document change count: {}", result.document_changes.len());
        for change in &result.document_changes {
            println!(
                "document change: {} kind={} base={} target={}",
                change.doc_id,
                change_kind_label(change.kind),
                change.base_revision_id.as_deref().unwrap_or("-"),
                change.target_revision_id.as_deref().unwrap_or("-"),
            );
        }
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.errors.is_empty() {
        println!("view diff: ok");
    } else {
        println!("view diff: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
    }
    summary.exit_code()
}

fn print_json(summary: &ViewDiffCliSummary) -> Result<i32, CliError> {
    let json = serde_json::to_string_pretty(summary)
        .map_err(|source| CliError::serialization("view diff summary", source))?;
    println!("{json}");
    Ok(summary.exit_code())
}

pub(super) fn handle(args: ViewDiffCliArgs) -> Result<i32, CliError> {
    let ViewDiffCliArgs {
        base_view_id,
        target_view_id,
        store_root,
        fail_on_diff,
        json,
        extra: _,
    } = args;
    let store_root = PathBuf::from(store_root);
    let mut summary = ViewDiffCliSummary::new(&store_root, fail_on_diff);

    match load_store_index_manifest(&store_root) {
        Ok(manifest) => match diff_governance_views(&manifest, &base_view_id, &target_view_id) {
            Ok(result) => summary.result = Some(result),
            Err(error) => summary.push_error(error.to_string()),
        },
        Err(error) => summary.push_error(format!("failed to read store index manifest: {error}")),
    }
    summary.notes.push(
        "policy changes are determined from canonical profile IDs; catalog policy values remain optional for legacy manifests"
            .to_string(),
    );
    summary.notes.push(
        "governance View comparison is separate from reader-facing accepted-head selection"
            .to_string(),
    );

    if json {
        print_json(&summary)
    } else {
        Ok(print_text(&summary))
    }
}
