use std::fmt::Write;

use super::{BundleMeta, FileBlock};

pub fn render(meta: &BundleMeta, files: &[FileBlock]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "<bundle tool=\"pithpack\" version=\"{}\" mode=\"{}\" tokenizer=\"{}\" files=\"{}\" tokens=\"{}\">",
        env!("CARGO_PKG_VERSION"),
        meta.mode,
        meta.tokenizer,
        files.len(),
        meta.tokens
    );
    if meta.tree {
        let paths: Vec<&str> = files.iter().map(|f| f.display).collect();
        out.push_str("<tree>\n");
        out.push_str(&super::file_tree(&paths));
        out.push_str("</tree>\n");
    }
    for file in files {
        let lang = crate::parser::lang_name(file.lang);
        let _ = writeln!(
            out,
            "<file path=\"{}\" lang=\"{lang}\" mode=\"{}\" tokens=\"{}\">",
            attr(file.display),
            file.mode,
            file.tokens
        );
        out.push_str(&file.text.replace("</file>", "<\\/file>"));
        if !file.text.is_empty() && !file.text.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("</file>\n");
    }
    out.push_str("</bundle>\n");
    out
}

fn attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
