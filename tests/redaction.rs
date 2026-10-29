mod common;

use common::{fixture, run};

/// Real-shaped fake secrets, assembled at run time so the source never holds a scannable token.
fn secrets() -> Vec<(&'static str, String)> {
    vec![
        ("aws-access-key", ["AKIA", "Z7QK2M4XW9TB3HNP"].concat()),
        (
            "github-token",
            ["ghp", "_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8"].concat(),
        ),
        (
            "anthropic-key",
            ["sk-", "ant-api03-Xy7Kq2Lm9Np4Rs8Tv3Wz6Ab1Cd5"].concat(),
        ),
        (
            "openai-key",
            ["sk-", "proj-Xy7Kq2Lm9Np4Rs8Tv3Wz6Ab1"].concat(),
        ),
        (
            "google-api-key",
            ["AIza", "SyB7k2Lm9Np4Rs8Tv3Wz6Ab1Cd5Ef0Gh2Jk"].concat(),
        ),
        ("slack-token", ["xoxb", "-1234567890-abcdefABCDEF"].concat()),
        (
            "stripe-key",
            ["sk_live", "_4eC39HqLyjWDarjtT1zdp7dc"].concat(),
        ),
        (
            "jwt",
            [
                "eyJhbGciOiJIUzI1NiJ9.",
                "eyJzdWIiOiIxMjM0In0.",
                "dBjftJeZ4CVPmB92K27uhbUJU1p1r_wW1gFWFOEjXk",
            ]
            .concat(),
        ),
        (
            "private-key",
            [
                "-----BEGIN RSA PRIVATE ",
                "KEY-----\nMIIEowIBAAKCAQEA7d8Kq\n-----END RSA PRIVATE KEY-----",
            ]
            .concat(),
        ),
    ]
}

fn source_with_every_secret() -> String {
    let mut source: String = secrets()
        .iter()
        .map(|(kind, value)| format!("const {}: &str = \"{value}\";\n", kind.replace('-', "_")))
        .collect();
    source.push_str("let db = \"postgres://admin:hunter2hunter2@db:5432/app\";\n");
    source.push_str("let api_token = \"q8Zr2Lx7Vm4Nt9Bw\";\n");
    source
}

#[test]
fn every_secret_shape_becomes_a_typed_marker_inside_its_quotes() {
    let dir = fixture(&[("src/keys.rs", &source_with_every_secret())]);

    let out = run(dir.path(), &["src"]);

    assert!(out.outcome.is_ok());
    for (kind, value) in secrets() {
        assert!(
            out.stdout.contains(&format!(
                "const {}: &str = \"[REDACTED:{kind}]\";\n",
                kind.replace('-', "_")
            )),
            "{kind} not redacted: {}",
            out.stdout
        );
        assert!(!out.stdout.contains(&value), "{kind} leaked");
        assert!(
            !out.stderr.contains(&value),
            "{kind} leaked into the Report"
        );
    }
    assert!(
        out.stdout
            .contains("let db = \"postgres://admin:[REDACTED:url-credentials]@db:5432/app\";\n")
    );
    assert!(
        out.stdout
            .contains("let api_token = \"[REDACTED:assigned-secret]\";\n")
    );
    assert!(!out.stdout.contains("hunter2"));
    assert!(!out.stdout.contains("q8Zr2Lx7Vm4Nt9Bw"));
}

#[test]
fn the_report_counts_redactions_per_file_and_in_total_without_the_secrets() {
    let aws = ["AKIA", "Z7QK2M4XW9TB3HNP"].concat();
    let slack = ["xoxb", "-1234567890-abcdefABCDEF"].concat();
    let dir = fixture(&[
        (
            "a.rs",
            &format!("let a = \"{aws}\";\nlet b = \"{slack}\";\n"),
        ),
        ("b.rs", "fn main() {}\n"),
    ]);

    let out = run(dir.path(), &["a.rs", "b.rs"]);

    let lines: Vec<&str> = out.stderr.lines().collect();
    assert!(lines[0].ends_with("  saved  redactions"), "{}", out.stderr);
    assert!(
        lines[1].starts_with(" a.rs ") && lines[1].ends_with("  2"),
        "{}",
        out.stderr
    );
    assert!(
        lines[2].starts_with(" b.rs ") && lines[2].ends_with("  0"),
        "{}",
        out.stderr
    );
    assert!(
        lines[4].starts_with(" 2 files ") && lines[4].ends_with("  2"),
        "{}",
        out.stderr
    );
    assert!(lines[5].contains(" · 2 redactions · "), "{}", out.stderr);
    assert!(!out.stderr.contains(&aws) && !out.stderr.contains(&slack));
}

#[test]
fn a_report_without_redactions_has_no_redaction_column_or_count() {
    let dir = fixture(&[("b.rs", "fn main() {}\n")]);

    let out = run(dir.path(), &["b.rs"]);

    assert!(
        out.stderr.lines().next().unwrap().ends_with("  saved"),
        "{}",
        out.stderr
    );
    assert!(!out.stderr.contains("redaction"), "{}", out.stderr);
}

#[test]
fn no_redact_passes_secrets_through_and_warns_even_when_quiet() {
    let aws = ["AKIA", "Z7QK2M4XW9TB3HNP"].concat();
    let dir = fixture(&[("a.rs", &format!("let a = \"{aws}\";\n"))]);

    let out = run(dir.path(), &["a.rs", "--no-redact", "-q"]);

    assert!(out.outcome.is_ok());
    assert!(out.stdout.contains(&aws));
    assert_eq!(
        out.stderr,
        "warning: Redaction is disabled by --no-redact; secrets in Source Files reach the Bundle\n"
    );
}
