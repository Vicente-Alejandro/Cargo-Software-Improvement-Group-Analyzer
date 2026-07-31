use crate::analysis::FunctionMetric;
use crate::coupling::CouplingGraph;
use crate::coverage::{Coverage, churn_weighted_coverage};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Score {
    pub stars: u8,
    pub code_stars: u8,
    pub cov_stars: Option<u8>,
    pub cov_pct: Option<f32>,
    pub volume_stars: u8,
    pub total_loc: usize,
    pub pct_moderate: f64,
    pub pct_high: f64,
    pub pct_very_high: f64,
}

#[derive(Clone, Copy)]
pub enum Risk {
    Low = 0,
    Moderate = 2,
    High = 5,
    VeryHigh = 10,
}

pub struct EvalCtx<'a> {
    pub metrics: &'a [FunctionMetric],
    pub dup: f32,
    pub bal: bool,
    pub graph: &'a CouplingGraph,
    pub cov: &'a Option<HashMap<PathBuf, Coverage>>,
    pub churns: &'a HashMap<PathBuf, usize>,
}

#[rustfmt::skip]
fn aggregate_risk(ctx: &EvalCtx) -> ([usize; 4], usize) {
    let mut t = [0; 4];
    for m in ctx.metrics {
        t[0] += m.lines_of_code;
        let r = categorize_risk(m, ctx.graph) as usize;
        if r == 10 { t[3] += m.lines_of_code; }
        else if r == 5 { t[2] += m.lines_of_code; }
        else if r == 2 { t[1] += m.lines_of_code; }
    }
    (t, t[0])
}

#[rustfmt::skip]
fn apply_coverage(mut score: Score, ctx: &EvalCtx) -> Score {
    if let Some(c_map) = ctx.cov {
        let pct = churn_weighted_coverage(c_map, ctx.churns);
        let c_stars = calculate_cov_stars(pct);
        score.cov_pct = Some(pct);
        score.cov_stars = Some(c_stars);
        score.stars = score.code_stars.min(c_stars);
    } else { score.stars = score.code_stars; }
    
    if score.volume_stars <= 2 && score.stars > 1 { score.stars -= 1; }
    score
}

#[rustfmt::skip]
pub fn evaluate(ctx: &EvalCtx) -> Score {
    if ctx.metrics.is_empty() { return Score { stars: 7, code_stars: 7, volume_stars: 7, ..Default::default() }; }
    let (t, total_loc) = aggregate_risk(ctx);
    let mut score = compute_score(t, ctx.dup, ctx.bal);
    score.total_loc = total_loc;
    score.volume_stars = calculate_volume_stars(total_loc);
    if !ctx.graph.detect_cycles().is_empty() && score.code_stars > 1 { score.code_stars = 1; }
    apply_coverage(score, ctx)
}

#[rustfmt::skip]
pub fn categorize_risk(m: &FunctionMetric, graph: &CouplingGraph) -> Risk {
    let f = graph.fan_out(&m.file_path);
    if is_vh(m, f) { return Risk::VeryHigh; }
    if is_h(m, f) { return Risk::High; }
    if is_m(m, f) { return Risk::Moderate; }
    Risk::Low
}

#[rustfmt::skip]
fn is_vh(m: &FunctionMetric, f: usize) -> bool {
    m.lines_of_code > 60 || m.cyclomatic_complexity > 25 || m.parameter_count > 7 || f > 10
}
#[rustfmt::skip]
fn is_h(m: &FunctionMetric, f: usize) -> bool {
    m.lines_of_code > 30 || m.cyclomatic_complexity > 10 || m.parameter_count > 5 || f > 7
}
#[rustfmt::skip]
fn is_m(m: &FunctionMetric, f: usize) -> bool {
    m.lines_of_code > 15 || m.cyclomatic_complexity > 5 || m.parameter_count > 4 || f > 5
}

#[rustfmt::skip]
fn compute_score(t: [usize; 4], dup: f32, balanced: bool) -> Score {
    let tot = if t[0] == 0 { 1.0 } else { t[0] as f64 };
    let (m, h, v) = ((t[1] as f64 / tot) * 100.0, (t[2] as f64 / tot) * 100.0, (t[3] as f64 / tot) * 100.0);
    let mut stars = calculate_stars(m, h, v, dup);
    if !balanced && stars > 5 { stars = 5; }
    Score { stars, code_stars: stars, cov_stars: None, cov_pct: None, volume_stars: 7, total_loc: 0, pct_moderate: m, pct_high: h, pct_very_high: v }
}

struct Threshold {
    v: f64,
    h: f64,
    m: f64,
    d: f64,
    stars: u8,
}

const THRESHOLDS: [Threshold; 6] = [
    Threshold {
        v: 0.0,
        h: 2.0,
        m: 10.0,
        d: 3.0,
        stars: 7,
    },
    Threshold {
        v: 0.0,
        h: 5.0,
        m: 15.0,
        d: 5.0,
        stars: 6,
    },
    Threshold {
        v: 2.0,
        h: 10.0,
        m: 20.0,
        d: 10.0,
        stars: 5,
    },
    Threshold {
        v: 5.0,
        h: 15.0,
        m: 30.0,
        d: 20.0,
        stars: 4,
    },
    Threshold {
        v: 10.0,
        h: 20.0,
        m: 40.0,
        d: 30.0,
        stars: 3,
    },
    Threshold {
        v: 15.0,
        h: 30.0,
        m: 50.0,
        d: 40.0,
        stars: 2,
    },
];

