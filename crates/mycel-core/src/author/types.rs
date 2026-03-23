use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;

use crate::protocol::BlockObject;
use crate::replay::DocumentState;
use crate::store::StoredObjectRecord;

#[derive(Debug, Clone)]
pub struct DocumentCreateParams {
    pub doc_id: String,
    pub title: String,
    pub language: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct PatchCreateParams {
    pub doc_id: String,
    pub base_revision: String,
    pub timestamp: u64,
    pub ops: Value,
}

#[derive(Debug, Clone)]
pub struct RevisionCommitParams {
    pub doc_id: String,
    pub parents: Vec<String>,
    pub patches: Vec<String>,
    pub merge_strategy: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct MergeRevisionCreateParams {
    pub doc_id: String,
    pub parents: Vec<String>,
    pub resolved_state: DocumentState,
    pub merge_strategy: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeOutcome {
    AutoMerged,
    MultiVariant,
    ManualCurationRequired,
}

impl MergeOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AutoMerged => "auto-merged",
            Self::MultiVariant => "multi-variant",
            Self::ManualCurationRequired => "manual-curation-required",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeReasonSubjectKind {
    Block,
    MetadataKey,
}

impl MergeReasonSubjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::MetadataKey => "metadata-key",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeReasonVariantKind {
    Content,
    Metadata,
    ParentPlacement,
    SiblingPlacement,
}

impl MergeReasonVariantKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Content => "content",
            Self::Metadata => "metadata",
            Self::ParentPlacement => "parent-placement",
            Self::SiblingPlacement => "sibling-placement",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeReasonKind {
    UnsupportedMetadataDeletion,
    SelectedNonPrimaryParentVariant,
    KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative,
    MultipleCompetingAlternativesRemainAfterSelectedVariant,
    MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant,
    NoMatchingParentVariant,
    NoMatchingParentPlacement,
    NoMatchingSiblingPlacement,
}

impl MergeReasonKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedMetadataDeletion => "unsupported-metadata-deletion",
            Self::SelectedNonPrimaryParentVariant => "selected-non-primary-parent-variant",
            Self::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative => {
                "kept-primary-parent-variant-over-competing-non-primary-alternative"
            }
            Self::MultipleCompetingAlternativesRemainAfterSelectedVariant => {
                "multiple-competing-alternatives-remain-after-selected-variant"
            }
            Self::MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant => {
                "multiple-competing-alternatives-remain-after-keeping-primary-variant"
            }
            Self::NoMatchingParentVariant => "no-matching-parent-variant",
            Self::NoMatchingParentPlacement => "no-matching-parent-placement",
            Self::NoMatchingSiblingPlacement => "no-matching-sibling-placement",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MergeReasonBranchKind {
    AdoptedNonPrimaryAddition,
    AdoptedNonPrimaryReplacement,
    AdoptedNonPrimaryRemoval,
    AdoptedNonPrimaryReplacementWhileCompetingRemovalRemains,
    AdoptedNonPrimaryReplacementWhileCompetingReplacementsAndRemovalRemain,
    AdoptedNonPrimaryRemovalWhileCompetingReplacementRemains,
    AdoptedNonPrimaryRemovalWhileCompetingReplacementAndRemovalsRemain,
    KeptPrimaryAbsenceOverNonPrimaryAddition,
    KeptPrimaryVariantOverNonPrimaryReplacement,
    KeptPrimaryVariantOverNonPrimaryRemoval,
    KeptPrimaryVariantOverMixedNonPrimaryAlternatives,
    KeptPrimaryVariantOverMultipleCompetingNonPrimaryReplacementsAndRemovals,
    MultipleCompetingNonPrimaryAdditions,
    MultipleCompetingNonPrimaryReplacements,
    MultipleCompetingNonPrimaryRemovals,
    MultipleCompetingMixedNonPrimaryAlternatives,
    MultipleCompetingNonPrimaryReplacementsAndRemovals,
}

impl MergeReasonBranchKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AdoptedNonPrimaryAddition => "adopted-non-primary-addition",
            Self::AdoptedNonPrimaryReplacement => "adopted-non-primary-replacement",
            Self::AdoptedNonPrimaryRemoval => "adopted-non-primary-removal",
            Self::AdoptedNonPrimaryReplacementWhileCompetingRemovalRemains => {
                "adopted-non-primary-replacement-while-competing-removal-remains"
            }
            Self::AdoptedNonPrimaryReplacementWhileCompetingReplacementsAndRemovalRemain => {
                "adopted-non-primary-replacement-while-competing-replacements-and-removal-remain"
            }
            Self::AdoptedNonPrimaryRemovalWhileCompetingReplacementRemains => {
                "adopted-non-primary-removal-while-competing-replacement-remains"
            }
            Self::AdoptedNonPrimaryRemovalWhileCompetingReplacementAndRemovalsRemain => {
                "adopted-non-primary-removal-while-competing-replacement-and-removals-remain"
            }
            Self::KeptPrimaryAbsenceOverNonPrimaryAddition => {
                "kept-primary-absence-over-non-primary-addition"
            }
            Self::KeptPrimaryVariantOverNonPrimaryReplacement => {
                "kept-primary-variant-over-non-primary-replacement"
            }
            Self::KeptPrimaryVariantOverNonPrimaryRemoval => {
                "kept-primary-variant-over-non-primary-removal"
            }
            Self::KeptPrimaryVariantOverMixedNonPrimaryAlternatives => {
                "kept-primary-variant-over-mixed-non-primary-alternatives"
            }
            Self::KeptPrimaryVariantOverMultipleCompetingNonPrimaryReplacementsAndRemovals => {
                "kept-primary-variant-over-multiple-competing-non-primary-replacements-and-removals"
            }
            Self::MultipleCompetingNonPrimaryAdditions => {
                "multiple-competing-non-primary-additions"
            }
            Self::MultipleCompetingNonPrimaryReplacements => {
                "multiple-competing-non-primary-replacements"
            }
            Self::MultipleCompetingNonPrimaryRemovals => "multiple-competing-non-primary-removals",
            Self::MultipleCompetingMixedNonPrimaryAlternatives => {
                "multiple-competing-mixed-non-primary-alternatives"
            }
            Self::MultipleCompetingNonPrimaryReplacementsAndRemovals => {
                "multiple-competing-non-primary-replacements-and-removals"
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MergeReasonDetail {
    pub subject_kind: MergeReasonSubjectKind,
    pub subject_id: String,
    pub variant_kind: MergeReasonVariantKind,
    pub reason_kind: MergeReasonKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_kind: Option<MergeReasonBranchKind>,
    pub primary_variant: String,
    pub resolved_variant: String,
    pub competing_variants: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentCreateSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub document_object_id: String,
    pub genesis_revision_id: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_objects: Vec<StoredObjectRecord>,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchCreateSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub patch_id: String,
    pub base_revision: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_object: StoredObjectRecord,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RevisionCommitSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub revision_id: String,
    pub parent_revision_ids: Vec<String>,
    pub patch_ids: Vec<String>,
    pub recomputed_state_hash: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_object: StoredObjectRecord,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergeRevisionCreateSummary {
    pub store_root: PathBuf,
    pub status: String,
    pub doc_id: String,
    pub merge_outcome: MergeOutcome,
    pub merge_reasons: Vec<String>,
    pub merge_reason_details: Vec<MergeReasonDetail>,
    pub parent_revision_ids: Vec<String>,
    pub patch_id: String,
    pub patch_op_count: usize,
    pub revision_id: String,
    pub recomputed_state_hash: String,
    pub written_object_count: usize,
    pub existing_object_count: usize,
    pub stored_objects: Vec<StoredObjectRecord>,
    pub index_manifest_path: Option<PathBuf>,
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManualCurationSummary {
    pub status: String,
    pub doc_id: String,
    pub merge_outcome: MergeOutcome,
    pub merge_reasons: Vec<String>,
    pub merge_reason_details: Vec<MergeReasonDetail>,
    pub parent_revision_ids: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct MergeAssessment {
    pub(crate) outcome: MergeOutcome,
    pub(crate) reasons: Vec<String>,
    pub(crate) reason_details: Vec<MergeReasonDetail>,
}

#[derive(Debug, Clone)]
pub(crate) struct BlockPlacement {
    pub(crate) block: BlockObject,
    pub(crate) parent_block_id: Option<String>,
    pub(crate) previous_sibling_id: Option<String>,
}
