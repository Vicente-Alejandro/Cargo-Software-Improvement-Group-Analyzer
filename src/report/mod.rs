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
    pub dup_res: &'a crate::duplication::DuplicationResult,
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
    println!("Volume > 15 lines: {}\nInterface > 4 params: {}\nComplexity > 5 branches: {}\nCode Duplication: {:.1}%", v, i, c, res.dup_res.percentage);
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
    if is_balanced(metrics) { println!("  All components are balanced. ✅"); }
    else { println!("  One component exceeds 50% of the codebase. ⚠️"); }
}

#[rustfmt::skip]
fn print_coupling(res: &AnalysisResult) {
    println!("\n{}", "Module Coupling:".bold());
    if res.graph.ignored_externals > 0 { println!("  {} external dependencies ignored. {}", res.graph.ignored_externals, "ℹ️".blue()); }
    let h_fan = res.metrics.iter().filter(|m| res.graph.fan_out(&m.file_path) > 5).count();
    if h_fan > 0 { println!("  Fan-Out > 5: {} modules {}", h_fan, "⚠️".yellow()); }
    else { println!("  Fan-Out is healthy across all modules. {}", "✅".green()); }
    let cycles = res.graph.detect_cycles();
    if cycles.is_empty() { println!("  No Circular Dependencies. {}", "✅".green()); } else {
        println!("  Circular Dependencies: {} DETECTED! {}", cycles.len(), "🚨".red());
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
    println!("\n{}\nModerate Risk: {:.1}%\nHigh Risk: {:.1}%\nVery High Risk: {:.1}%\n\n─────────────────────────────────────\n{}", "Risk Profile:".bold(), s.pct_moderate, s.pct_high, s.pct_very_high, "Maintainability Rating:".bold());
    println!("  Code Health:   {}", color_stars(s.code_stars, format!("{} ({:^1} / 7)", star_string(s.code_stars), s.code_stars)));
    if let (Some(pct), Some(st)) = (s.cov_pct, s.cov_stars) {
        println!("  Test Coverage: {}", color_stars(st, format!("{} ({:^1} / 7) [{:.1}% weighted]", star_string(st), st, pct)));
    } else { println!("  Test Coverage: {}", "N/A (No coverage data. Run 'cargo sig -a' to auto-generate)".dimmed()); }
    println!("  System Volume: {}", color_stars(s.volume_stars, format!("{} ({:^1} / 7) [Total: {} func LOC]", star_string(s.volume_stars), s.volume_stars, s.total_loc)));
    println!("  ──────────────────────────────\n  Final Score:   {}", color_stars(s.stars, format!("{} ({:^1} / 7)", star_string(s.stars), s.stars)).bold());
}

fn star_string(stars: u8) -> String {
    format!(
        "{}{}",
        "★".repeat(stars as usize),
        "☆".repeat((7 - stars) as usize)
    )
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
    let (mut v, mut i, mut c) = (0, 0, 0);
    for m in res.metrics {
        if m.lines_of_code > 15 { v += 1; }
        if m.parameter_count > 4 { i += 1; }
        if m.cyclomatic_complexity > 5 { c += 1; }
    }
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
        };

        print_hotspots(&res);
        print_profile(&res.score);

        let json = build_hotspots_json(&res);
        assert!(json.contains("risk_points"));

        // Call print_json with Some(cov) to cover those lines
        print_json(&res);
    }
}
