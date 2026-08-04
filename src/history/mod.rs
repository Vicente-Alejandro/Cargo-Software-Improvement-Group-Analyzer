use crate::analysis::FunctionMetric;
use crate::duplication::DuplicationResult;
use crate::scoring::Score;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryRecord {
    pub date: String,
    pub commit: String,
    pub stars: u8,
    pub code_stars: u8,
    pub cov_pct: Option<f32>,
    pub total_loc: usize,
    pub total_violations: usize,
    pub dup_pct: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct HistoryDelta {
    pub delta_stars: i8,
    pub delta_code_stars: i8,
    pub delta_cov: Option<f32>,
    pub delta_loc: isize,
    pub delta_violations: isize,
    pub delta_dup: f32,
}

#[rustfmt::skip]
#[must_use]
pub fn create_record(score: &Score, metrics: &[FunctionMetric], dup: &DuplicationResult, root: &Path) -> HistoryRecord {
    let (v_len, i_len, c_len) = crate::report::count_violations(metrics);
    HistoryRecord {
        date: get_current_date_str(),
        commit: get_git_commit(root),
        stars: score.stars,
        code_stars: score.code_stars,
        cov_pct: score.cov_pct,
        total_loc: score.total_loc,
        total_violations: v_len + i_len + c_len,
        dup_pct: dup.percentage,
    }
}

#[rustfmt::skip]
#[must_use]
pub fn compute_delta(curr: &HistoryRecord, prev: &HistoryRecord) -> HistoryDelta {
    let delta_cov = match (curr.cov_pct, prev.cov_pct) {
        (Some(c), Some(p)) => Some(c - p),
        (Some(c), None) => Some(c),
        _ => None,
    };
    let d_stars = i8::try_from(curr.stars).unwrap_or(0) - i8::try_from(prev.stars).unwrap_or(0);
    let d_code = i8::try_from(curr.code_stars).unwrap_or(0) - i8::try_from(prev.code_stars).unwrap_or(0);
    let d_loc = isize::try_from(curr.total_loc).unwrap_or(0) - isize::try_from(prev.total_loc).unwrap_or(0);
    let d_viols = isize::try_from(curr.total_violations).unwrap_or(0) - isize::try_from(prev.total_violations).unwrap_or(0);
    HistoryDelta { delta_stars: d_stars, delta_code_stars: d_code, delta_cov, delta_loc: d_loc, delta_violations: d_viols, delta_dup: curr.dup_pct - prev.dup_pct }
}

pub fn get_history_path(root_dir: &Path) -> PathBuf {
    root_dir.join("tools").join("cargo-sig").join(".sig_history.md")
}

#[must_use]
pub fn read_history(root_dir: &Path) -> Vec<HistoryRecord> {
    let path = get_history_path(root_dir);
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    content.lines().filter_map(parse_history_row).collect()
}

#[rustfmt::skip]
pub fn record_history(root_dir: &Path, rec: &HistoryRecord) -> std::io::Result<()> {
    let path = prepare_history_file(root_dir)?;
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    let cov = rec.cov_pct.map_or_else(|| "N/A".to_string(), |p| format!("{p:.1}%"));
    writeln!(
        f,
        "| {} | `{}` | {}/7 | {}/7 | {} | {} LOC | {} | {:.1}% |",
        rec.date, rec.commit, rec.stars, rec.code_stars, cov, rec.total_loc, rec.total_violations, rec.dup_pct
    )
}

#[rustfmt::skip]
fn prepare_history_file(root_dir: &Path) -> std::io::Result<PathBuf> {
    let out = root_dir.join("tools").join("cargo-sig");
    if !out.exists() { fs::create_dir_all(&out)?; }
    let path = out.join(".sig_history.md");
    if !path.exists() {
        let hdr = "# Cargo SIG Maintainability History\n\n| Date | Commit | Score | Code Health | Coverage | System Volume | Violations | Duplication |\n|---|---|---|---|---|---|---|---|\n";
        fs::write(&path, hdr)?;
    }
    Ok(path)
}

#[rustfmt::skip]
#[must_use]
pub fn parse_history_row(line: &str) -> Option<HistoryRecord> {
    let t = line.trim();
    if !t.starts_with('|') || t.contains("---|") || t.contains("Date | Commit") { return None; }
    let c: Vec<&str> = t.split('|').map(str::trim).collect();
    if c.len() < 10 { return None; }
    let (d, cm) = (c[1].to_string(), c[2].trim_matches('`').to_string());
    let (st, c_st) = (c[3].split('/').next()?.parse().ok()?, c[4].split('/').next()?.parse().ok()?);
    let cov = c[5].trim_end_matches('%').parse().ok();
    let loc = c[6].split_whitespace().next()?.parse().ok()?;
    let (v, dup) = (c[7].parse().ok()?, c[8].trim_end_matches('%').parse().ok()?);
    Some(HistoryRecord { date: d, commit: cm, stars: st, code_stars: c_st, cov_pct: cov, total_loc: loc, total_violations: v, dup_pct: dup })
}

#[must_use]
pub fn get_git_commit(root_dir: &Path) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(root_dir)
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { "HEAD".to_string() } else { s }
        }
        _ => "HEAD".to_string(),
    }
}

