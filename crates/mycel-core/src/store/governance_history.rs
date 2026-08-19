use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::governance_diff::diff_governance_records;
use super::{
    GovernanceViewDiffSummary, StoreIndexManifest, StoreRebuildError, ViewGovernanceRecord,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernanceHistoryIndex {
    pub profile_view_ids: BTreeMap<String, Vec<String>>,
    pub document_view_ids: BTreeMap<String, Vec<String>>,
    pub profile_document_view_ids: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceViewHistorySource {
    Persisted,
    Synthesized,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GovernanceViewHistoryRecord {
    pub view_id: String,
    pub maintainer: String,
    pub profile_id: String,
    pub governance_policy: Option<Value>,
    pub timestamp: u64,
    pub documents: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GovernanceViewHistorySummary {
    pub source: GovernanceViewHistorySource,
    pub profile_id: Option<String>,
    pub doc_id: Option<String>,
    pub timestamp_min: Option<u64>,
    pub timestamp_max: Option<u64>,
    pub record_count: usize,
    pub transition_count: usize,
    pub records: Vec<GovernanceViewHistoryRecord>,
    pub transitions: Vec<GovernanceViewDiffSummary>,
}

pub(super) fn build_governance_history_index(
    records: &[ViewGovernanceRecord],
) -> GovernanceHistoryIndex {
    let mut ordered_records = records.iter().collect::<Vec<_>>();
    ordered_records.sort_by(|left, right| {
        left.timestamp
            .cmp(&right.timestamp)
            .then_with(|| left.view_id.cmp(&right.view_id))
    });

    let mut index = GovernanceHistoryIndex::default();
    for record in ordered_records {
        index
            .profile_view_ids
            .entry(record.profile_id.clone())
            .or_default()
            .push(record.view_id.clone());
        for doc_id in record.documents.keys() {
            index
                .document_view_ids
                .entry(doc_id.clone())
                .or_default()
                .push(record.view_id.clone());
            index
                .profile_document_view_ids
                .entry(record.profile_id.clone())
                .or_default()
                .entry(doc_id.clone())
                .or_default()
                .push(record.view_id.clone());
        }
    }
    index
}

impl GovernanceViewHistorySummary {
    pub fn has_semantic_changes(&self) -> bool {
        self.transitions
            .iter()
            .any(GovernanceViewDiffSummary::has_semantic_change)
    }
}

fn matches_scope(
    record: &ViewGovernanceRecord,
    profile_id: Option<&str>,
    doc_id: Option<&str>,
    timestamp_min: Option<u64>,
    timestamp_max: Option<u64>,
) -> bool {
    let profile_matches = match profile_id {
        Some(profile_id) => record.profile_id == profile_id,
        None => true,
    };
    let document_matches = match doc_id {
        Some(doc_id) => record.documents.contains_key(doc_id),
        None => true,
    };
    let minimum_matches = match timestamp_min {
        Some(timestamp_min) => record.timestamp >= timestamp_min,
        None => true,
    };
    let maximum_matches = match timestamp_max {
        Some(timestamp_max) => record.timestamp <= timestamp_max,
        None => true,
    };
    profile_matches && document_matches && minimum_matches && maximum_matches
}

pub fn governance_view_history(
    manifest: &StoreIndexManifest,
    profile_id: Option<&str>,
    doc_id: Option<&str>,
    timestamp_min: Option<u64>,
    timestamp_max: Option<u64>,
) -> Result<GovernanceViewHistorySummary, StoreRebuildError> {
    if profile_id.is_none() && doc_id.is_none() {
        return Err(StoreRebuildError::new(
            "governance history requires a profile ID, document ID, or both",
        ));
    }
    if timestamp_min
        .zip(timestamp_max)
        .is_some_and(|(min, max)| min > max)
    {
        return Err(StoreRebuildError::new(
            "governance history timestamp minimum cannot be greater than timestamp maximum",
        ));
    }

    let synthesized_index;
    let (history_index, source) = match &manifest.governance_history {
        Some(index) => (index, GovernanceViewHistorySource::Persisted),
        None => {
            synthesized_index = build_governance_history_index(&manifest.view_governance);
            (&synthesized_index, GovernanceViewHistorySource::Synthesized)
        }
    };
    let indexed_view_ids = match (profile_id, doc_id) {
        (Some(profile_id), Some(doc_id)) => history_index
            .profile_document_view_ids
            .get(profile_id)
            .and_then(|documents| documents.get(doc_id)),
        (Some(profile_id), None) => history_index.profile_view_ids.get(profile_id),
        (None, Some(doc_id)) => history_index.document_view_ids.get(doc_id),
        (None, None) => unreachable!("unscoped governance history is rejected above"),
    };
    let records_by_id = manifest
        .view_governance
        .iter()
        .map(|record| (record.view_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    let mut matching_records = Vec::new();
    for view_id in indexed_view_ids.into_iter().flatten() {
        let record = records_by_id
            .get(view_id.as_str())
            .copied()
            .ok_or_else(|| {
                StoreRebuildError::new(format!(
                    "governance history index references missing view '{}'",
                    view_id
                ))
            })?;
        if !matches_scope(record, profile_id, doc_id, None, None) {
            return Err(StoreRebuildError::new(format!(
                "governance history index references out-of-scope view '{}'",
                view_id
            )));
        }
        if matches_scope(record, profile_id, doc_id, timestamp_min, timestamp_max) {
            matching_records.push(record);
        }
    }

    let transitions = matching_records
        .windows(2)
        .map(|pair| diff_governance_records(manifest, pair[0], pair[1], doc_id))
        .collect::<Vec<_>>();
    let records = matching_records
        .into_iter()
        .map(|record| GovernanceViewHistoryRecord {
            view_id: record.view_id.clone(),
            maintainer: record.maintainer.clone(),
            profile_id: record.profile_id.clone(),
            governance_policy: manifest
                .governance_profiles
                .get(&record.profile_id)
                .cloned(),
            timestamp: record.timestamp,
            documents: record.documents.clone(),
        })
        .collect::<Vec<_>>();

    Ok(GovernanceViewHistorySummary {
        source,
        profile_id: profile_id.map(str::to_string),
        doc_id: doc_id.map(str::to_string),
        timestamp_min,
        timestamp_max,
        record_count: records.len(),
        transition_count: transitions.len(),
        records,
        transitions,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;

    const LATE_TIMESTAMP: u64 = 30;

    fn record(
        view_id: &str,
        maintainer: &str,
        profile_id: &str,
        timestamp: u64,
        documents: &[(&str, &str)],
    ) -> ViewGovernanceRecord {
        ViewGovernanceRecord {
            view_id: view_id.to_string(),
            maintainer: maintainer.to_string(),
            profile_id: profile_id.to_string(),
            timestamp,
            documents: documents
                .iter()
                .map(|(doc_id, revision_id)| (doc_id.to_string(), revision_id.to_string()))
                .collect(),
            accepted_editor_keys: Vec::new(),
            maintainer_is_admitted_editor: false,
            admitted_editor_only_keys: Vec::new(),
        }
    }

    fn manifest() -> StoreIndexManifest {
        let mut manifest = StoreIndexManifest {
            version: "mycel-store-index/0.1".to_string(),
            stored_object_count: 0,
            object_ids_by_type: BTreeMap::new(),
            doc_revisions: BTreeMap::new(),
            revision_parents: BTreeMap::new(),
            author_patches: BTreeMap::new(),
            view_governance: vec![
                record(
                    "view:z",
                    "pk:second",
                    "hash:policy-a",
                    LATE_TIMESTAMP,
                    &[("doc:alpha", "rev:3"), ("doc:beta", "rev:b2")],
                ),
                record(
                    "view:a",
                    "pk:first",
                    "hash:policy-a",
                    10,
                    &[("doc:alpha", "rev:1"), ("doc:beta", "rev:b1")],
                ),
                record(
                    "view:m",
                    "pk:second",
                    "hash:policy-b",
                    20,
                    &[("doc:alpha", "rev:2"), ("doc:gamma", "rev:g1")],
                ),
            ],
            governance_history: None,
            governance_profiles: BTreeMap::from([
                ("hash:policy-a".to_string(), json!({"mode": "stable"})),
                ("hash:policy-b".to_string(), json!({"mode": "review"})),
            ]),
            maintainer_views: BTreeMap::new(),
            profile_views: BTreeMap::new(),
            document_views: BTreeMap::new(),
            latest_profile_views: BTreeMap::new(),
            latest_document_profile_views: BTreeMap::new(),
            current_governance: BTreeMap::new(),
            current_document_governance: BTreeMap::new(),
            current_maintainer_governance: BTreeMap::new(),
            profile_heads: BTreeMap::new(),
            doc_heads: BTreeMap::new(),
        };
        manifest.governance_history =
            Some(build_governance_history_index(&manifest.view_governance));
        manifest
    }

    #[test]
    fn document_history_is_chronological_and_document_scoped() {
        let history = governance_view_history(
            &manifest(),
            None,
            Some("doc:alpha"),
            Some(10),
            Some(LATE_TIMESTAMP),
        )
        .expect("document history should build");

        assert_eq!(history.record_count, 3);
        assert_eq!(history.source, GovernanceViewHistorySource::Persisted);
        assert_eq!(history.transition_count, 2);
        assert_eq!(
            history
                .records
                .iter()
                .map(|record| record.view_id.as_str())
                .collect::<Vec<_>>(),
            vec!["view:a", "view:m", "view:z"]
        );
        assert_eq!(
            history.records[1].governance_policy,
            Some(json!({"mode": "review"}))
        );
        assert!(history.transitions[0].policy_changed);
        assert_eq!(history.transitions[0].document_changes.len(), 1);
        assert_eq!(
            history.transitions[0].document_changes[0].doc_id,
            "doc:alpha"
        );
        assert_eq!(history.transitions[1].document_changes.len(), 1);
        assert_eq!(
            history.transitions[1].document_changes[0].doc_id,
            "doc:alpha"
        );
        assert!(history.has_semantic_changes());
    }

    #[test]
    fn profile_history_applies_inclusive_timestamp_range() {
        let history = governance_view_history(
            &manifest(),
            Some("hash:policy-a"),
            None,
            Some(LATE_TIMESTAMP),
            Some(LATE_TIMESTAMP),
        )
        .expect("profile history should build");

        assert_eq!(history.record_count, 1);
        assert_eq!(history.transition_count, 0);
        assert_eq!(history.records[0].view_id, "view:z");
        assert!(!history.has_semantic_changes());
    }

    #[test]
    fn persisted_history_index_keeps_profile_document_traversal_chronological() {
        let manifest = manifest();
        let index = manifest
            .governance_history
            .as_ref()
            .expect("history index should exist");
        assert_eq!(
            index.profile_document_view_ids["hash:policy-a"]["doc:alpha"],
            vec!["view:a", "view:z"]
        );

        let history = governance_view_history(
            &manifest,
            Some("hash:policy-a"),
            Some("doc:alpha"),
            None,
            None,
        )
        .expect("profile-document history should build");

        assert_eq!(history.source, GovernanceViewHistorySource::Persisted);
        assert_eq!(history.record_count, 2);
        assert_eq!(history.records[0].view_id, "view:a");
        assert_eq!(history.records[1].view_id, "view:z");
    }

    #[test]
    fn timestamp_only_history_is_not_a_semantic_change() {
        let mut manifest = manifest();
        manifest.view_governance = vec![
            record(
                "view:early",
                "pk:first",
                "hash:policy-a",
                10,
                &[("doc:alpha", "rev:1")],
            ),
            record(
                "view:late",
                "pk:first",
                "hash:policy-a",
                20,
                &[("doc:alpha", "rev:1")],
            ),
        ];
        manifest.governance_history =
            Some(build_governance_history_index(&manifest.view_governance));

        let history = governance_view_history(&manifest, None, Some("doc:alpha"), None, None)
            .expect("timestamp-only history should build");

        assert_eq!(history.transition_count, 1);
        assert!(history.transitions[0].timestamp_changed);
        assert!(history.transitions[0].is_different());
        assert!(!history.transitions[0].has_semantic_change());
        assert!(!history.has_semantic_changes());
    }

    #[test]
    fn legacy_history_synthesizes_the_missing_persisted_index() {
        let mut manifest = manifest();
        manifest.governance_history = None;

        let history = governance_view_history(&manifest, None, Some("doc:alpha"), None, None)
            .expect("legacy history should synthesize its traversal index");

        assert_eq!(history.source, GovernanceViewHistorySource::Synthesized);
        assert_eq!(history.record_count, 3);
        assert_eq!(history.records[0].view_id, "view:a");
        assert_eq!(history.records[2].view_id, "view:z");
    }

    #[test]
    fn persisted_history_rejects_missing_view_references() {
        let mut manifest = manifest();
        manifest
            .governance_history
            .as_mut()
            .expect("history index should exist")
            .document_view_ids
            .get_mut("doc:alpha")
            .expect("document history should exist")
            .push("view:missing".to_string());

        let error = governance_view_history(&manifest, None, Some("doc:alpha"), None, None)
            .expect_err("missing persisted history reference should fail");

        assert!(error.to_string().contains("references missing view"));
    }

    #[test]
    fn history_rejects_unscoped_and_inverted_queries() {
        let unscoped = governance_view_history(&manifest(), None, None, None, None)
            .expect_err("unscoped history should fail");
        assert!(unscoped.to_string().contains("requires a profile ID"));

        let inverted = governance_view_history(
            &manifest(),
            Some("hash:policy-a"),
            None,
            Some(LATE_TIMESTAMP + 1),
            Some(LATE_TIMESTAMP),
        )
        .expect_err("inverted history range should fail");
        assert!(inverted.to_string().contains("minimum cannot be greater"));
    }
}
