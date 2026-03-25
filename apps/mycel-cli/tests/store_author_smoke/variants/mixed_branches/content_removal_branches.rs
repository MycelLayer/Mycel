use super::*;

#[test]
fn store_merge_authoring_flow_reports_mixed_content_replacement_and_removal_branches() {
    let doc_id = "doc:author-smoke-content-mixed-replace-remove";
    let flow = VariantScenarioFlow::new(
        "store-merge-content-mixed-replace-remove-root",
        "store-merge-content-mixed-replace-remove-key",
        doc_id,
        "Author Smoke Content Mixed Replace Remove",
        "70",
    );
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-mixed-replace-remove-state",
        doc_id,
        &[("blk:author-smoke-mixed-001", "Base")],
    );
    let (_base_ops_dir, base_ops_path) = write_insert_block_ops_file(
        "store-merge-content-mixed-replace-remove-base-ops",
        "blk:author-smoke-mixed-001",
        "Base",
    );
    let (_replace_ops_dir, replace_ops_path) = write_content_variant_ops_for_block_file(
        "store-merge-content-mixed-replace-remove-replace-ops",
        "blk:author-smoke-mixed-001",
        "Right",
    );
    let (_delete_ops_dir, delete_ops_path) = write_content_delete_ops_for_block_file(
        "store-merge-content-mixed-replace-remove-delete-ops",
        "blk:author-smoke-mixed-001",
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let replace_ops_file = path_arg(&replace_ops_path);
    let delete_ops_file = path_arg(&delete_ops_path);

    let base_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &base_ops_file, "71", "72");
    let replace_revision_id =
        flow.commit_ops_revision(&base_revision_id, &replace_ops_file, "73", "74");
    let delete_revision_id =
        flow.commit_ops_revision(&base_revision_id, &delete_ops_file, "75", "76");
    let merge_json = flow.create_merge_revision(
        &[&base_revision_id, &replace_revision_id, &delete_revision_id],
        &resolved_state_file,
        "77",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-mixed-001' kept the primary parent variant over mixed competing non-primary alternatives",
                    )
                })
            })),
        "expected mixed keep-primary reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-mixed-001' kept the primary variant while multiple competing non-primary replacements and removals remained",
                    )
                })
            })),
        "expected mixed multiple-competing reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-mixed-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"]
                        == "kept-primary-variant-over-mixed-non-primary-alternatives"
            })),
        "expected mixed keep-primary branch kind detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-mixed-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-mixed-non-primary-alternatives"
            })),
        "expected mixed competing branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 0);
}

#[test]
fn store_merge_authoring_flow_reports_selected_content_removal_with_competing_removals() {
    let doc_id = "doc:author-smoke-content-select-removal-competing";
    let flow = VariantScenarioFlow::new(
        "store-merge-content-select-removal-competing-root",
        "store-merge-content-select-removal-competing-key",
        doc_id,
        "Author Smoke Content Select Removal Competing",
        "92",
    );
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-select-removal-competing-state",
        doc_id,
        &[("blk:author-smoke-unrelated", "Unrelated")],
    );
    let (_base_ops_dir, base_ops_path) = write_insert_block_ops_file(
        "store-merge-content-select-removal-competing-base-ops",
        "blk:author-smoke-remove-choice",
        "Base",
    );
    let (_unrelated_ops_dir, unrelated_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-select-removal-competing-unrelated-ops",
        "blk:author-smoke-unrelated",
        "Unrelated",
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let unrelated_ops_file = path_arg(&unrelated_ops_path);

    let base_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &base_ops_file, "93", "94");
    let unrelated_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &unrelated_ops_file, "95", "96");
    let merge_json = flow.create_merge_revision(
        &[
            &base_revision_id,
            flow.genesis_revision_id(),
            &unrelated_revision_id,
        ],
        &resolved_state_file,
        "97",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-remove-choice' adopted a non-primary parent removal",
                    )
                })
            })),
        "expected selected content removal reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-remove-choice' selected one non-primary removal while other competing non-primary removals remained",
                    )
                })
            })),
        "expected competing content removal reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-remove-choice"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-removal"
                    && detail["competing_variants"] == json!(["<absent>"])
            })),
        "expected selected content removal detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-remove-choice"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-removals"
                    && detail["competing_variants"] == json!(["<absent>", "<absent>"])
            })),
        "expected multiple competing content removals detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 2);
}

#[test]
fn store_merge_authoring_flow_reports_kept_primary_content_over_multiple_removals() {
    let doc_id = "doc:author-smoke-content-keep-primary-removals";
    let flow = VariantScenarioFlow::new(
        "store-merge-content-keep-primary-removals-root",
        "store-merge-content-keep-primary-removals-key",
        doc_id,
        "Author Smoke Content Keep Primary Removals",
        "98",
    );
    let (_resolved_dir, resolved_state_path) = write_content_entries_resolved_state_for_doc_file(
        "store-merge-content-keep-primary-removals-state",
        doc_id,
        &[
            ("blk:author-smoke-remove-keep", "Base"),
            ("blk:author-smoke-other", "Other"),
        ],
    );
    let (_base_ops_dir, base_ops_path) = write_insert_block_ops_file(
        "store-merge-content-keep-primary-removals-base-ops",
        "blk:author-smoke-remove-keep",
        "Base",
    );
    let (_other_ops_dir, other_ops_path) = write_content_addition_ops_for_block_file(
        "store-merge-content-keep-primary-removals-other-ops",
        "blk:author-smoke-other",
        "Other",
    );
    let resolved_state_file = path_arg(&resolved_state_path);
    let base_ops_file = path_arg(&base_ops_path);
    let other_ops_file = path_arg(&other_ops_path);

    let base_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &base_ops_file, "99", "100");
    let other_revision_id =
        flow.commit_ops_revision(flow.genesis_revision_id(), &other_ops_file, "101", "102");
    let merge_json = flow.create_merge_revision(
        &[
            &base_revision_id,
            flow.genesis_revision_id(),
            &other_revision_id,
        ],
        &resolved_state_file,
        "103",
    );

    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-remove-keep' kept the primary parent variant over a competing non-primary removal",
                    )
                })
            })),
        "expected keep-primary content removal reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "block 'blk:author-smoke-remove-keep' kept the primary variant while multiple competing non-primary removals remained",
                    )
                })
            })),
        "expected multiple competing content removals reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-remove-keep"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "kept-primary-parent-variant-over-competing-non-primary-alternative"
                    && detail["branch_kind"] == "kept-primary-variant-over-non-primary-removal"
                    && detail["competing_variants"] == json!(["<absent>", "<absent>"])
            })),
        "expected keep-primary content removal detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-remove-keep"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-keeping-primary-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-removals"
                    && detail["competing_variants"] == json!(["<absent>", "<absent>"])
            })),
        "expected multiple competing content removals detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
}
