use crate::analysis::FunctionMetric;
use crate::coverage::Coverage;
use crate::scoring::Score;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct AnalysisResult<'a> {
    pub metrics: &'a [FunctionMetric],
    pub churns: &'a HashMap<PathBuf, usize>,
    pub cov: &'a Option<HashMap<PathBuf, Coverage>>,
    pub score: &'a Score,
    pub dup_pct: f32,
    pub graph: &'a crate::coupling::CouplingGraph,
}

#[rustfmt::skip]
pub fn print_all(res: &AnalysisResult) {
    print_summary(res);
    print_balance(res.metrics);
    print_coupling(res);
    print_hotspots(res);
    print_profile(res.score);
}

#[rustfmt::skip]
fn print_summary(res: &AnalysisResult) {
    println!("\n{}", "Summary:".bold());
    println!("Total Functions: {}", res.metrics.len());
    let (mut v, mut i, mut c) = (0, 0, 0);
    for m in res.metrics {
        if m.lines_of_code > 15 { v += 1; }
        if m.parameter_count > 4 { i += 1; }
        if m.cyclomatic_complexity > 5 { c += 1; }
    }
    println!("Volume > 15 lines: {}\nInterface > 4 params: {}\nComplexity > 5 branches: {}\nCode Duplication: {:.1}%", v, i, c, res.dup_pct);
}

#[rustfmt::skip]
pub fn is_balanced(metrics: &[FunctionMetric]) -> bool {
    let (mut comp_loc, mut total_loc) = (HashMap::new(), 0);
    for m in metrics {
        if let Some(p) = m.file_path.parent() {
            let comp = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            *comp_loc.entry(comp).or_insert(0) += m.lines_of_code;
            total_loc += m.lines_of_code;
        }
    }
    if total_loc == 0 { return true; }
    !comp_loc.values().any(|&loc| (loc as f32 / total_loc as f32) > 0.5)
}

#[rustfmt::skip]
fn print_balance(metrics: &[FunctionMetric]) {
    println!("\n{}", "Component Balance:".bold());
    if is_balanced(metrics) { println!("  All components are balanced. {}", "✅".green()); }
    else { println!("  One component exceeds 50% of the codebase. {}", "⚠️".yellow()); }
}

#[rustfmt::skip]
fn print_coupling(res: &AnalysisResult) {
    println!("\n{}", "Module Coupling:".bold());
    if res.graph.ignored_externals > 0 { println!("  {} external dependencies ignored. {}", res.graph.ignored_externals, "ℹ️".blue()); }
    let h_fan = res.metrics.iter().filter(|m| res.graph.fan_out(&m.file_path) > 5).count();
    if h_fan > 0 { println!("  Fan-Out > 5: {} modules {}", h_fan, "⚠️".yellow()); }
    else { println!("  Fan-Out is healthy across all modules. {}", "✅".green()); }
    let cycles = res.graph.detect_cycles();
    if !cycles.is_empty() {
        println!("  Circular Dependencies: {} DETECTED! {}", cycles.len(), "🚨".red());
        let path: Vec<_> = cycles[0].iter().map(|p| p.file_name().unwrap_or_default().to_string_lossy()).collect();
        println!("     Example: {} -> {}", path.join(" -> "), path[0]);
    } else { println!("  No Circular Dependencies. {}", "✅".green()); }
}

#[rustfmt::skip]
fn compute_file_risk(metrics: &[FunctionMetric]) -> HashMap<PathBuf, usize> {
    let mut f_risk = HashMap::new();
    let g = crate::coupling::CouplingGraph::default();
    for m in metrics {
        let r = crate::scoring::categorize_risk(m, &g) as usize;
        *f_risk.entry(m.file_path.clone()).or_insert(0) += r;
    }
    f_risk
}

#[rustfmt::skip]
pub fn print_hotspots(res: &AnalysisResult) {
    let fr = compute_file_risk(res.metrics);
    let mut s: Vec<_> = fr.keys().collect();
    s.sort_by_key(|k| -( (fr.get(*k).unwrap_or(&0) * res.churns.get(*k).unwrap_or(&0)) as isize ));
    s.retain(|k| *fr.get(*k).unwrap_or(&0) > 0 && *res.churns.get(*k).unwrap_or(&0) > 0);
    if s.is_empty() { return; }
    println!("\n{} {}", "Hotspots (Risk + Churn):".bold().yellow(), "⚠️".yellow());
    for (i, p) in s.iter().take(5).enumerate() {
        print_hotspot_item(i, p, &fr, res);
    }
}

