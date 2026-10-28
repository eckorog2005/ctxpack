#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    Tsx,
    JavaScript,
    Jsx,
    Go,
    Html,
    Css,
}

impl Language {
    pub fn name(self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::TypeScript => "typescript",
            Language::Tsx => "tsx",
            Language::JavaScript => "javascript",
            Language::Jsx => "jsx",
            Language::Go => "go",
            Language::Html => "html",
            Language::Css => "css",
        }
    }
}

pub fn lang_name(lang: Option<Language>) -> &'static str {
    lang.map_or("text", Language::name)
}

/// `None` means Plain Text.
pub fn detect(path: &str) -> Option<Language> {
    let (_, ext) = path.rsplit_once('.')?;
    let lang = match ext.to_ascii_lowercase().as_str() {
        "rs" => Language::Rust,
        "py" | "pyi" => Language::Python,
        "ts" | "mts" | "cts" => Language::TypeScript,
        "tsx" => Language::Tsx,
        "js" | "mjs" | "cjs" => Language::JavaScript,
        "jsx" => Language::Jsx,
        "go" => Language::Go,
        "html" | "htm" => Language::Html,
        "css" => Language::Css,
        _ => return None,
    };
    Some(lang)
}
