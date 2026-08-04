use crate::analysis::FunctionMetric;
use crate::coverage::Coverage;
use crate::scoring::Score;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod gitignore;
pub mod html;
pub mod markdown;
pub mod pdf;

pub use gitignore::ensure_gitignored;
pub use html::generate_html_report;
pub use markdown::generate_markdown_report;
pub use pdf::generate_pdf_report;

pub struct AnalysisResult<'a> {
    pub metrics: &'a [FunctionMetric],
    pub churns: &'a HashMap<PathBuf, usize>,
    pub cov: &'a Option<HashMap<PathBuf, Coverage>>,
    pub score: &'a Score,
    pub dup_res: &'a crate::duplication::DuplicationResult,
    pub graph: &'a crate::coupling::CouplingGraph,
    pub delta: Option<&'a crate::history::HistoryDelta>,
    pub history: &'a [crate::history::HistoryRecord],
}

#[rustfmt::skip]
pub fn print_all(res: &AnalysisResult) {
    print_summary(res);
    print_balance(res.metrics);
    print_coupling(res);
    print_hotspots(res);
    print_profile(res);
}

#[rustfmt::skip]
fn print_summary(res: &AnalysisResult) {
    println!("\n{}", "Summary:".bold());
    println!("Total Functions: {}", res.metrics.len());
    let (v, i, c) = count_violations(res.metrics);
    let d_dup = res.delta.map_or(String::new(), |d| format!(" {}", crate::history::format_delta_pct(d.delta_dup)));
    println!("Volume > 15 lines: {}\nInterface > 4 params: {}\nComplexity > 5 branches: {}\nCode Duplication: {:.1}%{}", v, i, c, res.dup_res.percentage, d_dup);
}

pub(crate) fn count_violations(metrics: &[FunctionMetric]) -> (usize, usize, usize) {
    let (mut v, mut i, mut c) = (0, 0, 0);
    for m in metrics {
        if m.lines_of_code > 15 {
            v += 1;
        }
        if m.parameter_count > 4 {
            i += 1;
        }
        if m.cyclomatic_complexity > 5 {
            c += 1;
        }
    }
    (v, i, c)
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
    if is_balanced(metrics) { println!("  All components are balanced. {}", "[OK]".green()); }
    else { println!("  One component exceeds 50% of the codebase. {}", "[WARN]".yellow()); }
}

