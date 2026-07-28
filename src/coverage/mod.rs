use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Copy)]
pub struct Coverage {
    pub hit: usize,
    pub total: usize,
}

impl Coverage {
    pub fn percent(&self) -> f32 {
        if self.total == 0 {
            100.0
        } else {
            (self.hit as f32 / self.total as f32) * 100.0
        }
    }
}

pub fn read_lcov(project_dir: &Path) -> Option<HashMap<PathBuf, Coverage>> {
    let lcov_path = project_dir.join("coverage.lcov");
    let content = fs::read_to_string(&lcov_path).ok()?;
    parse_lcov_content(project_dir, &content)
}

fn parse_lcov_content(dir: &Path, content: &str) -> Option<HashMap<PathBuf, Coverage>> {
    let mut map: HashMap<PathBuf, Coverage> = HashMap::new();
    let mut current_file = None;
    for line in content.lines() {
        process_lcov_line(line, dir, &mut current_file, &mut map);
    }
    Some(map)
}

fn process_lcov_line(
    line: &str,
    dir: &Path,
    current_file: &mut Option<PathBuf>,
    map: &mut HashMap<PathBuf, Coverage>,
) {
    if let Some(file) = line.strip_prefix("SF:") {
        let p = dir.join(file.trim());
        *current_file = Some(p.canonicalize().unwrap_or(p));
    } else if let Some(da) = line.strip_prefix("DA:") {
        if let Some(path) = current_file {
            parse_da_line(da, path, map);
        }
    }
}

fn parse_da_line(da: &str, path: &Path, map: &mut HashMap<PathBuf, Coverage>) {
    let parts: Vec<&str> = da.split(',').collect();
    if parts.len() >= 2 {
        if let Ok(hits) = parts[1].trim().parse::<usize>() {
            let entry = map.entry(path.to_path_buf()).or_default();
            entry.total += 1;
            if hits > 0 {
                entry.hit += 1;
            }
        }
    }
}
