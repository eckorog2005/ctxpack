mod common;

use assert_cmd::Command;
use common::{fixture, run};
use pithpack::Outcome;

fn bundle_tokens(bundle: &str) -> usize {
    let header = bundle.lines().next().unwrap();
    let (_, rest) = header.split_once(" tokens=\"").unwrap();
    rest.split('"').next().unwrap().parse().unwrap()
}

fn tokens_of(args: &[&str]) -> usize {
    let dir = fixture(&[("a.txt", "hello world\n")]);
    bundle_tokens(&run(dir.path(), args).stdout)
}

#[test]
fn a_bundle_at_or_under_budget_is_ok_without_a_warning() {
    let total = tokens_of(&["a.txt"]).to_string();
    let dir = fixture(&[("a.txt", "hello world\n")]);

    let out = run(dir.path(), &["a.txt", "--budget", &total]);

    assert!(matches!(out.outcome, Ok(Outcome::Ok)));
    assert!(!out.stderr.contains("budget"), "{}", out.stderr);
}

#[test]
fn a_bundle_over_budget_is_still_written_and_warned_about() {
    let total = tokens_of(&["a.txt"]);
    let dir = fixture(&[("a.txt", "hello world\n")]);

    let out = run(dir.path(), &["a.txt", "--budget", &(total - 1).to_string()]);

    assert!(matches!(out.outcome, Ok(Outcome::OverBudget)));
    assert!(out.stdout.starts_with("<bundle "), "{}", out.stdout);
    assert!(
        out.stderr.contains(&format!(
            "warning: bundle is {total} tokens, budget {}\n",
            total - 1
        )),
        "{}",
        out.stderr
    );
}

#[test]
fn the_warning_groups_digits_and_survives_quiet() {
    let body = "word ".repeat(1500);
    let dir = fixture(&[("a.txt", &body)]);

    let out = run(dir.path(), &["a.txt", "--budget", "1", "-q"]);

    assert!(matches!(out.outcome, Ok(Outcome::OverBudget)));
    let total = bundle_tokens(&out.stdout);
    assert!(total >= 1000);
    let grouped = format!("{},{:03}", total / 1000, total % 1000);
    assert_eq!(
        out.stderr,
        format!("warning: bundle is {grouped} tokens, budget 1\n")
    );
}

#[test]
fn the_exit_code_is_3_over_budget_and_0_otherwise() {
    let dir = fixture(&[("a.txt", "hello world\n")]);
    let run = |budget: &str| {
        Command::cargo_bin("pithpack")
            .unwrap()
            .current_dir(dir.path())
            .args(["a.txt", "-q", "--budget", budget])
            .assert()
    };

    run("1")
        .code(3)
        .stdout(predicates::str::starts_with("<bundle "));
    run("100000").code(0);
}
