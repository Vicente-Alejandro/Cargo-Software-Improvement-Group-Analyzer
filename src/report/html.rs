use super::{
    AnalysisResult, collect_hotspot_rows, filter_complexity, filter_interface, filter_volume,
    format_rel_path, get_sorted_hotspots, star_string,
};
use crate::analysis::FunctionMetric;
use crate::scoring::Score;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CSS: &str = r#"
:root {
  --bg: #0d1117;
  --surface: #161b22;
  --surface-hover: #1c2128;
  --border: #30363d;
  --text-main: #f0f6fc;
  --text-muted: #8b949e;
  --accent: #58a6ff;
  --green: #3fb950;
  --amber: #d29922;
  --red: #f85149;
  --purple: #bc8cff;
}
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  background-color: var(--bg);
  color: var(--text-main);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  line-height: 1.5;
  padding: 2.5rem 1.5rem;
}
.container { max-width: 1100px; margin: 0 auto; }
header { margin-bottom: 2.5rem; border-bottom: 1px solid var(--border); padding-bottom: 1.5rem; }
.badge { display: inline-block; font-size: 0.75rem; font-weight: 600; padding: 0.2rem 0.6rem; border-radius: 999px; background: rgba(88,166,255,0.15); color: var(--accent); border: 1px solid rgba(88,166,255,0.3); margin-bottom: 0.5rem; }
h1 { font-size: 2rem; font-weight: 700; display: flex; align-items: center; gap: 0.5rem; }
.subtitle { color: var(--text-muted); font-size: 0.95rem; margin-top: 0.25rem; }
.grid-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1.25rem; margin-bottom: 2.5rem; }
.card { background: var(--surface); border: 1px solid var(--border); border-radius: 10px; padding: 1.25rem; box-shadow: 0 4px 12px rgba(0,0,0,0.2); }
.card-title { font-size: 0.85rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; font-weight: 600; margin-bottom: 0.5rem; }
.card-val { font-size: 1.6rem; font-weight: 700; color: var(--text-main); display: flex; align-items: baseline; gap: 0.5rem; }
.card-sub { font-size: 0.85rem; color: var(--text-muted); margin-top: 0.35rem; }
.stars { color: #e3b341; letter-spacing: 2px; }
.section { background: var(--surface); border: 1px solid var(--border); border-radius: 10px; padding: 1.5rem; margin-bottom: 2rem; }
.section-title { font-size: 1.25rem; font-weight: 600; margin-bottom: 1rem; display: flex; align-items: center; gap: 0.5rem; }
.risk-bar { display: flex; height: 12px; border-radius: 6px; overflow: hidden; background: #21262d; margin: 1rem 0; }
.risk-low { background: var(--green); }
.risk-mod { background: var(--amber); }
.risk-high { background: var(--red); }
.risk-vhigh { background: var(--purple); }
.legend { display: flex; gap: 1.5rem; font-size: 0.85rem; color: var(--text-muted); flex-wrap: wrap; }
.legend-item { display: flex; align-items: center; gap: 0.4rem; }
.dot { width: 10px; height: 10px; border-radius: 50%; display: inline-block; }
table { width: 100%; border-collapse: collapse; margin-top: 0.75rem; font-size: 0.9rem; }
th, td { text-align: left; padding: 0.65rem 0.85rem; border-bottom: 1px solid var(--border); }
th { background: rgba(255,255,255,0.02); color: var(--text-muted); font-weight: 600; font-size: 0.8rem; text-transform: uppercase; }
tr:hover { background: var(--surface-hover); }
code { font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace; font-size: 0.85rem; background: rgba(110,118,129,0.2); padding: 0.15rem 0.4rem; border-radius: 4px; }
.empty-msg { color: var(--green); font-size: 0.9rem; padding: 0.5rem 0; }
footer { text-align: center; color: var(--text-muted); font-size: 0.85rem; margin-top: 3rem; }
"#;

pub fn generate_html_report(res: &AnalysisResult, root_dir: &Path) -> io::Result<PathBuf> {
    let out_dir = root_dir.join("tools").join("cargo-sig");
    if !out_dir.exists() {
        fs::create_dir_all(&out_dir)?;
    }
    let report_path = out_dir.join("SIG_REPORT.html");
    fs::write(&report_path, render_html(res, root_dir))?;
    Ok(report_path)
}

#[rustfmt::skip]
#[must_use]
pub fn render_html(res: &AnalysisResult, root_dir: &Path) -> String {
    let mut out = String::with_capacity(16384);
    let mut ctx = HtmlCtx { out: &mut out, res, root_dir };
    ctx.render_doc_start(); ctx.render_header(); ctx.render_scorecard();
    ctx.render_risk_profile(); ctx.render_violations(); ctx.render_hotspots();
    ctx.render_architecture(); ctx.render_doc_end();
    out
}

struct HtmlCtx<'a> {
    out: &'a mut String,
    res: &'a AnalysisResult<'a>,
    root_dir: &'a Path,
}

impl HtmlCtx<'_> {
    fn render_doc_start(&mut self) {
        self.out
            .push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n");
        self.out.push_str(
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        self.out
            .push_str("<title>SIG Maintainability Report</title>\n");
        let _ = writeln!(
            self.out,
            "<style>{CSS}</style>\n</head>\n<body>\n<div class=\"container\">\n"
        );
    }

    fn render_doc_end(&mut self) {
        self.out.push_str("<footer>Generated by <strong>cargo-sig</strong> — Software Improvement Group Static Analyzer for Rust</footer>\n");
        self.out.push_str("</div>\n</body>\n</html>\n");
    }

    fn render_header(&mut self) {
        self.out
            .push_str("<header>\n<span class=\"badge\">SIG Quality Model</span>\n");
        self.out.push_str("<h1>📊 Maintainability Report</h1>\n");
        self.out.push_str("<p class=\"subtitle\">Static code quality, cyclomatic complexity, git churn, and coverage analysis.</p>\n</header>\n");
    }

    #[rustfmt::skip]
    fn render_scorecard(&mut self) {
        self.out.push_str("<div class=\"grid-cards\">\n");
        self.render_primary_cards();
        self.render_secondary_cards();
        self.out.push_str("</div>\n");
    }

    #[rustfmt::skip]
    fn render_primary_cards(&mut self) {
        let s = self.res.score;
        let c1 = CardMeta { title: "Final Rating", val: &format!("{}/7 Stars", s.stars), stars: &star_string(s.stars), sub: "Overall SIG Score" };
        let c2 = CardMeta { title: "Code Health", val: &format!("{}/7 Stars", s.code_stars), stars: &star_string(s.code_stars), sub: "Volume & Complexity" };
        self.render_card(c1); self.render_card(c2);
    }

    #[rustfmt::skip]
    fn render_secondary_cards(&mut self) {
        let s = self.res.score;
        self.render_cov_card(s);
        let c4 = CardMeta { title: "System Volume", val: &format!("{} LOC", s.total_loc), stars: &star_string(s.volume_stars), sub: &format!("{}/7 Stars Volume", s.volume_stars) };
        self.render_card(c4);
    }

    fn render_card(&mut self, c: CardMeta) {
        let _ = writeln!(
            self.out,
            "<div class=\"card\"><div class=\"card-title\">{}</div><div class=\"card-val\">{}</div><div class=\"stars\">{}</div><div class=\"card-sub\">{}</div></div>",
            c.title, c.val, c.stars, c.sub
        );
    }

    #[rustfmt::skip]
    fn render_cov_card(&mut self, s: &Score) {
        if let (Some(pct), Some(st)) = (s.cov_pct, s.cov_stars) {
            self.render_card(CardMeta { title: "Test Coverage", val: &format!("{pct:.1}%"), stars: &star_string(st), sub: &format!("{st}/7 Stars Weighted") });
        } else {
            self.render_card(CardMeta { title: "Test Coverage", val: "N/A", stars: "☆☆☆☆☆☆☆", sub: "No coverage data" });
        }
    }

    #[rustfmt::skip]
    fn render_risk_profile(&mut self) {
        let s = self.res.score;
        let low_pct = (100.0 - s.pct_moderate - s.pct_high - s.pct_very_high).max(0.0);
        self.out.push_str("<div class=\"section\"><h2 class=\"section-title\">🎯 Risk Profile Distribution</h2>\n");
        let _ = writeln!(self.out, "<div class=\"risk-bar\"><div class=\"risk-low\" style=\"width:{low_pct}%\"></div><div class=\"risk-mod\" style=\"width:{}%\"></div><div class=\"risk-high\" style=\"width:{}%\"></div><div class=\"risk-vhigh\" style=\"width:{}%\"></div></div>", s.pct_moderate, s.pct_high, s.pct_very_high);
        self.render_risk_legend(low_pct, s);
        self.out.push_str("</div>\n");
    }

    #[rustfmt::skip]
    fn render_risk_legend(&mut self, low: f64, s: &Score) {
        self.out.push_str("<div class=\"legend\">");
        let _ = writeln!(self.out, "<div class=\"legend-item\"><span class=\"dot risk-low\"></span> Low Risk ({low:.1}%)</div>");
        let _ = writeln!(self.out, "<div class=\"legend-item\"><span class=\"dot risk-mod\"></span> Moderate ({:.1}%)</div>", s.pct_moderate);
        let _ = writeln!(self.out, "<div class=\"legend-item\"><span class=\"dot risk-high\"></span> High ({:.1}%)</div>", s.pct_high);
        let _ = writeln!(self.out, "<div class=\"legend-item\"><span class=\"dot risk-vhigh\"></span> Very High ({:.1}%)</div>", s.pct_very_high);
        self.out.push_str("</div>\n");
    }

    fn render_violations(&mut self) {
        self.render_volume_table();
        self.render_complexity_table();
        self.render_interface_table();
        self.render_duplication_table();
    }

    fn render_volume_table(&mut self) {
        let meta = TableMeta {
            title: "📏 1. Unit Size Violations (> 15 LOC)",
            empty_msg: "No unit size violations detected. ✅",
            val_header: "Lines of Code",
        };
        self.render_table(meta, &filter_volume(self.res.metrics));
    }

    fn render_complexity_table(&mut self) {
        let meta = TableMeta {
            title: "🔀 2. Unit Complexity Violations (> 5 Branches)",
            empty_msg: "No unit complexity violations detected. ✅",
            val_header: "Complexity",
        };
        self.render_table(meta, &filter_complexity(self.res.metrics));
    }

    fn render_interface_table(&mut self) {
        let meta = TableMeta {
            title: "🔌 3. Unit Interface Violations (> 4 Parameters)",
            empty_msg: "No interface parameter violations detected. ✅",
            val_header: "Parameters",
        };
        self.render_table(meta, &filter_interface(self.res.metrics));
    }

    #[rustfmt::skip]
    fn render_table(&mut self, meta: TableMeta, rows: &[(&FunctionMetric, usize)]) {
        let _ = writeln!(self.out, "<div class=\"section\"><h2 class=\"section-title\">{}</h2>", meta.title);
        if rows.is_empty() { let _ = writeln!(self.out, "<p class=\"empty-msg\">{}</p></div>", meta.empty_msg); return; }
        let _ = writeln!(self.out, "<table><thead><tr><th>File</th><th>Function</th><th>Line</th><th>{}</th></tr></thead><tbody>", meta.val_header);
        for (m, val) in rows {
            let rel = format_rel_path(&m.file_path, self.root_dir);
            let _ = writeln!(self.out, "<tr><td><code>{rel}</code></td><td><code>{}</code></td><td>{}</td><td><strong>{val}</strong></td></tr>", m.function_name, m.start_line);
        }
        self.out.push_str("</tbody></table></div>\n");
    }

    #[rustfmt::skip]
    fn render_duplication_table(&mut self) {
        let dup = self.res.dup_res;
        let _ = writeln!(self.out, "<div class=\"section\"><h2 class=\"section-title\">👥 4. Code Duplication Spans ({:.1}% Total)</h2>", dup.percentage);
        if dup.blocks.is_empty() { self.out.push_str("<p class=\"empty-msg\">No duplicated code blocks detected. ✅</p></div>\n"); return; }
        self.out.push_str("<table><thead><tr><th>File</th><th>Line Span</th><th>Duplicated Lines</th></tr></thead><tbody>\n");
        for b in &dup.blocks {
            let rel = format_rel_path(&b.file_path, self.root_dir);
            let _ = writeln!(self.out, "<tr><td><code>{rel}</code></td><td>L{}-L{}</td><td><strong>{} lines</strong></td></tr>", b.start_line, b.end_line, (b.end_line - b.start_line) + 1);
        }
        self.out.push_str("</tbody></table></div>\n");
    }

    fn render_hotspots(&mut self) {
        let (hs, fr) = get_sorted_hotspots(self.res);
        self.out.push_str("<div class=\"section\"><h2 class=\"section-title\">⚡ Hotspots (Risk × Churn Matrix)</h2>\n");
        if hs.is_empty() {
            self.out.push_str("<p class=\"empty-msg\">No high-risk / high-churn hotspots detected. ✅</p></div>\n");
            return;
        }
        self.out.push_str("<table><thead><tr><th>Priority</th><th>File</th><th>Risk Points</th><th>Git Churn</th><th>Coverage</th><th>Recommendation</th></tr></thead><tbody>\n");
        self.render_hotspot_rows(&hs, &fr);
        self.out.push_str("</tbody></table></div>\n");
    }

    #[rustfmt::skip]
    fn render_hotspot_rows(&mut self, hs: &[PathBuf], fr: &HashMap<PathBuf, usize>) {
        for row in collect_hotspot_rows(hs, fr, self.res, self.root_dir) {
            let _ = writeln!(self.out, "<tr><td>#{}</td><td><code>{}</code></td><td>{}</td><td>{} commits</td><td>{}</td><td>{}</td></tr>", row.idx, row.rel_path, row.risk, row.churn, row.cov, row.rec);
        }
    }

    fn render_architecture(&mut self) {
        self.out.push_str("<div class=\"section\"><h2 class=\"section-title\">🏗️ Architecture & Component Balance</h2>\n");
        if super::is_balanced(self.res.metrics) {
            self.out.push_str("<p class=\"empty-msg\">Component Balance: All modules are balanced (< 50% codebase share each). ✅</p>\n");
        } else {
            self.out.push_str("<p style=\"color:var(--amber)\">Component Balance: ⚠️ One component exceeds 50% of the entire codebase.</p>\n");
        }
        self.render_cycles();
        self.out.push_str("</div>\n");
    }

    #[rustfmt::skip]
    fn render_cycles(&mut self) {
        let cycles = self.res.graph.detect_cycles();
        if cycles.is_empty() { self.out.push_str("<p class=\"empty-msg\">Circular Dependencies: None detected. ✅</p>\n"); return; }
        let _ = writeln!(self.out, "<p style=\"color:var(--red)\">Circular Dependencies: 🚨 {} detected!</p><ul>", cycles.len());
        for (i, c) in cycles.iter().take(5).enumerate() {
            let chain: Vec<String> = c.iter().map(|p| format!("<code>{}</code>", format_rel_path(p, self.root_dir))).collect();
            let _ = writeln!(self.out, "<li>Cycle #{}: {} &rarr; {}</li>", i + 1, chain.join(" &rarr; "), chain[0]);
        }
        self.out.push_str("</ul>\n");
    }
}

struct TableMeta<'a> {
    title: &'a str,
    empty_msg: &'a str,
    val_header: &'a str,
}

