use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

const WINDOW_SIZE: usize = 6;

use rayon::prelude::*;

#[must_use]
pub fn calculate_duplication(files: &[PathBuf]) -> f32 {
    let contents: Vec<String> = files
        .par_iter()
        .map(|f| fs::read_to_string(f).unwrap_or_default())
        .collect();

    let mut all_lines: Vec<(PathBuf, Vec<&str>)> = Vec::new();
    let mut hash_counts: HashMap<u64, usize> = HashMap::with_capacity(files.len() * 100);
    for (i, f) in files.iter().enumerate() {
        let lines = extract_lines(&contents[i]);
        count_hashes(&lines, &mut hash_counts);
        all_lines.push((f.clone(), lines));
    }
    compute_percentage(&all_lines, &hash_counts)
}

#[rustfmt::skip]
fn extract_lines(content: &str) -> Vec<&str> {
    let test_rows = get_test_rows(content);
    content.lines().enumerate()
        .filter(|(i, _)| !test_rows.iter().any(|&(s, e)| i >= &s && i <= &e))
        .map(|(_, l)| l.trim())
        .filter(|t| !t.is_empty() && !t.starts_with("//"))
        .collect()
}

fn get_test_rows(content: &str) -> Vec<(usize, usize)> {
    let mut rows = Vec::new();
    let mut parser = tree_sitter::Parser::new();
    if parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .is_ok()
    {
        if let Some(tree) = parser.parse(content, None) {
            find_test_modules(tree.root_node(), content.as_bytes(), &mut rows, 0);
        }
    }
    rows
}

#[rustfmt::skip]
fn find_test_modules(node: tree_sitter::Node, content: &[u8], test_rows: &mut Vec<(usize, usize)>, depth: usize) {
    if depth > 100 { return; }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let is_test_attr = child.kind() == "attribute_item" && child.utf8_text(content).unwrap_or("").contains("cfg(test)");
        if is_test_attr {
            if let Some(next) = skip_trivia(child.next_sibling()).filter(|n| n.kind() == "mod_item") {
                test_rows.push((next.start_position().row, next.end_position().row));
            }
        }
        find_test_modules(child, content, test_rows, depth + 1);
    }
}

fn skip_trivia(mut node: Option<tree_sitter::Node>) -> Option<tree_sitter::Node> {
    while let Some(n) = node {
        match n.kind() {
            "line_comment" | "block_comment" | "attribute_item" => node = n.next_sibling(),
            _ => return Some(n),
        }
    }
    None
}

fn count_hashes(lines: &[&str], counts: &mut HashMap<u64, usize>) {
    for w in lines.windows(WINDOW_SIZE) {
        *counts.entry(hash_window(w)).or_insert(0) += 1;
    }
}

fn hash_window(window: &[&str]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for line in window {
        line.hash(&mut hasher);
    }
    hasher.finish()
}

fn compute_percentage(all_lines: &[(PathBuf, Vec<&str>)], counts: &HashMap<u64, usize>) -> f32 {
    let (mut dup, mut tot) = (0, 0);
    for (_, lines) in all_lines {
        tot += lines.len();
        dup += count_dup_lines(lines, counts);
    }
    if tot == 0 {
        0.0
    } else {
        (dup as f32 / tot as f32) * 100.0
    }
}

fn count_dup_lines(lines: &[&str], counts: &HashMap<u64, usize>) -> usize {
    let mut is_dup = vec![false; lines.len()];
    for (i, w) in lines.windows(WINDOW_SIZE).enumerate() {
        if *counts.get(&hash_window(w)).unwrap_or(&0) > 1 {
            for j in 0..WINDOW_SIZE {
                is_dup[i + j] = true;
            }
        }
    }
    is_dup.into_iter().filter(|&d| d).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_extract_lines() {
        let content = "fn main() {\n    // comment\n\n    let x = 1;\n}";
        let lines = extract_lines(content);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "fn main() {");
        assert_eq!(lines[1], "let x = 1;");
        assert_eq!(lines[2], "}");
    }

    #[test]
    fn test_calculate_duplication_no_dups() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("f1.rs");
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "1\n2\n3\n4\n5\n6\n7\n8").unwrap();

        let file2 = dir.path().join("f2.rs");
        let mut f2 = File::create(&file2).unwrap();
        writeln!(f2, "a\nb\nc\nd\ne\nf\ng\nh").unwrap();

        let pct = calculate_duplication(&[file1, file2]);
        assert_eq!(pct, 0.0);
    }

    #[test]
    fn test_calculate_duplication_with_dups() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("f1.rs");
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "a\nb\nc\nd\ne\nf\ng").unwrap();

        let file2 = dir.path().join("f2.rs");
        let mut f2 = File::create(&file2).unwrap();
        writeln!(f2, "a\nb\nc\nd\ne\nf\ng").unwrap();

        let pct = calculate_duplication(&[file1, file2]);
        assert_eq!(pct, 100.0);
    }

    #[test]
    fn test_calculate_duplication_small_files() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("f1.rs");
        let mut f1 = File::create(&file1).unwrap();
        writeln!(f1, "a\nb\nc\nd\ne").unwrap();

        let file2 = dir.path().join("f2.rs");
        let mut f2 = File::create(&file2).unwrap();
        writeln!(f2, "a\nb\nc\nd\ne").unwrap();

        let pct = calculate_duplication(&[file1, file2]);
        assert_eq!(pct, 0.0);
    }
}
