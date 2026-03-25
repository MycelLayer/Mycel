use super::*;

#[test]
fn store_merge_authoring_flow_reports_added_metadata_from_non_primary_parent_as_multi_variant() {
    let doc_id = "doc:author-smoke-metadata-added";
    let flow = VariantScenarioFlow::new(
        "store-merge-metadata-added-root",
        "store-merge-metadata-added-key",
        doc_id,
        "Author Smoke Metadata Added",
        "40",
    );
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_for_doc_file(
        "store-merge-metadata-added-state",
        doc_id,
        "right",
    );
    let (_right_ops_dir, right_ops_path) =
        write_metadata_variant_ops_file("store-merge-metadata-added-right-ops", "right");
    let resolved_state_file = path_arg(&resolved_state_path);
    let right_ops_file = path_arg(&right_ops_path);

    let right_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &right_ops_file, "41", "42");
    let merge_json = flow.create_merge_revision(
        &[flow.genesis_revision_id(), &right_revision_id],
        &resolved_state_file,
        "43",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("metadata key 'topic' adopted a non-primary parent addition")
                })
            })),
        "expected metadata added-from-parent multi-variant reason, got {merge_json}"
    );
    assert!(
        !merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "metadata key 'topic' kept the primary variant while multiple competing non-primary additions remained",
                    )
                })
            })),
        "did not expect competing metadata reason with only one alternative, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-addition"
            })),
        "expected adopted non-primary metadata addition detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(2)
    );
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_metadata_over_non_primary_addition() {
    let doc_id = "doc:author-smoke-metadata-keep-primary";
    let flow = VariantScenarioFlow::new(
        "store-merge-metadata-keep-primary-root",
        "store-merge-metadata-keep-primary-key",
        doc_id,
        "Author Smoke Metadata Keep Primary",
        "40",
    );
    let (_right_ops_dir, right_ops_path) =
        write_metadata_variant_ops_file("store-merge-metadata-keep-primary-right-ops", "right");
    let (_empty_resolved_dir, empty_resolved_path) = write_empty_resolved_state_for_doc_file(
        "store-merge-metadata-keep-primary-empty-state",
        doc_id,
    );
    let right_ops_file = path_arg(&right_ops_path);
    let resolved_state_file = path_arg(&empty_resolved_path);

    let right_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &right_ops_file, "41", "42");
    let merge_json = flow.create_merge_revision(
        &[flow.genesis_revision_id(), &right_revision_id],
        &resolved_state_file,
        "43",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("metadata key 'topic' kept the primary absence over a competing non-primary addition")
                })
            })),
        "expected metadata keep-primary multi-variant reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_and_multiple_competing_metadata_additions() {
    let doc_id = "doc:author-smoke-metadata-keep-primary-multiple";
    let flow = VariantScenarioFlow::new(
        "store-merge-metadata-keep-primary-multiple-root",
        "store-merge-metadata-keep-primary-multiple-key",
        doc_id,
        "Author Smoke Metadata Keep Primary Multiple",
        "50",
    );
    let (_right_ops_dir, right_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-keep-primary-multiple-right-ops",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-keep-primary-multiple-center-ops",
        "center",
    );
    let (_empty_resolved_dir, empty_resolved_path) = write_empty_resolved_state_for_doc_file(
        "store-merge-metadata-keep-primary-multiple-empty-state",
        doc_id,
    );
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);
    let resolved_state_file = path_arg(&empty_resolved_path);

    let right_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &right_ops_file, "51", "52");
    let center_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &center_ops_file, "53", "54");
    let merge_json = flow.create_merge_revision(
        &[
            flow.genesis_revision_id(),
            &right_revision_id,
            &center_revision_id,
        ],
        &resolved_state_file,
        "55",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("metadata key 'topic' kept the primary absence over a competing non-primary addition")
                })
            })),
        "expected keep-primary metadata reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "metadata key 'topic' kept the primary variant while multiple competing non-primary additions remained",
                    )
                })
            })),
        "expected multiple-competing metadata reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-absence-over-non-primary-addition"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected keep-primary metadata detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected multiple-competing metadata detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

#[test]
fn store_merge_authoring_flow_preserves_duplicate_non_primary_metadata_additions_when_keeping_primary_absence(
) {
    let doc_id = "doc:author-smoke-metadata-keep-primary-duplicate-additions";
    let flow = VariantScenarioFlow::new(
        "store-merge-metadata-keep-primary-duplicate-additions-root",
        "store-merge-metadata-keep-primary-duplicate-additions-key",
        doc_id,
        "Author Smoke Metadata Keep Primary Duplicate Additions",
        "68",
    );
    let (_right_ops_dir, right_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-keep-primary-duplicate-additions-right-ops",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_metadata_variant_ops_file(
        "store-merge-metadata-keep-primary-duplicate-additions-center-ops",
        "right",
    );
    let (_empty_resolved_dir, empty_resolved_path) = write_empty_resolved_state_for_doc_file(
        "store-merge-metadata-keep-primary-duplicate-additions-empty-state",
        doc_id,
    );
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);
    let resolved_state_file = path_arg(&empty_resolved_path);

    let right_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &right_ops_file, "69", "70");
    let center_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &center_ops_file, "71", "72");
    let merge_json = flow.create_merge_revision(
        &[
            flow.genesis_revision_id(),
            &right_revision_id,
            &center_revision_id,
        ],
        &resolved_state_file,
        "73",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-absence-over-non-primary-addition"
                    && detail["competing_variants"] == json!(["\"right\"", "\"right\""])
            })),
        "expected keep-primary duplicate metadata additions detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"] == json!(["\"right\"", "\"right\""])
            })),
        "expected multiple competing duplicate metadata additions detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}
