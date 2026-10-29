pub(super) struct Rule {
    pub kind: &'static str,
    /// The first participating capture group is the secret span; without groups, the whole match.
    pub pattern: &'static str,
    /// Rejects a matched span that is shaped right but not secret enough.
    pub accept: fn(&str) -> bool,
}

fn always(_: &str) -> bool {
    true
}

fn high_entropy(literal: &str) -> bool {
    literal.chars().count() >= 16 && shannon_entropy(literal) >= 3.5
}

/// Bits per character.
fn shannon_entropy(text: &str) -> f64 {
    let mut counts = std::collections::HashMap::new();
    for c in text.chars() {
        *counts.entry(c).or_insert(0usize) += 1;
    }
    let len = text.chars().count() as f64;
    counts
        .values()
        .map(|&n| {
            let p = n as f64 / len;
            -p * p.log2()
        })
        .sum()
}

/// Where spans overlap, the earlier rule wins, so `sk-ant-…` is an Anthropic key, not OpenAI's.
pub(super) const BUILTIN: &[Rule] = &[
    Rule {
        kind: "aws-access-key",
        pattern: r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b",
        accept: always,
    },
    Rule {
        kind: "github-token",
        pattern: r"\b(?:gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{22,})\b",
        accept: always,
    },
    Rule {
        kind: "anthropic-key",
        pattern: r"\bsk-ant-[A-Za-z0-9_-]{20,}\b",
        accept: always,
    },
    Rule {
        kind: "openai-key",
        pattern: r"\bsk-(?:proj-)?[A-Za-z0-9_-]{20,}\b",
        accept: always,
    },
    Rule {
        kind: "google-api-key",
        pattern: r"\bAIza[0-9A-Za-z_-]{35}\b",
        accept: always,
    },
    Rule {
        kind: "slack-token",
        pattern: r"\bxox[abprs]-[A-Za-z0-9-]{10,}\b",
        accept: always,
    },
    Rule {
        kind: "stripe-key",
        pattern: r"\b(?:sk|rk)_(?:live|test)_[A-Za-z0-9]{20,}\b",
        accept: always,
    },
    Rule {
        kind: "jwt",
        pattern: r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b",
        accept: always,
    },
    Rule {
        kind: "private-key",
        // An unterminated block is redacted to the end so a truncated paste still leaks nothing.
        pattern: r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?(?:-----END [A-Z ]*PRIVATE KEY-----|\z)",
        accept: always,
    },
    Rule {
        kind: "url-credentials",
        pattern: r"[a-z]+://[^/\s:@]+:([^/\s@]+)@",
        accept: always,
    },
    Rule {
        kind: "assigned-secret",
        pattern: concat!(
            r"(?i)\b[a-z0-9_.-]*(?:secret|token|passw(?:or)?d|api_?key|auth)[a-z0-9_.-]*",
            r#"["']?\s*(?:=>|:=|=|:)\s*"#,
            r#"(?:"([^"\n]+)"|'([^'\n]+)'|`([^`\n]+)`)"#,
        ),
        accept: high_entropy,
    },
];
