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

pub fn load_or_generate_lcov(project_dir: &Path) -> Option<HashMap<PathBuf, Coverage>> {
    let lcov_path = project_dir.join("coverage.lcov");
    if let Ok(content) = fs::read_to_string(&lcov_path) {
        return parse_lcov_content(project_dir, &content);
    }

    // Check if cargo-llvm-cov is installed
    let version_status = std::process::Command::new("cargo")
        .args(["llvm-cov", "--version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if !version_status.is_ok_and(|s| s.success()) {
        return None; // Not installed
    }

    use owo_colors::OwoColorize;
    println!(
        "{} ⏳ Generating coverage data via cargo-llvm-cov...",
        "[cargo-sig]".bold().cyan()
    );

    let status = std::process::Command::new("cargo")
        .args(["llvm-cov", "--lcov", "--output-path", "coverage.lcov"])
        .current_dir(project_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;

    if !status.success() {
        return None;
    }

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

#[rustfmt::skip]
pub fn churn_weighted_coverage(cov: &HashMap<PathBuf, Coverage>, churns: &HashMap<PathBuf, usize>) -> f32 {
    let (mut total_churn, mut weighted_cov) = (0, 0.0);
    for (path, file_cov) in cov {
        let fc = churns.get(path).copied().unwrap_or(0);
        if fc > 0 { total_churn += fc; weighted_cov += file_cov.percent() * (fc as f32); }
    }
    if total_churn == 0 {
        let (mut t_hit, mut t_tot) = (0, 0);
        for c in cov.values() { t_hit += c.hit; t_tot += c.total; }
        return if t_tot == 0 { 100.0 } else { (t_hit as f32 / t_tot as f32) * 100.0 };
    }
    weighted_cov / (total_churn as f32)
}
