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
  --bg: #090d16;
  --surface-1: #0f172a;
  --surface-2: #1e293b;
  --surface-glass: rgba(15, 23, 42, 0.85);
  --border: rgba(255, 255, 255, 0.08);
  --border-subtle: rgba(255, 255, 255, 0.04);
  --text-main: #f8fafc;
  --text-secondary: #94a3b8;
  --text-muted: #64748b;
  --accent: #38bdf8;
  --accent-indigo: #6366f1;
  --accent-glow: rgba(56, 189, 248, 0.15);
  --green: #10b981;
  --green-glow: rgba(16, 185, 129, 0.15);
  --amber: #f59e0b;
  --amber-glow: rgba(245, 158, 11, 0.15);
  --red: #f43f5e;
  --red-glow: rgba(244, 63, 94, 0.15);
  --purple: #a855f7;
  --purple-glow: rgba(168, 85, 247, 0.15);
  --star-gold: #fbbf24;
  --radius: 12px;
  --radius-sm: 6px;
  --transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  background-color: var(--bg);
  background-image: 
    radial-gradient(at 0% 0%, rgba(99, 102, 241, 0.08) 0px, transparent 50%),
    radial-gradient(at 100% 0%, rgba(56, 189, 248, 0.08) 0px, transparent 50%);
  color: var(--text-main);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  line-height: 1.5;
  padding: 2.5rem 1.5rem;
  min-height: 100vh;
}
.container { max-width: 1200px; margin: 0 auto; }
.print-only { display: none !important; }
.screen-only { display: block; }
span.screen-only { display: inline; }
header {
  background: var(--surface-glass);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 1.75rem 2rem;
  margin-bottom: 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 1.5rem;
  box-shadow: 0 10px 30px -10px rgba(0, 0, 0, 0.5);
}
.brand-group { display: flex; flex-direction: column; gap: 0.35rem; }
.badge-row { display: flex; align-items: center; gap: 0.5rem; }
.badge {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0.25rem 0.65rem;
  border-radius: 999px;
  background: var(--accent-glow);
  color: var(--accent);
  border: 1px solid rgba(56, 189, 248, 0.3);
}
.badge-sig { background: rgba(99, 102, 241, 0.15); color: #818cf8; border-color: rgba(99, 102, 241, 0.3); }
h1 { font-size: 1.75rem; font-weight: 800; letter-spacing: -0.02em; color: #fff; }
.subtitle { color: var(--text-secondary); font-size: 0.9rem; }
.header-right { display: flex; align-items: center; gap: 1.25rem; flex-wrap: wrap; }
.btn-action {
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text-main);
  font-size: 0.82rem;
  font-weight: 600;
  padding: 0.65rem 1.1rem;
  border-radius: var(--radius-sm);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  transition: var(--transition);
}
.btn-action:hover {
  background: var(--accent-glow);
  border-color: var(--accent);
  color: var(--accent);
  transform: translateY(-1px);
}
.gauge-box {
  display: flex;
  align-items: center;
  gap: 1rem;
  background: var(--surface-2);
  padding: 0.75rem 1.25rem;
  border-radius: var(--radius);
  border: 1px solid var(--border);
}
.gauge-svg { width: 68px; height: 68px; transform: rotate(-90deg); flex-shrink: 0; }
.gauge-bg { fill: none; stroke: rgba(255, 255, 255, 0.08); stroke-width: 6; }
.gauge-fill { fill: none; stroke-width: 6; stroke-linecap: round; transition: stroke-dashoffset 1s ease-out; }
.gauge-text-group { display: flex; flex-direction: column; }
.gauge-val { font-size: 1.35rem; font-weight: 800; color: #fff; line-height: 1.1; }
.gauge-label { font-size: 0.75rem; color: var(--text-muted); font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; }

.grid-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1.25rem; margin-bottom: 2rem; }
.card {
  background: var(--surface-1);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 1.35rem;
  position: relative;
  overflow: hidden;
  transition: var(--transition);
}
.card:hover { transform: translateY(-2px); border-color: rgba(255, 255, 255, 0.15); box-shadow: 0 12px 24px -10px rgba(0,0,0,0.5); }
.card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
.card-title { font-size: 0.78rem; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.06em; font-weight: 700; }
.card-pill { font-size: 0.7rem; font-weight: 600; padding: 0.15rem 0.5rem; border-radius: 999px; }
.pill-green { background: var(--green-glow); color: var(--green); border: 1px solid rgba(16,185,129,0.3); }
.pill-amber { background: var(--amber-glow); color: var(--amber); border: 1px solid rgba(245,158,11,0.3); }
.pill-red { background: var(--red-glow); color: var(--red); border: 1px solid rgba(244,63,94,0.3); }
.card-val { font-size: 1.75rem; font-weight: 800; color: #fff; line-height: 1.1; display: flex; align-items: baseline; gap: 0.5rem; }
.stars { color: var(--star-gold); letter-spacing: 2px; font-size: 0.95rem; margin-top: 0.35rem; }
.card-sub { font-size: 0.8rem; color: var(--text-muted); margin-top: 0.35rem; }

.tabs { display: flex; gap: 0.5rem; border-bottom: 1px solid var(--border); margin-bottom: 1.75rem; overflow-x: auto; padding-bottom: 0.35rem; }
.tab-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 0.88rem;
  font-weight: 600;
  padding: 0.65rem 1.1rem;
  border-radius: var(--radius-sm);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  transition: var(--transition);
  white-space: nowrap;
}
.tab-btn:hover { color: var(--text-main); background: var(--surface-2); }
.tab-btn.active { color: #fff; background: var(--surface-2); box-shadow: inset 0 -2px 0 var(--accent); }
.tab-badge { font-size: 0.7rem; padding: 0.1rem 0.45rem; border-radius: 999px; background: rgba(255,255,255,0.08); color: var(--text-secondary); }

.tab-pane { display: none; }
.tab-pane.active { display: block; animation: fadeIn 0.25s ease-in-out; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: translateY(0); } }

.section {
  background: var(--surface-1);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 1.5rem;
  margin-bottom: 1.75rem;
}
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
.section-title { font-size: 1.15rem; font-weight: 700; color: #fff; display: flex; align-items: center; gap: 0.5rem; }

.risk-bar {
  display: flex;
  height: 14px;
  border-radius: 999px;
  overflow: hidden;
  background: var(--surface-2);
  margin: 1.25rem 0 1rem 0;
  border: 1px solid var(--border-subtle);
}
.risk-seg { transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1); }
.risk-low { background: var(--green); }
.risk-mod { background: var(--amber); }
.risk-high { background: var(--red); }
.risk-vhigh { background: var(--purple); }

.legend-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 0.85rem; margin-top: 1rem; }
.legend-card {
  background: var(--surface-2);
  padding: 0.75rem 1rem;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-subtle);
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.dot { width: 10px; height: 10px; border-radius: 50%; display: inline-block; flex-shrink: 0; }
.legend-info { display: flex; flex-direction: column; }
.legend-name { font-size: 0.8rem; font-weight: 600; color: var(--text-main); }
.legend-pct { font-size: 1rem; font-weight: 800; color: #fff; }

.table-wrap { width: 100%; overflow-x: auto; margin-top: 0.75rem; }
table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.88rem;
  text-align: left;
  table-layout: fixed;
}
th {
  background: var(--surface-2);
  color: var(--text-secondary);
  font-weight: 700;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
td {
  padding: 0.85rem 1rem;
  border-bottom: 1px solid var(--border-subtle);
  vertical-align: middle;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
tr:hover td { background: var(--surface-2); }
code {
  font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, monospace;
  font-size: 0.82rem;
  background: rgba(255, 255, 255, 0.05);
  color: #e2e8f0;
  padding: 0.2rem 0.45rem;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  max-width: 100%;
  display: inline-block;
  overflow: hidden;
  text-overflow: ellipsis;
  vertical-align: middle;
}
.th-center, .td-center, .cell-center {
  text-align: center !important;
  overflow: visible !important;
  text-overflow: clip !important;
}
.cell-num { font-family: "JetBrains Mono", monospace; font-weight: 700; color: #fff; }
.tag-crit {
  display: inline-block;
  padding: 0.2rem 0.5rem;
  font-size: 0.7rem;
  font-weight: 700;
  border-radius: 4px;
  background: var(--red-glow);
  color: var(--red);
  border: 1px solid rgba(244,63,94,0.3);
  white-space: nowrap !important;
  overflow: visible !important;
  text-overflow: clip !important;
}
.tag-warn {
  display: inline-block;
  padding: 0.2rem 0.5rem;
  font-size: 0.7rem;
  font-weight: 700;
  border-radius: 4px;
  background: var(--amber-glow);
  color: var(--amber);
  border: 1px solid rgba(245,158,11,0.3);
  white-space: nowrap !important;
  overflow: visible !important;
  text-overflow: clip !important;
}
.tag-ok {
  display: inline-block;
  padding: 0.2rem 0.5rem;
  font-size: 0.7rem;
  font-weight: 700;
  border-radius: 4px;
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(16,185,129,0.3);
  white-space: nowrap !important;
  overflow: visible !important;
  text-overflow: clip !important;
}
.empty-msg { color: var(--green); font-size: 0.9rem; padding: 1rem; background: var(--green-glow); border-radius: var(--radius-sm); border: 1px solid rgba(16,185,129,0.2); margin-top: 0.5rem; }

.expand-btn {
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--accent);
  font-size: 0.75rem;
  font-weight: 600;
  padding: 0.3rem 0.65rem;
  border-radius: var(--radius-sm);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  transition: var(--transition);
  white-space: nowrap;
}
.expand-btn:hover {
  background: var(--accent-glow);
  border-color: var(--accent);
  color: #fff;
}
.code-expand-row { display: none; }
.code-expand-row.open { display: table-row; }
.code-expand-cell {
  padding: 0.85rem 1rem 1.25rem 1rem !important;
  background: rgba(6, 10, 20, 0.98) !important;
  white-space: normal !important;
  max-width: 0;
  width: 100%;
}
.code-box {
  background: #030610;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 0.85rem 1rem;
  overflow-x: auto;
  font-family: "JetBrains Mono", monospace;
  font-size: 0.8rem;
  line-height: 1.5;
  color: #e2e8f0;
  max-height: 380px;
  box-sizing: border-box;
  max-width: 100%;
  white-space: pre;
  word-break: normal;
}
.code-box code {
  background: transparent !important;
  border: none !important;
  padding: 0 !important;
  font-size: inherit !important;
  color: inherit !important;
  display: block;
  min-width: max-content;
  white-space: pre;
}
.ln {
  color: #64748b;
  user-select: none;
  margin-right: 0.75rem;
  font-weight: 500;
}

.arch-card { display: flex; flex-direction: column; gap: 1rem; }
.arch-item { display: flex; align-items: center; justify-content: space-between; padding: 0.85rem 1rem; background: var(--surface-2); border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); }
.cycle-list { list-style: none; display: flex; flex-direction: column; gap: 0.5rem; margin-top: 0.75rem; }
.cycle-item { padding: 0.75rem 1rem; background: var(--red-glow); border: 1px solid rgba(244,63,94,0.3); border-radius: var(--radius-sm); font-size: 0.85rem; color: #fecdd3; }

footer {
  text-align: center;
  color: var(--text-muted);
  font-size: 0.82rem;
  margin-top: 3.5rem;
  border-top: 1px solid var(--border);
  padding-top: 1.75rem;
}
footer .brand { color: var(--accent); font-weight: 700; }
footer a { color: var(--text-secondary); text-decoration: none; transition: var(--transition); }
footer a:hover { color: var(--accent); text-decoration: underline; }

@media (max-width: 768px) {
  body { padding: 1.25rem 0.75rem; }
  header {
    flex-direction: column;
    align-items: stretch;
    padding: 1.25rem 1rem;
    gap: 1.25rem;
  }
  .header-right {
    flex-direction: column;
    align-items: stretch;
    width: 100%;
    gap: 1rem;
  }
  .btn-action { justify-content: center; width: 100%; }
  .gauge-box { justify-content: center; width: 100%; }
  .grid-cards { grid-template-columns: 1fr; gap: 0.85rem; }
  .tabs { gap: 0.35rem; padding-bottom: 0.5rem; -webkit-overflow-scrolling: touch; }
  .tab-btn { padding: 0.5rem 0.85rem; font-size: 0.82rem; }
  .section { padding: 1rem; }
  .legend-grid { grid-template-columns: 1fr 1fr; gap: 0.5rem; }
  table { min-width: 620px; }
  .table-wrap {
    -webkit-overflow-scrolling: touch;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }
}

@media (max-width: 480px) {
  h1 { font-size: 1.35rem; }
  .subtitle { font-size: 0.82rem; }
  .legend-grid { grid-template-columns: 1fr; }
  .card-val { font-size: 1.4rem; }
}

@media print {
  @page {
    size: A4 portrait;
    margin: 15mm 15mm 15mm 15mm;
  }
  *, *::before, *::after {
    box-sizing: border-box !important;
  }
  body {
    background: #ffffff !important;
    color: #0f172a !important;
    padding: 0 !important;
    margin: 0 !important;
    font-size: 8.5pt;
    line-height: 1.35;
    width: 100% !important;
  }
  .screen-only {
    display: none !important;
  }
  .print-only {
    display: block !important;
  }
  span.print-only {
    display: inline !important;
  }
  .container {
    max-width: 100% !important;
    width: 100% !important;
    margin: 0 !important;
    padding: 0 !important;
  }
  header {
    background: #ffffff !important;
    border: 1px solid #cbd5e1 !important;
    border-radius: 0 !important;
    box-shadow: none !important;
    padding: 1rem 1.25rem !important;
    margin-bottom: 1.25rem !important;
    display: flex !important;
    justify-content: space-between !important;
    align-items: center !important;
    page-break-inside: avoid;
    break-inside: avoid;
    page-break-after: avoid;
    break-after: avoid;
  }
  .badge, .badge-sig, .card-pill, .tag-crit, .tag-warn, .tag-ok, .cycle-item {
    border-radius: 0 !important;
  }
  .btn-action, .tabs, .expand-btn, .col-action, th.col-action, td.col-action, col.col-action {
    display: none !important;
    width: 0 !important;
    padding: 0 !important;
    margin: 0 !important;
    border: none !important;
    visibility: hidden !important;
  }
  .gauge-box {
    background: #f8fafc !important;
    border: 1px solid #cbd5e1 !important;
    border-radius: 0 !important;
    padding: 0.4rem 0.85rem !important;
  }
  .gauge-svg {
    width: 52px !important;
    height: 52px !important;
  }
  .grid-cards {
    display: grid !important;
    grid-template-columns: repeat(4, 1fr) !important;
    gap: 0.65rem !important;
    margin-bottom: 1.25rem !important;
    page-break-inside: avoid;
    break-inside: avoid;
  }
  .card {
    background: #ffffff !important;
    border: 1px solid #cbd5e1 !important;
    border-radius: 0 !important;
    box-shadow: none !important;
    padding: 0.75rem 0.85rem !important;
    page-break-inside: avoid;
    break-inside: avoid;
  }
  .card-title {
    color: #475569 !important;
    font-weight: 700 !important;
    font-size: 7pt !important;
  }
  .card-val {
    font-size: 1.2rem !important;
    font-weight: 800 !important;
    color: #0f172a !important;
  }
  .card-sub {
    color: #475569 !important;
    font-weight: 600 !important;
    font-size: 7pt !important;
  }
  .tab-pane {
    display: block !important;
    opacity: 1 !important;
    visibility: visible !important;
  }
  .print-page-break {
    page-break-before: always !important;
    break-before: page !important;
  }
  .section {
    background: #ffffff !important;
    border: 1px solid #cbd5e1 !important;
    border-radius: 0 !important;
    box-shadow: none !important;
    padding: 1rem 1.25rem !important;
    margin-bottom: 1.25rem !important;
    page-break-inside: auto !important;
    break-inside: auto !important;
  }
  .section-header {
    margin-bottom: 0.75rem !important;
    page-break-after: avoid !important;
    break-after: avoid !important;
  }
  .section-title {
    font-size: 0.95rem !important;
    font-weight: 800 !important;
    color: #0f172a !important;
  }
  .risk-bar {
    border-radius: 0 !important;
    height: 10px !important;
    margin: 0.75rem 0 0.5rem 0 !important;
  }
  .legend-grid {
    display: grid !important;
    grid-template-columns: repeat(4, 1fr) !important;
    gap: 0.5rem !important;
    margin-top: 0.75rem !important;
  }
  .legend-card, .arch-item {
    background: #f8fafc !important;
    border: 1px solid #e2e8f0 !important;
    border-radius: 0 !important;
    padding: 0.5rem 0.75rem !important;
  }
  .legend-name {
    color: #475569 !important;
    font-weight: 700 !important;
    font-size: 7.5pt !important;
  }
  .legend-pct {
    color: #0f172a !important;
    font-weight: 800 !important;
    font-size: 9pt !important;
  }
  .table-wrap {
    overflow: visible !important;
    margin-top: 0.65rem !important;
    width: 100% !important;
  }
  colgroup {
    display: none !important;
  }
  table {
    table-layout: auto !important;
    width: 100% !important;
    font-size: 8pt !important;
    border-collapse: collapse !important;
    page-break-inside: auto !important;
    break-inside: auto !important;
  }
  thead {
    display: table-header-group !important;
  }
  tfoot {
    display: table-footer-group !important;
  }
  tr {
    page-break-inside: avoid !important;
    break-inside: avoid !important;
  }
  th {
    background: #f8fafc !important;
    color: #0f172a !important;
    font-weight: 800 !important;
    padding: 0.45rem 0.6rem !important;
    border-bottom: 2px solid #cbd5e1 !important;
    font-size: 7.5pt !important;
    letter-spacing: 0.05em !important;
    overflow: visible !important;
    text-overflow: clip !important;
    white-space: nowrap !important;
  }
  td {
    padding: 0.45rem 0.6rem !important;
    border-bottom: 1px solid #e2e8f0 !important;
    color: #0f172a !important;
    font-weight: 500 !important;
    overflow: visible !important;
    text-overflow: clip !important;
    white-space: nowrap !important;
    page-break-inside: avoid !important;
    break-inside: avoid !important;
  }
  td:first-child code, td:nth-child(2) code {
    white-space: normal !important;
    word-break: break-word !important;
    max-width: none !important;
    overflow: visible !important;
    text-overflow: clip !important;
  }
  td code {
    color: #0f172a !important;
    background: #f1f5f9 !important;
    border: 1px solid #cbd5e1 !important;
    border-radius: 0 !important;
    font-weight: 600 !important;
    padding: 0.1rem 0.35rem !important;
  }
  .cell-num {
    color: #0f172a !important;
    font-weight: 700 !important;
    text-align: center !important;
  }
  .tag-crit {
    background: #fee2e2 !important;
    color: #991b1b !important;
    border: 1px solid #f87171 !important;
    font-weight: 800 !important;
    font-size: 6.8pt !important;
    padding: 0.15rem 0.45rem !important;
    white-space: nowrap !important;
    overflow: visible !important;
    text-overflow: clip !important;
    display: inline-block !important;
  }
  .tag-warn {
    background: #fef3c7 !important;
    color: #92400e !important;
    border: 1px solid #fbbf24 !important;
    font-weight: 800 !important;
    font-size: 6.8pt !important;
    padding: 0.15rem 0.45rem !important;
    white-space: nowrap !important;
    overflow: visible !important;
    text-overflow: clip !important;
    display: inline-block !important;
  }
  .tag-ok {
    background: #d1fae5 !important;
    color: #065f46 !important;
    border: 1px solid #34d399 !important;
    font-weight: 800 !important;
    font-size: 6.8pt !important;
    padding: 0.15rem 0.45rem !important;
    white-space: nowrap !important;
    overflow: visible !important;
    text-overflow: clip !important;
    display: inline-block !important;
  }
  .code-expand-row {
    display: none !important;
  }
  .code-expand-row.open {
    display: table-row !important;
  }
  .code-expand-cell {
    background: #f8fafc !important;
    border-color: #cbd5e1 !important;
    padding: 0.5rem 0.75rem !important;
  }
  .code-box {
    background: #f1f5f9 !important;
    color: #0f172a !important;
    border: 1px solid #cbd5e1 !important;
    border-radius: 0 !important;
    max-height: none !important;
    font-size: 7pt !important;
    line-height: 1.35 !important;
    padding: 0.5rem !important;
    white-space: pre-wrap !important;
    word-break: break-word !important;
  }
  .code-box code {
    white-space: pre-wrap !important;
    word-break: break-word !important;
    color: #0f172a !important;
  }
  .ln {
    color: #475569 !important;
    font-weight: 700 !important;
  }
  footer {
    display: none !important;
  }
}
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

#[must_use]
pub fn render_html(res: &AnalysisResult, root_dir: &Path) -> String {
    let mut out = String::with_capacity(32768);
    let mut ctx = HtmlCtx {
        out: &mut out,
        res,
        root_dir,
    };
    ctx.render_doc_start();
    ctx.render_header();
    ctx.render_scorecard();
    ctx.render_tabs();
    ctx.render_tab_overview();
    ctx.render_tab_violations();
    ctx.render_tab_hotspots();
    ctx.render_tab_duplication();
    ctx.render_tab_architecture();
    ctx.render_script();
    ctx.render_doc_end();
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
            .push_str("<title>SIG Maintainability Audit Dashboard</title>\n");
        let _ = writeln!(
            self.out,
            "<style>{CSS}</style>\n</head>\n<body>\n<div class=\"container\">\n"
        );
    }

    fn render_doc_end(&mut self) {
        self.out.push_str("<footer>Generated by <strong class=\"brand\">cargo-sig</strong> — Software Improvement Group Static Analyzer for Rust &bull; <a href=\"https://github.com/Vicente-Alejandro/Cargo-Software-Improvement-Group-Analyzer\" target=\"_blank\" rel=\"noopener noreferrer\">GitHub</a> &bull; <a href=\"https://crates.io/crates/cargo-sig\" target=\"_blank\" rel=\"noopener noreferrer\">Crates.io</a></footer>\n");
        self.out.push_str("</div>\n</body>\n</html>\n");
    }

    fn render_header(&mut self) {
        let s = self.res.score;
        let color = get_gauge_color(s.stars);
        let offset = compute_gauge_offset(s.stars);
        self.out
            .push_str("<header>\n<div class=\"brand-group\">\n<div class=\"badge-row\">\n");
        self.out.push_str("<span class=\"badge\">SIG Quality Model</span>\n<span class=\"badge badge-sig\"><span class=\"screen-only\">ISO 25010</span><span class=\"print-only\">ISO/IEC 25010</span></span>\n</div>\n");
        self.out.push_str("<h1 class=\"screen-only\">Maintainability Dashboard</h1>\n<p class=\"subtitle screen-only\">Static code health, cyclomatic complexity, git churn, and coverage intelligence.</p>\n<h1 class=\"print-only\">Software Maintainability Audit Report</h1>\n<p class=\"subtitle print-only\">Comprehensive evaluation of static code health, cyclomatic complexity, git churn hotspots, and test coverage.</p>\n</div>\n");
        self.out.push_str("<div class=\"header-right\">\n<button class=\"btn-action\" onclick=\"window.print()\" title=\"Export PDF or Print Report\"><svg width=\"15\" height=\"15\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M6 9V2h12v7\"></path><path d=\"M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2\"></path><rect x=\"6\" y=\"14\" width=\"12\" height=\"8\"></rect></svg><span>Export PDF / Print</span></button>\n");
        let _ = writeln!(
            self.out,
            "<div class=\"gauge-box\"><svg class=\"gauge-svg\" viewBox=\"0 0 100 100\"><circle class=\"gauge-bg\" cx=\"50\" cy=\"50\" r=\"42\"></circle><circle class=\"gauge-fill\" cx=\"50\" cy=\"50\" r=\"42\" stroke=\"{color}\" stroke-dasharray=\"263.89\" stroke-dashoffset=\"{offset:.1}\"></circle></svg><div class=\"gauge-text-group\"><span class=\"gauge-val\">{}/7</span><span class=\"gauge-label\">Overall Rating</span></div></div></div></header>\n",
            s.stars
        );
    }

    fn render_scorecard(&mut self) {
        self.out.push_str("<div class=\"grid-cards\">\n");
        self.render_card_final();
        self.render_card_health();
        self.render_card_coverage();
        self.render_card_volume();
        self.out.push_str("</div>\n");
    }

    fn render_card_final(&mut self) {
        let s = self.res.score;
        let pill = get_star_pill(s.stars);
        let _ = writeln!(
            self.out,
            "<div class=\"card\"><div class=\"card-header\"><span class=\"card-title\">Final Rating</span><span class=\"card-pill {}\">{}</span></div><div class=\"card-val\">{}/7 Stars</div><div class=\"stars\">{}</div><div class=\"card-sub\">Aggregate Maintainability</div></div>",
            pill.cls,
            pill.label,
            s.stars,
            star_string(s.stars)
        );
    }

    fn render_card_health(&mut self) {
        let s = self.res.score;
        let pill = get_star_pill(s.code_stars);
        let _ = writeln!(
            self.out,
            "<div class=\"card\"><div class=\"card-header\"><span class=\"card-title\">Code Health</span><span class=\"card-pill {}\">{}</span></div><div class=\"card-val\">{}/7 Stars</div><div class=\"stars\">{}</div><div class=\"card-sub\">Volume & Complexity</div></div>",
            pill.cls,
            pill.label,
            s.code_stars,
            star_string(s.code_stars)
        );
    }

    fn render_card_coverage(&mut self) {
        let s = self.res.score;
        if let (Some(pct), Some(st)) = (s.cov_pct, s.cov_stars) {
            let pill = get_star_pill(st);
            let _ = writeln!(
                self.out,
                "<div class=\"card\"><div class=\"card-header\"><span class=\"card-title\">Test Coverage</span><span class=\"card-pill {}\">{}</span></div><div class=\"card-val\">{pct:.1}%</div><div class=\"stars\">{}</div><div class=\"card-sub\">{st}/7 Stars Weighted</div></div>",
                pill.cls,
                pill.label,
                star_string(st)
            );
        } else if !crate::coverage::has_llvm_cov() {
            let _ = writeln!(
                self.out,
                "<div class=\"card\"><div class=\"card-header\"><span class=\"card-title\">Test Coverage</span><span class=\"card-pill pill-amber\">NOT INSTALLED</span></div><div class=\"card-val\">N/A</div><div class=\"stars\">☆☆☆☆☆☆☆</div><div class=\"card-sub\">Run 'cargo install cargo-llvm-cov'</div></div>"
            );
        } else {
            let _ = writeln!(
                self.out,
                "<div class=\"card\"><div class=\"card-header\"><span class=\"card-title\">Test Coverage</span><span class=\"card-pill pill-amber\">UNAVAILABLE</span></div><div class=\"card-val\">N/A</div><div class=\"stars\">☆☆☆☆☆☆☆</div><div class=\"card-sub\">Run 'cargo sig -a'</div></div>"
            );
        }
    }

    fn render_card_volume(&mut self) {
        let s = self.res.score;
        let pill = get_star_pill(s.volume_stars);
        let _ = writeln!(
            self.out,
            "<div class=\"card\"><div class=\"card-header\"><span class=\"card-title\">System Volume</span><span class=\"card-pill {}\">{}</span></div><div class=\"card-val\">{} LOC</div><div class=\"stars\">{}</div><div class=\"card-sub\">{}/7 Stars Scale</div></div>",
            pill.cls,
            pill.label,
            s.total_loc,
            star_string(s.volume_stars),
            s.volume_stars
        );
    }

    fn render_tabs(&mut self) {
        let (v, i, c) = super::count_violations(self.res.metrics);
        let total_v = v + i + c;
        let (hs, _) = get_sorted_hotspots(self.res);
        let _ = writeln!(
            self.out,
            "<div class=\"tabs\"><button class=\"tab-btn active\" onclick=\"showTab('tab-overview')\">📊 Overview</button><button class=\"tab-btn\" onclick=\"showTab('tab-violations')\">⚠️ Violations <span class=\"tab-badge\">{total_v}</span></button><button class=\"tab-btn\" onclick=\"showTab('tab-hotspots')\">⚡ Hotspots <span class=\"tab-badge\">{}</span></button><button class=\"tab-btn\" onclick=\"showTab('tab-duplication')\">👥 Duplication <span class=\"tab-badge\">{:.1}%</span></button><button class=\"tab-btn\" onclick=\"showTab('tab-architecture')\">🏗️ Architecture</button></div>",
            hs.len(),
            self.res.dup_res.percentage
        );
    }

    fn render_tab_overview(&mut self) {
        self.out
            .push_str("<div id=\"tab-overview\" class=\"tab-pane active\">\n");
        self.render_risk_section();
        self.out.push_str("</div>\n");
    }

    fn render_risk_section(&mut self) {
        let s = self.res.score;
        let low_pct = (100.0 - s.pct_moderate - s.pct_high - s.pct_very_high).max(0.0);
        self.out.push_str("<div class=\"section\"><div class=\"section-header\"><h2 class=\"section-title\">🎯 Risk Profile Distribution</h2></div>\n");
        let _ = writeln!(
            self.out,
            "<div class=\"risk-bar\"><div class=\"risk-seg risk-low\" style=\"width:{low_pct}%\"></div><div class=\"risk-seg risk-mod\" style=\"width:{}%\"></div><div class=\"risk-seg risk-high\" style=\"width:{}%\"></div><div class=\"risk-seg risk-vhigh\" style=\"width:{}%\"></div></div>",
            s.pct_moderate, s.pct_high, s.pct_very_high
        );
        self.render_risk_legends(low_pct, s);
        self.out.push_str("</div>\n");
    }

    fn render_risk_legends(&mut self, low: f64, s: &Score) {
        self.out.push_str("<div class=\"legend-grid\">");
        let _ = writeln!(
            self.out,
            "<div class=\"legend-card\"><span class=\"dot risk-low\"></span><div class=\"legend-info\"><span class=\"legend-name\">Low Risk</span><span class=\"legend-pct\">{low:.1}%</span></div></div>"
        );
        let _ = writeln!(
            self.out,
            "<div class=\"legend-card\"><span class=\"dot risk-mod\"></span><div class=\"legend-info\"><span class=\"legend-name\">Moderate</span><span class=\"legend-pct\">{:.1}%</span></div></div>",
            s.pct_moderate
        );
        let _ = writeln!(
            self.out,
            "<div class=\"legend-card\"><span class=\"dot risk-high\"></span><div class=\"legend-info\"><span class=\"legend-name\">High Risk</span><span class=\"legend-pct\">{:.1}%</span></div></div>",
            s.pct_high
        );
        let _ = writeln!(
            self.out,
            "<div class=\"legend-card\"><span class=\"dot risk-vhigh\"></span><div class=\"legend-info\"><span class=\"legend-name\">Very High</span><span class=\"legend-pct\">{:.1}%</span></div></div>",
            s.pct_very_high
        );
        self.out.push_str("</div>");
    }

    fn render_tab_violations(&mut self) {
        self.out
            .push_str("<div id=\"tab-violations\" class=\"tab-pane\">\n");
        self.render_volume_table();
        self.render_complexity_table();
        self.render_interface_table();
        self.out.push_str("</div>\n");
    }

    fn render_volume_table(&mut self) {
        let meta = TableMeta {
            prefix: "vol",
            title: "📏 1. Unit Size Violations (> 15 LOC)",
            empty_msg: "No unit size violations detected. ✅",
            val_header: "Lines of Code",
            tag_label: "CRITICAL",
            tag_cls: "tag-crit",
        };
        self.render_table(meta, &filter_volume(self.res.metrics));
    }

    fn render_complexity_table(&mut self) {
        let meta = TableMeta {
            prefix: "comp",
            title: "🔀 2. Unit Complexity Violations (> 5 Branches)",
            empty_msg: "No unit complexity violations detected. ✅",
            val_header: "Complexity",
            tag_label: "WARNING",
            tag_cls: "tag-warn",
        };
        self.render_table(meta, &filter_complexity(self.res.metrics));
    }

    fn render_interface_table(&mut self) {
        let meta = TableMeta {
            prefix: "int",
            title: "🔌 3. Unit Interface Violations (> 4 Parameters)",
            empty_msg: "No interface parameter violations detected. ✅",
            val_header: "Parameters",
            tag_label: "INFO",
            tag_cls: "tag-ok",
        };
        self.render_table(meta, &filter_interface(self.res.metrics));
    }

    fn render_table(&mut self, meta: TableMeta, rows: &[(&FunctionMetric, usize)]) {
        let _ = writeln!(
            self.out,
            "<div class=\"section print-page-break\"><div class=\"section-header\"><h2 class=\"section-title\">{}</h2></div>",
            meta.title
        );
        if rows.is_empty() {
            let _ = writeln!(
                self.out,
                "<p class=\"empty-msg\">{}</p></div>",
                meta.empty_msg
            );
            return;
        }
        let _ = writeln!(
            self.out,
            "<div class=\"table-wrap\"><table><colgroup><col style=\"width:25%\"><col style=\"width:23%\"><col style=\"width:9%\"><col style=\"width:15%\"><col style=\"width:14%\"><col class=\"col-action\" style=\"width:14%\"></colgroup><thead><tr><th>File</th><th>Function</th><th class=\"th-center\">Line</th><th class=\"th-center\">{}</th><th class=\"th-center\">Severity</th><th class=\"th-center col-action\">Action</th></tr></thead><tbody>",
            meta.val_header
        );
        for (i, row_data) in rows.iter().enumerate() {
            render_table_row(self.out, &meta, *row_data, i + 1, self.root_dir);
        }
        self.out.push_str("</tbody></table></div></div>\n");
    }

    fn render_tab_hotspots(&mut self) {
        let (hs, fr) = get_sorted_hotspots(self.res);
        self.out.push_str("<div id=\"tab-hotspots\" class=\"tab-pane print-page-break\">\n<div class=\"section\"><div class=\"section-header\"><h2 class=\"section-title\">⚡ Hotspots (Risk × Churn Matrix)</h2></div>\n");
        if hs.is_empty() {
            self.out.push_str("<p class=\"empty-msg\">No high-risk / high-churn hotspots detected. ✅</p></div></div>\n");
            return;
        }
        self.out.push_str("<div class=\"table-wrap\"><table><colgroup><col style=\"width:8%\"><col style=\"width:32%\"><col style=\"width:14%\"><col style=\"width:14%\"><col style=\"width:12%\"><col style=\"width:20%\"></colgroup><thead><tr><th class=\"th-center\">Rank</th><th>File</th><th class=\"th-center\">Risk Points</th><th class=\"th-center\">Git Churn</th><th class=\"th-center\">Coverage</th><th>Recommendation</th></tr></thead><tbody>\n");
        self.render_hotspot_rows(&hs, &fr);
        self.out.push_str("</tbody></table></div></div></div>\n");
    }

    fn render_hotspot_rows(&mut self, hs: &[PathBuf], fr: &HashMap<PathBuf, usize>) {
        for row in collect_hotspot_rows(hs, fr, self.res, self.root_dir) {
            let _ = writeln!(
                self.out,
                "<tr><td class=\"cell-center cell-num\">#{}</td><td><code>{}</code></td><td class=\"cell-center cell-num\">{}</td><td class=\"cell-center\">{} commits</td><td class=\"cell-center cell-num\">{}</td><td><span class=\"tag-warn\">{}</span></td></tr>",
                row.idx, row.rel_path, row.risk, row.churn, row.cov, row.rec
            );
        }
    }

    fn render_tab_duplication(&mut self) {
        let dup = self.res.dup_res;
        self.out.push_str("<div id=\"tab-duplication\" class=\"tab-pane print-page-break\">\n<div class=\"section\"><div class=\"section-header\">");
        let _ = writeln!(
            self.out,
            "<h2 class=\"section-title\">👥 Code Duplication Spans ({:.1}% Total)</h2></div>",
            dup.percentage
        );
        if dup.blocks.is_empty() {
            self.out.push_str(
                "<p class=\"empty-msg\">No duplicated code blocks detected. ✅</p></div></div>\n",
            );
            return;
        }
        self.out.push_str("<div class=\"table-wrap\"><table><colgroup><col style=\"width:45%\"><col style=\"width:18%\"><col style=\"width:20%\"><col style=\"width:17%\"></colgroup><thead><tr><th>File</th><th class=\"th-center\">Line Span</th><th class=\"th-center\">Duplicated Lines</th><th class=\"th-center\">Status</th></tr></thead><tbody>\n");
        for b in &dup.blocks {
            let rel = format_rel_path(&b.file_path, self.root_dir);
            let lines = (b.end_line - b.start_line) + 1;
            let _ = writeln!(
                self.out,
                "<tr><td><code>{rel}</code></td><td class=\"cell-center\">L{}-L{}</td><td class=\"cell-center cell-num\">{lines} lines</td><td class=\"cell-center\"><span class=\"tag-warn\">DUPLICATE</span></td></tr>",
                b.start_line, b.end_line
            );
        }
        self.out.push_str("</tbody></table></div></div></div>\n");
    }

    fn render_tab_architecture(&mut self) {
        self.out.push_str("<div id=\"tab-architecture\" class=\"tab-pane print-page-break\">\n<div class=\"section\"><div class=\"section-header\"><h2 class=\"section-title\">🏗️ Architecture & Component Balance</h2></div><div class=\"arch-card\">\n");
        if super::is_balanced(self.res.metrics) {
            self.out.push_str("<div class=\"arch-item\"><span>Component Balance</span><span class=\"tag-ok\">BALANCED (&lt; 50% Share Each) ✅</span></div>\n");
        } else {
            self.out.push_str("<div class=\"arch-item\"><span>Component Balance</span><span class=\"tag-warn\">UNBALANCED (&gt; 50% Single Component) ⚠️</span></div>\n");
        }
        self.render_cycles();
        self.out.push_str("</div></div></div>\n");
    }

    fn render_cycles(&mut self) {
        let cycles = self.res.graph.detect_cycles();
        if cycles.is_empty() {
            self.out.push_str("<div class=\"arch-item\"><span>Circular Dependencies</span><span class=\"tag-ok\">NONE DETECTED ✅</span></div>\n");
            return;
        }
        let _ = writeln!(
            self.out,
            "<div class=\"arch-item\"><span>Circular Dependencies</span><span class=\"tag-crit\">{} DETECTED 🚨</span></div><ul class=\"cycle-list\">",
            cycles.len()
        );
        for (i, c) in cycles.iter().take(5).enumerate() {
            let chain: Vec<String> = c
                .iter()
                .map(|p| format!("<code>{}</code>", format_rel_path(p, self.root_dir)))
                .collect();
            let _ = writeln!(
                self.out,
                "<li class=\"cycle-item\">Cycle #{}: {} &rarr; {}</li>",
                i + 1,
                chain.join(" &rarr; "),
                chain[0]
            );
        }
        self.out.push_str("</ul>\n");
    }

    fn render_script(&mut self) {
        self.out.push_str(r"<script>
function showTab(tabId) {
  document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
  document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
  const target = document.getElementById(tabId);
  if (target) target.classList.add('active');
  const clicked = Array.from(document.querySelectorAll('.tab-btn')).find(b => {
    const attr = b.getAttribute('onclick');
    return attr && attr.includes(tabId);
  });
  if (clicked) clicked.classList.add('active');
}
function toggleCode(btn, rowId) {
  const target = document.getElementById(rowId);
  if (!target) return;
  const isAlreadyOpen = target.classList.contains('open');
  document.querySelectorAll('.code-expand-row.open').forEach(r => r.classList.remove('open'));
  document.querySelectorAll('.expand-btn').forEach(b => { b.innerHTML = '🔍 View Code'; });
  if (!isAlreadyOpen) {
    target.classList.add('open');
    btn.innerHTML = '▲ Hide Code';
  }
}
</script>
");
    }
}

fn render_table_row(
    out: &mut String,
    meta: &TableMeta,
    (m, val): (&FunctionMetric, usize),
    idx: usize,
    root: &Path,
) {
    let rel = format_rel_path(&m.file_path, root);
    let row_id = format!("{}-code-{idx}", meta.prefix);
    let source = read_function_source(&m.file_path, m.start_line, m.lines_of_code);
    let _ = writeln!(
        out,
        "<tr><td><code>{rel}</code></td><td><code>{}</code></td><td class=\"cell-center\">L{}</td><td class=\"cell-center cell-num\">{val}</td><td class=\"cell-center\"><span class=\"{}\">{}</span></td><td class=\"cell-center col-action\"><button class=\"expand-btn\" onclick=\"toggleCode(this, '{row_id}')\">🔍 View Code</button></td></tr>",
        m.function_name, m.start_line, meta.tag_cls, meta.tag_label
    );
    let _ = writeln!(
        out,
        "<tr id=\"{row_id}\" class=\"code-expand-row\"><td colspan=\"6\" class=\"code-expand-cell\"><pre class=\"code-box\"><code>{source}</code></pre></td></tr>"
    );
}

fn read_function_source(file_path: &Path, start_line: usize, loc: usize) -> String {
    if start_line == 0 || loc == 0 {
        return String::from("<em>Source line data unavailable</em>");
    }
    let Ok(content) = fs::read_to_string(file_path) else {
        return String::from("<em>Source file unavailable</em>");
    };
    format_source_lines(&content, start_line, loc)
}

fn format_source_lines(content: &str, start_line: usize, loc: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start_idx = start_line.saturating_sub(1);
    let end_idx = (start_idx + loc).min(lines.len());
    if start_idx >= lines.len() {
        return String::from("<em>Source range out of bounds</em>");
    }
    let mut code_block = String::with_capacity(loc * 40);
    for (i, line) in lines[start_idx..end_idx].iter().enumerate() {
        let line_no = start_line + i;
        let escaped = escape_html_str(line);
        let _ = writeln!(
            code_block,
            "<span class=\"ln\">{line_no:>4} |</span> {escaped}"
        );
    }
    code_block
}

fn escape_html_str(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() + 10);
    for c in s.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn compute_gauge_offset(stars: u8) -> f64 {
    let circumference = 263.89;
    let frac = f64::from(stars.min(7)) / 7.0;
    circumference * (1.0 - frac)
}

fn get_gauge_color(stars: u8) -> &'static str {
    match stars {
        6..=7 => "#10b981",
        4..=5 => "#f59e0b",
        _ => "#f43f5e",
    }
}

struct StarPill {
    cls: &'static str,
    label: &'static str,
}

fn get_star_pill(stars: u8) -> StarPill {
    match stars {
        6..=7 => StarPill {
            cls: "pill-green",
            label: "EXCELLENT",
        },
        4..=5 => StarPill {
            cls: "pill-amber",
            label: "MODERATE",
        },
        _ => StarPill {
            cls: "pill-red",
            label: "CRITICAL",
        },
    }
}

struct TableMeta<'a> {
    prefix: &'a str,
    title: &'a str,
    empty_msg: &'a str,
    val_header: &'a str,
    tag_label: &'a str,
    tag_cls: &'a str,
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
        assert!(html.contains("Maintainability Dashboard"));
        assert!(html.contains("Software Maintainability Audit Report"));
        assert!(html.contains("7/7"));
        assert!(html.contains("showTab"));
        assert!(html.contains("toggleCode"));
        assert!(html.contains("Export PDF / Print"));
    }

    #[test]
    fn test_render_html_with_data() {
        let dir = tempdir().unwrap();
        let main_file = dir.path().join("src/main.rs");
        fs::create_dir_all(main_file.parent().unwrap()).unwrap();
        fs::write(
            &main_file,
            "fn test_fn() {\n    println!(\"hello\");\n}\n",
        )
        .unwrap();

        let m1 = FunctionMetric {
            file_path: main_file,
            function_name: "test_fn".to_string(),
            start_line: 1,
            lines_of_code: 3,
            parameter_count: 5,
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
        let dup_res = crate::duplication::DuplicationResult {
            percentage: 3.5,
            blocks: vec![crate::duplication::DuplicationBlock {
                file_path: dir.path().join("src/main.rs"),
                start_line: 1,
                end_line: 3,
            }],
        };
        let mut edges = HashMap::new();
        let mut neighbors = std::collections::HashSet::new();
        neighbors.insert(dir.path().join("src/lib.rs"));
        edges.insert(dir.path().join("src/main.rs"), neighbors);
        let graph = crate::coupling::CouplingGraph {
            edges,
            ignored_externals: 0,
        };
        let score = Score {
            stars: 4,
            code_stars: 4,
            cov_stars: Some(5),
            cov_pct: Some(80.0),
            volume_stars: 6,
            total_loc: 120,
            pct_moderate: 15.0,
            pct_high: 5.0,
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
        assert!(html.contains("src/main.rs"));
        assert!(html.contains("test_fn"));
        assert!(html.contains("WARNING"));
        assert!(html.contains("View Code"));
        assert!(html.contains("hello"));

        let report_path = generate_html_report(&res, dir.path()).unwrap();
        assert!(report_path.exists());
    }

    #[test]
    fn test_compute_gauge_offset_and_color() {
        assert!(compute_gauge_offset(7) < 1.0);
        assert_eq!(get_gauge_color(7), "#10b981");
        assert_eq!(get_gauge_color(4), "#f59e0b");
        assert_eq!(get_gauge_color(2), "#f43f5e");
    }

    #[test]
    fn test_read_function_source() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("foo.rs");
        fs::write(&f, "fn foo() {\n    let x = 1 < 2;\n}\n").unwrap();
        let src = read_function_source(&f, 1, 3);
        assert!(src.contains("1 &lt; 2"));
        assert!(src.contains("1 |"));

        let empty = read_function_source(&f, 0, 0);
        assert!(empty.contains("unavailable"));
    }
}
