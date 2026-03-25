use super::*;

const CONTENT_VARIANT_DOC_ID: &str = "doc:author-smoke-content-variant";
const CONTENT_VARIANT_TITLE: &str = "Author Smoke Content Variant";

#[test]
fn store_merge_authoring_flow_reports_block_added_from_non_primary_parent_as_multi_variant() {
    let flow = VariantScenarioFlow::new(
        "store-merge-content-added-root",
        "store-merge-content-added-key",
        CONTENT_VARIANT_DOC_ID,
        CONTENT_VARIANT_TITLE,
        "40",
    );
    let (_resolved_dir, resolved_state_path) =
        write_content_variant_resolved_state_file("store-merge-content-added-state", "right");
    let (_right_ops_dir, right_ops_path) =
        write_content_addition_ops_file("store-merge-content-added-right-ops", "right");
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
                    reason.contains("block 'blk:author-smoke-variant-001' adopted a non-primary parent addition")
                })
            })),
        "expected added-from-parent content reason, got {merge_json}"
    );
    assert!(
        !merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-variant-001' kept the primary variant while multiple competing non-primary additions remained",
                    )
                })
            })),
        "did not expect competing content reason with only one alternative, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-addition"
            })),
        "expected adopted non-primary content addition detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(2)
    );
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_absence_over_non_primary_block_addition() {
    let flow = VariantScenarioFlow::new(
        "store-merge-content-keep-primary-root",
        "store-merge-content-keep-primary-key",
        CONTENT_VARIANT_DOC_ID,
        CONTENT_VARIANT_TITLE,
        "40",
    );
    let (_right_ops_dir, right_ops_path) =
        write_content_addition_ops_file("store-merge-content-keep-primary-right-ops", "right");
    let (_empty_resolved_dir, empty_resolved_path) = write_empty_resolved_state_for_doc_file(
        "store-merge-content-keep-primary-empty-state",
        CONTENT_VARIANT_DOC_ID,
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
                    reason.contains(
                        "block 'blk:author-smoke-variant-001' kept the primary absence over a competing non-primary addition",
                    )
                })
            })),
        "expected keep-primary content reason, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_and_multiple_competing_block_additions() {
    let doc_id = "doc:author-smoke-content-keep-primary-multiple";
    let flow = VariantScenarioFlow::new(
        "store-merge-content-keep-primary-multiple-root",
        "store-merge-content-keep-primary-multiple-key",
        doc_id,
        "Author Smoke Content Keep Primary Multiple",
        "50",
    );
    let (_right_ops_dir, right_ops_path) = write_content_addition_ops_file(
        "store-merge-content-keep-primary-multiple-right-ops",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_content_addition_ops_file(
        "store-merge-content-keep-primary-multiple-center-ops",
        "center",
    );
    let (_empty_resolved_dir, empty_resolved_path) = write_empty_resolved_state_for_doc_file(
        "store-merge-content-keep-primary-multiple-empty-state",
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
                    reason.contains(
                        "block 'blk:author-smoke-variant-001' kept the primary absence over a competing non-primary addition",
                    )
                })
            })),
        "expected keep-primary content reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-variant-001' kept the primary variant while multiple competing non-primary additions remained",
                    )
                })
            })),
        "expected multiple-competing content reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-absence-over-non-primary-addition"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected keep-primary content detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected multiple-competing content detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

#[test]
fn store_merge_authoring_flow_preserves_duplicate_non_primary_content_additions_when_keeping_primary_absence(
) {
    let doc_id = "doc:author-smoke-content-keep-primary-duplicate-additions";
    let flow = VariantScenarioFlow::new(
        "store-merge-content-keep-primary-duplicate-additions-root",
        "store-merge-content-keep-primary-duplicate-additions-key",
        doc_id,
        "Author Smoke Content Keep Primary Duplicate Additions",
        "56",
    );
    let (_right_ops_dir, right_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-keep-primary-duplicate-additions-right-ops",
        "blk:author-smoke-added-keep-duplicate",
        "right",
    );
    let (_center_ops_dir, center_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-keep-primary-duplicate-additions-center-ops",
        "blk:author-smoke-added-keep-duplicate",
        "right",
    );
    let (_empty_resolved_dir, empty_resolved_path) = write_empty_resolved_state_for_doc_file(
        "store-merge-content-keep-primary-duplicate-additions-empty-state",
        doc_id,
    );
    let right_ops_file = path_arg(&right_ops_path);
    let center_ops_file = path_arg(&center_ops_path);
    let resolved_state_file = path_arg(&empty_resolved_path);

    let right_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &right_ops_file, "57", "58");
    let center_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &center_ops_file, "59", "60");
    let merge_json = flow.create_merge_revision(
        &[
            flow.genesis_revision_id(),
            &right_revision_id,
            &center_revision_id,
        ],
        &resolved_state_file,
        "61",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-added-keep-duplicate"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-absence-over-non-primary-addition"
                    && detail["competing_variants"] == json!([
                        "{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-keep-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}",
                        "{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-keep-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}"
                    ])
            })),
        "expected keep-primary duplicate content additions detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-added-keep-duplicate"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-additions"
                    && detail["competing_variants"] == json!([
                        "{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-keep-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}",
                        "{\"attrs\":{},\"block_id\":\"blk:author-smoke-added-keep-duplicate\",\"block_type\":\"paragraph\",\"content\":\"right\"}"
                    ])
            })),
        "expected multiple competing duplicate content additions detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}
