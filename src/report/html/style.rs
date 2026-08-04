pub const CSS: &str = r#"
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
.card-badges { display: flex; align-items: center; gap: 0.35rem; }
.card-pill { font-size: 0.7rem; font-weight: 600; padding: 0.15rem 0.5rem; border-radius: 999px; }
.badge-delta { font-size: 0.68rem; font-weight: 700; padding: 0.15rem 0.45rem; border-radius: 999px; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }
.delta-pos { background: var(--green-glow); color: var(--green); border: 1px solid rgba(16,185,129,0.3); }
.delta-neg { background: var(--red-glow); color: var(--red); border: 1px solid rgba(244,63,94,0.3); }
.delta-same { background: rgba(255,255,255,0.06); color: var(--text-muted); border: 1px solid var(--border); }
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
    -webkit-print-color-adjust: exact !important;
    print-color-adjust: exact !important;
    color-adjust: exact !important;
  }
  body {
    background: #ffffff !important;
    color: #0f172a !important;
    padding: 0 !important;
    margin: 0 !important;
    font-size: 8.5pt;
    line-height: 1.35;
    width: 100% !important;
    -webkit-print-color-adjust: exact !important;
    print-color-adjust: exact !important;
    color-adjust: exact !important;
  }
  h1, h2, h3, h4, h5, h6 {
    color: #0f172a !important;
  }
  h1 {
    font-size: 1.5rem !important;
    font-weight: 800 !important;
    color: #0f172a !important;
    margin: 0.25rem 0 !important;
  }
  .subtitle {
    color: #475569 !important;
    font-size: 8pt !important;
    margin: 0 !important;
  }
  .gauge-val {
    color: #0f172a !important;
    font-size: 1.25rem !important;
    font-weight: 800 !important;
  }
  .gauge-label {
    color: #475569 !important;
    font-size: 6.5pt !important;
    font-weight: 700 !important;
  }
  .gauge-bg {
    stroke: #e2e8f0 !important;
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
  .badge {
    background: #e0f2fe !important;
    color: #0369a1 !important;
    border: 1px solid #7dd3fc !important;
    border-radius: 0 !important;
  }
  .badge-sig {
    background: #e0e7ff !important;
    color: #4338ca !important;
    border: 1px solid #a5b4fc !important;
    border-radius: 0 !important;
  }
  .pill-green {
    background: #dcfce7 !important;
    color: #15803d !important;
    border: 1px solid #86efac !important;
  }
  .pill-amber {
    background: #fef3c7 !important;
    color: #b45309 !important;
    border: 1px solid #fde68a !important;
  }
  .pill-red {
    background: #fee2e2 !important;
    color: #b91c1c !important;
    border: 1px solid #fca5a5 !important;
  }
  .card-pill, .tag-crit, .tag-warn, .tag-ok, .cycle-item {
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
  .badge-delta {
    border: 1px solid #cbd5e1 !important;
    background: #f1f5f9 !important;
    color: #0f172a !important;
    font-size: 6.5pt !important;
    font-weight: 700 !important;
    padding: 0.1rem 0.35rem !important;
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

pub const JS_SCRIPT: &str = r"<script>
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
  document.querySelectorAll('.expand-btn').forEach(b => { b.textContent = 'View Code'; });
  if (!isAlreadyOpen) {
    target.classList.add('open');
    btn.textContent = 'Hide Code';
  }
}
</script>
";
