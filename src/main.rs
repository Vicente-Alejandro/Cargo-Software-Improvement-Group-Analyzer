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
    let res_score = run_app(&args)?;
    report::enforce_gate(res_score, args.fail_below);
    Ok(())
}

fn run_app(args: &SigArgs) -> anyhow::Result<u8> {
    let dir = std::env::current_dir()?;
    let metrics = analysis::run_analysis(&dir)?;
    let mut f: Vec<_> = metrics.iter().map(|m| m.file_path.clone()).collect();
    f.sort();
    f.dedup();
    let dup_pct = duplication::calculate_duplication(&f);
    let graph = coupling::CouplingGraph::build(&dir, &f);
    let churns = churn::get_frequencies(&dir).unwrap_or_default();
    let cov = coverage::load_or_generate_lcov(&dir);
    let is_balanced = report::is_balanced(&metrics);
    let ctx = scoring::EvalCtx {
        metrics: &metrics,
        dup: dup_pct,
        bal: is_balanced,
        graph: &graph,
        cov: &cov,
        churns: &churns,
    };
    let score = scoring::evaluate(&ctx);
    let res = report::AnalysisResult {
        metrics: &metrics,
        churns: &churns,
        cov: &cov,
        score: &score,
        dup_pct,
        graph: &graph,
    };
    if args.format == "json" {
        report::print_json(&res);
    } else {
        report::print_all(&res);
    }
    Ok(score.stars)
}

#[rustfmt::skip]
fn parse_args() -> SigArgs {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() && args[0] == "sig" { args.remove(0); }
    SigArgs::parse(args.into_iter())
}