#[rustfmt::skip]
fn calculate_stars(m: f64, h: f64, v: f64, d: f32) -> u8 {
    THRESHOLDS.iter().find(|t| v <= t.v && h <= t.h && m <= t.m && (d as f64) <= t.d).map(|t| t.stars).unwrap_or(1)
}

#[rustfmt::skip]
fn calculate_cov_stars(cov: f32) -> u8 {
    let th = [(95.0, 7), (80.0, 6), (60.0, 5), (40.0, 4), (20.0, 3)];
    th.iter().find(|(t, _)| cov >= *t).map(|(_, s)| *s).unwrap_or(1)
}

#[rustfmt::skip]
fn calculate_volume_stars(loc: usize) -> u8 {
    let th = [(10_000, 7), (30_000, 6), (75_000, 5), (150_000, 4), (300_000, 3), (600_000, 2)];
    th.iter().find(|(t, _)| loc < *t).map(|(_, s)| *s).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_score() {
        let score = compute_score([100, 0, 0, 0], 0.0, true);
        assert_eq!(score.stars, 7);
    }

    #[test]
    fn test_empty_metrics() {
        let graph = crate::coupling::CouplingGraph::default();
        let churns = HashMap::new();
        let ctx = EvalCtx {
            metrics: &[],
            dup: 0.0,
            bal: true,
            graph: &graph,
            cov: &None,
            churns: &churns,
        };
        let score = evaluate(&ctx);
        assert_eq!(score.stars, 7);
    }

    #[test]
    fn test_calculate_cov_stars() {
        assert_eq!(calculate_cov_stars(100.0), 7);
        assert_eq!(calculate_cov_stars(95.0), 7);
        assert_eq!(calculate_cov_stars(85.0), 6);
        assert_eq!(calculate_cov_stars(65.0), 5);
        assert_eq!(calculate_cov_stars(45.0), 4);
        assert_eq!(calculate_cov_stars(25.0), 3);
        assert_eq!(calculate_cov_stars(10.0), 1);
    }

    #[test]
    fn test_calculate_volume_stars() {
        assert_eq!(calculate_volume_stars(5_000), 7);
        assert_eq!(calculate_volume_stars(15_000), 6);
        assert_eq!(calculate_volume_stars(50_000), 5);
        assert_eq!(calculate_volume_stars(100_000), 4);
        assert_eq!(calculate_volume_stars(200_000), 3);
        assert_eq!(calculate_volume_stars(400_000), 2);
        assert_eq!(calculate_volume_stars(700_000), 1);
    }

    #[test]
    fn test_categorize_risk() {
        let graph = CouplingGraph::default();
        let m_low = FunctionMetric {
            file_path: PathBuf::new(),
            function_name: String::new(),
            lines_of_code: 10,
            parameter_count: 2,
            cyclomatic_complexity: 2,
        };
        assert!(matches!(categorize_risk(&m_low, &graph), Risk::Low));

        let m_mod = FunctionMetric {
            file_path: PathBuf::new(),
            function_name: String::new(),
            lines_of_code: 20,
            parameter_count: 2,
            cyclomatic_complexity: 2,
        };
        assert!(matches!(categorize_risk(&m_mod, &graph), Risk::Moderate));

        let m_high = FunctionMetric {
            file_path: PathBuf::new(),
            function_name: String::new(),
            lines_of_code: 40,
            parameter_count: 2,
            cyclomatic_complexity: 2,
        };
        assert!(matches!(categorize_risk(&m_high, &graph), Risk::High));

        let m_vh = FunctionMetric {
            file_path: PathBuf::new(),
            function_name: String::new(),
            lines_of_code: 70,
            parameter_count: 2,
            cyclomatic_complexity: 2,
        };
        assert!(matches!(categorize_risk(&m_vh, &graph), Risk::VeryHigh));
    }

    #[test]
    fn test_compute_score_thresholds() {
        let s2 = compute_score([100, 0, 0, 12], 0.0, true);
        assert_eq!(s2.stars, 2);

        let s_unbalanced = compute_score([100, 0, 0, 0], 0.0, false);
        assert_eq!(s_unbalanced.stars, 5);
    }

    #[test]
    fn test_evaluate() {
        let graph = CouplingGraph::default();
        let m1 = FunctionMetric {
            file_path: PathBuf::from("a.rs"),
            function_name: String::new(),
            lines_of_code: 10,
            parameter_count: 2,
            cyclomatic_complexity: 2,
        };

        let churns = HashMap::new();
        let mut cov_map = HashMap::new();
        cov_map.insert(PathBuf::from("a.rs"), Coverage { hit: 1, total: 2 });
        let cov = Some(cov_map);

        let metrics = vec![m1];
        let ctx = EvalCtx {
            metrics: &metrics,
            dup: 0.0,
            bal: true,
            graph: &graph,
            cov: &cov,
            churns: &churns,
        };

        let score = evaluate(&ctx);
        assert_eq!(score.stars, 4);
    }
}
