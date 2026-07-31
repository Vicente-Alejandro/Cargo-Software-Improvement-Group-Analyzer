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

pub fn load_or_generate_lcov(
    project_dir: &Path,
    skip_auto: bool,
) -> Option<HashMap<PathBuf, Coverage>> {
    let lcov_path = project_dir.join("coverage.lcov");
    if let Ok(content) = fs::read_to_string(&lcov_path) {
        return parse_lcov_content(project_dir, &content);
    }

    if skip_auto || !generate_lcov(project_dir) {
        return None;
    }

    let content = fs::read_to_string(&lcov_path).ok()?;
    parse_lcov_content(project_dir, &content)
}

#[rustfmt::skip]
fn generate_lcov(dir: &Path) -> bool {
    if std::env::var_os("LLVM_PROFILE_FILE").is_some() { return false; }
    let v_stat = std::process::Command::new("cargo").args(["llvm-cov", "--version"]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
    if !v_stat.is_ok_and(|s| s.success()) { return false; }
    use owo_colors::OwoColorize;
    use std::io::Write;
    let Ok(mut c) = std::process::Command::new("cargo").args(["llvm-cov", "--lcov", "--output-path", "coverage.lcov"]).current_dir(dir).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn() else { return false; };
    
    let mut progress: f32 = 0.0;
    for _ in 0..360 {
        if let Ok(Some(s)) = c.try_wait() { 
            print!("\r{} ⏳ Generating coverage data via cargo-llvm-cov... 100%   \n", "[cargo-sig]".bold().cyan());
            let _ = std::io::stdout().flush();
            return s.success(); 
        }
        progress += (99.0 - progress) * 0.05;
        print!("\r{} ⏳ Generating coverage data via cargo-llvm-cov... {:.0}%   ", "[cargo-sig]".bold().cyan(), progress);
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    let _ = c.kill();
    false
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
    if let Some(path_str) = line.strip_prefix("SF:") {
        let p = dir.join(path_str.trim());
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
    let (mut tc, mut wc) = (0, 0.0);
    for (path, file_cov) in cov {
        let fc = churns.get(path).copied().unwrap_or(0);
        if fc > 0 { tc += fc; wc += file_cov.percent() * (fc as f32); }
    }
    if tc > 0 { return wc / (tc as f32); }
    let t_tot: usize = cov.values().map(|c| c.total).sum();
    let t_hit: usize = cov.values().map(|c| c.hit).sum();
    if t_tot == 0 { 100.0 } else { (t_hit as f32 / t_tot as f32) * 100.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_coverage_percent() {
        let cov = Coverage { hit: 5, total: 10 };
        assert_eq!(cov.percent(), 50.0);
        let cov_empty = Coverage { hit: 0, total: 0 };
        assert_eq!(cov_empty.percent(), 100.0);
    }

    #[test]
    fn test_parse_lcov_content() {
        let content = "TN:\nSF:src/main.rs\nDA:1,1\nDA:2,0\nend_of_record\n";
        let dir = PathBuf::from(".");
        let map = parse_lcov_content(&dir, content).unwrap();
        let path = dir
            .join("src/main.rs")
            .canonicalize()
            .unwrap_or(dir.join("src/main.rs"));
        let cov = map.get(&path).unwrap();
        assert_eq!(cov.total, 2);
        assert_eq!(cov.hit, 1);
    }

    #[test]
    fn test_churn_weighted_coverage() {
        let mut cov = HashMap::new();
        let path = PathBuf::from("src/main.rs");
        cov.insert(path.clone(), Coverage { hit: 5, total: 10 });

        let mut churns = HashMap::new();
        churns.insert(path.clone(), 10);

        let result = churn_weighted_coverage(&cov, &churns);
        assert_eq!(result, 50.0);

        let churns_empty = HashMap::new();
        let result_fallback = churn_weighted_coverage(&cov, &churns_empty);
        assert_eq!(result_fallback, 50.0);
    }

    #[test]
    fn test_load_lcov_skip_auto() {
        let dir = tempdir().unwrap();
        let lcov_path = dir.path().join("coverage.lcov");
        let mut file = File::create(&lcov_path).unwrap();
        writeln!(file, "SF:test.rs\nDA:1,1\n").unwrap();

        let map = load_or_generate_lcov(dir.path(), true).unwrap();
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_load_lcov_no_file_skip_auto() {
        let dir = tempdir().unwrap();
        let map = load_or_generate_lcov(dir.path(), true);
        assert!(map.is_none());
    }

    #[test]
    fn test_generate_lcov() {
        let dir = tempfile::tempdir().unwrap();
        let result = generate_lcov(dir.path());
        assert_eq!(result, false);
    }
}