#[rustfmt::skip]
fn print_coupling(res: &AnalysisResult) {
    println!("\n{}", "Module Coupling:".bold());
    if res.graph.ignored_externals > 0 { println!("  {} external dependencies ignored.", res.graph.ignored_externals); }
    let h_fan = res.metrics.iter().filter(|m| res.graph.fan_out(&m.file_path) > 5).count();
    if h_fan > 0 { println!("  Fan-Out > 5: {} modules {}", h_fan, "[WARN]".yellow()); }
    else { println!("  Fan-Out is healthy across all modules. {}", "[OK]".green()); }
    let cycles = res.graph.detect_cycles();
    if cycles.is_empty() { println!("  No Circular Dependencies. {}", "[OK]".green()); } else {
        println!("  Circular Dependencies: {} DETECTED! {}", cycles.len(), "[CRITICAL]".red());
        let path: Vec<_> = cycles[0].iter().map(|p| p.file_name().unwrap_or_default().to_string_lossy()).collect();
        println!("     Example: {} -> {}", path.join(" -> "), path[0]);
    }
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
    println!("\n{}", "Hotspots (Risk + Churn):".bold().yellow());
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
fn print_profile(res: &AnalysisResult) {
    let s = res.score;
    println!("\n{}\nModerate Risk: {:.1}%\nHigh Risk: {:.1}%\nVery High Risk: {:.1}%\n\n─────────────────────────────────────\n{}", "Risk Profile:".bold(), s.pct_moderate, s.pct_high, s.pct_very_high, "Maintainability Rating:".bold());
    let d_code = res.delta.map_or(String::new(), |d| format!(" {}", crate::history::format_delta_stars(d.delta_code_stars)));
    println!("  Code Health:   {}{}", color_stars(s.code_stars, format!("{} ({:^1} / 7)", star_string(s.code_stars), s.code_stars)), d_code);
    print_coverage_line(res);
    let d_vol = res.delta.map_or(String::new(), |d| format!(" {}", crate::history::format_delta_num(d.delta_loc)));
    println!("  System Volume: {}{}", color_stars(s.volume_stars, format!("{} ({:^1} / 7) [Total: {} func LOC]", star_string(s.volume_stars), s.volume_stars, s.total_loc)), d_vol);
    let d_final = res.delta.map_or(String::new(), |d| format!(" {}", crate::history::format_delta_stars(d.delta_stars)));
    println!("  ──────────────────────────────\n  Final Score:   {}{}", color_stars(s.stars, format!("{} ({:^1} / 7)", star_string(s.stars), s.stars)).bold(), d_final);
    println!("\n{}", "Tip: Run 'cargo sig -r' (Markdown), 'cargo sig -w' (HTML), or 'cargo sig -p' (PDF) for full reports. 'cargo sig -h' for help.".dimmed());
}

#[rustfmt::skip]
fn print_coverage_line(res: &AnalysisResult) {
    let s = res.score;
    let d_cov = res.delta.and_then(|d| d.delta_cov).map_or(String::new(), |c| format!(" {}", crate::history::format_delta_pct(c)));
    if let (Some(pct), Some(st)) = (s.cov_pct, s.cov_stars) {
        println!("  Test Coverage: {}{}", color_stars(st, format!("{} ({:^1} / 7) [{:.1}% weighted]", star_string(st), st, pct)), d_cov);
    } else if !crate::coverage::has_llvm_cov() {
        println!("  Test Coverage: {}", "N/A (cargo-llvm-cov not installed. Run 'cargo install cargo-llvm-cov')".dimmed());
    } else {
        println!("  Test Coverage: {}", "N/A (No coverage data. Run 'cargo sig -a' to auto-generate)".dimmed());
    }
}

pub fn star_string(stars: u8) -> String {
    format!(
        "{}{}",
        "★".repeat(stars as usize),
        "☆".repeat((7 - stars) as usize)
    )
}

pub fn format_rel_path(path: &std::path::Path, root_dir: &std::path::Path) -> String {
    path.strip_prefix(root_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn color_stars(stars: u8, text: String) -> String {
    match stars {
        6..=7 => text.green().to_string(),
        4..=5 => text.yellow().to_string(),
        _ => text.red().to_string(),
    }
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
    let (v, i, c) = count_violations(res.metrics);
    format!("{{\"total_functions\":{},\"volume_violations\":{},\"interface_violations\":{},\"complexity_violations\":{},\"duplication_pct\":{:.1}}}", res.metrics.len(), v, i, c, res.dup_res.percentage)
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
        let cv = res.cov.as_ref().and_then(|c| c.get(*p)).map_or("null".to_string(), |c| c.percent().to_string());
        format!("{{\"file\":\"{}\",\"risk_points\":{},\"churn_commits\":{},\"coverage_pct\":{}}}", n, fr.get(*p).unwrap_or(&0), res.churns.get(*p).unwrap_or(&0), cv)
    }).collect();
    js.join(",")
}

#[rustfmt::skip]
pub fn print_json(res: &AnalysisResult) {
    let bal = is_balanced(res.metrics);
    let h_fan = res.metrics.iter().filter(|m| res.graph.fan_out(&m.file_path) > 15).count();
    let cycles = res.graph.detect_cycles().len();
    let cov_stars_str = res.score.cov_stars.map_or("null".to_string(), |s| s.to_string());
    let cov_pct_str = res.score.cov_pct.map_or("null".to_string(), |p| format!("{p:.1}"));
    println!("{{\n  \"summary\": {},\n  \"component_balance\": {{\"is_balanced\":{}}},\n  \"module_coupling\": {{\"ignored_externals\":{},\"fan_out_violations\":{},\"circular_dependencies\":{}}},\n  \"hotspots\": [{}],\n  \"risk_profile\": {{\"moderate_pct\":{:.1},\"high_pct\":{:.1},\"very_high_pct\":{:.1}}},\n  \"rating\": {{\"final_stars\":{},\"code_stars\":{},\"coverage_stars\":{},\"coverage_pct\":{},\"volume_stars\":{},\"total_func_loc\":{},\"max_stars\":7}}\n}}", 
        build_summary_json(res), bal, res.graph.ignored_externals, h_fan, cycles, build_hotspots_json(res), res.score.pct_moderate, res.score.pct_high, res.score.pct_very_high, res.score.stars, res.score.code_stars, cov_stars_str, cov_pct_str, res.score.volume_stars, res.score.total_loc);
}

pub(crate) fn get_sorted_hotspots(res: &AnalysisResult) -> (Vec<PathBuf>, HashMap<PathBuf, usize>) {
    let mut fr = HashMap::new();
    let g = crate::coupling::CouplingGraph::default();
    for m in res.metrics {
        *fr.entry(m.file_path.clone()).or_insert(0) +=
            crate::scoring::categorize_risk(m, &g) as usize;
    }
    let mut hs: Vec<_> = fr.keys().cloned().collect();
    hs.sort_by_key(|k| -((fr.get(k).unwrap_or(&0) * res.churns.get(k).unwrap_or(&0)) as isize));
    hs.retain(|k| *fr.get(k).unwrap_or(&0) > 0 && *res.churns.get(k).unwrap_or(&0) > 0);
    (hs, fr)
}

pub(crate) fn get_coverage_display(
    p: &std::path::Path,
    cov: Option<&HashMap<PathBuf, crate::coverage::Coverage>>,
) -> String {
    cov.and_then(|cv| cv.get(p))
        .map_or_else(|| "N/A".to_string(), |c| format!("{:.1}%", c.percent()))
}

pub(crate) fn hotspot_recommendation(r: usize, c: usize) -> &'static str {
    match (r > 10, c > 5, r > 5) {
        (true, true, _) => "Critical: Modular refactoring and test harness required.",
        (_, _, true) => "High: Split long functions and decrease branching complexity.",
        _ => "Moderate: Monitor churn and increase unit test coverage.",
    }
}

