use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

const WINDOW_SIZE: usize = 6;

use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct DuplicationBlock {
    pub file_path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Default)]
pub struct DuplicationResult {
    pub percentage: f32,
    pub blocks: Vec<DuplicationBlock>,
}

#[must_use]
pub fn calculate_duplication(files: &[PathBuf]) -> DuplicationResult {
    let contents: Vec<String> = files
        .par_iter()
        .map(|f| fs::read_to_string(f).unwrap_or_default())
        .collect();

    let mut all_lines: Vec<(PathBuf, Vec<(usize, &str)>)> = Vec::new();
    let mut hash_counts: HashMap<u64, usize> = HashMap::with_capacity(files.len() * 100);
    for (i, f) in files.iter().enumerate() {
        let lines = extract_lines(&contents[i]);
        count_hashes(&lines, &mut hash_counts);
        all_lines.push((f.clone(), lines));
    }
    compute_percentage(&all_lines, &hash_counts)
}

#[rustfmt::skip]
fn extract_lines(content: &str) -> Vec<(usize, &str)> {
    let test_rows = get_test_rows(content);
    content.lines().enumerate()
        .filter(|(i, _)| !test_rows.iter().any(|&(s, e)| i >= &s && i <= &e))
        .map(|(i, l)| (i, l.trim()))
        .filter(|(_, t)| !t.is_empty() && !t.starts_with("//"))
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
        check_test_mod(child, content, test_rows);
        find_test_modules(child, content, test_rows, depth + 1);
    }
}

fn check_test_mod(child: tree_sitter::Node, content: &[u8], test_rows: &mut Vec<(usize, usize)>) {
    let is_test_attr = child.kind() == "attribute_item"
        && child.utf8_text(content).unwrap_or("").contains("cfg(test)");
    if is_test_attr {
        if let Some(next) = skip_trivia(child.next_sibling()).filter(|n| n.kind() == "mod_item") {
            test_rows.push((next.start_position().row, next.end_position().row));
        }
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

fn count_hashes(lines: &[(usize, &str)], counts: &mut HashMap<u64, usize>) {
    for w in lines.windows(WINDOW_SIZE) {
        *counts.entry(hash_window(w)).or_insert(0) += 1;
    }
}

fn hash_window(window: &[(usize, &str)]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for (_, line) in window {
        line.hash(&mut hasher);
    }
    hasher.finish()
}

#[rustfmt::skip]
fn compute_percentage(all_lines: &[(PathBuf, Vec<(usize, &str)>)], counts: &HashMap<u64, usize>) -> DuplicationResult {
    let (mut dup, mut tot, mut blocks) = (0, 0, Vec::new());
    for (path, lines) in all_lines {
        tot += lines.len();
        let file_blocks = get_dup_blocks(lines, counts);
        dup += file_blocks.iter().map(|(s, e)| (e - s) + 1).sum::<usize>();
        collect_dup_blocks(path, lines, &file_blocks, &mut blocks);
    }
    let percentage = if tot == 0 { 0.0 } else { (dup as f32 / tot as f32) * 100.0 };
    DuplicationResult { percentage, blocks }
}

fn collect_dup_blocks(
    path: &Path,
    lines: &[(usize, &str)],
    file_blocks: &[(usize, usize)],
    blocks: &mut Vec<DuplicationBlock>,
) {
    for &(s, e) in file_blocks {
        blocks.push(DuplicationBlock {
            file_path: path.to_path_buf(),
            start_line: lines[s].0 + 1,
            end_line: lines[e].0 + 1,
        });
    }
}

fn mark_dup_lines(lines: &[(usize, &str)], counts: &HashMap<u64, usize>) -> Vec<bool> {
    let mut is_dup = vec![false; lines.len()];
    for (i, w) in lines.windows(WINDOW_SIZE).enumerate() {
        if *counts.get(&hash_window(w)).unwrap_or(&0) > 1 {
            for j in 0..WINDOW_SIZE {
                is_dup[i + j] = true;
            }
        }
    }
    is_dup
}

#[rustfmt::skip]
fn get_dup_blocks(lines: &[(usize, &str)], counts: &HashMap<u64, usize>) -> Vec<(usize, usize)> {
    let is_dup = mark_dup_lines(lines, counts);
    let (mut blocks, mut start) = (Vec::new(), None);
    for (i, &d) in is_dup.iter().enumerate() {
        process_dup_step(d, i, &mut start, &mut blocks);
    }
    if let Some(s) = start { blocks.push((s, is_dup.len() - 1)); }
    blocks
}

fn process_dup_step(
    d: bool,
    i: usize,
    start: &mut Option<usize>,
    blocks: &mut Vec<(usize, usize)>,
) {
    if d && start.is_none() {
        *start = Some(i);
    } else if !d && start.is_some() {
        blocks.push((start.unwrap(), i - 1));
        *start = None;
    }
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
        assert_eq!(lines[0].1, "fn main() {");
        assert_eq!(lines[0].0, 0);
        assert_eq!(lines[1].1, "let x = 1;");
        assert_eq!(lines[1].0, 3);
        assert_eq!(lines[2].1, "}");
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

        let res = calculate_duplication(&[file1, file2]);
        assert!(res.percentage.abs() < f32::EPSILON);
        assert!(res.blocks.is_empty());
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

        let res = calculate_duplication(&[file1, file2]);
        assert!((res.percentage - 100.0).abs() < f32::EPSILON);
        assert_eq!(res.blocks.len(), 2);
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

        let res = calculate_duplication(&[file1, file2]);
        assert!(res.percentage.abs() < f32::EPSILON);
    }
}
