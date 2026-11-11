//! `--help` is a request, not a failure (#1525): it goes to stdout with exit 0, like `--version`.
//! Bare `forja-cli` is a usage error: the help on stderr, exit 2.

use std::process::Command;

#[test]
fn help_flags_print_help_to_stdout_and_exit_zero() {
    for args in [&["--help"][..], &["-h"], &["help"]] {
        let out = Command::new(env!("CARGO_BIN_EXE_forja-cli")).args(args).output().expect("run forja-cli");
        assert!(out.status.success(), "{args:?}: {:?}", out.status);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.starts_with("forja-cli: headless Forja\n"), "{args:?}: {stdout:?}");
        assert!(String::from_utf8_lossy(&out.stderr).is_empty(), "{args:?}: stderr should be empty");
    }
}

#[test]
fn bare_invocation_reports_on_stderr() {
    let out = Command::new(env!("CARGO_BIN_EXE_forja-cli")).output().expect("run forja-cli");
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty(), "bare invocation keeps its help on stderr");
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("forja-cli: headless Forja\n"));
}
