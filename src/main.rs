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
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let args = parse_args();
    println!("{} - Running check...", "Cargo SIG".bold().cyan());

    let dir = std::env::current_dir()?;
    let metrics = analysis::run_analysis(&dir)?;
    let churns = churn::get_frequencies(&dir).unwrap_or_default();
    let score = scoring::evaluate(&metrics);

    print_summary(&metrics);
    print_hotspots(&metrics, &churns);
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
    println!("\n{}", "Summary:".bold());
    println!("Total Functions: {}", metrics.len());
    println!("Volume > 15 lines: {}", color(loc));
    println!("Interface > 4 params: {}", color(prm));
    println!("Complexity > 5 branches: {}", color(cmp));
}

fn color(val: usize) -> String {
    if val > 0 {
        val.red().to_string()
    } else {
        val.green().to_string()
    }
}

fn print_hotspots(metrics: &[FunctionMetric], churns: &HashMap<PathBuf, usize>) {
    let mut h = match_hotspots(&compute_file_risk(metrics), churns);
    if h.is_empty() {
        println!("\n{} No Hotspots.", "✅ [OK]".green().bold());
        return;
    }
    h.sort_by_key(|i| std::cmp::Reverse(i.1 * i.2));
    println!("\n{}", "⚠️ Hotspots:".bold().yellow());
    for (i, (p, r, c)) in h.iter().take(5).enumerate() {
        let n = p.file_name().unwrap_or_default().to_string_lossy();
        println!("  {}. {} ({} commits, {} risk LOC)", i + 1, n.red(), c, r);
    }
}

fn compute_file_risk(metrics: &[FunctionMetric]) -> HashMap<PathBuf, usize> {
    let mut f_risk = HashMap::new();
    for m in metrics {
        if m.lines_of_code > 30 || m.cyclomatic_complexity > 10 {
            let key = m
                .file_path
                .canonicalize()
                .unwrap_or_else(|_| m.file_path.clone());
            *f_risk.entry(key).or_insert(0) += m.lines_of_code;
        }
    }
    f_risk
}

fn match_hotspots(
    f_risk: &HashMap<PathBuf, usize>,
    ch: &HashMap<PathBuf, usize>,
) -> Vec<(PathBuf, usize, usize)> {
    let mut hotspots = Vec::new();
    for (path, &risk_loc) in f_risk {
        let commits = ch.get(path).copied().unwrap_or(0);
        if commits > 1 {
            hotspots.push((path.clone(), risk_loc, commits));
        }
    }
    hotspots
}

fn print_profile(score: &Score) {
    println!("\n{}", "Risk Profile:".bold());
    println!("Moderate Risk: {:.1}%", score.pct_moderate.yellow());
    println!("High Risk: {:.1}%", score.pct_high.bright_red());
    println!(
        "Very High Risk: {:.1}%",
        score.pct_very_high.red().on_black()
    );
    println!("\n─────────────────────────────────────");
    let c = format_stars(score.stars);
    println!("Maintainability Rating: {} ({} / 7)", c, score.stars);
}

fn format_stars(stars: u8) -> String {
    let s = "★".repeat(stars as usize) + &"☆".repeat((7 - stars) as usize);
    match stars {
        7 | 6 => s.green().bold().to_string(),
        5 | 4 => s.yellow().bold().to_string(),
        _ => s.red().bold().to_string(),
    }
}

fn enforce_gate(stars: u8, gate: u8) {
    if gate == 0 {
        return;
    }
    if stars < gate {
        println!(
            "\n{} Rating {} below gate {}.",
            "❌ [ERROR]".red().bold(),
            stars,
            gate
        );
        std::process::exit(1);
    }
    println!(
        "\n{} Passed gate ({} >= {}).",
        "✅ [OK]".green().bold(),
        stars,
        gate
    );
}