#[rustfmt::skip]
fn print_hotspot_item(i: usize, p: &PathBuf, fr: &HashMap<PathBuf, usize>, res: &AnalysisResult) {
    let (cwd, r, c) = (std::env::current_dir().unwrap_or_default(), fr.get(p).unwrap_or(&0), res.churns.get(p).unwrap_or(&0));
    let name = p.strip_prefix(&cwd).unwrap_or(p).display();
    let cv_str = if let Some(cv) = res.cov {
        if let Some(c) = cv.get(p) { format!("Coverage: {:.1}%", c.percent()) } else { "Coverage: N/A".to_string() }
    } else { "Coverage: N/A".to_string() };
    println!("  {}. {} (High Risk: {} points, High Churn: {} commits, {})", i + 1, name, r, c, cv_str);
}

#[rustfmt::skip]
fn print_profile(s: &Score) {
    println!("\n{}", "Risk Profile:".bold());
    println!("Moderate Risk: {:.1}%\nHigh Risk: {:.1}%\nVery High Risk: {:.1}%", s.pct_moderate, s.pct_high, s.pct_very_high);
    println!("\n─────────────────────────────────────");
    let stars_str = format!("{} ({:^1} / 7)", star_string(s.stars), s.stars);
    let colored_stars = match s.stars {
        6..=7 => stars_str.green().to_string(),
        4..=5 => stars_str.yellow().to_string(),
        _ => stars_str.red().to_string(),
    };
    println!("{}: {}", "Maintainability Rating".bold(), colored_stars);
}

fn star_string(stars: u8) -> String {
    format!(
        "{}{}",
        "★".repeat(stars as usize),
        "☆".repeat((7 - stars) as usize)
    )
}

pub fn enforce_gate(stars: u8, fail_below: u8) {
    if stars < fail_below {
        eprintln!(
            "\n{} Rating {} is below the required {}",
            "ERROR:".red().bold(),
            stars,
            fail_below
        );
        std::process::exit(1);
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[rustfmt::skip]
fn build_summary_json(res: &AnalysisResult) -> String {
    let (mut v, mut i, mut c) = (0, 0, 0);
    for m in res.metrics {
        if m.lines_of_code > 15 { v += 1; }
        if m.parameter_count > 4 { i += 1; }
        if m.cyclomatic_complexity > 5 { c += 1; }
    }
    format!("{{\"total_functions\":{},\"volume_violations\":{},\"interface_violations\":{},\"complexity_violations\":{},\"duplication_pct\":{:.1}}}", res.metrics.len(), v, i, c, res.dup_pct)
}

#[rustfmt::skip]
fn build_hotspots_json(res: &AnalysisResult) -> String {
    let fr = compute_file_risk(res.metrics);
    let mut hs: Vec<_> = fr.keys().collect();
    hs.sort_by_key(|k| -( (fr.get(*k).unwrap_or(&0) * res.churns.get(*k).unwrap_or(&0)) as isize ));
    hs.retain(|k| *fr.get(*k).unwrap_or(&0) > 0 && *res.churns.get(*k).unwrap_or(&0) > 0);
    let cwd = std::env::current_dir().unwrap_or_default();
    let js: Vec<String> = hs.iter().take(5).map(|p| {
        let n = escape_json(&p.strip_prefix(&cwd).unwrap_or(p).display().to_string());
        let cv = res.cov.as_ref().and_then(|c| c.get(*p)).map(|c| c.percent().to_string()).unwrap_or("null".to_string());
        format!("{{\"file\":\"{}\",\"risk_points\":{},\"churn_commits\":{},\"coverage_pct\":{}}}", n, fr.get(*p).unwrap_or(&0), res.churns.get(*p).unwrap_or(&0), cv)
    }).collect();
    js.join(",")
}

#[rustfmt::skip]
pub fn print_json(res: &AnalysisResult) {
    let bal = is_balanced(res.metrics);
    let cycles = res.graph.detect_cycles().len();
    let h_fan = res.metrics.iter().filter(|m| res.graph.fan_out(&m.file_path) > 5).count();
    println!("{{");
    println!("  \"summary\": {},", build_summary_json(res));
    println!("  \"component_balance\": {{\"is_balanced\":{}}},", bal);
    println!("  \"module_coupling\": {{\"ignored_externals\":{},\"fan_out_violations\":{},\"circular_dependencies\":{}}},", res.graph.ignored_externals, h_fan, cycles);
    println!("  \"hotspots\": [{}],", build_hotspots_json(res));
    println!("  \"risk_profile\": {{\"moderate_pct\":{:.1},\"high_pct\":{:.1},\"very_high_pct\":{:.1}}},", res.score.pct_moderate, res.score.pct_high, res.score.pct_very_high);
    println!("  \"rating\": {{\"stars\":{},\"max_stars\":7}}", res.score.stars);
    println!("}}");
}
