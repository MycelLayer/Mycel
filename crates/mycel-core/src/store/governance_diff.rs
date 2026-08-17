use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::Value;

use super::{StoreIndexManifest, StoreRebuildError, ViewGovernanceRecord};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceViewDiffComparison {
    Match,
    Different,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceDocumentChangeKind {
    Added,
    Removed,
    Changed,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GovernanceDocumentChange {
    pub doc_id: String,
    pub kind: GovernanceDocumentChangeKind,
    pub base_revision_id: Option<String>,
    pub target_revision_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GovernanceViewDiffSummary {
    pub comparison: GovernanceViewDiffComparison,
    pub difference_count: usize,
    pub base_view_id: String,
    pub target_view_id: String,
    pub base_maintainer: String,
    pub target_maintainer: String,
    pub maintainer_changed: bool,
    pub base_profile_id: String,
    pub target_profile_id: String,
    pub policy_changed: bool,
    pub base_governance_policy: Option<Value>,
    pub target_governance_policy: Option<Value>,
    pub base_timestamp: u64,
    pub target_timestamp: u64,
    pub timestamp_changed: bool,
    pub unchanged_document_count: usize,
    pub document_changes: Vec<GovernanceDocumentChange>,
}

impl GovernanceViewDiffSummary {
    pub fn is_different(&self) -> bool {
        self.comparison == GovernanceViewDiffComparison::Different
    }
}

fn find_view_record<'a>(
    manifest: &'a StoreIndexManifest,
    view_id: &str,
) -> Result<&'a ViewGovernanceRecord, StoreRebuildError> {
    manifest
        .view_governance
        .iter()
        .find(|record| record.view_id == view_id)
        .ok_or_else(|| {
            StoreRebuildError::new(format!(
                "view '{}' was not found in persisted governance indexes",
                view_id
            ))
        })
}

pub fn diff_governance_views(
    manifest: &StoreIndexManifest,
    base_view_id: &str,
    target_view_id: &str,
) -> Result<GovernanceViewDiffSummary, StoreRebuildError> {
    let base = find_view_record(manifest, base_view_id)?;
    let target = find_view_record(manifest, target_view_id)?;
    Ok(diff_governance_records(manifest, base, target, None))
}

pub(super) fn diff_governance_records(
    manifest: &StoreIndexManifest,
    base: &ViewGovernanceRecord,
    target: &ViewGovernanceRecord,
    doc_id: Option<&str>,
) -> GovernanceViewDiffSummary {
    let maintainer_changed = base.maintainer != target.maintainer;
    let policy_changed = base.profile_id != target.profile_id;
    let timestamp_changed = base.timestamp != target.timestamp;
    let mut unchanged_document_count = 0;
    let mut document_changes = Vec::new();

    let doc_ids = match doc_id {
        Some(doc_id) => BTreeSet::from([doc_id.to_string()]),
        None => base
            .documents
            .keys()
            .chain(target.documents.keys())
            .cloned()
            .collect::<BTreeSet<_>>(),
    };
    for doc_id in doc_ids {
        let base_revision_id = base.documents.get(&doc_id);
        let target_revision_id = target.documents.get(&doc_id);
        let kind = match (base_revision_id, target_revision_id) {
            (Some(base_revision_id), Some(target_revision_id))
                if base_revision_id == target_revision_id =>
            {
                unchanged_document_count += 1;
                continue;
            }
            (Some(_), Some(_)) => GovernanceDocumentChangeKind::Changed,
            (Some(_), None) => GovernanceDocumentChangeKind::Removed,
            (None, Some(_)) => GovernanceDocumentChangeKind::Added,
            (None, None) => continue,
        };
        document_changes.push(GovernanceDocumentChange {
            doc_id,
            kind,
            base_revision_id: base_revision_id.cloned(),
            target_revision_id: target_revision_id.cloned(),
        });
    }

    let difference_count = usize::from(maintainer_changed)
        + usize::from(policy_changed)
        + usize::from(timestamp_changed)
        + document_changes.len();
    let comparison = if difference_count == 0 {
        GovernanceViewDiffComparison::Match
    } else {
        GovernanceViewDiffComparison::Different
    };

    GovernanceViewDiffSummary {
        comparison,
        difference_count,
        base_view_id: base.view_id.clone(),
        target_view_id: target.view_id.clone(),
        base_maintainer: base.maintainer.clone(),
        target_maintainer: target.maintainer.clone(),
        maintainer_changed,
        base_profile_id: base.profile_id.clone(),
        target_profile_id: target.profile_id.clone(),
        policy_changed,
        base_governance_policy: manifest.governance_profiles.get(&base.profile_id).cloned(),
        target_governance_policy: manifest
            .governance_profiles
            .get(&target.profile_id)
            .cloned(),
        base_timestamp: base.timestamp,
        target_timestamp: target.timestamp,
        timestamp_changed,
        unchanged_document_count,
        document_changes,
    }
}
