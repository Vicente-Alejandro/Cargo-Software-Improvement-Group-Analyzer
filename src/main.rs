mod analysis;
mod churn;
mod cli;
mod coverage;
mod duplication;
mod report;
mod scoring;

use cli::SigArgs;
use owo_colors::OwoColorize;

fn main() -> anyhow::Result<()> {
    let args = parse_args();
    println!("{} - Running check...", "Cargo SIG".bold().cyan());

    let dir = std::env::current_dir()?;
    let metrics = analysis::run_analysis(&dir)?;
    let dup_pct = get_dup(&metrics);

    let churns = churn::get_frequencies(&dir).unwrap_or_default();

    let cov = coverage::read_lcov(&dir);
    let is_balanced = report::is_balanced(&metrics);
    let score = scoring::evaluate(&metrics, dup_pct, is_balanced);

    let res = report::AnalysisResult {
        metrics: &metrics,
        churns: &churns,
        cov: &cov,
        score: &score,
        dup_pct,
    };
    report::print_all(&res);
    report::enforce_gate(score.stars, args.fail_below);
    Ok(())
}

fn parse_args() -> SigArgs {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() && args[0] == "sig" {
        args.remove(0);
    }
    SigArgs::parse(args.into_iter())
}

fn get_dup(metrics: &[analysis::FunctionMetric]) -> f32 {
    let mut f: Vec<_> = metrics.iter().map(|m| m.file_path.clone()).collect();
    f.sort();
    f.dedup();
    duplication::calculate_duplication(&f)
}
