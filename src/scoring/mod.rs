use crate::analysis::FunctionMetric;

#[derive(Debug, Default)]
pub struct Score {
    pub stars: u8,
    pub pct_moderate: f64,
    pub pct_high: f64,
    pub pct_very_high: f64,
}

enum Risk {
    Low,
    Moderate,
    High,
    VeryHigh,
}

pub fn evaluate(metrics: &[FunctionMetric], dup: f32) -> Score {
    if metrics.is_empty() {
        return Score {
            stars: 7,
            ..Default::default()
        };
    }
    let mut totals = [0, 0, 0, 0];
    for m in metrics {
        totals[0] += m.lines_of_code;
        add_risk(&mut totals, categorize_risk(m), m.lines_of_code);
    }
    compute_score(totals, dup)
}

fn add_risk(t: &mut [usize; 4], r: Risk, loc: usize) {
    match r {
        Risk::VeryHigh => t[3] += loc,
        Risk::High => t[2] += loc,
        Risk::Moderate => t[1] += loc,
        Risk::Low => {}
    }
}

fn categorize_risk(m: &FunctionMetric) -> Risk {
    if is_vhigh(m) {
        return Risk::VeryHigh;
    }
    if is_high(m) {
        return Risk::High;
    }
    if is_mod(m) {
        return Risk::Moderate;
    }
    Risk::Low
}

fn is_vhigh(m: &FunctionMetric) -> bool {
    m.lines_of_code > 60 || m.cyclomatic_complexity > 25
}
fn is_high(m: &FunctionMetric) -> bool {
    m.lines_of_code > 30 || m.cyclomatic_complexity > 10
}
fn is_mod(m: &FunctionMetric) -> bool {
    m.lines_of_code > 15 || m.cyclomatic_complexity > 5
}

fn compute_score(t: [usize; 4], dup: f32) -> Score {
    let tot = if t[0] == 0 { 1.0 } else { t[0] as f64 };
    Score {
        stars: calculate_stars(
            (t[1] as f64 / tot) * 100.0,
            (t[2] as f64 / tot) * 100.0,
            (t[3] as f64 / tot) * 100.0,
            dup,
        ),
        pct_moderate: (t[1] as f64 / tot) * 100.0,
        pct_high: (t[2] as f64 / tot) * 100.0,
        pct_very_high: (t[3] as f64 / tot) * 100.0,
    }
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

fn calculate_stars(m: f64, h: f64, v: f64, d: f32) -> u8 {
    let d64 = d as f64;
    for t in THRESHOLDS {
        if v <= t.v && h <= t.h && m <= t.m && d64 <= t.d {
            return t.stars;
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_score() {
        let score = compute_score([100, 0, 0, 0], 0.0);
        assert_eq!(score.stars, 7);
    }

    #[test]
    fn test_one_star() {
        let score = compute_score([100, 0, 0, 60], 0.0);
        assert_eq!(score.stars, 1);
    }

    #[test]
    fn test_moderate_penalty() {
        let score = compute_score([100, 20, 0, 0], 0.0);
        assert_eq!(score.stars, 5);
    }

    #[test]
    fn test_dup_penalty() {
        let score = compute_score([100, 0, 0, 0], 25.0);
        assert_eq!(score.stars, 3);
    }

    #[test]
    fn test_empty_metrics() {
        let score = evaluate(&[], 0.0);
        assert_eq!(score.stars, 7);
    }
}
