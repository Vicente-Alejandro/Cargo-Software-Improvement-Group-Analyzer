use crate::analysis::FunctionMetric;
use crate::coverage::Coverage;
use crate::scoring::Score;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct AnalysisResult<'a> {
    pub metrics: &'a [FunctionMetric],
    pub churns: &'a HashMap<PathBuf, usize>,
    pub cov: &'a Option<HashMap<PathBuf, Coverage>>,
    pub score: &'a Score,
    pub dup_pct: f32,
}

pub fn print_all(res: &AnalysisResult) {
    print_summary(res.metrics, res.dup_pct);
    print_balance(res.metrics);
    print_hotspots(res.metrics, res.churns, res.cov);
    print_profile(res.score);
}

fn print_summary(metrics: &[FunctionMetric], dup: f32) {
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
    print_stats(metrics.len(), loc, prm, cmp, dup);
}

fn print_stats(tot: usize, loc: usize, prm: usize, cmp: usize, dup: f32) {
    println!("\n{}", "Summary:".bold());
    println!("Total Functions: {}", tot);
    println!("Volume > 15 lines: {}", color(loc));
    println!("Interface > 4 params: {}", color(prm));
    println!("Complexity > 5 branches: {}", color(cmp));

    let d_col = if dup > 5.0 {
        format!("{:.1}%", dup).red().to_string()
    } else {
        format!("{:.1}%", dup).green().to_string()
    };
    println!("Code Duplication: {}", d_col);
}

fn color(val: usize) -> String {
    if val > 0 {
        val.red().to_string()
    } else {
        val.green().to_string()
    }
}

fn print_balance(metrics: &[FunctionMetric]) {
    println!("\n{}", "Component Balance:".bold());
    let mut d = HashMap::new();
    let mut tot = 0;
    for m in metrics {
        let p = m.file_path.parent().unwrap_or(Path::new("")).to_path_buf();
        *d.entry(p).or_insert(0) += m.lines_of_code;
        tot += m.lines_of_code;
    }
    check_balance(d, tot);
}

fn check_balance(dirs: HashMap<PathBuf, usize>, tot: usize) {
    let mut ok = true;
    for (d, loc) in dirs {
        let pct = (loc as f32 / tot as f32) * 100.0;
        if pct > 50.0 {
            print_imbalance(&d, pct);
            ok = false;
        }
    }
    if ok {
        println!("  {} All components are balanced.", "✅".green());
    }
}

fn print_imbalance(d: &Path, pct: f32) {
    let n = d.file_name().unwrap_or_default().to_string_lossy();
    println!(
        "  {} {} contains {:.1}% of total code.",
        "⚠️".yellow(),
        n.bold(),
        pct
    );
}

#[rustfmt::skip]
fn print_hotspots(m: &[FunctionMetric], ch: &HashMap<PathBuf, usize>, cov: &Option<HashMap<PathBuf, Coverage>>) {
    let mut h = match_hotspots(&compute_file_risk(m), ch, cov);
    if h.is_empty() {
        println!("\n{} No Hotspots.", "✅ [OK]".green().bold());
        return;
    }
    h.sort_by_key(|b| std::cmp::Reverse(b.1 * b.2));
    println!("\n{}", "⚠️ Hotspots (Risk + Churn):".bold().yellow());
    for (i, (p, r, c, cv)) in h.iter().take(5).enumerate() {
        print_hotspot_item(i, p, *r, *c, *cv);
    }
}

fn print_hotspot_item(i: usize, p: &Path, r: usize, c: usize, cv: Option<f32>) {
    let n = p.file_name().unwrap_or_default().to_string_lossy();
    let c_str = cv
        .map(|v| format!("{:.0}% cov", v))
        .unwrap_or_else(|| "no cov data".to_string());
    println!(
        "  {}. {} ({} com, {} r_loc, {})",
        i + 1,
        n.red(),
        c,
        r,
        c_str
    );
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
    cov: &Option<HashMap<PathBuf, Coverage>>,
) -> Vec<(PathBuf, usize, usize, Option<f32>)> {
    let mut hotspots = Vec::new();
    for (path, &risk_loc) in f_risk {
        let commits = ch.get(path).copied().unwrap_or(0);
        let cv = cov.as_ref().and_then(|c| c.get(path)).map(|c| c.percent());
        if commits > 1 {
            hotspots.push((path.clone(), risk_loc, commits, cv));
        }
    }
    hotspots
}

fn print_profile(score: &Score) {
    println!("\n{}", "Risk Profile:".bold());
    println!("Moderate Risk: {:.1}%", score.pct_moderate.yellow());
    println!("High Risk: {:.1}%", score.pct_high.bright_red());
    println!("Very High Risk: {:.1}%", score.pct_very_high.red().bold());
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

pub fn enforce_gate(stars: u8, gate: u8) {
    if gate == 0 {
        return;
    }
    if stars < gate {
        gate_fail(stars, gate);
    }
    println!("\n{} Passed gate.", "✅ [OK]".green().bold());
}

fn gate_fail(stars: u8, gate: u8) {
    println!(
        "\n{} Rating {} below gate {}.",
        "❌ [ERROR]".red().bold(),
        stars,
        gate
    );
    std::process::exit(1);
}
