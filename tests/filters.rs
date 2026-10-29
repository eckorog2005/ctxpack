mod common;

use common::{file_paths, fixture, run};

#[test]
fn pithignore_entries_are_honoured_with_gitignore_syntax() {
    let dir = fixture(&[
        (".pithignore", "fixtures/\n*.snap\n!keep.snap\n"),
        ("fixtures/big.json", "{}"),
        ("src/a.snap", ""),
        ("src/keep.snap", ""),
        ("src/lib.rs", ""),
    ]);

    let out = run(dir.path(), &["."]);

    assert!(out.outcome.is_ok());
    assert_eq!(
        file_paths(&out.stdout),
        [".pithignore", "src/keep.snap", "src/lib.rs"]
    );
}

#[test]
fn include_keeps_only_the_listed_extensions_and_still_respects_ignore_files() {
    let dir = fixture(&[
        (".gitignore", "target/\n"),
        ("Cargo.toml", ""),
        ("README.md", ""),
        ("src/lib.rs", ""),
        ("target/gen.rs", ""),
    ]);

    let out = run(dir.path(), &[".", "-i", "rs,toml"]);

    assert_eq!(file_paths(&out.stdout), ["Cargo.toml", "src/lib.rs"]);
}

#[test]
fn include_cannot_admit_an_excluded_file() {
    let dir = fixture(&[
        (".env", "API_KEY=hunter2\n"),
        ("prod.env", ""),
        ("a.rs", ""),
    ]);

    let out = run(dir.path(), &[".", "-i", "env"]);

    assert!(out.outcome.is_ok());
    assert_eq!(file_paths(&out.stdout), Vec::<&str>::new());
}

#[test]
fn repeatable_excludes_drop_matching_files_relative_to_each_root() {
    let dir = fixture(&[
        ("src/lib.rs", ""),
        ("src/lib.snap", ""),
        ("src/fixtures/data.rs", ""),
        ("fixtures/top.rs", ""),
    ]);

    let out = run(
        dir.path(),
        &["src", "fixtures", "-e", "*.snap", "-e", "/fixtures/"],
    );

    assert_eq!(file_paths(&out.stdout), ["fixtures/top.rs", "src/lib.rs"]);
}

#[test]
fn files_over_the_max_file_size_are_skipped_whether_walked_or_named() {
    let dir = fixture(&[
        ("src/big.rs", &"x".repeat(2049)),
        ("src/edge.rs", &"x".repeat(2048)),
        ("huge.txt", &"x".repeat(3000)),
    ]);

    let out = run(dir.path(), &["src", "huge.txt", "--max-file-size", "2KiB"]);

    assert!(out.outcome.is_ok());
    assert_eq!(file_paths(&out.stdout), ["src/edge.rs"]);
}

#[test]
fn the_default_max_file_size_is_one_mebibyte() {
    let dir = fixture(&[
        ("big.txt", &"x".repeat(1024 * 1024 + 1)),
        ("edge.txt", &"x".repeat(1024 * 1024)),
    ]);

    let out = run(dir.path(), &[".", "-q"]);

    assert_eq!(file_paths(&out.stdout), ["edge.txt"]);
}

#[test]
fn a_file_with_a_nul_in_its_first_8_kib_is_skipped_as_binary() {
    let late_nul = format!("{}\0", "x".repeat(8 * 1024));
    let dir = fixture(&[
        ("logo.png", "\u{89}PNG\r\n\0\0\0IHDR"),
        ("late.txt", &late_nul),
        ("a.rs", "fn a() {}\n"),
    ]);

    let out = run(dir.path(), &[".", "logo.png"]);

    assert!(out.outcome.is_ok());
    assert_eq!(file_paths(&out.stdout), ["a.rs", "late.txt"]);
}

#[cfg(unix)]
fn set_mode(path: &std::path::Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(unix)]
fn make_unreadable(path: &std::path::Path) -> bool {
    set_mode(path, 0o000);
    // Root can open a mode-000 path, so the test would prove nothing.
    std::fs::read_dir(path).is_err() && std::fs::File::open(path).is_err()
}

