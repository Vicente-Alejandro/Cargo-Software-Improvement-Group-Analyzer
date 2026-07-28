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
        .filter_map(|path| parse_and_analyze(&engine, path))
        .flatten()
        .collect();
    Ok(metrics)
}

fn parse_and_analyze(engine: &VolumeEngine, path: PathBuf) -> Option<Vec<FunctionMetric>> {
    let source = std::fs::read_to_string(&path).ok()?;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .ok()?;
    let tree = parser.parse(&source, None)?;
    Some(engine.analyze_tree(&path, &source, &tree))
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
