#![warn(clippy::pedantic)]
#![deny(clippy::all)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::unused_self)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::struct_excessive_bools)]

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
    let dup_res = duplication::calculate_duplication(&f);
    let graph = coupling::CouplingGraph::build(dir, &f);
    let churns = churn::get_frequencies(dir).unwrap_or_default();
    let cov = coverage::load_or_generate_lcov(dir, !args.auto_cov);
    let ctx = scoring::EvalCtx { metrics: &metrics, dup: dup_res.percentage, bal: report::is_balanced(&metrics), graph: &graph, cov: &cov, churns: &churns };
    let score = scoring::evaluate(&ctx);
    let res = report::AnalysisResult { metrics: &metrics, churns: &churns, cov: &cov, score: &score, dup_res: &dup_res, graph: &graph };
    if args.format == "json" { report::print_json(&res); } else { report::print_all(&res); }
    if args.report || args.html || args.pdf { emit_reports(args, &res, dir)?; }
    Ok(score.stars)
}

fn emit_reports(
    args: &SigArgs,
    res: &report::AnalysisResult,
    dir: &std::path::Path,
) -> anyhow::Result<()> {
    report::ensure_gitignored(dir)?;
    if args.report {
        emit_md(res, dir)?;
    }
    if args.html {
        emit_html(res, dir)?;
    }
    if args.pdf {
        emit_pdf(res, dir)?;
    }
    Ok(())
}

fn emit_md(res: &report::AnalysisResult, dir: &std::path::Path) -> anyhow::Result<()> {
    let path = report::generate_markdown_report(res, dir)?;
    let rel = path.strip_prefix(dir).unwrap_or(&path);
    println!(
        "\n{} {}",
        "Full Markdown report generated:".green().bold(),
        rel.display()
    );
    Ok(())
}

fn emit_html(res: &report::AnalysisResult, dir: &std::path::Path) -> anyhow::Result<()> {
    let path = report::generate_html_report(res, dir)?;
    let rel = path.strip_prefix(dir).unwrap_or(&path);
    println!(
        "\n{} {}",
        "Full HTML report generated:".green().bold(),
        rel.display()
    );
    Ok(())
}

fn emit_pdf(res: &report::AnalysisResult, dir: &std::path::Path) -> anyhow::Result<()> {
    let html_path = report::generate_html_report(res, dir)?;
    match report::generate_pdf_report(&html_path, dir) {
        Ok(pdf_path) => {
            let rel = pdf_path.strip_prefix(dir).unwrap_or(&pdf_path);
            println!(
                "\n{} {}",
                "Full PDF report generated:".green().bold(),
                rel.display()
            );
        }
        Err(e) => {
            let html_rel = html_path.strip_prefix(dir).unwrap_or(&html_path);
            println!(
                "\n{} Unable to generate PDF automatically: {e}",
                "[WARN]".yellow().bold()
            );
            println!(
                "Tip: Open '{}' in any web browser and click 'Export PDF / Print'.",
                html_rel.display()
            );
        }
    }
    Ok(())
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
            report: true,
            html: true,
            pdf: true,
        };
        let stars = run_app(&args, dir.path()).unwrap();
        assert_eq!(stars, 5); // 5 stars because it's unbalanced (1 file = 100%)
        assert!(dir.path().join("tools/cargo-sig/SIG_REPORT.html").exists());
        assert!(dir.path().join("tools/cargo-sig/SIG_REPORT.md").exists());
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
