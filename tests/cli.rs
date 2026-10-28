mod common;

use std::path::Path;

use assert_cmd::Command;
use common::{fixture, write};
use predicates::prelude::*;

/// Isolates the run from the developer's own global gitignore and git config.
fn pithpack(cwd: &Path, home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("pithpack").unwrap();
    cmd.current_dir(cwd)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env_remove("GIT_CONFIG_GLOBAL");
    cmd
}

#[test]
fn success_writes_only_the_bundle_to_stdout_and_the_report_to_stderr() {
    let dir = fixture(&[("src/lib.rs", "pub fn a() {}\n")]);
    let home = tempfile::tempdir().unwrap();

    pithpack(dir.path(), home.path())
        .arg("./src")
        .assert()
        .code(0)
        .stdout(
            predicate::str::starts_with("<bundle ").and(predicate::str::ends_with("</bundle>\n")),
        )
        .stderr(predicate::str::contains("src/lib.rs").and(predicate::str::contains(" bundle: ")));
}

#[test]
fn quiet_success_leaves_stderr_empty() {
    let dir = fixture(&[("src/lib.rs", "pub fn a() {}\n")]);
    let home = tempfile::tempdir().unwrap();

    pithpack(dir.path(), home.path())
        .args(["-q", "src"])
        .assert()
        .code(0)
        .stderr("");
}

#[test]
fn a_missing_path_exits_1_naming_it() {
    let dir = fixture(&[]);
    let home = tempfile::tempdir().unwrap();

    pithpack(dir.path(), home.path())
        .arg("nope/missing.rs")
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains("nope/missing.rs"));
}

#[test]
fn a_usage_error_exits_2() {
    let dir = fixture(&[]);
    let home = tempfile::tempdir().unwrap();

    pithpack(dir.path(), home.path())
        .args([".", "--no-such-flag"])
        .assert()
        .code(2)
        .stdout("");
}

#[test]
fn the_global_gitignore_is_respected() {
    let dir = fixture(&[("keep.rs", ""), ("scratch.tmp", "")]);
    let home = tempfile::tempdir().unwrap();
    write(home.path(), ".config/git/ignore", "*.tmp\n");

    pithpack(dir.path(), home.path())
        .arg(".")
        .assert()
        .code(0)
        .stdout(
            predicate::str::contains("keep.rs").and(predicate::str::contains("scratch.tmp").not()),
        );
}

#[test]
fn the_same_inputs_produce_byte_identical_output() {
    let dir = fixture(&[
        ("b/x.rs", "x\n"),
        ("a/y.py", "y\n"),
        ("a/z/w.go", "w"),
        ("c.txt", "</file>"),
    ]);
    let home = tempfile::tempdir().unwrap();

    let first = pithpack(dir.path(), home.path())
        .args(["a", "b", "c.txt"])
        .output()
        .unwrap();
    let second = pithpack(dir.path(), home.path())
        .args(["a", "b", "c.txt"])
        .output()
        .unwrap();

    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
}
