use super::*;

struct VariantScenarioFlow {
    flow: StoreAuthoringFlow,
    doc_id: &'static str,
    genesis_revision_id: String,
}

impl VariantScenarioFlow {
    fn new(
        store_prefix: &str,
        key_prefix: &str,
        doc_id: &'static str,
        title: &str,
        timestamp: &str,
    ) -> Self {
        let flow = StoreAuthoringFlow::new(store_prefix, key_prefix);
        let genesis_revision_id = flow.create_document(doc_id, title, "en", timestamp);
        Self {
            flow,
            doc_id,
            genesis_revision_id,
        }
    }

    fn genesis_revision_id(&self) -> &str {
        &self.genesis_revision_id
    }

    fn commit_ops_revision(
        &self,
        base_revision_id: &str,
        ops_file: &str,
        patch_timestamp: &str,
        revision_timestamp: &str,
    ) -> String {
        let patch_id =
            self.flow
                .create_patch(self.doc_id, base_revision_id, ops_file, patch_timestamp);
        self.flow
            .commit_revision(self.doc_id, base_revision_id, &patch_id, revision_timestamp)
    }

    fn create_merge_revision(
        &self,
        parent_revision_ids: &[&str],
        resolved_state_file: &str,
        timestamp: &str,
    ) -> serde_json::Value {
        self.flow.create_merge_revision(
            self.doc_id,
            parent_revision_ids,
            resolved_state_file,
            timestamp,
        )
    }
}

fn write_empty_resolved_state_for_doc_file(
    prefix: &str,
    doc_id: &str,
) -> (common::TempDir, PathBuf) {
    let dir = create_temp_dir(prefix);
    let path = dir.path().join("resolved-state.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "doc_id": doc_id,
            "blocks": [],
            "metadata": {}
        }))
        .expect("empty resolved state JSON should serialize"),
    )
    .expect("empty resolved state JSON should write");
    (dir, path)
}

fn assert_content_variant_merge_reasons(merge_json: &serde_json::Value) {
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("adopted a non-primary parent replacement")
                })
            })),
        "expected content variant multi-variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "selected one non-primary replacement while other competing non-primary replacements remained",
                    )
                })
            })),
        "expected competing content variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_kind"] == "block"
                    && detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-replacement"
                    && detail["resolved_variant"]
                        .as_str()
                        .is_some_and(|variant| variant.contains("Right variant"))
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 1)
            })),
        "expected structured content variant detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_kind"] == "block"
                    && detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-replacements"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected competing content branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(3)
    );
}

fn assert_duplicate_non_primary_content_replacement_reasons(merge_json: &serde_json::Value) {
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| {
                            variants.len() == 1
                                && variants.iter().all(|variant| {
                                    variant.as_str().is_some_and(|variant| {
                                        variant.contains("\"content\":\"right\"")
                                    })
                                })
                        })
            })),
        "expected selected duplicate content replacement detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-replacements"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| {
                            variants.len() == 2
                                && variants.iter().all(|variant| {
                                    variant.as_str().is_some_and(|variant| {
                                        variant.contains("\"content\":\"right\"")
                                    })
                                })
                        })
            })),
        "expected duplicate competing content replacements detail, got {merge_json}"
    );
}

fn assert_duplicate_non_primary_metadata_replacement_reasons(merge_json: &serde_json::Value) {
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["competing_variants"] == json!(["\"right\""])
            })),
        "expected selected duplicate metadata replacement detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-replacements"
                    && detail["competing_variants"] == json!(["\"right\"", "\"right\""])
            })),
        "expected duplicate competing metadata replacements detail, got {merge_json}"
    );
}

mod additions;
mod mixed_branches;
mod replacements;
