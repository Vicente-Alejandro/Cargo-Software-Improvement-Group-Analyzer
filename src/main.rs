pub mod analysis;
pub mod churn;
pub mod cli;
pub mod coupling;
pub mod coverage;
pub mod duplication;
mod report;
mod scoring;

use cli::SigArgs;
use owo_colors::OwoColorize;

#[rustfmt::skip]
fn main() -> anyhow::Result<()> {
    let args = parse_args();
    if args.format != "json" {
        println!("{} - Running check...", "Cargo SIG".bold().cyan());
    }
    let dir = std::env::current_dir()?;
    let res_score = run_app(&args, &dir)?;
    report::enforce_gate(res_score, args.fail_below);
    Ok(())
}

#[rustfmt::skip]
fn run_app(args: &SigArgs, dir: &std::path::Path) -> anyhow::Result<u8> {
    let metrics = analysis::run_analysis(dir)?;
    let mut f: Vec<_> = metrics.iter().map(|m| m.file_path.clone()).collect();
    f.sort(); f.dedup();
    let dup = duplication::calculate_duplication(&f);
    let graph = coupling::CouplingGraph::build(dir, &f);
    let churns = churn::get_frequencies(dir).unwrap_or_default();
    let cov = coverage::load_or_generate_lcov(dir, !args.auto_cov);
    let ctx = scoring::EvalCtx { metrics: &metrics, dup, bal: report::is_balanced(&metrics), graph: &graph, cov: &cov, churns: &churns };
    let score = scoring::evaluate(&ctx);
    let res = report::AnalysisResult { metrics: &metrics, churns: &churns, cov: &cov, score: &score, dup_pct: dup, graph: &graph };
    if args.format == "json" { report::print_json(&res); } else { report::print_all(&res); }
    Ok(score.stars)
}

#[rustfmt::skip]
fn parse_args() -> SigArgs {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() && args[0] == "sig" { args.remove(0); }
    SigArgs::parse(args.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_run_app() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let args = SigArgs {
            fail_below: 0,
            format: "terminal".to_string(),
            auto_cov: false,
        };
        let stars = run_app(&args, dir.path()).unwrap();
        assert_eq!(stars, 5); // 5 stars because it's unbalanced (1 file = 100%)
    }

    #[test]
    fn test_parse_args() {
        let _ = parse_args();
    }

    #[test]
    fn test_main() {
        // Just run main. It will parse test runner args, run analysis on current dir, and return Ok
        let res = main();
        assert!(res.is_ok());
    }
}
