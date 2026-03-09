mod common;

use common::{run_mycel, stderr_text, stdout_text};

fn assert_usage_text(stdout: &str) {
    assert!(
        stdout.contains("mycel <command> [path]"),
        "expected usage header, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Commands:"),
        "expected Commands section, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Sim options:"),
        "expected Sim options section, stdout: {stdout}"
    );
    assert!(
        stdout.contains("Validate options:"),
        "expected Validate options section, stdout: {stdout}"
    );
}

#[test]
fn help_command_prints_usage_and_succeeds() {
    let output = run_mycel(&["help"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stderr_text(&output), "");
    assert_usage_text(&stdout_text(&output));
}

#[test]
fn no_arguments_prints_usage_and_succeeds() {
    let output = run_mycel(&[]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stderr_text(&output), "");
    assert_usage_text(&stdout_text(&output));
}

#[test]
fn unknown_command_prints_usage_and_fails_with_error() {
    let output = run_mycel(&["bogus"]);

    assert_eq!(output.status.code(), Some(2));
    assert_usage_text(&stdout_text(&output));

    let stderr = stderr_text(&output);
    assert!(
        stderr.contains("unknown command: bogus"),
        "expected unknown command error, stderr: {stderr}"
    );
}
