mod rules;

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

use regex::{Regex, RegexSet};

#[derive(Debug)]
pub struct Rules {
    set: RegexSet,
    rules: Vec<CompiledRule>,
}

#[derive(Debug)]
struct CompiledRule {
    kind: Cow<'static, str>,
    regex: Regex,
    accept: fn(&str) -> bool,
}

/// A config-supplied Redaction rule.
#[derive(Debug, Clone)]
pub struct PatternSpec {
    pub kind: String,
    pub regex: String,
}

#[derive(Debug)]
pub struct InvalidPattern {
    pub kind: String,
    pub source: regex::Error,
}

impl std::fmt::Display for InvalidPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid redaction pattern `{}`: {}",
            self.kind, self.source
        )
    }
}

impl std::error::Error for InvalidPattern {}

impl Rules {
    pub fn builtin() -> &'static Rules {
        static BUILTIN: LazyLock<Rules> =
            LazyLock::new(|| Rules::with_extra(&[]).expect("built-in Redaction rules are valid"));
        &BUILTIN
    }

    /// The built-in rules followed by `extra`; built-ins win where spans overlap.
    pub fn with_extra(extra: &[PatternSpec]) -> Result<Rules, InvalidPattern> {
        let compile = |kind: Cow<'static, str>, pattern: &str, accept| {
            Regex::new(pattern)
                .map(|regex| CompiledRule {
                    kind: kind.clone(),
                    regex,
                    accept,
                })
                .map_err(|source| InvalidPattern {
                    kind: kind.into_owned(),
                    source,
                })
        };
        let mut rules = Vec::new();
        for r in rules::BUILTIN {
            rules.push(compile(Cow::Borrowed(r.kind), r.pattern, r.accept)?);
        }
        for spec in extra {
            rules.push(compile(Cow::Owned(spec.kind.clone()), &spec.regex, |_| {
                true
            })?);
        }
        let set = RegexSet::new(rules.iter().map(|r| r.regex.as_str()))
            .expect("every pattern compiled on its own");
        Ok(Rules { set, rules })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RedactionHit {
    pub kind: Cow<'static, str>,
    /// Byte range of the secret in the original text.
    pub range: Range<usize>,
}

pub struct Redacted<'a> {
    pub text: Cow<'a, str>,
    pub hits: Vec<RedactionHit>,
}

