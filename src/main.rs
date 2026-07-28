mod analysis;
mod churn;
mod cli;
mod coverage;
mod duplication;
mod report;
mod scoring;

use analysis::FunctionMetric;
use cli::SigArgs;
use owo_colors::OwoColorize;
use scoring::Score;

fn main() -> anyhow::Result<()> {
    let args = parse_args();
    println!("{} - Running check...", "Cargo SIG Analyzer".bold().cyan());

    let metrics = analysis::run_analysis(&std::env::current_dir()?)?;
    let score = scoring::evaluate(&metrics);

    print_summary(&metrics);
    print_profile(&score);
    enforce_gate(score.stars, args.fail_below);
    Ok(())
}

fn parse_args() -> SigArgs {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() && args[0] == "sig" {
        args.remove(0);
    }
    SigArgs::parse(args.into_iter())
}

fn print_summary(metrics: &[FunctionMetric]) {
    let (mut loc, mut prm, mut cmp) = (0, 0, 0);
    for m in metrics {
        if m.lines_of_code > 15 {
            loc += 1;
        }
        if m.parameter_count > 4 {
            prm += 1;
        }
        if m.cyclomatic_complexity > 5 {
            cmp += 1;
        }
    }

    println!("\n{}", "Analysis Summary:".bold());
    println!("Total Functions Analyzed: {}", metrics.len());
    println!(
        "Functions > 15 lines (Volume): {}",
        if loc > 0 {
            loc.red().to_string()
        } else {
            loc.green().to_string()
        }
    );
    println!(
        "Functions > 4 parameters (Interfaces): {}",
        if prm > 0 {
            prm.red().to_string()
        } else {
            prm.green().to_string()
        }
    );
    println!(
        "Functions > 5 branches (Complexity): {}",
        if cmp > 0 {
            cmp.red().to_string()
        } else {
            cmp.green().to_string()
        }
    );
}

fn print_profile(score: &Score) {
    println!("\n{}", "Risk Profile:".bold());
    println!("Moderate Risk Code: {:.1}%", score.pct_moderate.yellow());
    println!("High Risk Code: {:.1}%", score.pct_high.bright_red());
    println!(
        "Very High Risk Code: {:.1}%",
        score.pct_very_high.red().on_black()
    );

    println!("\n─────────────────────────────────────");
    let stars = "★".repeat(score.stars as usize) + &"☆".repeat((7 - score.stars) as usize);
    let c = match score.stars {
        7 | 6 => stars.green().bold().to_string(),
        5 | 4 => stars.yellow().bold().to_string(),
        _ => stars.red().bold().to_string(),
    };
    println!("Maintainability Rating: {} ({} / 7)", c, score.stars);
}

fn enforce_gate(stars: u8, gate: u8) {
    if gate > 0 {
        if stars < gate {
            println!(
                "\n{} Rating {} is below the required gate of {} stars.",
                "❌ [ERROR]".red().bold(),
                stars,
                gate
            );
            std::process::exit(1);
        } else {
            println!(
                "\n{} Passed the quality gate ({} >= {}).",
                "✅ [OK]".green().bold(),
                stars,
                gate
            );
        }
    }
}
