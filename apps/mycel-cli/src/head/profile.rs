use std::path::PathBuf;

use clap::{Args, Subcommand};
use mycel_core::head::{
    inspect_head_profile_from_path, list_head_profiles_from_path, HeadProfileInspectSummary,
    HeadProfileListSummary, HeadProfileSummary,
};

use crate::{emit_error_line, CliError};

use super::{
    humanize_tokenized_label, profile_inspect_hint, profile_retry_hint, unexpected_extra,
    viewer_review_state_text,
};

#[derive(Args)]
pub(super) struct HeadProfileCliArgs {
    #[command(subcommand)]
    command: Option<HeadProfileSubcommand>,
}

#[derive(Subcommand)]
enum HeadProfileSubcommand {
    #[command(about = "List fixed reader profiles declared by a head input bundle")]
    List(HeadProfileListCliArgs),
    #[command(about = "Inspect one fixed reader profile from a head input bundle")]
    Inspect(HeadProfileInspectCliArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args)]
struct HeadProfileListCliArgs {
    #[arg(
        long,
        value_name = "PATH_OR_FIXTURE",
        help = "Input bundle path or repo fixture name",
        required = true
    )]
    input: String,
    #[arg(long, help = "Emit machine-readable profile listing output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

#[derive(Args)]
struct HeadProfileInspectCliArgs {
    #[arg(
        long,
        value_name = "PATH_OR_FIXTURE",
        help = "Input bundle path or repo fixture name",
        required = true
    )]
    input: String,
    #[arg(
        long,
        value_name = "PROFILE_ID",
        help = "Select one named fixed reader profile from the input bundle; discover choices with mycel head profile list and inspect one with mycel head profile inspect"
    )]
    profile_id: Option<String>,
    #[arg(long, help = "Emit machine-readable profile inspection output")]
    json: bool,
    #[arg(hide = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

fn profile_reuse_hint(profile_id: &str) -> String {
    format!("reuse this profile with: --profile-id {profile_id}")
}

fn head_profile_label(profile: &HeadProfileSummary) -> &str {
    profile.profile_id.as_deref().unwrap_or("default")
}

fn summarize_profile_admitted_keys(keys: &[String]) -> String {
    if keys.is_empty() {
        "none".to_string()
    } else {
        keys.join(", ")
    }
}

fn viewer_score_mode_text(profile: &HeadProfileSummary) -> String {
    if profile.viewer_score.enabled {
        humanize_tokenized_label(&viewer_review_state_text(&profile.viewer_score.mode))
    } else {
        "disabled".to_string()
    }
}

fn print_head_profile_entry_human(profile: &HeadProfileSummary) {
    println!("- {}", head_profile_label(profile));
    println!(
        "  source={} policy hash={} selector epoch={}",
        profile.source, profile.policy_hash, profile.selector_epoch
    );
    println!(
        "  effective selection time={} epoch seconds={} epoch zero timestamp={}",
        profile.effective_selection_time, profile.epoch_seconds, profile.epoch_zero_timestamp
    );
    println!(
        "  admission window epochs={} min valid views for admission={} min valid views per epoch={} weight cap per key={}",
        profile.admission_window_epochs,
        profile.min_valid_views_for_admission,
        profile.min_valid_views_per_epoch,
        profile.weight_cap_per_key
    );
    println!(
        "  editor admission={} admitted keys={}",
        profile.editor_admission.mode,
        summarize_profile_admitted_keys(&profile.editor_admission.admitted_keys)
    );
    println!(
        "  view admission={} admitted keys={}",
        profile.view_admission.mode,
        summarize_profile_admitted_keys(&profile.view_admission.admitted_keys)
    );
    if profile.viewer_score.enabled {
        println!(
            "  viewer score={} bonus cap={} penalty cap={} signal weight cap={} admission required={} min identity tier={} min reputation band={}",
            viewer_score_mode_text(profile),
            profile.viewer_score.bonus_cap,
            profile.viewer_score.penalty_cap,
            profile.viewer_score.signal_weight_cap,
            profile.viewer_score.admission_required,
            viewer_review_state_text(&profile.viewer_score.min_identity_tier),
            viewer_review_state_text(&profile.viewer_score.min_reputation_band)
        );
    } else {
        println!("  viewer score=disabled");
    }
}