pub fn redact<'a>(text: &'a str, rules: &Rules) -> Redacted<'a> {
    let mut hits = Vec::new();
    for i in rules.set.matches(text).iter() {
        let CompiledRule { regex, accept, .. } = &rules.rules[i];
        for caps in regex.captures_iter(text) {
            let span = caps
                .iter()
                .skip(1)
                .flatten()
                .next()
                .unwrap_or_else(|| caps.get(0).unwrap());
            if accept(span.as_str()) {
                hits.push((i, span.range()));
            }
        }
    }
    hits.sort_by_key(|(rule, range)| (range.start, std::cmp::Reverse(range.end), *rule));
    let mut merged: Vec<RedactionHit> = Vec::new();
    for (rule, range) in hits {
        match merged.last_mut() {
            // Merging rather than dropping a partial overlap keeps its tail from leaking.
            Some(last) if range.start < last.range.end => {
                last.range.end = last.range.end.max(range.end);
            }
            _ => merged.push(RedactionHit {
                kind: rules.rules[rule].kind.clone(),
                range,
            }),
        }
    }
    let hits = merged;
    if hits.is_empty() {
        return Redacted {
            text: Cow::Borrowed(text),
            hits,
        };
    }
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    for hit in &hits {
        out.push_str(&text[at..hit.range.start]);
        out.push_str(&format!("[REDACTED:{}]", hit.kind));
        at = hit.range.end;
    }
    out.push_str(&text[at..]);
    Redacted {
        text: Cow::Owned(out),
        hits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn redacted(text: &str) -> String {
        redact(text, Rules::builtin()).text.into_owned()
    }

    /// Fake secrets are assembled at run time so the source never holds a scannable token.
    fn secret(parts: &[&str]) -> String {
        parts.concat()
    }

    #[test]
    fn each_rule_replaces_only_the_secret_span_with_its_kind() {
        let jwt = secret(&[
            "eyJhbGciOiJIUzI1NiJ9.",
            "eyJzdWIiOiIxMjM0In0.",
            "dBjftJeZ4CVPmB92K27uhbUJU1p1r_wW1gFWFOEjXk",
        ]);
        let cases = [
            (secret(&["AKIA", "Z7QK2M4XW9TB3HNP"]), "aws-access-key"),
            (secret(&["ASIA", "Z7QK2M4XW9TB3HNP"]), "aws-access-key"),
            (
                secret(&["ghp", "_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8"]),
                "github-token",
            ),
            (
                secret(&["ghs", "_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8s9"]),
                "github-token",
            ),
            (
                secret(&["github_pat", "_11ABCDEFG0123456789_abcdefghijABCDEF"]),
                "github-token",
            ),
            (
                secret(&["sk-", "ant-api03-Xy7Kq2Lm9Np4Rs8Tv3Wz6Ab1Cd5"]),
                "anthropic-key",
            ),
            (
                secret(&["sk-", "proj-Xy7Kq2Lm9Np4Rs8Tv3Wz6Ab1"]),
                "openai-key",
            ),
            (secret(&["sk-", "Xy7Kq2Lm9Np4Rs8Tv3Wz6Ab1"]), "openai-key"),
            (
                secret(&["AIza", "SyB7k2Lm9Np4Rs8Tv3Wz6Ab1Cd5Ef0Gh2Jk"]),
                "google-api-key",
            ),
            (secret(&["xoxb", "-1234567890-abcdefABCDEF"]), "slack-token"),
            (
                secret(&["sk_live", "_4eC39HqLyjWDarjtT1zdp7dc"]),
                "stripe-key",
            ),
            (
                secret(&["rk_test", "_4eC39HqLyjWDarjtT1zdp7dc"]),
                "stripe-key",
            ),
            (jwt, "jwt"),
        ];
        for (value, kind) in cases {
            assert_eq!(
                redacted(&format!("let v = \"{value}\";\n")),
                format!("let v = \"[REDACTED:{kind}]\";\n"),
                "{kind}: {value}"
            );
        }
    }

    #[test]
    fn url_credentials_lose_only_the_password() {
        assert_eq!(
            redacted("DATABASE_URL=postgres://admin:hunter2@db.internal:5432/app\n"),
            "DATABASE_URL=postgres://admin:[REDACTED:url-credentials]@db.internal:5432/app\n"
        );
    }

    #[test]
    fn a_private_key_block_is_redacted_whole() {
        let block = secret(&[
            "-----BEGIN RSA PRIVATE ",
            "KEY-----\nMIIEowIBAAKCAQEA7d8Kq\nZ2xLm9Np4Rs8Tv3W==\n-----END RSA PRIVATE KEY-----",
        ]);

        assert_eq!(
            redacted(&format!("const KEY: &str = \"{block}\";\n")),
            "const KEY: &str = \"[REDACTED:private-key]\";\n"
        );
    }

    #[test]
    fn an_unterminated_private_key_is_redacted_to_the_end_of_the_text() {
        let block = secret(&["-----BEGIN OPENSSH PRIVATE ", "KEY-----\nb3BlbnNzaC1rZXk\n"]);

        assert_eq!(
            redacted(&format!("key:\n{block}")),
            "key:\n[REDACTED:private-key]"
        );
    }

    #[test]
    fn a_high_entropy_literal_assigned_to_a_secret_name_is_redacted() {
        for (line, expected) in [
            (
                "const API_KEY = \"q8Zr2Lx7Vm4Nt9Bw\";",
                "const API_KEY = \"[REDACTED:assigned-secret]\";",
            ),
            (
                "client_secret: 'Hq7tW2zP9mXc4Rv8Ld'",
                "client_secret: '[REDACTED:assigned-secret]'",
            ),
            (
                "{\"authToken\": \"Zq81Lm3Vx94Nw2Rt7Ka\"}",
                "{\"authToken\": \"[REDACTED:assigned-secret]\"}",
            ),
            (
                "dbPassword := `Wn4Xc8Vb2Mz6Lk1Jh5`",
                "dbPassword := `[REDACTED:assigned-secret]`",
            ),
            (
                "passwd => \"abcdefghIIJJKKLL\"",
                "passwd => \"[REDACTED:assigned-secret]\"",
            ),
        ] {
            assert_eq!(redacted(line), expected);
        }
    }

    #[test]
    fn short_low_entropy_or_unassigned_literals_are_kept() {
        for line in [
            "password = \"passwordpassword\"",
            "secret = \"abcdefgIIJJKKLLL\"",
            "token = \"q8Zr2Lx7Vm4Nt9B\"",
            "username = \"q8Zr2Lx7Vm4Nt9Bw\"",
            "api_key = os.environ[\"API_KEY_FOR_PRODUCTION\"]",
            "if token == \"q8Zr2Lx7Vm4Nt9Bw\" {}",
        ] {
            assert_eq!(redacted(line), line);
        }
    }

    #[test]
    fn partially_overlapping_spans_are_redacted_as_one_so_no_tail_survives() {
        let line = "token = \"a://u:q8Zr2Lx7Vm4Nt9Bw\"Zk9Q@host";

        let out = redact(line, Rules::builtin());

        assert_eq!(out.text, "token = \"[REDACTED:assigned-secret]@host");
        assert_eq!(out.hits.len(), 1);
    }

    #[test]
    fn text_without_secrets_is_borrowed_unchanged() {
        let text = "fn main() {}\n";

        let out = redact(text, Rules::builtin());

        assert!(matches!(out.text, Cow::Borrowed(t) if t == text));
        assert!(out.hits.is_empty());
    }

    #[test]
    fn near_misses_are_left_alone() {
        for text in [
            secret(&["AKIA", "Z7QK2M4XW9TB3HN"]),
            secret(&["AKIA", "z7qk2m4xw9tb3hnp"]),
            secret(&["XAKIA", "Z7QK2M4XW9TB3HNP"]),
            secret(&["AKIA", "Z7QK2M4XW9TB3HNPQ"]),
            secret(&["ghp", "_A1b2C3d4E5f6G7h8I9j0"]),
            secret(&["ghx", "_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8"]),
            secret(&["github_pat", "_short"]),
            secret(&["sk-", "short"]),
            secret(&["task-", "Xy7Kq2Lm9Np4Rs8Tv3Wz6Ab1"]),
            secret(&["AIza", "SyB7k2Lm9Np4Rs8Tv3Wz"]),
            secret(&["xoxq", "-1234567890-abcdef"]),
            secret(&["xoxb", "-12345"]),
            secret(&["sk_prod", "_4eC39HqLyjWDarjtT1zdp7dc"]),
            secret(&["sk_live", "_4eC39Hq"]),
            secret(&["eyJhbGciOiJIUzI1NiJ9.", "eyJzdWIiOiIxMjM0In0"]),
            "https://admin@example.com/".to_string(),
            "https://example.com:8080/path".to_string(),
            "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkq\n-----END PUBLIC KEY-----".to_string(),
        ] {
            assert_eq!(redacted(&text), text);
        }
    }
}