#[cfg(unix)]
#[test]
fn unreadable_files_and_directories_are_skipped_and_named_in_a_warning() {
    let dir = fixture(&[("locked.rs", "x"), ("vault/inner.rs", "y"), ("a.rs", "")]);
    if !make_unreadable(&dir.path().join("locked.rs"))
        || !make_unreadable(&dir.path().join("vault"))
    {
        return;
    }

    let walked = run(dir.path(), &["."]);
    let named = run(dir.path(), &["locked.rs", "a.rs"]);

    for out in [&walked, &named] {
        assert!(out.outcome.is_ok(), "{:?}", out.outcome);
        assert_eq!(file_paths(&out.stdout), ["a.rs"]);
        assert!(
            out.stderr
                .contains("warning: skipped unreadable `locked.rs`: permission denied\n"),
            "{}",
            out.stderr
        );
    }
    assert!(
        walked
            .stderr
            .contains("warning: skipped unreadable `vault`: permission denied\n"),
        "{}",
        walked.stderr
    );
    set_mode(&dir.path().join("vault"), 0o755);
}

#[cfg(unix)]
fn symlinked_fixture() -> tempfile::TempDir {
    use std::os::unix::fs::symlink;
    let dir = fixture(&[
        ("outside/shared.rs", "shared"),
        ("outside/lib/deep.rs", "deep"),
        ("secrets/.env", "API_KEY=hunter2\n"),
        ("repo/a.rs", ""),
    ]);
    let root = dir.path();
    symlink(root.join("outside/shared.rs"), root.join("repo/shared.rs")).unwrap();
    symlink(root.join("outside/lib"), root.join("repo/lib")).unwrap();
    symlink(root.join("secrets/.env"), root.join("repo/settings.txt")).unwrap();
    dir
}

#[cfg(unix)]
#[test]
fn symlinks_are_not_followed_by_default() {
    let dir = symlinked_fixture();

    let out = run(&dir.path().join("repo"), &["."]);

    assert!(out.outcome.is_ok(), "{:?}", out.outcome);
    assert_eq!(file_paths(&out.stdout), ["a.rs"]);
}

#[cfg(unix)]
#[test]
fn follow_symlinks_follows_links_but_excludes_a_link_to_a_secret_file() {
    let dir = symlinked_fixture();
    let repo = dir.path().join("repo");

    let walked = run(&repo, &[".", "--follow-symlinks"]);
    let named = run(&repo, &["settings.txt", "a.rs", "--follow-symlinks"]);

    assert!(walked.outcome.is_ok(), "{:?}", walked.outcome);
    assert_eq!(
        file_paths(&walked.stdout),
        ["a.rs", "lib/deep.rs", "shared.rs"]
    );
    assert_eq!(file_paths(&named.stdout), ["a.rs"]);
    for out in [&walked, &named] {
        assert!(!out.stdout.contains("hunter2"), "{}", out.stdout);
    }
}

#[cfg(unix)]
#[test]
fn following_a_link_back_to_an_ancestor_is_not_an_error() {
    let dir = fixture(&[("repo/a.rs", "")]);
    std::os::unix::fs::symlink(dir.path().join("repo"), dir.path().join("repo/loop")).unwrap();

    let out = run(&dir.path().join("repo"), &[".", "--follow-symlinks"]);

    assert!(out.outcome.is_ok(), "{:?}", out.outcome);
    assert_eq!(file_paths(&out.stdout), ["a.rs"]);
}

#[test]
fn the_report_footer_counts_skips_per_reason() {
    let dir = fixture(&[
        (".env", ""),
        ("logo.png", "\0"),
        ("big.txt", &"x".repeat(2048)),
        ("a.rs", ""),
    ]);

    let out = run(dir.path(), &[".", "--max-file-size", "1KiB"]);

    assert!(
        out.stderr
            .contains(" · 3 skipped (1 secret, 1 binary, 1 too large) · "),
        "{}",
        out.stderr
    );
}
