use std::{
    fs,
    io::Write,
    process::{Command, Output, Stdio},
};

fn diagramdiff(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_diagramdiff"))
        .args(args)
        // clap colours help when CLICOLOR_FORCE is set; NO_COLOR wins over it.
        .env("NO_COLOR", "1")
        .output()
        .unwrap()
}

fn golden(case: &str) -> (Output, String) {
    let dir = format!("tests/fixtures/{case}");
    let output = Command::new(env!("CARGO_BIN_EXE_diagramdiff"))
        .args(["old.excalidraw", "new.excalidraw"])
        .current_dir(&dir)
        .output()
        .unwrap();
    (
        output,
        fs::read_to_string(format!("{dir}/expected.txt")).unwrap(),
    )
}

fn assert_golden(case: &str) {
    let (output, expected) = golden(case);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn redrawn_box_is_removed_and_added() {
    assert_golden("redrawn");
}

#[test]
fn nodes_show_and_sort_by_their_label() {
    assert_golden("labels");
}

#[test]
fn tombstones_are_left_out() {
    assert_golden("tombstones");
}

#[test]
fn a_pipe_is_read_once() {
    // Like `diagramdiff <(git show HEAD~1:a.excalidraw) a.excalidraw`: a second read finds the pipe empty.
    let mut child = Command::new(env!("CARGO_BIN_EXE_diagramdiff"))
        .args(["/dev/stdin", "tests/fixtures/redrawn/new.excalidraw"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&fs::read("tests/fixtures/redrawn/old.excalidraw").unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "/dev/stdin → tests/fixtures/redrawn/new.excalidraw: 1 removed, 1 added\n\
         - rectangle \"API\"\n\
         + rectangle \"API\"\n"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_reader_that_stops_early_is_not_an_error() {
    // Like `| head -0`. If diagramdiff writes before the close, the pipe buffers it: no flaky failure.
    let mut child = Command::new(env!("CARGO_BIN_EXE_diagramdiff"))
        .args([
            "tests/fixtures/redrawn/old.excalidraw",
            "tests/fixtures/redrawn/new.excalidraw",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let output = child.wait_with_output().unwrap();

    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn no_changes_gives_no_output() {
    let file = "tests/fixtures/redrawn/old.excalidraw";
    let output = diagramdiff(&[file, file]);

    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn version_and_help() {
    let version = diagramdiff(&["--version"]);
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "diagramdiff 0.0.1\n"
    );

    let help = diagramdiff(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .contains("Usage: diagramdiff <OLD> <NEW>")
    );
}

#[test]
fn missing_file_is_an_error() {
    let output = diagramdiff(&["missing.excalidraw", "Cargo.toml"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("diagramdiff: missing.excalidraw: "),
        "{stderr}"
    );
}

#[test]
fn unreadable_new_file_is_an_error() {
    let output = diagramdiff(&["tests/fixtures/redrawn/old.excalidraw", "src"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("diagramdiff: src: "), "{stderr}");
}

#[test]
fn other_json_is_not_excalidraw() {
    let (output, expected) = golden("not-excalidraw");

    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), expected);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_file_that_is_not_json_is_not_excalidraw() {
    let output = diagramdiff(&["Cargo.toml", "tests/cli.rs"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("diagramdiff: Cargo.toml: not an Excalidraw file: "),
        "{stderr}"
    );
}
