/// `paths` must be sorted, which keeps every directory's entries contiguous.
pub fn file_tree(paths: &[&str]) -> String {
    let mut out = String::new();
    let mut open: Vec<&str> = Vec::new();
    for path in paths {
        let mut parts: Vec<&str> = path.split('/').collect();
        let file = parts.pop().unwrap_or_default();
        let shared = open.iter().zip(&parts).take_while(|(a, b)| a == b).count();
        open.truncate(shared);
        for dir in &parts[shared..] {
            push_line(&mut out, open.len(), dir, "/");
            open.push(dir);
        }
        push_line(&mut out, open.len(), file, "");
    }
    out
}

fn push_line(out: &mut String, depth: usize, name: &str, suffix: &str) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    out.push_str(name);
    out.push_str(suffix);
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::file_tree;

    #[test]
    fn nests_files_under_their_directories() {
        let tree = file_tree(&[
            ".github/ci.yml",
            "Cargo.toml",
            "src/lib.rs",
            "src/parser/language.rs",
            "src/parser/mod.rs",
            "src/scanner/mod.rs",
        ]);

        assert_eq!(
            tree,
            "\
.github/
  ci.yml
Cargo.toml
src/
  lib.rs
  parser/
    language.rs
    mod.rs
  scanner/
    mod.rs
"
        );
    }

    #[test]
    fn reopens_nothing_when_a_sibling_sorts_between_a_file_and_its_directory() {
        let tree = file_tree(&["a.rs", "a/b.rs", "a0.rs"]);

        assert_eq!(tree, "a.rs\na/\n  b.rs\na0.rs\n");
    }
}
