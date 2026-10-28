mod tree;
mod xml;

pub use tree::file_tree;

use crate::parser::Language;

pub struct FileBlock<'a> {
    pub display: &'a str,
    pub lang: Option<Language>,
    pub mode: &'static str,
    pub text: &'a str,
}

pub struct BundleMeta {
    pub mode: &'static str,
    pub tree: bool,
}

pub fn render(meta: &BundleMeta, files: &[FileBlock]) -> String {
    xml::render(meta, files)
}
