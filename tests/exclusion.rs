mod common;

use common::{fixture, run};

fn file_paths(bundle: &str) -> Vec<&str> {
    bundle
        .lines()
        .filter_map(|l| l.strip_prefix("<file path=\""))
        .map(|l| l.split('"').next().unwrap())
        .collect()
}

#[test]
fn a_gitignore_re_include_does_not_admit_a_secret_file() {
    let dir = fixture(&[
        (".gitignore", "*.local\n!.env\n!.env.local\n"),
        (".env", "API_KEY=hunter2\n"),
        (".env.local", "API_KEY=hunter3\n"),
        ("certs/server.PEM", "-----BEGIN CERTIFICATE-----\n"),
        ("src/main.rs", "fn main() {}\n"),
    ]);

    let out = run(dir.path(), &["."]);

    assert!(out.outcome.is_ok());
    assert_eq!(file_paths(&out.stdout), [".gitignore", "src/main.rs"]);
    assert!(!out.stdout.contains("hunter"), "{}", out.stdout);
}

#[test]
fn an_explicit_secret_file_argument_is_excluded_without_the_override() {
    let dir = fixture(&[(".env", "API_KEY=hunter2\n"), ("a.rs", "")]);

    let out = run(dir.path(), &[".env", "a.rs"]);

    assert!(out.outcome.is_ok());
    assert_eq!(file_paths(&out.stdout), ["a.rs"]);
}

#[test]
fn allow_secret_file_admits_a_secret_file_named_exactly_on_the_command_line() {
    let dir = fixture(&[("config/.env", "API_KEY=hunter2\n")]);

    let out = run(dir.path(), &["config/.env", "--allow-secret-file"]);

    assert!(out.outcome.is_ok());
    assert_eq!(file_paths(&out.stdout), ["config/.env"]);
}

#[test]
fn allow_secret_file_has_no_effect_on_walked_files() {
    let dir = fixture(&[
        ("config/.env", "API_KEY=hunter2\n"),
        ("config/app.toml", ""),
    ]);

    let out = run(dir.path(), &["config", "--allow-secret-file"]);

    assert_eq!(file_paths(&out.stdout), ["config/app.toml"]);
}

#[cfg(unix)]
#[test]
fn excluded_files_are_never_opened() {
    use std::os::unix::fs::PermissionsExt;
    let dir = fixture(&[(".env", "API_KEY=hunter2\n"), ("a.rs", "")]);
    let env = dir.path().join(".env");
    std::fs::set_permissions(&env, std::fs::Permissions::from_mode(0o000)).unwrap();
    // Root can open a mode-000 file, so the test would prove nothing.
    if std::fs::File::open(&env).is_ok() {
        return;
    }

    let walked = run(dir.path(), &["."]);
    let explicit = run(dir.path(), &[".env", "a.rs"]);

    assert!(walked.outcome.is_ok(), "{:?}", walked.outcome);
    assert!(explicit.outcome.is_ok(), "{:?}", explicit.outcome);
}

#[test]
fn the_report_counts_excluded_files_without_naming_them() {
    let dir = fixture(&[(".env", ""), ("deploy/id_rsa", ""), ("a.rs", "")]);

    let out = run(dir.path(), &["."]);

    assert!(
        out.stderr.contains(" · 2 files excluded as secrets · "),
        "{}",
        out.stderr
    );
    assert!(!out.stderr.contains(".env") && !out.stderr.contains("id_rsa"));
}

#[test]
fn the_report_mentions_no_exclusions_when_there_are_none() {
    let dir = fixture(&[("a.rs", "")]);

    let out = run(dir.path(), &["a.rs"]);

    assert!(!out.stderr.contains("excluded"), "{}", out.stderr);
}

#[test]
fn a_secret_file_both_named_and_walked_is_admitted_once_and_not_counted() {
    let dir = fixture(&[(".env", "API_KEY=hunter2\n"), ("a.rs", "")]);

    let out = run(dir.path(), &[".", ".env", "--allow-secret-file"]);

    assert_eq!(file_paths(&out.stdout), [".env", "a.rs"]);
    assert!(!out.stderr.contains("excluded"), "{}", out.stderr);
}
