use crate::analysis::FunctionMetric;

#[derive(Debug, Default)]
pub struct Score {
    pub stars: u8,
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

#[rustfmt::skip]
pub fn evaluate(metrics: &[FunctionMetric], dup: f32, bal: bool, graph: &crate::coupling::CouplingGraph) -> Score {
    if metrics.is_empty() { return Score { stars: 7, ..Default::default() }; }
    let mut t = [0; 4];
    for m in metrics {
        t[0] += m.lines_of_code;
        let r = categorize_risk(m, graph) as usize;
        if r == 10 { t[3] += m.lines_of_code; }
        else if r == 5 { t[2] += m.lines_of_code; }
        else if r == 2 { t[1] += m.lines_of_code; }
    }
    let mut score = compute_score(t, dup, bal);
    if !graph.detect_cycles().is_empty() && score.stars > 1 { score.stars = 1; }
    score
}

#[rustfmt::skip]
pub fn categorize_risk(m: &FunctionMetric, graph: &crate::coupling::CouplingGraph) -> Risk {
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
    Score { stars, pct_moderate: m, pct_high: h, pct_very_high: v }
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
        let score = evaluate(&[], 0.0, true, &graph);
        assert_eq!(score.stars, 7);
    }

    #[test]
    fn test_interface_size_penalty() {
        let metrics = vec![FunctionMetric {
            function_name: "test_fn".to_string(),
            file_path: PathBuf::new(),
            lines_of_code: 10,
            cyclomatic_complexity: 1,
            parameter_count: 8,
        }];
        let graph = crate::coupling::CouplingGraph::default();
        let score = evaluate(&metrics, 0.0, true, &graph);
        assert_eq!(score.stars, 1);
    }
}
