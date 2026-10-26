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

fn golden(case: &str, flags: &[&str]) -> (Output, String) {
    let dir = format!("tests/fixtures/{case}");
    let output = Command::new(env!("CARGO_BIN_EXE_diagramdiff"))
        .args(flags)
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
    assert_golden_with_warnings(case, "");
}

fn assert_golden_with_warnings(case: &str, warnings: &str) {
    let (output, expected) = golden(case, &[]);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), warnings);
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
fn other_kinds_show_their_kind_and_label() {
    assert_golden("other-kinds");
}

#[test]
fn text_and_unknown_kinds_show_as_written() {
    assert_golden("kinds");
}

#[test]
fn bound_text_without_its_container_is_standalone() {
    assert_golden("missing-container");
}

#[test]
fn an_mcp_download_diffs_like_any_other_file() {
    assert_golden("mcp-sample");
}

#[test]
fn an_element_without_id_is_skipped_with_a_warning() {
    assert_golden_with_warnings(
        "no-id",
        "diagramdiff: new.excalidraw: elements[1] skipped: no id\n",
    );
}

#[test]
fn on_a_duplicate_id_the_first_is_kept_with_a_warning() {
    assert_golden_with_warnings(
        "duplicate-id",
        "diagramdiff: new.excalidraw: elements[3] skipped: id #api is used by elements[1]\n",
    );
}

#[test]
fn a_warning_cannot_send_escape_codes() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("warning-escape-codes");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("a.excalidraw"),
        r#"{"type": "excalidraw", "elements": [
            {"id": "x\u001b[2K\u202ey", "type": "text"},
            {"id": "x\u001b[2K\u202ey", "type": "text"}
        ]}"#,
    )
    .unwrap();

    fs::write(
        dir.join("b.excalidraw"),
        r#"{"type": "excalidraw", "elements": [{"type": "text"}]}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_diagramdiff"))
        .args(["a.excalidraw", "b.excalidraw"])
        .current_dir(&dir)
        .output()
        .unwrap();

    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "diagramdiff: a.excalidraw: elements[1] skipped: id #x [2K y is used by elements[0]\n\
         diagramdiff: b.excalidraw: elements[0] skipped: no id\n"
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "a.excalidraw → b.excalidraw: 1 removed, 0 added\n\
         - text #x [2K y\n"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn layout_is_not_a_change() {
    assert_golden("layout-only");
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
    let cases: [(&[&str], i32); 2] = [(&[], 0), (&["--exit-code"], 1)];
    for (flags, code) in cases {
        let mut child = Command::new(env!("CARGO_BIN_EXE_diagramdiff"))
            .args(flags)
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

        assert_eq!(String::from_utf8(output.stderr).unwrap(), "", "{flags:?}");
        assert_eq!(output.status.code(), Some(code), "{flags:?}");
    }
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
fn exit_code_is_1_for_changes() {
    let (output, expected) = golden("redrawn", &["--exit-code"]);

    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn exit_code_is_0_for_no_changes() {
    let file = "tests/fixtures/redrawn/old.excalidraw";
    let output = diagramdiff(&["--exit-code", file, file]);

    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn errors_exit_2_with_or_without_exit_code() {
    let errors: [&[&str]; 3] = [
        &["missing.excalidraw", "Cargo.toml"],
        &["Cargo.toml", "Cargo.toml"],
        &["a", "b", "c"],
    ];
    let flag_states: [&[&str]; 2] = [&[], &["--exit-code"]];
    for flags in flag_states {
        for args in errors {
            let output = diagramdiff(&[flags, args].concat());

            assert_eq!(output.status.code(), Some(2), "{flags:?} {args:?}");
        }
    }
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
            .contains("Usage: diagramdiff [OPTIONS] <OLD> <NEW>")
    );
}

fn assert_readme_shows(command: &str, output: Output) {
    let example = format!(
        "$ {command}\n{}```",
        String::from_utf8(output.stdout).unwrap()
    );

    assert!(
        fs::read_to_string("README.md").unwrap().contains(&example),
        "{example}"
    );
}

#[test]
fn the_readme_shows_real_output() {
    let (output, _) = golden("sample", &[]);
    assert_readme_shows("diagramdiff old.excalidraw new.excalidraw", output);
}

#[test]
fn the_readme_shows_the_real_help() {
    assert_readme_shows("diagramdiff --help", diagramdiff(&["--help"]));
}

/// The shell loop of a README CI job, from `git diff` to `done`.
fn ci_loop(job: &str) -> String {
    let start = job.find("git diff").unwrap();
    let end = start + job[start..].find("done").unwrap();
    let lines: Vec<&str> = job[start..end].lines().map(str::trim_start).collect();
    format!("{}\ndone", lines.join("\n"))
}

#[cfg(unix)]
#[test]
fn the_readme_ci_jobs_diff_the_diagrams_changed_in_place() {
    let readme = fs::read_to_string("README.md").unwrap();
    let jobs: Vec<String> = readme.split("```yaml").skip(1).map(ci_loop).collect();
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[0], jobs[1]);

    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("ci-loop");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("docs")).unwrap();
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args(["-c", "user.name=test", "-c", "user.email=test@example.com"])
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .current_dir(&dir)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    let fixture = |case: &str, file: &str, to: &str| {
        fs::copy(format!("tests/fixtures/{case}/{file}"), dir.join(to)).unwrap();
    };
    git(&["init", "-q"]);
    fixture("redrawn", "old.excalidraw", "docs/architecture.excalidraw");
    fixture("redrawn", "old.excalidraw", "-lead.excalidraw");
    fixture("sample", "old.excalidraw", "deleted.excalidraw");
    git(&["add", "-A"]);
    git(&["commit", "-qm", "base"]);
    fixture("redrawn", "new.excalidraw", "docs/architecture.excalidraw");
    fixture("redrawn", "new.excalidraw", "-lead.excalidraw");
    fixture("sample", "new.excalidraw", "added.excalidraw");
    fs::remove_file(dir.join("deleted.excalidraw")).unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-qm", "pull request"]);

    let bin = std::path::Path::new(env!("CARGO_BIN_EXE_diagramdiff"));
    let path = format!(
        "{}:{}",
        bin.parent().unwrap().display(),
        std::env::var("PATH").unwrap()
    );
    let output = Command::new("bash")
        .args(["-eo", "pipefail", "-c", &jobs[0]])
        .env("BASE", "HEAD^1")
        .env("PATH", path)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .current_dir(&dir)
        .output()
        .unwrap();

    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "-lead.excalidraw.old → -lead.excalidraw: 1 removed, 1 added\n\
         - rectangle \"API\"\n\
         + rectangle \"API\"\n\
         docs/architecture.excalidraw.old → docs/architecture.excalidraw: 1 removed, 1 added\n\
         - rectangle \"API\"\n\
         + rectangle \"API\"\n"
    );
    assert_eq!(output.status.code(), Some(0));
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
    let (output, expected) = golden("not-excalidraw", &[]);

    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), expected);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn an_error_cannot_send_escape_codes() {
    let (output, expected) = golden("escape-codes", &[]);

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
        stderr.contains("Usage: diagramdiff [OPTIONS] <OLD> <NEW>\n"),
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