pub(crate) struct HotspotRow<'a> {
    pub idx: usize,
    pub rel_path: String,
    pub risk: usize,
    pub churn: usize,
    pub cov: String,
    pub rec: &'static str,
    pub _marker: std::marker::PhantomData<&'a ()>,
}

#[rustfmt::skip]
pub(crate) fn collect_hotspot_rows<'a>(
    hs: &'a [PathBuf], fr: &HashMap<PathBuf, usize>, res: &'a AnalysisResult<'a>, root: &std::path::Path,
) -> Vec<HotspotRow<'a>> {
    hs.iter().take(10).enumerate().map(|(i, p)| {
        let rel_path = format_rel_path(p, root);
        let risk = *fr.get(p).unwrap_or(&0);
        let churn = *res.churns.get(p).unwrap_or(&0);
        let cov = get_coverage_display(p, res.cov.as_ref());
        let rec = hotspot_recommendation(risk, churn);
        HotspotRow { idx: i + 1, rel_path, risk, churn, cov, rec, _marker: std::marker::PhantomData }
    }).collect()
}

pub(crate) fn filter_volume(metrics: &[FunctionMetric]) -> Vec<(&FunctionMetric, usize)> {
    metrics
        .iter()
        .filter(|m| m.lines_of_code > 15)
        .map(|m| (m, m.lines_of_code))
        .collect()
}

pub(crate) fn filter_complexity(metrics: &[FunctionMetric]) -> Vec<(&FunctionMetric, usize)> {
    metrics
        .iter()
        .filter(|m| m.cyclomatic_complexity > 5)
        .map(|m| (m, m.cyclomatic_complexity))
        .collect()
}

