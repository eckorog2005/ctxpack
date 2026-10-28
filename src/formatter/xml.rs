use std::fmt::Write;

use super::{BundleMeta, FileBlock};

pub fn render(meta: &BundleMeta, files: &[FileBlock]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "<bundle tool=\"pithpack\" version=\"{}\" mode=\"{}\" files=\"{}\">",
        env!("CARGO_PKG_VERSION"),
        meta.mode,
        files.len()
    );
    if meta.tree {
        let paths: Vec<&str> = files.iter().map(|f| f.display).collect();
        out.push_str("<tree>\n");
        out.push_str(&super::file_tree(&paths));
        out.push_str("</tree>\n");
    }
    for file in files {
        let lang = file.lang.map_or("text", |l| l.name());
        let _ = writeln!(
            out,
            "<file path=\"{}\" lang=\"{lang}\" mode=\"{}\">",
            attr(file.display),
            file.mode
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
