use std::fmt::Write;

use super::{BundleMeta, FileBlock};

pub fn render(meta: &BundleMeta, files: &[FileBlock]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# pithpack bundle\n\n\
         - version: {}\n\
         - mode: {}\n\
         - tokenizer: {}\n\
         - files: {}\n\
         - tokens: {}",
        env!("CARGO_PKG_VERSION"),
        meta.mode,
        meta.tokenizer,
        files.len(),
        meta.tokens
    );
    if meta.tree {
        let paths: Vec<&str> = files.iter().map(|f| f.display).collect();
        out.push_str("\n## File tree\n\n");
        push_fenced(&mut out, "text", &super::file_tree(&paths));
    }
    for file in files {
        let _ = write!(
            out,
            "\n## {}\n\nmode: {} · tokens: {}\n\n",
            file.display, file.mode, file.tokens
        );
        push_fenced(&mut out, crate::parser::lang_name(file.lang), file.text);
    }
    out
}

fn push_fenced(out: &mut String, info: &str, text: &str) {
    let fence = fence_for(text);
    let _ = writeln!(out, "{fence}{info}");
    out.push_str(text);
    if !text.is_empty() && !text.ends_with('\n') {
        out.push('\n');
    }
    let _ = writeln!(out, "{fence}");
}

/// One backtick longer than the longest run in `text`, and never under three.
fn fence_for(text: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for byte in text.bytes() {
        if byte == b'`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    "`".repeat((longest + 1).max(3))
}

#[cfg(test)]
mod tests {
    use super::fence_for;

    #[test]
    fn fence_is_three_backticks_when_the_text_has_none() {
        assert_eq!(fence_for("fn main() {}\n"), "```");
    }

    #[test]
    fn fence_stays_at_three_for_runs_shorter_than_three() {
        assert_eq!(fence_for("`a` and ``b``"), "```");
    }

    #[test]
    fn fence_outgrows_a_triple_backtick_run() {
        assert_eq!(fence_for("```rust\ncode\n```\n"), "````");
    }

    #[test]
    fn fence_is_one_longer_than_the_longest_run_anywhere() {
        assert_eq!(fence_for("`` then ````` then ```"), "``````");
    }
}
