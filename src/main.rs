mod analysis;
mod churn;
mod cli;
mod coverage;
mod duplication;
mod report;
mod scoring;

use clap::Parser;
use cli::CargoCli;

fn main() -> anyhow::Result<()> {
    // Cargo passes the subcommand name "sig" as the first argument when invoked as `cargo sig`.
    // If invoked directly as `cargo-sig`, that argument is missing.
    // We intercept the arguments to ensure both invocations work seamlessly.
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() == 1 || (args.len() > 1 && args[1] != "sig") {
        args.insert(1, "sig".to_string());
    }

    let CargoCli::Sig(args) = CargoCli::parse_from(args);

    println!("Cargo SIG Analyzer - Running check...");

    // 1. AST Analysis (Volume, Complexity)
    let current_dir = std::env::current_dir()?;
    let metrics = analysis::run_analysis(&current_dir)?;

    let mut high_risk_loc = 0;
    let mut high_risk_params = 0;

    for metric in &metrics {
        if metric.lines_of_code > 15 {
            println!(
                "  [WARN] Function '{}' in {:?} exceeds 15 lines ({} lines)",
                metric.function_name,
                metric.file_path.file_name().unwrap_or_default(),
                metric.lines_of_code
            );
            high_risk_loc += 1;
        }
        if metric.parameter_count > 4 {
            println!(
                "  [WARN] Function '{}' in {:?} has too many parameters ({})",
                metric.function_name,
                metric.file_path.file_name().unwrap_or_default(),
                metric.parameter_count
            );
            high_risk_params += 1;
        }
    }

    println!("\nAnalysis Summary:");
    println!("Total Functions Analyzed: {}", metrics.len());
    println!("Functions > 15 lines: {}", high_risk_loc);
    println!("Functions > 4 parameters: {}", high_risk_params);

    if args.fail_below > 0 {
        println!(
            "Quality gate active: must score at least {} stars",
            args.fail_below
        );
    }

    Ok(())
}
