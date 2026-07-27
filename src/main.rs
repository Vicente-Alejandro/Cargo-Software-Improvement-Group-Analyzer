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
