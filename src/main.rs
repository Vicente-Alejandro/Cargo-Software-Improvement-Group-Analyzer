mod cli;
mod analysis;
mod duplication;
mod churn;
mod coverage;
mod scoring;
mod report;

use cli::SigArgs;
use owo_colors::OwoColorize;

fn main() -> anyhow::Result<()> {
    let mut args_iter = std::env::args();
    let _ = args_iter.next();
    
    let mut args = args_iter.collect::<Vec<String>>();
    if !args.is_empty() && args[0] == "sig" {
        args.remove(0);
    }

    let parsed_args = SigArgs::parse(args.into_iter());

    println!("{} - Running check...", "Cargo SIG Analyzer".bold().cyan());
    
    let current_dir = std::env::current_dir()?;
    let metrics = analysis::run_analysis(&current_dir)?;
    
    let mut high_risk_loc = 0;
    let mut high_risk_params = 0;
    let mut high_risk_complexity = 0;

    for metric in &metrics {
        if metric.lines_of_code > 15 {
            high_risk_loc += 1;
        }
        if metric.parameter_count > 4 {
            high_risk_params += 1;
        }
        if metric.cyclomatic_complexity > 5 {
            high_risk_complexity += 1;
        }
    }

    let score = scoring::evaluate(&metrics);

    println!("\n{}", "Analysis Summary:".bold());
    println!("Total Functions Analyzed: {}", metrics.len());
    
    let loc_str = if high_risk_loc > 0 { high_risk_loc.red().to_string() } else { high_risk_loc.green().to_string() };
    let param_str = if high_risk_params > 0 { high_risk_params.red().to_string() } else { high_risk_params.green().to_string() };
    let comp_str = if high_risk_complexity > 0 { high_risk_complexity.red().to_string() } else { high_risk_complexity.green().to_string() };
    
    println!("Functions > 15 lines (Volume): {}", loc_str);
    println!("Functions > 4 parameters (Interfaces): {}", param_str);
    println!("Functions > 5 branches (Complexity): {}", comp_str);

    println!("\n{}", "Risk Profile:".bold());
    println!("Moderate Risk Code: {:.1}%", score.pct_moderate.yellow());
    println!("High Risk Code: {:.1}%", score.pct_high.bright_red());
    println!("Very High Risk Code: {:.1}%", score.pct_very_high.red().on_black());

    println!("\n─────────────────────────────────────");
    
    let stars_visual = "★".repeat(score.stars as usize) + &"☆".repeat((7 - score.stars) as usize);
    let stars_colored = match score.stars {
        7 | 6 => stars_visual.green().bold().to_string(),
        5 | 4 => stars_visual.yellow().bold().to_string(),
        _ => stars_visual.red().bold().to_string(),
    };
    
    println!("Maintainability Rating: {} ({} / 7)", stars_colored, score.stars);

    if parsed_args.fail_below > 0 {
        if score.stars < parsed_args.fail_below {
            println!("\n{} Rating {} is below the required gate of {} stars.", "❌ [ERROR]".red().bold(), score.stars, parsed_args.fail_below);
            std::process::exit(1);
        } else {
            println!("\n{} Passed the quality gate ({} >= {}).", "✅ [OK]".green().bold(), score.stars, parsed_args.fail_below);
        }
    }

    Ok(())
}
