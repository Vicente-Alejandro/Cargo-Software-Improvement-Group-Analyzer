pub mod volume;

pub use volume::{FunctionMetric, VolumeEngine};

pub fn run_analysis(dir: &std::path::Path) -> anyhow::Result<Vec<FunctionMetric>> {
    let engine = VolumeEngine::new();
    let mut all_metrics = Vec::new();

    // We'll use the `ignore` crate (part of cargo-sig deps? actually walkdir is usually standard, but let's just use walkdir or ignore)
    // Actually we didn't explicitly add `ignore` or `walkdir` to Cargo.toml. Let's add `ignore` later if needed, or just standard `std::fs::read_dir`.
    // We can just use `std::fs` for a basic recursive walk, but let's assume we want to analyze the whole project.

    // For now, let's do a simple recursive traverse using std::fs to keep dependencies low, or use walkdir if it's already there.
    fn walk_dir(
        dir: &std::path::Path,
        metrics: &mut Vec<FunctionMetric>,
        engine: &VolumeEngine,
    ) -> anyhow::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                // Skip target and hidden dirs
                if path.is_dir() {
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if file_name.starts_with('.') || file_name == "target" {
                        continue;
                    }
                    walk_dir(&path, metrics, engine)?;
                } else if path.extension().is_some_and(|ext| ext == "rs")
                    && let Ok(source) = std::fs::read_to_string(&path)
                    && let Ok(file_metrics) = engine.analyze_file(&path, &source)
                {
                    metrics.extend(file_metrics);
                }
            }
        }
        Ok(())
    }

    walk_dir(dir, &mut all_metrics, &engine)?;

    Ok(all_metrics)
}
