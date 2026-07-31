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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_parse_churn() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("file1.rs");
        File::create(&file1_path).unwrap();

        let stdout = "file1.rs\n\nfile1.rs\nfile2.rs\n";
        let freqs = parse_churn(dir.path(), stdout).unwrap();

        assert_eq!(freqs.len(), 2);
        let key1 = file1_path.canonicalize().unwrap_or(file1_path);
        let key2 = dir.path().join("file2.rs");

        assert_eq!(freqs.get(&key1), Some(&2));
        assert_eq!(freqs.get(&key2), Some(&1));
    }

    #[test]
    fn test_get_frequencies_no_git() {
        let dir = tempdir().unwrap();
        let freqs = get_frequencies(dir.path()).unwrap();
        assert!(freqs.is_empty());
    }
}
