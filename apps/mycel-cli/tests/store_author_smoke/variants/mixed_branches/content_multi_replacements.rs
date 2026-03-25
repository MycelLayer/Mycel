use super::*;

#[test]
fn store_merge_authoring_flow_reports_selected_content_replacement_with_multiple_replacements_and_removal(
) {
    let doc_id = "doc:author-smoke-content-select-many";
    let flow = VariantScenarioFlow::new(
        "store-merge-content-select-many-root",
        "store-merge-content-select-many-key",
        doc_id,
        "Author Smoke Content Select Many",
        "110",
    );
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-select-many-state",
        doc_id,
        &[("blk:author-smoke-select-many-001", "Right A")],
    );
    let (_base_ops_dir, base_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-select-many-base-ops",
        "blk:author-smoke-select-many-001",
        "Base",
    );
    let (_replace_a_ops_dir, replace_a_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-select-many-replace-a-ops",
        "blk:author-smoke-select-many-001",
        "Right A",
    );
    let (_replace_b_ops_dir, replace_b_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-select-many-replace-b-ops",
        "blk:author-smoke-select-many-001",
        "Right B",
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_a_ops_file = path_arg(&replace_a_ops_path);
    let replace_b_ops_file = path_arg(&replace_b_ops_path);

    let base_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &base_ops_file, "111", "112");
    let replace_a_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_a_ops_file, "113", "114");
    let replace_b_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_b_ops_file, "115", "116");
    let merge_json = flow.create_merge_revision(
        &[
            &base_revision_id,
            &replace_a_revision_id,
            &replace_b_revision_id,
            flow.genesis_revision_id(),
        ],
        &resolved_state_file,
        "117",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"].as_array().is_some_and(|reasons| reasons
            .iter()
            .any(|reason| reason.as_str().is_some_and(|reason| reason.contains(
                "block 'blk:author-smoke-select-many-001' adopted a non-primary parent replacement while competing non-primary replacements and a removal remained"
            )))),
        "expected richer selected content replacement reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-select-many-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"]
                        == "adopted-non-primary-replacement-while-competing-replacements-and-removal-remain"
            })),
        "expected richer selected content branch detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-select-many-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"]
                        == "multiple-competing-non-primary-replacements-and-removals"
            })),
        "expected richer competing content branch detail, got {merge_json}"
    );
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_content_over_multiple_replacements_and_removals()
{
    let doc_id = "doc:author-smoke-content-keep-many";
    let flow = VariantScenarioFlow::new(
        "store-merge-content-keep-many-root",
        "store-merge-content-keep-many-key",
        doc_id,
        "Author Smoke Content Keep Many",
        "118",
    );
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-keep-many-state",
        doc_id,
        &[("blk:author-smoke-keep-many-001", "Base")],
    );
    let (_base_ops_dir, base_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-keep-many-base-ops",
        "blk:author-smoke-keep-many-001",
        "Base",
    );
    let (_replace_a_ops_dir, replace_a_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-keep-many-replace-a-ops",
        "blk:author-smoke-keep-many-001",
        "Right A",
    );
    let (_replace_b_ops_dir, replace_b_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-keep-many-replace-b-ops",
        "blk:author-smoke-keep-many-001",
        "Right B",
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_a_ops_file = path_arg(&replace_a_ops_path);
    let replace_b_ops_file = path_arg(&replace_b_ops_path);

    let base_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &base_ops_file, "119", "120");
    let replace_a_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_a_ops_file, "121", "122");
    let replace_b_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_b_ops_file, "123", "124");
    let merge_json = flow.create_merge_revision(
        &[
            &base_revision_id,
            &replace_a_revision_id,
            &replace_b_revision_id,
            flow.genesis_revision_id(),
        ],
        &resolved_state_file,
        "125",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"].as_array().is_some_and(|reasons| reasons
            .iter()
            .any(|reason| reason.as_str().is_some_and(|reason| reason.contains(
                "block 'blk:author-smoke-keep-many-001' kept the primary parent variant over multiple competing non-primary replacements and removals"
            )))),
        "expected richer kept-primary content reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-keep-many-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"]
                        == "kept-primary-variant-over-multiple-competing-non-primary-replacements-and-removals"
            })),
        "expected richer kept-primary content branch detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-keep-many-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"]
                        == "multiple-competing-non-primary-replacements-and-removals"
            })),
        "expected richer multiple competing kept-primary content branch detail, got {merge_json}"
    );
}