struct CardMeta<'a> {
    title: &'a str,
    val: &'a str,
    stars: &'a str,
    sub: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coverage::Coverage;
    use tempfile::tempdir;

    #[test]
    fn test_render_html_empty() {
        let dir = tempdir().unwrap();
        let graph = crate::coupling::CouplingGraph::default();
        let churns = HashMap::new();
        let dup_res = crate::duplication::DuplicationResult::default();
        let score = Score {
            stars: 7,
            code_stars: 7,
            cov_stars: Some(7),
            cov_pct: Some(100.0),
            volume_stars: 7,
            total_loc: 0,
            pct_moderate: 0.0,
            pct_high: 0.0,
            pct_very_high: 0.0,
        };
        let res = AnalysisResult {
            metrics: &[],
            churns: &churns,
            cov: &None,
            score: &score,
            dup_res: &dup_res,
            graph: &graph,
        };

        let html = render_html(&res, dir.path());
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Maintainability Report"));
        assert!(html.contains("7/7 Stars"));
    }

    #[test]
    fn test_render_html_with_data() {
        let dir = tempdir().unwrap();
        let m1 = FunctionMetric {
            file_path: dir.path().join("src/main.rs"),
            function_name: "test_fn".to_string(),
            start_line: 1,
            lines_of_code: 20,
            parameter_count: 2,
            cyclomatic_complexity: 6,
        };
        let metrics = vec![m1];
        let mut churns = HashMap::new();
        churns.insert(dir.path().join("src/main.rs"), 4);
        let mut cov_map = HashMap::new();
        cov_map.insert(
            dir.path().join("src/main.rs"),
            Coverage { hit: 8, total: 10 },
        );
        let cov = Some(cov_map);
        let dup_res = crate::duplication::DuplicationResult::default();
        let graph = crate::coupling::CouplingGraph::default();
        let score = Score {
            stars: 5,
            code_stars: 5,
            cov_stars: Some(6),
            cov_pct: Some(80.0),
            volume_stars: 7,
            total_loc: 20,
            pct_moderate: 10.0,
            pct_high: 0.0,
            pct_very_high: 0.0,
        };

        let res = AnalysisResult {
            metrics: &metrics,
            churns: &churns,
            cov: &cov,
            score: &score,
            dup_res: &dup_res,
            graph: &graph,
        };

        let html = render_html(&res, dir.path());
        assert!(html.contains("`src/main.rs`") || html.contains("src/main.rs"));
        assert!(html.contains("test_fn"));

        let report_path = generate_html_report(&res, dir.path()).unwrap();
        assert!(report_path.exists());
    }
}
