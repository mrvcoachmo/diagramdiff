use std::process::{Command, Output};

fn runediff(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_runediff"))
        .args(args)
        // clap colours help when CLICOLOR_FORCE is set; NO_COLOR wins over it.
        .env("NO_COLOR", "1")
        .output()
        .unwrap()
}

#[test]
fn version_and_help() {
    let version = runediff(&["--version"]);
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "runediff 0.0.1\n"
    );

    let help = runediff(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .contains("Usage: runediff <OLD> <NEW>")
    );
}

#[test]
fn missing_file_is_an_error() {
    let output = runediff(&["missing.excalidraw", "Cargo.toml"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("runediff: missing.excalidraw: "),
        "{stderr}"
    );
}

#[test]
fn unreadable_new_file_is_an_error() {
    let output = runediff(&["Cargo.toml", "src"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("runediff: src: "), "{stderr}");
}

#[test]
fn two_readable_files_are_not_implemented_yet() {
    let output = runediff(&["Cargo.toml", "tests/cli.rs"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "runediff: not implemented yet\n"
    );
}
