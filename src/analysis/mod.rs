pub mod volume;

use rayon::prelude::*;
use std::path::{Path, PathBuf};
pub use volume::{FunctionMetric, VolumeEngine};

pub fn run_analysis(dir: &Path) -> anyhow::Result<Vec<FunctionMetric>> {
    let engine = VolumeEngine::new();
    let mut files = Vec::new();
    gather_files(dir, &mut files)?;
    let metrics = files
        .into_par_iter()
        .filter_map(|path| process_file(&engine, path))
        .flatten()
        .collect();
    Ok(metrics)
}

fn process_file(engine: &VolumeEngine, path: PathBuf) -> Option<Vec<FunctionMetric>> {
    match parse_and_analyze(engine, &path) {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("⚠️ Skipping {}: {}", path.display(), e);
            None
        }
    }
}

fn parse_and_analyze(engine: &VolumeEngine, path: &Path) -> anyhow::Result<Vec<FunctionMetric>> {
    if std::fs::metadata(path)?.len() > 2_000_000 {
        anyhow::bail!("File exceeds 2MB limit (DoS protection)");
    }
    let source = std::fs::read_to_string(path)?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    let tree = parser
        .parse(&source, None)
        .ok_or_else(|| anyhow::anyhow!("Parse fail"))?;
    Ok(engine.analyze_tree(path, &source, &tree))
}

fn gather_files(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        process_entry(&entry?.path(), files)?;
    }
    Ok(())
}

fn process_entry(path: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if path.is_dir() {
        if is_valid_dir(path) {
            gather_files(path, files)?;
        }
    } else if path.extension().is_some_and(|e| e == "rs") {
        files.push(path.to_path_buf());
    }
    Ok(())
}

fn is_valid_dir(path: &Path) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    !name.starts_with('.') && name != "target"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_is_valid_dir() {
        assert!(is_valid_dir(Path::new("src")));
        assert!(!is_valid_dir(Path::new(".git")));
        assert!(!is_valid_dir(Path::new("target")));
    }

    #[test]
    fn test_gather_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn test() {{}}").unwrap();

        let sub_dir = dir.path().join("sub");
        std::fs::create_dir(&sub_dir).unwrap();
        let sub_file = sub_dir.join("sub_test.rs");
        let mut file2 = File::create(&sub_file).unwrap();
        writeln!(file2, "fn sub_test() {{}}").unwrap();

        let git_dir = dir.path().join(".git");
        std::fs::create_dir(&git_dir).unwrap();
        let git_file = git_dir.join("ignore.rs");
        let mut file3 = File::create(&git_file).unwrap();
        writeln!(file3, "fn ignore() {{}}").unwrap();

        let mut gathered = Vec::new();
        gather_files(dir.path(), &mut gathered).unwrap();

        assert_eq!(gathered.len(), 2);
    }

    #[test]
    fn test_run_analysis_and_parse() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{\n    println!(\"Hello\");\n}}").unwrap();

        let metrics = run_analysis(dir.path()).unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].function_name, "main");
    }
}
