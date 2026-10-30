mod common;

use std::path::Path;

use assert_cmd::Command;
use common::{fixture, run};

fn pithpack(cwd: &Path) -> Command {
    let mut cmd = Command::cargo_bin("pithpack").unwrap();
    cmd.current_dir(cwd).env_remove("PITHPACK_CLIPBOARD");
    cmd
}

#[test]
fn output_file_receives_the_bundle_and_stdout_stays_empty() {
    let dir = fixture(&[("src/lib.rs", "pub fn a() {}\n")]);

    let out = run(dir.path(), &["src", "-q", "-o", "ctx.xml"]);
    let expected = run(dir.path(), &["src", "-q"]);

    assert!(out.outcome.is_ok());
    assert_eq!(out.stdout, "");
    let written = std::fs::read_to_string(dir.path().join("ctx.xml")).unwrap();
    assert_eq!(written, expected.stdout);
    assert!(written.starts_with("<bundle "));
}

#[test]
fn output_file_replaces_an_existing_file_and_leaves_no_temporary_behind() {
    let dir = fixture(&[("src/lib.rs", "pub fn a() {}\n"), ("out/ctx.xml", "old")]);

    let out = run(dir.path(), &["src", "-q", "-o", "out/ctx.xml"]);

    assert!(out.outcome.is_ok());
    let names: Vec<_> = std::fs::read_dir(dir.path().join("out"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, ["ctx.xml"]);
    let written = std::fs::read_to_string(dir.path().join("out/ctx.xml")).unwrap();
    assert!(written.starts_with("<bundle "));
}

#[test]
fn unwritable_output_path_exits_1_with_a_message_and_creates_nothing() {
    let dir = fixture(&[("src/lib.rs", "pub fn a() {}\n")]);

    pithpack(dir.path())
        .args(["src", "-q", "-o", "missing/ctx.xml"])
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicates::str::contains("missing/ctx.xml"));

    assert!(!dir.path().join("missing").exists());
}

#[test]
fn clipboard_hook_receives_the_exact_bundle_and_stdout_stays_empty() {
    let dir = fixture(&[("src/lib.rs", "pub fn a() {}\n")]);
    let expected = run(dir.path(), &["src", "-q"]).stdout;
    let hook = dir.path().join("clip.txt");

    pithpack(dir.path())
        .args(["src", "-q", "-c"])
        .env("PITHPACK_CLIPBOARD", format!("file:{}", hook.display()))
        .assert()
        .code(0)
        .stdout("");

    assert_eq!(std::fs::read_to_string(hook).unwrap(), expected);
}

#[test]
fn clipboard_hook_write_failure_exits_1_with_a_message() {
    let dir = fixture(&[("src/lib.rs", "pub fn a() {}\n")]);
    let hook = dir.path().join("no/such/dir/clip.txt");

    pithpack(dir.path())
        .args(["src", "-q", "-c"])
        .env("PITHPACK_CLIPBOARD", format!("file:{}", hook.display()))
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicates::str::contains("clipboard"));
}

#[test]
fn clipboard_and_output_file_are_mutually_exclusive() {
    let dir = fixture(&[("src/lib.rs", "")]);

    pithpack(dir.path())
        .args(["src", "-c", "-o", "x"])
        .assert()
        .code(2);
}
