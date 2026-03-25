use super::*;

#[test]
fn store_merge_authoring_flow_reports_selected_replacement_with_competing_removal_branch() {
    let doc_id = "doc:author-smoke-content-select-replace-with-removal";
    let flow = VariantScenarioFlow::new(
        "store-merge-content-select-replace-with-removal-root",
        "store-merge-content-select-replace-with-removal-key",
        doc_id,
        "Author Smoke Content Select Replace With Removal",
        "78",
    );
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-select-replace-with-removal-state",
        doc_id,
        &[("blk:author-smoke-select-001", "Right")],
    );
    let (_base_ops_dir, base_ops_path) = write_insert_block_ops_file(
        "store-merge-content-select-replace-with-removal-base-ops",
        "blk:author-smoke-select-001",
        "Base",
    );
    let (_replace_ops_dir, replace_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-select-replace-with-removal-replace-ops",
        "blk:author-smoke-select-001",
        "Right",
    );
    let (_delete_ops_dir, delete_ops_path) = write_content_delete_ops_for_block_file(
        "store-merge-content-select-replace-with-removal-delete-ops",
        "blk:author-smoke-select-001",
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_ops_file = path_arg(&replace_ops_path);
    let delete_ops_file = path_arg(&delete_ops_path);

    let base_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &base_ops_file, "79", "80");
    let replace_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_ops_file, "81", "82");
    let delete_revision_id =
        flow.commit_ops_revision(&base_revision_id, &delete_ops_file, "83", "84");
    let merge_json = flow.create_merge_revision(
        &[&base_revision_id, &replace_revision_id, &delete_revision_id],
        &resolved_state_file,
        "85",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-select-001' adopted a non-primary parent replacement while a competing non-primary removal remained",
                    )
                })
            })),
        "expected mixed selected replacement reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-select-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"]
                        == "adopted-non-primary-replacement-while-competing-removal-remains"
            })),
        "expected mixed selected replacement branch kind detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-select-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-mixed-non-primary-alternatives"
            })),
        "expected mixed competing content branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}

#[test]
fn store_merge_authoring_flow_reports_selected_metadata_replacement_with_competing_removal_branch()
{
    let doc_id = "doc:author-smoke-metadata-select-replace-with-removal";
    let flow = VariantScenarioFlow::new(
        "store-merge-metadata-select-replace-with-removal-root",
        "store-merge-metadata-select-replace-with-removal-key",
        doc_id,
        "Author Smoke Metadata Select Replace With Removal",
        "86",
    );
    let (_resolved_dir, resolved_state_path) = write_metadata_variant_resolved_state_for_doc_file(
        "store-merge-metadata-select-replace-with-removal-state",
        doc_id,
        "right",
    );
    let (_base_ops_dir, base_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-select-replace-with-removal-base-ops",
        &[("topic", "base")],
    );
    let (_replace_ops_dir, replace_ops_path) = write_metadata_entries_ops_file(
        "store-merge-metadata-select-replace-with-removal-replace-ops",
        &[("topic", "right")],
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_ops_file = path_arg(&replace_ops_path);

    let base_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &base_ops_file, "87", "88");
    let replace_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_ops_file, "89", "90");
    let merge_json = flow.create_merge_revision(
        &[
            &base_revision_id,
            &replace_revision_id,
            flow.genesis_revision_id(),
        ],
        &resolved_state_file,
        "91",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "metadata key 'topic' adopted a non-primary parent replacement while a competing non-primary removal remained",
                    )
                })
            })),
        "expected mixed selected metadata replacement reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"]
                        == "adopted-non-primary-replacement-while-competing-removal-remains"
            })),
        "expected mixed selected metadata replacement branch kind detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-mixed-non-primary-alternatives"
            })),
        "expected mixed competing metadata branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}
