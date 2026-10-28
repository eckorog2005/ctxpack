mod common;

use common::{fixture, run};

#[test]
fn bundles_a_directory_into_file_blocks() {
    let dir = fixture(&[("src/lib.rs", "pub fn a() {}\n")]);

    let out = run(dir.path(), &["src", "--no-tree"]);

    assert!(out.outcome.is_ok());
    assert_eq!(
        out.stdout,
        format!(
            "<bundle tool=\"pithpack\" version=\"{}\" mode=\"none\" files=\"1\">\n\
             <file path=\"src/lib.rs\" lang=\"rust\" mode=\"none\">\n\
             pub fn a() {{}}\n\
             </file>\n\
             </bundle>\n",
            env!("CARGO_PKG_VERSION")
        )
    );
}

#[test]
fn every_bundle_starts_with_a_file_tree() {
    let dir = fixture(&[("src/lib.rs", "a\n"), ("src/x/mod.rs", "b\n")]);

    let out = run(dir.path(), &["src"]);

    let after_header = out.stdout.split_once('\n').unwrap().1;
    assert!(
        after_header.starts_with("<tree>\nsrc/\n  lib.rs\n  x/\n    mod.rs\n</tree>\n<file "),
        "{}",
        out.stdout
    );
}

fn file_paths(bundle: &str) -> Vec<&str> {
    bundle
        .lines()
        .filter_map(|l| l.strip_prefix("<file path=\""))
        .map(|l| l.split('"').next().unwrap())
        .collect()
}

#[test]
fn several_paths_combine_into_one_bundle_ordered_by_display_path() {
    let dir = fixture(&[
        ("src/z.rs", ""),
        ("src/a/b.rs", ""),
        ("Cargo.toml", ""),
        ("docs/x.md", ""),
    ]);

    let out = run(dir.path(), &["src", "./Cargo.toml", "src/a"]);

    assert_eq!(
        file_paths(&out.stdout),
        ["Cargo.toml", "src/a/b.rs", "src/z.rs"]
    );
    assert!(out.stdout.starts_with("<bundle ") && out.stdout.contains(" files=\"3\">"));
}

#[test]
fn ignore_files_are_respected_and_hidden_files_walked_but_git_never_included() {
    let dir = fixture(&[
        (".gitignore", "target/\n"),
        (".ignore", "*.log\n"),
        (".github/workflows/ci.yml", ""),
        (".git/config", ""),
        (".git/HEAD", ""),
        ("target/debug/out", ""),
        ("build.log", ""),
        ("src/main.rs", ""),
    ]);

    let out = run(dir.path(), &["."]);

    assert_eq!(
        file_paths(&out.stdout),
        [
            ".github/workflows/ci.yml",
            ".gitignore",
            ".ignore",
            "src/main.rs"
        ]
    );
}

#[test]
fn content_is_not_xml_escaped_except_a_closing_file_tag() {
    let dir = fixture(&[("t.ts", "if (a < b && c > d) { s = \"</file>\"; }")]);

    let out = run(dir.path(), &["t.ts", "--no-tree"]);

    assert!(
        out.stdout.contains(
            "<file path=\"t.ts\" lang=\"typescript\" mode=\"none\">\n\
             if (a < b && c > d) { s = \"<\\/file>\"; }\n\
             </file>\n"
        ),
        "{}",
        out.stdout
    );
}

#[test]
fn display_paths_use_forward_slashes_whatever_the_argument_separator() {
    let dir = fixture(&[("src/a/b/c.rs", "")]);
    let arg = std::path::Path::new(".").join("src").join("a");

    let out = run(dir.path(), &[arg.to_str().unwrap(), "--no-tree"]);

    assert_eq!(file_paths(&out.stdout), ["src/a/b/c.rs"]);
}

#[test]
fn an_absolute_argument_keeps_its_root_with_forward_slashes() {
    let dir = fixture(&[("src/a.rs", "")]);
    let arg = dir.path().join("src");

    let out = run(dir.path(), &[arg.to_str().unwrap(), "--no-tree"]);

    let paths = file_paths(&out.stdout);
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("/src/a.rs"), "{}", paths[0]);
    assert!(
        !paths[0].contains('\\') && !paths[0].contains("//"),
        "{}",
        paths[0]
    );
}
