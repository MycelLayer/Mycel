use std::sync::{Mutex, MutexGuard, OnceLock};

mod common;

use common::{
    assert_exit_code, assert_stderr_contains, assert_stdout_contains, assert_success,
    parse_json_stdout, run_report, run_sim, stdout_text,
};

fn sim_run_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn report_inspect_json_reports_ok_for_example_report() {
    let output = run_report(&[
        "report",
        "inspect",
        "sim/reports/report.example.json",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["run_id"], "run:example-001");
    assert_eq!(json["result"], "pass");
    assert_eq!(json["peer_count"], 2);
    assert_eq!(json["event_count"], 3);
    assert_eq!(json["failure_count"], 0);
    assert_eq!(json["validation_status"], "ok");
    assert_eq!(json["seed_source"], "derived");
    assert_eq!(json["fault_plan_count"], 0);
}

#[test]
fn report_inspect_text_reports_summary_for_example_report() {
    let output = run_report(&["report", "inspect", "sim/reports/report.example.json"]);

    assert_success(&output);
    assert_stdout_contains(&output, "report path: sim/reports/report.example.json");
    assert_stdout_contains(&output, "run id: run:example-001");
    assert_stdout_contains(&output, "result: pass");
    assert_stdout_contains(&output, "events: 3");
    assert_stdout_contains(&output, "report inspection: ok");
}

#[test]
fn report_inspect_events_json_reports_event_trace_for_example_report() {
    let output = run_report(&[
        "report",
        "inspect",
        "sim/reports/report.example.json",
        "--events",
        "--json",
    ]);

    assert_success(&output);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["event_count"], 3);
    let events = json["events"]
        .as_array()
        .expect("events should be an array");
    assert_eq!(events.len(), 3);
    assert_eq!(events[0]["action"], "load-fixture");
    assert_eq!(events[1]["action"], "seed-advertise");
}

#[test]
fn report_inspect_events_text_reports_event_trace_for_example_report() {
    let output = run_report(&[
        "report",
        "inspect",
        "sim/reports/report.example.json",
        "--events",
    ]);

    assert_success(&output);
    assert_stdout_contains(&output, "events: 3");
    assert_stdout_contains(
        &output,
        "event #1 phase=load action=load-fixture outcome=ok",
    );
    assert_stdout_contains(
        &output,
        "event #2 phase=sync action=seed-advertise outcome=ok",
    );
}

#[test]
fn report_inspect_generated_report_path_round_trips() {
    let _guard = sim_run_lock();
    let sim_output = run_sim(&[
        "sim",
        "run",
        "sim/tests/three-peer-consistency.example.json",
        "--json",
    ]);
    assert_success(&sim_output);

    let sim_json = parse_json_stdout(&sim_output);
    let report_path = sim_json["report_path"]
        .as_str()
        .expect("report_path should be a string")
        .to_owned();

    let output = run_report(&["report", "inspect", &report_path, "--json"]);
    assert_success(&output);

    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["run_id"], "run:three-peer-consistency");
    assert_eq!(json["result"], "pass");
    assert_eq!(json["validation_status"], "ok");
    assert_eq!(json["matched_expected_outcomes"][0], "sync-success");
}

#[test]
fn report_inspect_failures_json_reports_failures_for_generated_negative_report() {
    let _guard = sim_run_lock();
    let sim_output = run_sim(&[
        "sim",
        "run",
        "sim/tests/hash-mismatch.example.json",
        "--json",
    ]);
    assert_success(&sim_output);

    let sim_json = parse_json_stdout(&sim_output);
    let report_path = sim_json["report_path"]
        .as_str()
        .expect("report_path should be a string")
        .to_owned();

    let output = run_report(&["report", "inspect", &report_path, "--failures", "--json"]);
    assert_success(&output);

    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["result"], "fail");
    assert_eq!(json["failure_count"], 2);
    let failures = json["failures"]
        .as_array()
        .expect("failures should be an array");
    assert!(
        failures.iter().any(|entry| {
            entry["description"]
                .as_str()
                .is_some_and(|description| description.contains("Reader rejected planned fault"))
        }),
        "expected reader rejection failure, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_inspect_requires_target_path() {
    let output = run_report(&["report", "inspect"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "missing report inspect target");
}

#[test]
fn report_inspect_rejects_schema_file_targets() {
    let output = run_report(&[
        "report",
        "inspect",
        "sim/reports/report.schema.json",
        "--json",
    ]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("schema files are not inspect targets"))
            })),
        "expected schema-target error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_inspect_missing_target_fails_cleanly() {
    let output = run_report(&["report", "inspect", "does-not-exist.report.json", "--json"]);

    assert_exit_code(&output, 1);
    let json = parse_json_stdout(&output);
    assert_eq!(json["status"], "failed");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|errors| errors.iter().any(|entry| {
                entry
                    .as_str()
                    .is_some_and(|message| message.contains("report path does not exist"))
            })),
        "expected missing-path error, stdout: {}",
        stdout_text(&output)
    );
}

#[test]
fn report_inspect_rejects_conflicting_filter_flags() {
    let output = run_report(&[
        "report",
        "inspect",
        "sim/reports/report.example.json",
        "--events",
        "--failures",
    ]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(
        &output,
        "report inspect accepts only one of --events or --failures",
    );
}

#[test]
fn report_rejects_unknown_subcommand() {
    let output = run_report(&["report", "bogus"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "unknown report subcommand: bogus");
}

#[test]
fn report_requires_subcommand() {
    let output = run_report(&["report"]);

    assert_exit_code(&output, 2);
    assert_stderr_contains(&output, "missing report subcommand");
}
