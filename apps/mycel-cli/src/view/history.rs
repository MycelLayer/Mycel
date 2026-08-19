use std::path::{Path, PathBuf};

use mycel_core::store::{
    governance_view_history, load_store_index_manifest, GovernanceViewDiffSummary,
    GovernanceViewHistorySource, GovernanceViewHistorySummary,
};
use serde::Serialize;

use crate::{emit_error_line, CliError};

use super::ViewHistoryCliArgs;

#[derive(Debug, Clone, Serialize)]
struct ViewHistoryCliSummary {
    store_root: PathBuf,
    manifest_path: PathBuf,
    status: String,
    fail_on_change: bool,
    semantic_change_detected: bool,
    result: Option<GovernanceViewHistorySummary>,
    notes: Vec<String>,
    errors: Vec<String>,
}

impl ViewHistoryCliSummary {
    fn new(store_root: &Path, fail_on_change: bool) -> Self {
        Self {
            store_root: store_root.to_path_buf(),
            manifest_path: store_root.join("indexes").join("manifest.json"),
            status: "ok".to_string(),
            fail_on_change,
            semantic_change_detected: false,
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
        i32::from(!self.errors.is_empty() || (self.fail_on_change && self.semantic_change_detected))
    }
}

fn optional_label(value: Option<&str>) -> &str {
    value.unwrap_or("-")
}

fn optional_timestamp_label(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn print_transition(transition: &GovernanceViewDiffSummary) {
    println!(
        "history transition: {} -> {} differences={} maintainer-changed={} policy-changed={}",
        transition.base_view_id,
        transition.target_view_id,
        transition.difference_count,
        transition.maintainer_changed,
        transition.policy_changed,
    );
    for change in &transition.document_changes {
        println!(
            "history document change: {} base={} target={}",
            change.doc_id,
            optional_label(change.base_revision_id.as_deref()),
            optional_label(change.target_revision_id.as_deref()),
        );
    }
}

fn print_text(summary: &ViewHistoryCliSummary) -> i32 {
    println!("store root: {}", summary.store_root.display());
    println!("manifest path: {}", summary.manifest_path.display());
    println!("fail on change: {}", summary.fail_on_change);
    println!(
        "semantic change detected: {}",
        summary.semantic_change_detected
    );
    if let Some(result) = &summary.result {
        println!(
            "profile id: {}",
            optional_label(result.profile_id.as_deref())
        );
        println!("document id: {}", optional_label(result.doc_id.as_deref()));
        println!(
            "timestamp minimum: {}",
            optional_timestamp_label(result.timestamp_min)
        );
        println!(
            "timestamp maximum: {}",
            optional_timestamp_label(result.timestamp_max)
        );
        println!("history record count: {}", result.record_count);
        println!("history transition count: {}", result.transition_count);
        println!(
            "history source: {}",
            match result.source {
                GovernanceViewHistorySource::Persisted => "persisted",
                GovernanceViewHistorySource::Synthesized => "synthesized",
            }
        );
        for record in &result.records {
            println!(
                "history record: timestamp={} view={} maintainer={} profile={} documents={}",
                record.timestamp,
                record.view_id,
                record.maintainer,
                record.profile_id,
                record.documents.len(),
            );
        }
        for transition in &result.transitions {
            print_transition(transition);
        }
    }
    for note in &summary.notes {
        println!("note: {note}");
    }

    if summary.errors.is_empty() {
        println!("view history: ok");
    } else {
        println!("view history: failed");
        for error in &summary.errors {
            emit_error_line(error);
        }
    }
    summary.exit_code()
}

fn print_json(summary: &ViewHistoryCliSummary) -> Result<i32, CliError> {
    let json = serde_json::to_string_pretty(summary)
        .map_err(|source| CliError::serialization("view history summary", source))?;
    println!("{json}");
    Ok(summary.exit_code())
}

pub(super) fn handle(args: ViewHistoryCliArgs) -> Result<i32, CliError> {
    let ViewHistoryCliArgs {
        store_root,
        profile_id,
        doc_id,
        timestamp_min,
        timestamp_max,
        fail_on_change,
        json,
        extra: _,
    } = args;
    if timestamp_min
        .zip(timestamp_max)
        .is_some_and(|(min, max)| min > max)
    {
        return Err(CliError::usage(
            "view history timestamp-min cannot be greater than timestamp-max".to_string(),
        ));
    }

    let store_root = PathBuf::from(store_root);
    let mut summary = ViewHistoryCliSummary::new(&store_root, fail_on_change);
    match load_store_index_manifest(&store_root) {
        Ok(manifest) => match governance_view_history(
            &manifest,
            profile_id.as_deref(),
            doc_id.as_deref(),
            timestamp_min,
            timestamp_max,
        ) {
            Ok(result) => {
                summary.semantic_change_detected = result.has_semantic_changes();
                summary.notes.push(match result.source {
                    GovernanceViewHistorySource::Persisted => {
                        "history traversal used the persisted governance history index".to_string()
                    }
                    GovernanceViewHistorySource::Synthesized => {
                        "history traversal synthesized an index from legacy manifest records; rebuild the store to persist it"
                            .to_string()
                    }
                });
                summary.result = Some(result);
            }
            Err(error) => summary.push_error(error.to_string()),
        },
        Err(error) => summary.push_error(format!("failed to read store index manifest: {error}")),
    }
    summary.notes.push(
        "records are ordered by timestamp and View ID; transitions compare adjacent records"
            .to_string(),
    );
    summary.notes.push(
        "governance history is separate from reader-facing accepted-head selection".to_string(),
    );
    summary.notes.push(
        "semantic change detection includes maintainer, policy, and document mapping changes but ignores timestamp-only transitions"
            .to_string(),
    );

    if json {
        print_json(&summary)
    } else {
        Ok(print_text(&summary))
    }
}