fn print_head_profile_list_human(summary: &HeadProfileListSummary) -> i32 {
    println!(
        "Head profiles: {}",
        if summary.is_ok() { "ok" } else { "failed" }
    );
    println!();
    println!("Input");
    println!("- input: {}", summary.input_path.display());
    println!("- profile count: {}", summary.profile_count);
    if !summary.available_profile_ids.is_empty() {
        println!(
            "- available profiles: {}",
            summary.available_profile_ids.join(", ")
        );
        let example_profile_id = &summary.available_profile_ids[0];
        println!("- {}", profile_reuse_hint(example_profile_id));
        println!(
            "- inspect profile details with: mycel head profile inspect --input <same-input> --profile-id {example_profile_id}"
        );
    }
    if !summary.profiles.is_empty() {
        println!();
        println!("Profiles");
        for profile in &summary.profiles {
            print_head_profile_entry_human(profile);
        }
    }
    if !summary.notes.is_empty() {
        println!();
        println!("Notes");
        for note in &summary.notes {
            println!("- {note}");
        }
    }

    if summary.is_ok() {
        0
    } else {
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_head_profile_list_json(summary: &HeadProfileListSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization("head profile list summary", source)),
    }
}

fn print_head_profile_inspect_human(summary: &HeadProfileInspectSummary) -> i32 {
    println!(
        "Head profile: {}",
        if summary.is_ok() { "ok" } else { "failed" }
    );
    println!();
    println!("Input");
    println!("- input: {}", summary.input_path.display());
    if let Some(requested_profile_id) = &summary.requested_profile_id {
        println!("- requested profile: {requested_profile_id}");
    }
    if !summary.available_profile_ids.is_empty() {
        println!(
            "- available profiles: {}",
            summary.available_profile_ids.join(", ")
        );
        if let Some(hint) = profile_retry_hint(&summary.available_profile_ids, &summary.errors) {
            println!("- {hint}");
        }
        if let Some(hint) = profile_inspect_hint(&summary.available_profile_ids, &summary.errors) {
            println!("- {hint}");
        }
    }
    if let Some(profile) = &summary.profile {
        if let Some(profile_id) = &profile.profile_id {
            println!("- {}", profile_reuse_hint(profile_id));
        }
        println!();
        println!("Profile");
        print_head_profile_entry_human(profile);
    }
    if !summary.notes.is_empty() {
        println!();
        println!("Notes");
        for note in &summary.notes {
            println!("- {note}");
        }
    }

    if summary.is_ok() {
        0
    } else {
        for error in &summary.errors {
            emit_error_line(error);
        }
        1
    }
}

fn print_head_profile_inspect_json(summary: &HeadProfileInspectSummary) -> Result<i32, CliError> {
    match serde_json::to_string_pretty(summary) {
        Ok(json) => {
            println!("{json}");
            if summary.is_ok() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        Err(source) => Err(CliError::serialization(
            "head profile inspect summary",
            source,
        )),
    }
}

fn head_profile_list(input_path: PathBuf, json: bool) -> Result<i32, CliError> {
    let summary = list_head_profiles_from_path(&input_path);
    if json {
        print_head_profile_list_json(&summary)
    } else {
        Ok(print_head_profile_list_human(&summary))
    }
}

fn head_profile_inspect(
    input_path: PathBuf,
    profile_id: Option<String>,
    json: bool,
) -> Result<i32, CliError> {
    let summary = inspect_head_profile_from_path(&input_path, profile_id.as_deref());
    if json {
        print_head_profile_inspect_json(&summary)
    } else {
        Ok(print_head_profile_inspect_human(&summary))
    }
}

pub(super) fn handle(args: HeadProfileCliArgs) -> Result<i32, CliError> {
    match args.command {
        Some(HeadProfileSubcommand::List(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "head profile list") {
                return Err(CliError::usage(message));
            }

            head_profile_list(PathBuf::from(args.input), args.json)
        }
        Some(HeadProfileSubcommand::Inspect(args)) => {
            if let Some(message) = unexpected_extra(&args.extra, "head profile inspect") {
                return Err(CliError::usage(message));
            }

            head_profile_inspect(PathBuf::from(args.input), args.profile_id, args.json)
        }
        Some(HeadProfileSubcommand::External(args)) => {
            let other = args.first().map(String::as_str).unwrap_or("<unknown>");
            Err(CliError::usage(format!(
                "unknown head profile subcommand: {other}"
            )))
        }
        None => Err(CliError::usage("missing head profile subcommand")),
    }
}
