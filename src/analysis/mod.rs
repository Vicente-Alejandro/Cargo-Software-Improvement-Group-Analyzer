pub mod volume;

pub use volume::{FunctionMetric, VolumeEngine};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn run_analysis(dir: &Path) -> anyhow::Result<Vec<FunctionMetric>> {
    let engine = VolumeEngine::new();
    
    // 1. Gather all .rs files recursively using std::fs to maintain zero-bloat
    let mut files = Vec::new();
    gather_files(dir, &mut files)?;
    
    // 2. Process all files in parallel using rayon
    let all_metrics: Vec<FunctionMetric> = files
        .into_par_iter()
        .filter_map(|path| {
            if let Ok(source) = std::fs::read_to_string(&path) {
                if let Ok(metrics) = engine.analyze_file(&path, &source) {
                    return Some(metrics);
                }
            }
            None
        })
        .flatten()
        .collect();

    Ok(all_metrics)
}

fn gather_files(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                // Skip target and hidden directories
                if file_name.starts_with('.') || file_name == "target" {
                    continue;
                }
                gather_files(&path, files)?;
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    Ok(())
}
