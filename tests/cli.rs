#![expect(clippy::unwrap_used, reason = "in a test, a panic is the failure")]

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
fn edges_show_from_to_and_label() {
    assert_golden("sample");
}

#[test]
fn old_bindings_give_the_same_edges() {
    assert_golden("old-bindings");
}

#[test]
fn an_end_not_bound_to_a_node_is_unconnected() {
    assert_golden("dangling");
}

#[test]
fn ends_are_named_from_the_diagram_of_their_edge() {
    assert_golden("relabelled");
}

#[test]
fn edges_sort_by_from_to_and_label_after_nodes() {
    assert_golden("edge-order");
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

#[cfg(target_os = "linux")]
#[test]
fn a_failed_write_names_stdout() {
    // Like a full disk: /dev/full refuses every write.
    let output = Command::new(env!("CARGO_BIN_EXE_diagramdiff"))
        .args([
            "tests/fixtures/redrawn/old.excalidraw",
            "tests/fixtures/redrawn/new.excalidraw",
        ])
        .stdout(fs::File::create("/dev/full").unwrap())
        .output()
        .unwrap();

    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "diagramdiff: stdout: No space left on device (os error 28)\n"
    );
    assert_eq!(output.status.code(), Some(2));
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
fn an_error_cannot_send_escape_codes() {
    let (output, expected) = golden("escape-codes");

    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), expected);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_closed_stderr_still_exits_2() {
    // Like `2>&1 | head -0`. If diagramdiff writes before the close, the pipe buffers it: no flaky failure.
    let mut child = Command::new(env!("CARGO_BIN_EXE_diagramdiff"))
        .args(["Cargo.toml", "Cargo.toml"])
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stderr.take());

    assert_eq!(child.wait().unwrap().code(), Some(2));
}

#[test]
fn an_argument_cannot_send_escape_codes() {
    let output = diagramdiff(&["a", "b", "c\u{1b}[31md\u{202e}\rX"]);

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains(['\u{1b}', '\u{202e}', '\r']), "{stderr:?}");
    assert!(
        stderr.starts_with("error: unexpected argument 'cd  X' found\n"),
        "{stderr}"
    );
    assert!(
        stderr.contains("Usage: diagramdiff <OLD> <NEW>\n"),
        "{stderr}"
    );
    assert_eq!(output.status.code(), Some(2));
}

#[cfg(unix)]
#[test]
fn a_path_cannot_send_escape_codes() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("escape-codes");
    fs::create_dir_all(&dir).unwrap();
    let old = "a\u{1b}[31mb\u{202e}.excalidraw";
    let new = "c\u{1b}[2Kd\ne\u{2028}f\u{2029}g.excalidraw";
    fs::copy("tests/fixtures/redrawn/old.excalidraw", dir.join(old)).unwrap();
    fs::copy("tests/fixtures/redrawn/new.excalidraw", dir.join(new)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_diagramdiff"))
        .args([old, new])
        .current_dir(&dir)
        .output()
        .unwrap();

    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "a [31mb .excalidraw → c [2Kd e f g.excalidraw: 1 removed, 1 added\n\
         - rectangle \"API\"\n\
         + rectangle \"API\"\n"
    );
    assert_eq!(output.status.code(), Some(0));
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
