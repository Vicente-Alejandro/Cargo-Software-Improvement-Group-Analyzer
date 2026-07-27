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
    let CargoCli::Sig(args) = CargoCli::parse();

    println!("Cargo SIG Analyzer - Running check...");

    // Future execution logic:
    // 1. AST Analysis (Volume, Complexity)
    // 2. Duplication (jscpd-rs)
    // 3. Churn vs Coverage
    // 4. Scoring Engine

    if args.fail_below > 0 {
        println!(
            "Quality gate active: must score at least {} stars",
            args.fail_below
        );
    }

    Ok(())
}