pub(crate) fn filter_interface(metrics: &[FunctionMetric]) -> Vec<(&FunctionMetric, usize)> {
    metrics
        .iter()
        .filter(|m| m.parameter_count > 4)
        .map(|m| (m, m.parameter_count))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("a\\b\"c"), "a\\\\b\\\"c");
    }

    #[test]
    fn test_star_string() {
        assert_eq!(star_string(0), "☆☆☆☆☆☆☆");
        assert_eq!(star_string(3), "★★★☆☆☆☆");
        assert_eq!(star_string(7), "★★★★★★★");
    }

    #[test]
    fn test_is_balanced() {
        let m1 = FunctionMetric {
            start_line: 0,
            file_path: PathBuf::from("comp1/a.rs"),
            function_name: "f1".into(),
            lines_of_code: 100,
            parameter_count: 0,
            cyclomatic_complexity: 0,
        };
        let m2 = FunctionMetric {
            start_line: 0,
            file_path: PathBuf::from("comp2/b.rs"),
            function_name: "f2".into(),
            lines_of_code: 10,
            parameter_count: 0,
            cyclomatic_complexity: 0,
        };
        let metrics = vec![m1.clone(), m2.clone()];
        // comp1 has 100, comp2 has 10. Total 110. comp1 is > 50%.
        assert!(!is_balanced(&metrics));

        let m3 = FunctionMetric {
            start_line: 0,
            file_path: PathBuf::from("comp2/c.rs"),
            function_name: "f3".into(),
            lines_of_code: 90,
            parameter_count: 0,
            cyclomatic_complexity: 0,
        };
        let metrics_bal = vec![m1, m2, m3];
        // comp1 has 100, comp2 has 100. Total 200. None > 50%.
        assert!(is_balanced(&metrics_bal));
    }

    #[test]
    fn test_build_summary_json() {
        let metrics = vec![FunctionMetric {
            start_line: 0,
            file_path: PathBuf::from("a.rs"),
            function_name: "f1".into(),
            lines_of_code: 20,
            parameter_count: 5,
            cyclomatic_complexity: 6,
        }];
        let graph = crate::coupling::CouplingGraph::default();
        let churns = HashMap::new();
        let cov = None;
        let score = Score {
            code_stars: 1,
            cov_stars: None,
            cov_pct: None,
            volume_stars: 1,
            stars: 1,
            pct_moderate: 0.0,
            pct_high: 0.0,
            pct_very_high: 0.0,
            total_loc: 20,
        };
        let res = AnalysisResult {
            metrics: &metrics,
            churns: &churns,
            cov: &cov,
            score: &score,
            dup_res: &crate::duplication::DuplicationResult {
                percentage: 1.5,
                blocks: vec![],
            },
            graph: &graph,
            delta: None,
            history: &[],
        };
        let json = build_summary_json(&res);
        assert_eq!(
            json,
            "{\"total_functions\":1,\"volume_violations\":1,\"interface_violations\":1,\"complexity_violations\":1,\"duplication_pct\":1.5}"
        );
    }

    #[test]
    fn test_print_all_and_json() {
        let metrics = vec![FunctionMetric {
            start_line: 0,
            file_path: PathBuf::from("a.rs"),
            function_name: "f1".into(),
            lines_of_code: 20,
            parameter_count: 5,
            cyclomatic_complexity: 6,
        }];
        let graph = crate::coupling::CouplingGraph::default();
        let churns = HashMap::new();
        let cov = None;
        let score = Score {
            code_stars: 1,
            cov_stars: None,
            cov_pct: None,
            volume_stars: 1,
            stars: 1,
            pct_moderate: 0.0,
            pct_high: 0.0,
            pct_very_high: 0.0,
            total_loc: 20,
        };
        let res = AnalysisResult {
            metrics: &metrics,
            churns: &churns,
            cov: &cov,
            score: &score,
            dup_res: &crate::duplication::DuplicationResult {
                percentage: 1.5,
                blocks: vec![],
            },
            graph: &graph,
            delta: None,
            history: &[],
        };

        print_all(&res);
        print_json(&res);
    }

    #[test]
    fn test_enforce_gate_pass() {
        enforce_gate(5, 4);
    }

    #[test]
    fn test_print_hotspots_with_data() {
        let metrics = vec![FunctionMetric {
            start_line: 0,
            file_path: PathBuf::from("a.rs"),
            function_name: "f1".into(),
            lines_of_code: 70,
            parameter_count: 8,
            cyclomatic_complexity: 30,
        }];
        let mut edges = HashMap::new();
        edges.insert(PathBuf::from("a.rs"), std::collections::HashSet::new());
        let graph = crate::coupling::CouplingGraph {
            edges,
            ignored_externals: 0,
        };
        let mut churns = HashMap::new();
        churns.insert(PathBuf::from("a.rs"), 5);
        let mut cov_map = HashMap::new();
        cov_map.insert(PathBuf::from("a.rs"), Coverage { hit: 5, total: 10 });
        let cov = Some(cov_map);
        let score = Score {
            code_stars: 1,
            cov_stars: Some(4),
            cov_pct: Some(50.0),
            volume_stars: 1,
            stars: 1,
            pct_moderate: 0.0,
            pct_high: 0.0,
            pct_very_high: 0.0,
            total_loc: 20,
        };
        let res = AnalysisResult {
            metrics: &metrics,
            churns: &churns,
            cov: &cov,
            score: &score,
            dup_res: &crate::duplication::DuplicationResult::default(),
            graph: &graph,
            delta: None,
            history: &[],
        };

        print_hotspots(&res);
        print_profile(&res);

        let json = build_hotspots_json(&res);
        assert!(json.contains("risk_points"));

        // Call print_json with Some(cov) to cover those lines
        print_json(&res);
    }
}
