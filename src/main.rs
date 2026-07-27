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
    // Cargo passes the subcommand name "sig" as the first argument when invoked as `cargo sig`.
    // If invoked directly as `cargo-sig`, that argument is missing.
    let mut args_iter = std::env::args();
    // skip the binary name (cargo-sig)
    let _ = args_iter.next();

    // Check if the next arg is "sig"
    let mut args = args_iter.collect::<Vec<String>>();
    if !args.is_empty() && args[0] == "sig" {
        args.remove(0); // pop "sig"
    }

    let parsed_args = SigArgs::parse(args.into_iter());

    println!("{} - Running check...", "Cargo SIG Analyzer".bold().cyan());

    // 1. AST Analysis (Volume, Complexity)
    let current_dir = std::env::current_dir()?;
    let metrics = analysis::run_analysis(&current_dir)?;

    let mut high_risk_loc = 0;
    let mut high_risk_params = 0;

    for metric in &metrics {
        if metric.lines_of_code > 15 {
            println!(
                "  {} Function '{}' in {:?} exceeds 15 lines ({} lines)",
                "[WARN]".yellow().bold(),
                metric.function_name.cyan(),
                metric.file_path.file_name().unwrap_or_default(),
                metric.lines_of_code.red()
            );
            high_risk_loc += 1;
        }
        if metric.parameter_count > 4 {
            println!(
                "  {} Function '{}' in {:?} has too many parameters ({})",
                "[WARN]".yellow().bold(),
                metric.function_name.cyan(),
                metric.file_path.file_name().unwrap_or_default(),
                metric.parameter_count.red()
            );
            high_risk_params += 1;
        }
    }

    println!("\n{}", "Analysis Summary:".bold());
    println!("Total Functions Analyzed: {}", metrics.len());

    let loc_str = if high_risk_loc > 0 {
        high_risk_loc.red().to_string()
    } else {
        high_risk_loc.green().to_string()
    };
    let param_str = if high_risk_params > 0 {
        high_risk_params.red().to_string()
    } else {
        high_risk_params.green().to_string()
    };

    println!("Functions > 15 lines: {}", loc_str);
    println!("Functions > 4 parameters: {}", param_str);

    if parsed_args.fail_below > 0 {
        println!(
            "Quality gate active: must score at least {} stars",
            parsed_args.fail_below.cyan().bold()
        );
    }

    Ok(())
}
