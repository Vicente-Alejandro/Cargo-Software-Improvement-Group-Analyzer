use crate::analysis::FunctionMetric;

#[derive(Debug)]
pub struct Score {
    pub stars: u8,
    pub pct_moderate: f64,
    pub pct_high: f64,
    pub pct_very_high: f64,
}

pub fn evaluate(metrics: &[FunctionMetric]) -> Score {
    if metrics.is_empty() {
        return Score {
            stars: 7,
            pct_moderate: 0.0,
            pct_high: 0.0,
            pct_very_high: 0.0,
        };
    }

    let mut total_loc = 0;
    let mut moderate_loc = 0;
    let mut high_loc = 0;
    let mut very_high_loc = 0;

    for m in metrics {
        total_loc += m.lines_of_code;

        let is_very_high = m.lines_of_code > 60 || m.cyclomatic_complexity > 25;
        let is_high = !is_very_high && (m.lines_of_code > 30 || m.cyclomatic_complexity > 10);
        let is_moderate = !is_very_high && !is_high && (m.lines_of_code > 15 || m.cyclomatic_complexity > 5);

        if is_very_high {
            very_high_loc += m.lines_of_code;
        } else if is_high {
            high_loc += m.lines_of_code;
        } else if is_moderate {
            moderate_loc += m.lines_of_code;
        }
    }

    if total_loc == 0 {
        total_loc = 1; // Prevent division by zero
    }

    let pct_moderate = (moderate_loc as f64 / total_loc as f64) * 100.0;
    let pct_high = (high_loc as f64 / total_loc as f64) * 100.0;
    let pct_very_high = (very_high_loc as f64 / total_loc as f64) * 100.0;

    let stars = calculate_stars(pct_moderate, pct_high, pct_very_high);

    Score {
        stars,
        pct_moderate,
        pct_high,
        pct_very_high,
    }
}

fn calculate_stars(mod_pct: f64, high_pct: f64, vhigh_pct: f64) -> u8 {
    // 7 Stars (Perfection): 0% Very High, <= 2% High, <= 10% Moderate
    if vhigh_pct == 0.0 && high_pct <= 2.0 && mod_pct <= 10.0 {
        return 7;
    }
    // 6 Stars (Excellent): 0% Very High, <= 5% High, <= 15% Moderate
    if vhigh_pct == 0.0 && high_pct <= 5.0 && mod_pct <= 15.0 {
        return 6;
    }
    // 5 Stars (Good): <= 2% Very High, <= 10% High, <= 20% Moderate
    if vhigh_pct <= 2.0 && high_pct <= 10.0 && mod_pct <= 20.0 {
        return 5;
    }
    // 4 Stars (Average): <= 5% Very High, <= 15% High, <= 30% Moderate
    if vhigh_pct <= 5.0 && high_pct <= 15.0 && mod_pct <= 30.0 {
        return 4;
    }
    // 3 Stars (Mediocre): <= 10% Very High, <= 20% High, <= 40% Moderate
    if vhigh_pct <= 10.0 && high_pct <= 20.0 && mod_pct <= 40.0 {
        return 3;
    }
    // 2 Stars (Poor): <= 15% Very High, <= 30% High, <= 50% Moderate
    if vhigh_pct <= 15.0 && high_pct <= 30.0 && mod_pct <= 50.0 {
        return 2;
    }
    // 1 Star (Critical): Anything worse
    1
}
