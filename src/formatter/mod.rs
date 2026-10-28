mod tree;
mod xml;

pub use tree::file_tree;

use crate::parser::Language;

pub struct FileBlock<'a> {
    pub display: &'a str,
    pub lang: Option<Language>,
    pub mode: &'static str,
    pub tokens: usize,
    pub text: &'a str,
}

pub struct BundleMeta {
    pub mode: &'static str,
    pub tokenizer: &'static str,
    /// The whole Bundle's count, known only after a first render.
    pub tokens: usize,
    pub tree: bool,
}

pub fn render(meta: &BundleMeta, files: &[FileBlock]) -> String {
    xml::render(meta, files)
}
