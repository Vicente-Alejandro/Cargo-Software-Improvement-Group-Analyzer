use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn get_frequencies(dir: &Path) -> anyhow::Result<HashMap<PathBuf, usize>> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(["log", "--since=6 months ago", "--name-only", "--format="])
        .output();

    let stdout = match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
        _ => return Ok(HashMap::new()),
    };

    parse_churn(dir, &stdout)
}

fn parse_churn(dir: &Path, stdout: &str) -> anyhow::Result<HashMap<PathBuf, usize>> {
    let mut freqs = HashMap::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let path = dir.join(line);
        let key = path.canonicalize().unwrap_or(path);
        *freqs.entry(key).or_insert(0) += 1;
    }
    Ok(freqs)
}
