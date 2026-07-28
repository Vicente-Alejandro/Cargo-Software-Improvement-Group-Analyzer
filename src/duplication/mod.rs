use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

const WINDOW_SIZE: usize = 6;

pub fn calculate_duplication(files: &[PathBuf]) -> f32 {
    let mut all_lines: Vec<(PathBuf, Vec<String>)> = Vec::new();
    let mut hash_counts: HashMap<u64, usize> = HashMap::new();
    for f in files {
        let lines = extract_lines(f);
        count_hashes(&lines, &mut hash_counts);
        all_lines.push((f.clone(), lines));
    }
    compute_percentage(&all_lines, &hash_counts)
}

fn extract_lines(path: &Path) -> Vec<String> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            lines.push(trimmed.to_string());
        }
    }
    lines
}

fn count_hashes(lines: &[String], counts: &mut HashMap<u64, usize>) {
    for w in lines.windows(WINDOW_SIZE) {
        *counts.entry(hash_window(w)).or_insert(0) += 1;
    }
}

fn hash_window(window: &[String]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for line in window {
        line.hash(&mut hasher);
    }
    hasher.finish()
}

fn compute_percentage(all_lines: &[(PathBuf, Vec<String>)], counts: &HashMap<u64, usize>) -> f32 {
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

fn count_dup_lines(lines: &[String], counts: &HashMap<u64, usize>) -> usize {
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