#[must_use]
pub fn get_current_date_str() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    format_timestamp(secs)
}

#[must_use]
pub fn format_timestamp(secs: u64) -> String {
    let days = secs / 86400;
    let day_secs = secs % 86400;
    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let (y, m, d) = get_ymd(days);
    format!("{y:04}-{m:02}-{d:02} {hour:02}:{min:02}")
}

#[rustfmt::skip]
#[must_use]
fn get_year(mut days: u64) -> (u64, u64) {
    let mut year = 1970;
    loop {
        let y_days = if is_leap(year) { 366 } else { 365 };
        if days < y_days { return (year, days); }
        days -= y_days;
        year += 1;
    }
}

#[rustfmt::skip]
#[must_use]
fn get_ymd(days: u64) -> (u64, u64, u64) {
    let (year, mut rem) = get_year(days);
    let m_days = [31, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1;
    for &d in &m_days {
        if rem < d { break; }
        rem -= d;
        month += 1;
    }
    (year, month, rem + 1)
}

#[must_use]
const fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

#[must_use]
pub fn format_delta_num(delta: isize) -> String {
    match delta {
        0 => "(=)".to_string(),
        d if d > 0 => format!("(+{d})"),
        d => format!("({d})"),
    }
}

#[must_use]
pub fn format_delta_stars(delta: i8) -> String {
    match delta {
        0 => "(=)".to_string(),
        d if d > 0 => format!("(+{d})"),
        d => format!("({d})"),
    }
}

#[must_use]
pub fn format_delta_pct(delta: f32) -> String {
    if delta.abs() < 0.05 {
        "(=)".to_string()
    } else if delta > 0.0 {
        format!("(+{delta:.1}%)")
    } else {
        format!("({delta:.1}%)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_format_timestamp() {
        let ts = 1_700_000_000; // 2023-11-14 22:13:20 UTC
        let s = format_timestamp(ts);
        assert_eq!(s, "2023-11-14 22:13");
    }

    #[test]
    fn test_compute_delta() {
        let prev = HistoryRecord {
            date: "2026-08-01 10:00".to_string(),
            commit: "abc1234".to_string(),
            stars: 6,
            code_stars: 6,
            cov_pct: Some(90.0),
            total_loc: 1500,
            total_violations: 4,
            dup_pct: 2.0,
        };
        let curr = HistoryRecord {
            date: "2026-08-03 12:00".to_string(),
            commit: "def5678".to_string(),
            stars: 7,
            code_stars: 7,
            cov_pct: Some(95.0),
            total_loc: 1600,
            total_violations: 0,
            dup_pct: 1.0,
        };
        let delta = compute_delta(&curr, &prev);
        assert_eq!(delta.delta_stars, 1);
        assert_eq!(delta.delta_code_stars, 1);
        assert_eq!(delta.delta_cov, Some(5.0));
        assert_eq!(delta.delta_loc, 100);
        assert_eq!(delta.delta_violations, -4);
        assert!((delta.delta_dup - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_record_and_read_history() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let rec1 = HistoryRecord {
            date: "2026-08-01 10:00".to_string(),
            commit: "abc1234".to_string(),
            stars: 6,
            code_stars: 6,
            cov_pct: Some(90.0),
            total_loc: 1500,
            total_violations: 4,
            dup_pct: 2.0,
        };
        let rec2 = HistoryRecord {
            date: "2026-08-03 12:00".to_string(),
            commit: "def5678".to_string(),
            stars: 7,
            code_stars: 7,
            cov_pct: None,
            total_loc: 1600,
            total_violations: 0,
            dup_pct: 1.0,
        };
        record_history(root, &rec1).unwrap();
        record_history(root, &rec2).unwrap();

        let history = read_history(root);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].commit, "abc1234");
        assert_eq!(history[0].stars, 6);
        assert_eq!(history[1].commit, "def5678");
        assert_eq!(history[1].stars, 7);
        assert_eq!(history[1].cov_pct, None);
    }

    #[test]
    fn test_format_delta_strings() {
        assert_eq!(format_delta_stars(0), "(=)");
        assert_eq!(format_delta_stars(1), "(+1)");
        assert_eq!(format_delta_stars(-2), "(-2)");
        assert_eq!(format_delta_num(15), "(+15)");
        assert_eq!(format_delta_pct(1.5), "(+1.5%)");
        assert_eq!(format_delta_pct(-0.4), "(-0.4%)");
        assert_eq!(format_delta_pct(0.01), "(=)");
    }
}
