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
fn aggregate_risk(ctx: &EvalCtx) -> [usize; 4] {
    let mut t = [0; 4];
    for m in ctx.metrics {
        t[0] += m.lines_of_code;
        let r = categorize_risk(m, ctx.graph) as usize;
        if r == 10 { t[3] += m.lines_of_code; }
        else if r == 5 { t[2] += m.lines_of_code; }
        else if r == 2 { t[1] += m.lines_of_code; }
    }
    t
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
    score
}

#[rustfmt::skip]
pub fn evaluate(ctx: &EvalCtx) -> Score {
    if ctx.metrics.is_empty() { return Score { stars: 7, code_stars: 7, ..Default::default() }; }
    let mut score = compute_score(aggregate_risk(ctx), ctx.dup, ctx.bal);
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
    Score { stars, code_stars: stars, cov_stars: None, cov_pct: None, pct_moderate: m, pct_high: h, pct_very_high: v }
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
    if cov >= 95.0 { 7 }
    else if cov >= 80.0 { 6 }
    else if cov >= 60.0 { 5 }
    else if cov >= 40.0 { 4 }
    else if cov >= 20.0 { 3 }
    else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
}
