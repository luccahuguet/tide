// Regenerates the paper's figures from committed records. Run from the repository root:
//   node paper/figures/build.mjs           writes data/, excerpts/ and svg/ under paper/figures/
//   node paper/figures/build.mjs --check   regenerates in memory and exits 1 if any committed
//                                           file differs, is missing, or is no longer generated
// Inputs: the records named below, records/ (tool output captured at a named commit), spec/
// (hand-transcribed inputs whose quotes are checked against their sources) and
// data/reference-sections.json (written by count-reference.mjs). See README.md.
import { readFileSync, writeFileSync, readdirSync, existsSync, mkdirSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { scaleLinear, scaleLog } from "d3-scale";

const FIG = "paper/figures";
const read = (p) => readFileSync(p, "utf8");
const json = (p) => JSON.parse(read(p));
const out = new Map();
const emit = (path, text) => out.set(`${FIG}/${path}`, text);
const emitJSON = (path, value) => emit(path, JSON.stringify(value, null, 2) + "\n");
function fail(msg) { throw new Error(msg); }
function need(cond, msg) { if (!cond) fail(msg); }
function grab(s, re, what) { const m = s.match(re); need(m, `${what}: nothing matches ${re}`); return m; }
function quoted(file, quote) { need(read(file).includes(quote), `${file} no longer contains: ${quote}`); }

// ---- Style: one palette and two typefaces for every figure ------------------
// Okabe-Ito colours (colourblind-safe) with distinct grey levels; every mark also carries a
// label, so no figure depends on colour. Typst embeds both typefaces in the PDF.
const C = { ink: "#222222", muted: "#666666", rule: "#b8b8b8", faint: "#ececec",
  blue: "#0072B2", orange: "#E69F00", verm: "#D55E00", sky: "#9fd3f0" };
const SERIF = "Libertinus Serif", MONO = "DejaVu Sans Mono";
const W = 459; // the paper's text width in points; Typst places each SVG at width 100%

const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const num = (v) => (Number.isInteger(v) ? String(v) : String(Math.round(v * 100) / 100));
const attrs = (a) => Object.entries(a).filter(([, v]) => v !== undefined && v !== null)
  .map(([k, v]) => ` ${k}="${typeof v === "number" ? num(v) : esc(v)}"`).join("");
const el = (tag, a = {}, body) => (body === undefined ? `<${tag}${attrs(a)}/>` : `<${tag}${attrs(a)}>${body}</${tag}>`);
const thousands = (n) => String(n).replace(/\B(?=(\d{3})+(?!\d))/g, ",");

// Advance widths, in em, close to Libertinus Serif's; layout only needs them to be generous.
function charWidth(ch) {
  if (" ".includes(ch)) return 0.25;
  if ("iljtfr.,:;'!|()[]".includes(ch)) return 0.3;
  if ("mwMW".includes(ch)) return 0.8;
  if (/[A-Z]/.test(ch)) return 0.66;
  if (/[0-9]/.test(ch)) return 0.5;
  return 0.49;
}
// Text with `code` spans: widths and tspans use the monospace face inside backticks.
function spans(text) { return text.split("`").map((t, i) => ({ t, mono: i % 2 === 1 })).filter((s) => s.t); }
function width(text, size) {
  return spans(text).reduce((w, s) => w + (s.mono ? 0.6 * size * s.t.length : [...s.t].reduce((a, c) => a + charWidth(c) * size, 0)), 0);
}
function wrap(text, max, size) {
  const lines = [];
  let line = "";
  for (const word of text.split(" ")) {
    const next = line ? `${line} ${word}` : word;
    if (line && width(next, size) > max) { lines.push(line); line = word; } else line = next;
  }
  if (line) lines.push(line);
  // Keep each line's code spans balanced.
  for (let i = 0; i < lines.length; i++) {
    if ((lines[i].match(/`/g) || []).length % 2 === 1) { lines[i] += "`"; if (i + 1 < lines.length) lines[i + 1] = "`" + lines[i + 1]; }
  }
  return lines;
}
function text(x, y, s, o = {}) {
  const size = o.size ?? 8;
  const body = spans(s).map((p) => (p.mono ? el("tspan", { "font-family": MONO, "font-size": size * 0.88 }, esc(p.t)) : esc(p.t))).join("");
  return el("text", { x, y, "font-size": size, "text-anchor": o.anchor, "font-weight": o.bold ? "bold" : undefined,
    "font-style": o.italic ? "italic" : undefined, fill: o.fill ?? C.ink }, body);
}
function lines(x, y, ls, o = {}) { const lead = (o.size ?? 8) * 1.25; return ls.map((l, i) => text(x, y + i * lead, l, o)).join(""); }
const rect = (x, y, w, h, o = {}) => el("rect", { x, y, width: w, height: h, rx: o.rx, fill: o.fill ?? "none", stroke: o.stroke, "stroke-width": o.sw, "stroke-dasharray": o.dash });
const line = (x1, y1, x2, y2, o = {}) => el("line", { x1, y1, x2, y2, stroke: o.stroke ?? C.ink, "stroke-width": o.sw ?? 0.6, "stroke-dasharray": o.dash, "marker-end": o.arrow ? "url(#arrow)" : undefined });
const defs = el("defs", {}, el("marker", { id: "arrow", viewBox: "0 0 10 10", refX: 9, refY: 5, markerWidth: 6, markerHeight: 6, orient: "auto" }, el("path", { d: "M0,1 L9,5 L0,9 z", fill: C.ink })));
function svg(name, h, body) {
  emit(`svg/${name}.svg`, `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${num(h)}" viewBox="0 0 ${W} ${num(h)}" font-family="${SERIF}">${defs}${body}</svg>\n`);
}
function axis(x, y, lo, hi, ticks, unit, o = {}) {
  return line(x(lo), y, x(hi), y) + ticks.map((t) => line(x(t), y, x(t), y + 3) + text(x(t), y + 11, `${t}${unit}`, { size: 7, anchor: "middle", fill: C.muted })).join("");
}
function box(x, y, w, h, title, body, o = {}) {
  const tl = wrap(title, w - 10, 8.5), bl = wrap(body, w - 10, 7.5);
  need(12 + tl.length * 10.6 + bl.length * 9.4 <= h + 2, `box "${title}" overflows`);
  return rect(x, y, w, h, { stroke: o.stroke ?? C.ink, sw: 0.6, rx: 2, fill: o.fill ?? "white" })
    + lines(x + 5, y + 12, tl, { size: 8.5, bold: true }) + lines(x + 5, y + 13 + tl.length * 10.6, bl, { size: 7.5 });
}

// ---- Records: tool output at a named commit ---------------------------------
// Each records/*.txt file starts with `# ` header lines (commit, command, normalization), then a
// blank line, then the output.
function record(name) {
  const t = read(`${FIG}/records/${name}`);
  const i = t.indexOf("\n\n");
  need(t.startsWith("# ") && i > 0, `${name}: missing header`);
  return t.slice(i + 2);
}
function section(textBody, heading) {
  const ls = textBody.split("\n"), i = ls.findIndex((l) => l.startsWith(`== ${heading}`));
  need(i >= 0, `demo record has no step "${heading}"`);
  const rest = ls.slice(i + 1), j = rest.findIndex((l) => l.startsWith("== "));
  return rest.slice(0, j < 0 ? undefined : j).filter((l, k, a) => !(l === "" && k === a.length - 1));
}

// ---- Code and output excerpts (Typst sets them as raw blocks) ---------------
function fnExcerpt(file, sig) {
  const ls = read(file).split("\n"), i = ls.findIndex((l) => l.startsWith(sig));
  need(i >= 0, `${file}: no line starting with ${sig}`);
  const j = ls.findIndex((l, k) => k > i && l === "}");
  return ls.slice(i, j + 1).join("\n") + "\n";
}
emit("excerpts/code-rust.rs", fnExcerpt("examples/capabilities/ambient_read.rs", "pub fn load("));
emit("excerpts/code-tide.rs", fnExcerpt("examples/capabilities/config.rs", "pub fn load("));
const admitRejected = record("admit-ambient_read.txt");
need(/TD0501/.test(admitRejected) && /^exit 1$/m.test(admitRejected), "admission record: expected TD0501 and exit 1");
need(/^exit 0$/m.test(record("admit-config.txt")), "admission record of config.rs: expected exit 0");
emit("excerpts/diagnostic.txt", admitRejected.split("\n").filter((l) => l && !l.startsWith("exit ")).join("\n") + "\n");
const demo = record("demo-run.txt");
const diff = section(demo, "4.");
need(diff.some((l) => /hits::serve \(function\) widened/.test(l)) && diff.includes("      + metrics: net"), "demo record: trust diff lacks the widened serve");
emit("excerpts/trust-diff.txt", diff.join("\n") + "\n");

// The manifest: a subset of its keys; hashes cut to 16 hex digits, the dependency list and the
// trace (40 events) elided, each cut marked with `…`, and a final `"…"` entry in each object
// naming the recorded keys it leaves out.
const manifest = json(`${FIG}/records/demo-seed0.manifest.json`);
need(manifest.format === "tide-replay/2" && manifest.seed === "0", "manifest: not the seed-0 tide-replay/2 recording");
const cut = (s) => s.replace(/[0-9a-f]{64}/g, (h) => `${h.slice(0, 16)}…`);
const shown = {
  format: manifest.format, test: manifest.test, seed: manifest.seed,
  build: { source: manifest.build.source, library: manifest.build.library,
    dependencies: `… ${manifest.build.dependencies.length} entries: each library file and its SHA-256`,
    compiler: manifest.build.compiler.split(";")[0] + ";…", profile: manifest.build.profile, driver: manifest.build.driver },
  roots: manifest.roots, injected: manifest.injected,
  outcome: manifest.outcome.split("\\n")[0] + "…",
  trace: `… ${manifest.trace.trim().split("\n").length} events, one JSON object each`,
  trace_hash: manifest.trace_hash, input_hash: manifest.input_hash,
};
for (const [all, kept] of [[manifest.build, shown.build], [manifest, shown]]) {
  const left = Object.keys(all).filter((k) => !(k in kept));
  if (left.length) kept["…"] = `also recorded, not shown: ${left.join(", ")}`;
}
emit("excerpts/manifest.txt", cut(JSON.stringify(shown, null, 2)).replace(/"…/g, "\"…").replace(/\[\n\s*\{\n\s*"ordinal": (\d+),\n\s*"seed": "(\d+)",\n\s*"inputs": \[\]\n\s*\}\n\s*\]/, '[{ "ordinal": $1, "seed": "$2", "inputs": [] }]') + "\n");

// ---- Figure: replay ------------------------------------------------------------
{
  const search = section(demo, "1.").join("\n"), fixed = section(demo, "3.").join("\n"), rec = section(demo, "2.");
  const m = grab(search, /(\d+) passed; (\d+) failed \((\d+) seeds\)/, "demo record, step 1"), f = grab(fixed, /(\d+) passed; (\d+) failed \((\d+) seeds\)/, "demo record, step 3");
  const failing = grab(search, /failing seeds: ([\d ]+) \(/, "demo record, step 1")[1].split(" ").map(Number);
  const seeds = Number(m[3]);
  need(failing.length === Number(m[2]) && Number(f[2]) === 0 && Number(f[3]) === seeds, "demo record: seed counts disagree");
  const runs = rec.map((l) => l.match(/^(recording|replay-\d)\s+(\d+)\s+([0-9a-f]+)\s+([0-9a-f]+)$/)).filter(Boolean)
    .map((r) => ({ run: r[1], events: Number(r[2]), event_trace: r[3], output_trace: r[4] }));
  need(runs.length === 3 && runs.every((r) => r.event_trace === runs[0].event_trace && r.output_trace === runs[0].output_trace), "demo record: replays differ");
  const data = { source: `${FIG}/records/demo-run.txt`, seeds, buggy_failing: failing, fixed_failing: [], recorded_seed: failing[0], runs };
  emitJSON("data/replay.json", data);
  const x0 = 92, cw = 19, cell = 15;
  let b = text(x0, 12, "Seed", { size: 7.5, fill: C.muted });
  for (let s = 0; s < seeds; s++) b += text(x0 + s * cw + cell / 2, 24, String(s), { size: 7.5, anchor: "middle" });
  [["Buggy revision", failing], ["Fixed revision", []]].forEach(([label, fails], r) => {
    const y = 30 + r * 22;
    b += text(x0 - 6, y + 11, label, { size: 8, anchor: "end" });
    for (let s = 0; s < seeds; s++) {
      const bad = fails.includes(s);
      b += rect(x0 + s * cw, y, cell, cell, { fill: bad ? C.verm : "white", stroke: bad ? C.verm : C.muted, sw: 0.7 });
      // Drawn marks, not glyphs, so no fallback font is needed: a cross for fail, a tick for pass.
      const cx = x0 + s * cw;
      b += bad ? el("path", { d: `M${cx + 4},${y + 4} L${cx + 11},${y + 11} M${cx + 11},${y + 4} L${cx + 4},${y + 11}`, stroke: "white", "stroke-width": 1.6, fill: "none" })
        : el("path", { d: `M${cx + 3.5},${y + 8} L${cx + 6.5},${y + 11} L${cx + 11.5},${y + 4.5}`, stroke: C.muted, "stroke-width": 1.2, fill: "none" });
    }
    b += text(x0 + seeds * cw + 2, y + 11, `${fails.length} of ${seeds} fail`, { size: 8, bold: r === 0 });
  });
  const y = 92;
  b += text(0, y, `Seed ${data.recorded_seed}: the recording and its two replays`, { size: 8.5, bold: true });
  const cols = [0, 92, 150, 260];
  ["Run", "Events", "Event-trace hash", "Output-trace hash"].forEach((h, i) => (b += text(cols[i], y + 14, h, { size: 7.5, fill: C.muted })));
  runs.forEach((r, i) => {
    const yy = y + 27 + i * 12;
    b += text(cols[0], yy, r.run, { size: 8 }) + text(cols[1], yy, String(r.events), { size: 8 })
      + text(cols[2], yy, `\`${r.event_trace}\``, { size: 8 }) + text(cols[3], yy, `\`${r.output_trace}\``, { size: 8 });
  });
  b += text(cols[3] + 112, y + 39, "identical", { size: 8, italic: true, fill: C.blue });
  svg("replay", y + 58, b);
}

// ---- Figure: reference budget ---------------------------------------------------
{
  const counts = json(`${FIG}/data/reference-sections.json`);
  const blob = execFileSync("git", ["hash-object", counts.source], { encoding: "utf8" }).trim();
  need(blob === counts.blob, `${counts.source} changed since its sections were counted: rerun node ${FIG}/count-reference.mjs`);
  const companion = grab(read("design/reference-companion.md"), /Core count: ([\d,]+) tokens/, "design/reference-companion.md");
  const budget = Number(grab(read(counts.source), /Budget: ([\d,]+) tokens/, counts.source)[1].replace(/,/g, ""));
  const whole = Number(companion[1].replace(/,/g, ""));
  const sum = counts.sections.reduce((a, s) => a + s.tokens, 0);
  emitJSON("data/budget.json", { budget, whole, whole_source: "design/reference-companion.md (Core count)", sections_sum: sum, sections: counts.sections });
  const label = (s) => s.heading.replace(/^(\d+)\. /, "§$1 ");
  const L = Math.ceil(Math.max(...counts.sections.map((s) => width(label(s), 7.5)))) + 8;
  const x = scaleLinear().domain([0, budget]).range([L, W - 40]);
  let b = text(0, 12, "Whole reference", { size: 8.5, bold: true });
  b += rect(x(0), 4, x(budget) - x(0), 12, { fill: C.faint });
  b += rect(x(0), 4, x(whole) - x(0), 12, { fill: C.blue });
  b += line(x(budget), 0, x(budget), 20, { sw: 1.2 }) + text(x(budget) + 3, 9, "budget", { size: 7, fill: C.muted }) + text(x(budget) + 3, 18, thousands(budget), { size: 7, fill: C.muted });
  b += text(x(0) + 4, 13.5, `${thousands(whole)} tokens`, { size: 8, fill: "white", bold: true });
  const top = 36, rh = 11.5, sx = scaleLinear().domain([0, Math.max(...counts.sections.map((s) => s.tokens))]).range([0, W - L - 40]);
  b += text(0, top - 4, "By section, each counted alone", { size: 8.5, bold: true });
  counts.sections.forEach((s, i) => {
    const yy = top + 4 + i * rh;
    b += text(L - 6, yy + 8, label(s), { size: 7.5, anchor: "end" });
    b += rect(L, yy + 1, sx(s.tokens), 9, { fill: C.sky });
    b += text(L + sx(s.tokens) + 3, yy + 8, thousands(s.tokens), { size: 7.5 });
  });
  const yEnd = top + 4 + counts.sections.length * rh + 10;
  b += text(0, yEnd, `The sections, counted alone, sum to ${thousands(sum)}; the whole file counts ${thousands(whole)}. Counted with scripts/count-tokens.sh on ${counts.counted}.`, { size: 7, fill: C.muted });
  svg("budget", yEnd + 4, b);
}

// ---- Figure: red teams -------------------------------------------------------------
// Every route verdict, finding heading and classification line must be read by a rule, or the
// build fails: a reworded record cannot silently count as zero.
const VERDICTS = {
  held: ["rejected", "seccomp denies", "Landlock denies", "denied", "`ENOSYS`", "the sandbox covers constructors", "the loader starts but cannot read the shell"],
  admitted: ["admitted", "admitted (SINK-1)", "admitted, inert (the driver's `--crate-type` wins)"],
};
const ITEM = /^### [A-Z]\d+\.|^\*\*Classification|^- \*\*[^*]*\b[A-Z]\d+\.\s/;
function deriveRedteams(spec, rd) {
  // A held-attacks section is a table (data rows, less the header) or, with `item`, a list
  // (lines matching it); a section with any other line shape fails the build.
  const heldRows = ({ record: f, heading, item }) => {
    const ls = rd(f).split("\n"), i = ls.indexOf(heading);
    need(i >= 0, `${f}: no "${heading}"`);
    const rows = [];
    for (const l of ls.slice(i + 1)) {
      if (l.startsWith("## ")) break;
      if (item) { if (new RegExp(item).test(l)) rows.push(l); else need(l === "" || !/^\s*[-*|]/.test(l), `${f}: a held item not read by ${item}: ${l}`); }
      else if (l.startsWith("| ") && !l.startsWith("| ---")) rows.push(l);
    }
    need(rows.length > 0, `${f}: no held attacks under "${heading}"`);
    return item ? rows.length : rows.length - 1;
  };
  return spec.teams.map((t) => {
    let held = 0, blocking = 0, nonblocking = 0, open = 0, controls = 0;
    if (t.routes) {
      const r = rd(t.routes.record), total = Number(grab(r, new RegExp(t.routes.total, "m"), t.routes.record)[1]);
      const rows = r.split("\n").filter((l) => /^\| \d+[a-z]? \|/.test(l)).map((l) => l.split("|").map((c) => c.trim()));
      for (const c of rows) need(VERDICTS.held.includes(c[3]) || VERDICTS.admitted.includes(c[3]), `${t.routes.record}: unclassified route verdict: ${c[3]}`);
      const ctrl = rows.filter((c) => c[2].startsWith("controls"));
      blocking = rows.filter((c) => VERDICTS.held.includes(c[3]) && c[4].startsWith("**admitted at")).length;
      // The record's form: the routes found admitted, then fixed, are marked "**admitted at <commit>**".
      need(blocking > 0 && blocking === rows.filter((c) => /admitted at/i.test(c[4])).length, `${t.routes.record}: the admitted-then-fixed routes no longer read "**admitted at"`);
      open = rows.filter((c) => VERDICTS.admitted.includes(c[3]) && !c[2].startsWith("controls")).length;
      controls = total - (rows.length - ctrl.length);
      held = total - controls - blocking - open;
      need(ctrl.length === 1 && held > 0 && held === rows.filter((c) => VERDICTS.held.includes(c[3])).length - blocking, "route table changed shape");
      nonblocking = open;
    } else {
      held = heldRows(t.held);
      const sels = [...(t.finding ?? []), ...(t.nonblocking ?? [])];
      for (const f of t.finding ?? []) {
        let n = 0;
        for (const l of rd(f.record).split("\n")) {
          const m = l.match(new RegExp(f.line)); if (!m) continue;
          n++;
          if (/non-blocking|\bnit\b/.test(m[1])) nonblocking++; else if (/blocking/.test(m[1])) blocking++; else fail(`${f.record}: unclassified finding: ${l}`);
        }
        need(n > 0, `${f.record}: ${f.line} matches nothing`);
      }
      for (const f of t.nonblocking ?? []) {
        const n = rd(f.record).split("\n").filter((l) => new RegExp(f.line).test(l)).length;
        need(n > 0, `${f.record}: ${f.line} matches nothing`);
        nonblocking += n;
      }
      for (const rec of new Set(sels.map((f) => f.record))) for (const l of rd(rec).split("\n"))
        if (ITEM.test(l)) need(sels.some((f) => f.record === rec && new RegExp(f.line).test(l)), `${rec}: no rule reads this line: ${l}`);
    }
    for (const q of [t.fixed ?? []].flat()) need(rd(q.record).includes(q.quote), `${q.record} no longer contains: ${q.quote}`);
    need(blocking === 0 || t.fixed, `${t.name}: blocking findings with no fix record`);
    return { name: t.name, when: t.when, routes: Boolean(t.routes), held, blocking, blocking_fixed: blocking, nonblocking, controls };
  });
}
{
  const spec = json(`${FIG}/spec/redteams.json`);
  const teams = deriveRedteams(spec, read);
  // Falsifier, run on every build: a reworded finding line must fail the build, not count as zero.
  const reworded = (f) => (f === "reviews/fx-ff13-redteam-codex.md" ? read(f).replace(/^\*\*Classification: (.*)\*\*$/gm, "**Classification:** $1") : read(f));
  let threw = false;
  try { deriveRedteams(spec, reworded); } catch { threw = true; }
  need(threw, "falsifier: a reworded red-team record did not fail the build");
  emitJSON("data/redteams.json", { source: `${FIG}/spec/redteams.json`, teams });
  const x = scaleLinear().domain([0, Math.max(...teams.map((t) => t.held + t.blocking + t.nonblocking))]).range([0, W - 150 - 120]);
  const kinds = [["held", "Attacks or routes that held", C.faint, C.ink], ["blocking", "Blocking finds, all fixed", C.blue, "white"], ["nonblocking", "Non-blocking, or left open as a stated limit", C.orange, C.ink]];
  let b = "", lx = 0;
  kinds.forEach(([, label, fill]) => { b += rect(lx, 2, 9, 9, { fill, stroke: C.muted, sw: 0.5 }) + text(lx + 12, 10, label, { size: 7.5 }); lx += 16 + width(label, 7.5) + 14; });
  teams.forEach((t, i) => {
    const y = 24 + i * 30;
    b += text(144, y + 8, t.name.replace(/^(Proc::command|Net::get|Net::resolve|Rand)/, "`$1`"), { size: 8, anchor: "end", bold: true });
    b += text(144, y + 18, t.when + (t.controls ? `; plus ${t.controls} controls` : ""), { size: 7, anchor: "end", fill: C.muted });
    let at = 150;
    for (const [k, , fill, ink] of kinds) {
      const n = t[k]; if (!n) continue;
      const w = x(n);
      b += rect(at, y + 2, w, 15, { fill, stroke: C.muted, sw: 0.4 });
      if (k === "held") b += text(at + w / 2, y + 12.5, String(n), { size: 7.5, anchor: "middle", fill: ink });
      at += w;
    }
    const notes = [t.blocking ? `${t.blocking} blocking, fixed` : "no blocking find", t.nonblocking ? `${t.nonblocking} ${t.routes ? "left open" : "non-blocking"}` : ""].filter(Boolean);
    b += text(at + 5, y + 12.5, notes.join("; "), { size: 7.5 });
  });
  svg("redteams", 24 + teams.length * 30, b);
}

// ---- Figure: search record -------------------------------------------------------------
// Every count is read from its round's record by the selectors in spec/search.json.
const WORDS = { none: 0, no: 0, nothing: 0, one: 1, ten: 10 };
function stageValue(body, st, vars, where) {
  if (st.sum) return st.sum.reduce((a, v) => a + vars[v], 0) - (st.minus ?? []).reduce((a, v) => a + vars[v], 0);
  if (st.count) {
    const n = body.split("\n").filter((l) => new RegExp(st.count).test(l)).length;
    need(n > 0, `${where}: ${st.count} matches nothing`);
    return n;
  }
  const ms = [...body.matchAll(new RegExp(st.re, "gm"))];
  need(ms.length === (st.matches ?? 1), `${where}: ${st.re} matches ${ms.length} times, not ${st.matches ?? 1}`);
  return ms.reduce((a, m) => a + m.slice(1).reduce((t, g) => {
    const n = /^[\d,]+$/.test(g) ? Number(g.replace(/,/g, "")) : WORDS[g.toLowerCase()];
    need(n !== undefined, `${where}: cannot read "${g}" as a number`);
    return t + n;
  }, 0), 0);
}
function deriveSearch(spec, readRecord) {
  return spec.rounds.map((r) => {
    const body = readRecord(r.record), vars = {};
    for (const [k, st] of Object.entries(r.vars ?? {})) vars[k] = stageValue(body, st, vars, `${r.round} ${k}`);
    const fill = (u) => (u ?? "").replace(/\{(\w+)\}/g, (_, k) => thousands(vars[k]));
    const stage = (k) => (r[k] === null ? null : { n: stageValue(body, r[k], vars, `${r.round} ${k}`), unit: fill(r[k].unit) });
    for (const q of r.note?.quote ?? []) need(body.includes(q), `${r.record} no longer contains: ${q}`);
    return { round: r.round, record: r.record, catch: Boolean(r.catch), same_catch_as: r.same_catch_as, note: r.note?.text,
      pool: stage("pool"), screened: stage("screened"), eligible: stage("eligible"), caught: stage("caught") };
  });
}
{
  const spec = json(`${FIG}/spec/search.json`);
  const rounds = deriveSearch(spec, read);
  // Falsifier, run on every build: if research/47 reported a catch, the derived data must show it.
  const tampered = deriveSearch(spec, (f) => (f === "research/47-malware-rescreen.md" ? read(f).replace("**Nothing was caught.**", "**One was caught.**") : read(f)));
  need(tampered.find((r) => r.round === "fx-zu4f.8").caught.n === 1, "falsifier: a changed search result did not change the derived data");
  for (const r of rounds.filter((r) => r.same_catch_as)) need(rounds.some((o) => o.round === r.same_catch_as && o.catch), `${r.round}: same_catch_as names no catch row`);
  emitJSON("data/search.json", rounds);
  const cols = [{ k: "pool", h: "Pool" }, { k: "screened", h: "Screened" }, { k: "eligible", h: "Eligible" }, { k: "caught", h: "Caught" }];
  const cx = [66, 196, 326, 396], rh = 22;
  const bar = scaleLog().domain([1, 10000]).range([0, 96]);
  let b = text(0, 12, "Round", { size: 7.5, fill: C.muted });
  cols.forEach((c, i) => (b += text(cx[i], 12, c.h, { size: 8, bold: true })));
  let y = 22;
  for (const r of rounds) {
    const h = r.note ? rh + 11 : rh;
    if (r.catch) b += rect(-2, y - 2, W + 2, h - 2, { fill: "#fbe7c2" });
    b += text(0, y + 11, `\`${r.round}\``, { size: 7.5, bold: r.catch });
    cols.forEach((c, j) => {
      const v = r[c.k], x = cx[j];
      if (j > 0) b += text(x - 12, y + 11, "›", { size: 10, fill: C.muted });
      if (v === null) { b += text(x, y + 11, "—", { size: 8, fill: C.muted }); return; }
      const big = j < 2;
      if (big) b += rect(x, y + 13, Math.max(bar(Math.max(v.n, 1)), 1.5), 3, { fill: r.catch ? C.orange : C.rule });
      const label = big ? `${thousands(v.n)} ${v.unit}` : `${v.n}${v.unit ? ` ${v.unit}` : ""}`;
      const hit = c.k === "caught" && v.n > 0 && !r.same_catch_as;
      b += text(x, y + 10, label, { size: 7.5, bold: hit, fill: hit ? C.verm : C.ink });
    });
    if (r.note) b += text(cx[0], y + 26, r.note, { size: 7, italic: true });
    y += h;
  }
  b += text(0, y + 6, "Bars under Pool and Screened: the count on a log scale (1 to 10,000), in each round's own unit.", { size: 7, fill: C.muted });
  svg("search", y + 10, b);
}

// ---- Figure: E3 ------------------------------------------------------------------------
{
  const e3 = json("trials/e3-pilot/result.json");
  need(e3.schema === "tide-e3-result/1", "unexpected E3 schema");
  const threshold = Number(grab(e3.sentence, /D0's ([0-9.]+)x threshold/, "trials/e3-pilot/result.json sentence")[1]);
  const S = e3.gate.S;
  let b = text(0, 10, "Sessions that caught the planted widening", { size: 8.5, bold: true });
  const x = scaleLinear().domain([0, S]).range([58, 215]);
  [["trust diff", e3.gate.K_T, C.blue], ["plain diff", e3.gate.K_P, C.muted]].forEach(([name, k, fill], i) => {
    const y = 20 + i * 20;
    b += text(54, y + 10, name, { size: 8, anchor: "end" }) + rect(x(0), y, x(k) - x(0), 13, { fill });
    b += text(x(k) + 3, y + 10, `${k} of ${S}`, { size: 8 });
  });
  const ceil = (2 * S) / 3;
  b += line(x(ceil), 16, x(ceil), 58, { stroke: C.verm, dash: "2 2", sw: 0.8 });
  b += axis(x, 62, 0, S, [0, 24, 48, 72], "") + lines(0, 86, wrap(`Dashed: the ceiling 2S/3 = ${Math.round(ceil)}; above it, 1.5 times the plain arm's catches exceeds the S = ${S} sessions an arm has.`, 220, 7), { size: 7, fill: C.muted });
  const lo = 0.8, hi = Math.max(threshold + 0.1, e3.interval.b);
  const rx = scaleLinear().domain([lo, hi]).range([250, W - 8]);
  b += text(250, 10, "Ratio of catches, trust to plain", { size: 8.5, bold: true });
  b += line(rx(1), 16, rx(1), 58, { stroke: C.muted, dash: "1 2" }) + line(rx(threshold), 16, rx(threshold), 58, { stroke: C.verm, dash: "4 2", sw: 0.9 });
  b += text(rx(threshold) - 3, 24, `D0 threshold, ${threshold.toFixed(1)}×`, { size: 7.5, anchor: "end", fill: C.verm });
  const yr = 42;
  b += line(rx(e3.interval.a), yr, rx(e3.interval.b), yr, { stroke: C.blue, sw: 1.4 });
  for (const v of [e3.interval.a, e3.interval.b]) b += line(rx(v), yr - 4, rx(v), yr + 4, { stroke: C.blue, sw: 1 });
  b += el("circle", { cx: rx(e3.ratio.value), cy: yr, r: 3.2, fill: C.blue });
  b += text(rx(1) + 6, yr + 2, `R = ${e3.ratio.text}, 95% interval ${e3.interval.text}`, { size: 7.5 });
  b += axis(rx, 62, lo, hi, [0.8, 1.0, 1.2, 1.4, 1.6].filter((t) => t <= hi + 1e-9).map((t) => Number(t.toFixed(1))), "×");
  b += lines(250, 86, wrap(`Gate: ${e3.gate.result}.`, W - 258, 7), { size: 7, fill: C.muted });
  emitJSON("data/e3.json", { source: "trials/e3-pilot/result.json", S, K_T: e3.gate.K_T, K_P: e3.gate.K_P, ratio: e3.ratio.value, interval: [e3.interval.a, e3.interval.b], threshold, gate: e3.gate.result });
  svg("e3", 100, b);
}

// ---- Figure: benchmarks -----------------------------------------------------------------
{
  const path = "research/36-benchmarks/2026-11-18-bf6b958/records.jsonl";
  const recs = read(path).split("\n").filter((l) => l.trim()).map((l) => JSON.parse(l));
  const summaries = recs.filter((r) => r.record === "summary");
  const LOSS = 20; // research/36-benchmarks.md §1: slower in at least 20 of the 30 paired rounds
  const pairs = ["contracts", "files", "tasks", "tcp"];
  const row = (kind, phase, tide, rust, pair) => {
    const sum = (config) => { const f = summaries.filter((r) => r.phase === phase && r.config === config && r.pair === pair); need(f.length === 1, `missing summary ${phase} ${config} ${pair}`); return f[0]; };
    const rounds = (config) => new Map(recs.filter((r) => r.record === kind && r.config === config && r.pair === pair).map((r) => [r.round, r.wall_s]));
    const t = rounds(tide), u = rounds(rust);
    need(t.size === u.size && t.size > 0, `unpaired rounds for ${pair}`);
    const slower = [...t].filter(([k, w]) => w > u.get(k)).length;
    const verdict = slower >= LOSS ? "loss" : t.size - slower >= LOSS ? "win" : "tie";
    return { pair, ratio: sum(tide).median_s / sum(rust).median_s, slower, rounds: t.size, verdict };
  };
  const panels = [
    { label: "Run time in release, against Rust built with Tide's profile flags", rows: pairs.map((p) => row("run", "run", "tide-release", "rust-release-tide-flags", p)) },
    { label: "Cold debug build, against Rust's debug build", rows: pairs.map((p) => row("build", "build", "tide-debug", "rust-debug", p)) },
  ];
  const cvs = summaries.map((r) => r.cv), stable = summaries.filter((r) => r.stable === true).length;
  const pct3 = (r) => { const p = 100 * r; return `${p.toFixed(p >= 10 ? 1 : 2)}%`; };
  const tasksPlain = row("run", "run", "tide-release", "rust-release", "tasks").ratio;
  emitJSON("data/bench.json", { source: path, loss_rounds: LOSS, stable, series: summaries.length, cv_min: Math.min(...cvs), cv_max: Math.max(...cvs), panels,
    tasks_twin_ratio: panels[0].rows.find((r) => r.pair === "tasks").ratio, tasks_plain_ratio: tasksPlain });
  let b = lines(0, 10, wrap(`Unstable runs. ${stable} of ${summaries.length} series in the main run met the benchmark tool's stability criterion (CVs ${pct3(Math.min(...cvs))} to ${pct3(Math.max(...cvs))}). These medians are descriptive, not settled measurements.`, W, 8), { size: 8, fill: C.verm });
  let y = 36;
  for (const p of panels) {
    const hi = Math.ceil(Math.max(...p.rows.map((r) => r.ratio)) * 2) / 2;
    const ticks = []; for (let t = 0; t <= hi + 1e-9; t += hi <= 2 ? 0.5 : 1) ticks.push(t);
    const x = scaleLinear().domain([0, hi]).range([60, 290]);
    b += text(0, y, p.label, { size: 8.5, bold: true });
    b += line(x(1), y + 5, x(1), y + 8 + p.rows.length * 15, { stroke: C.muted, dash: "1 2" });
    p.rows.forEach((r, i) => {
      const yy = y + 14 + i * 15, col = r.verdict === "loss" ? C.verm : C.ink;
      b += text(54, yy + 3, `\`${r.pair}\``, { size: 8, anchor: "end" });
      b += line(x(Math.min(1, r.ratio)), yy, x(Math.max(1, r.ratio)), yy, { stroke: col, sw: 1.1 });
      b += r.verdict === "loss" ? el("circle", { cx: x(r.ratio), cy: yy, r: 3.2, fill: col }) : el("circle", { cx: x(r.ratio), cy: yy, r: 3, fill: "white", stroke: col, "stroke-width": 1.1 });
      b += text(300, yy + 3, `${r.ratio.toFixed(2)}× ${r.verdict} (Tide slower in ${r.slower} of ${r.rounds} rounds)`, { size: 7.5, fill: col, bold: r.verdict === "loss" });
    });
    y += 14 + p.rows.length * 15;
    b += axis(x, y - 6, 0, hi, ticks.map((t) => Number(t.toFixed(1))), "×");
    y += 22;
  }
  b += text(0, y - 4, "Filled dot: a loss; open dot: a tie. Dotted line: parity, 1×.", { size: 7, fill: C.muted });
  svg("bench", y, b);
}

// ---- Diagrams ------------------------------------------------------------------------------
const D = json(`${FIG}/spec/diagrams.json`);
{
  const p = D.pipeline;
  for (const s of p.src) quoted(p.source, s);
  const n = p.boxes.length, gap = 14, bw = (W - 2 - gap * (n - 1)) / n, top = 18, bh = 96;
  let b = text(0, 10, `In: ${p.input}`, { size: 7.5, fill: C.muted }) + text(W, 10, `Out: ${p.output}`, { size: 7.5, fill: C.muted, anchor: "end" });
  p.boxes.forEach((s, i) => {
    const x = 1 + i * (bw + gap);
    b += box(x, top, bw, bh, s.title, s.body, { fill: i === 1 ? C.faint : "white" });
    if (i > 0) b += line(x - gap + 1, top + bh / 2, x - 1, top + bh / 2, { arrow: true });
  });
  svg("pipeline", top + bh + 2, b);
}
{
  const r = D.related, colW = (W - 70) / r.columns.length;
  let b = "";
  r.columns.forEach((c, i) => (b += lines(70 + i * colW + 3, 10, wrap(c, colW - 6, 8), { size: 8, bold: true })));
  r.rows.forEach((row, i) => {
    const y = 30 + i * 26;
    b += line(0, y - 2, W, y - 2, { stroke: C.rule, sw: 0.4 }) + text(0, y + 10, row.tool, { size: 8.5, bold: row.tool === "Tide" });
    row.cells.forEach((cell, j) => {
      const x = 70 + j * colW + 3;
      if (!cell) { b += text(x, y + 10, r.none, { size: 8, fill: C.muted }); return; }
      quoted(r.source, cell[1]);
      const yes = cell[0].startsWith("yes") && cell[2] !== "unshaded"; // "unshaded": a design claim, not a v0 result
      const ls = wrap(cell[0], colW - 10, yes ? 8.2 : 7.5);
      if (yes) b += rect(x - 3, y, colW - 2, Math.max(14, ls.length * 9.4 + 4), { fill: "#dcecf7" });
      b += lines(x, y + 9, ls, { size: 7.5, bold: yes });
    });
  });
  const yEnd = 30 + r.rows.length * 26 + 8;
  b += text(0, yEnd, `${r.none}: the section makes no claim. Shaded: the property holds, with the stated scope.`, { size: 7, fill: C.muted });
  svg("related", yEnd + 4, b);
}
{
  const s = D.sim_model, readme = read(s.source);
  const at = readme.indexOf("### Simulator fidelity"), table = readme.slice(at, readme.indexOf("\n\n", readme.indexOf("| --- |", at)));
  let b = text(70, 10, "The v0 simulator models", { size: 8.5, bold: true }) + text(70 + (W - 70) / 2 + 4, 10, "It does not model", { size: 8.5, bold: true });
  let y = 18;
  const cw = (W - 70) / 2 - 8;
  for (const row of s.rows) {
    need(table.includes(row.models[1]) && table.includes(row.omits[1]), `README simulator table no longer says: ${row.models[1]} / ${row.omits[1]}`);
    const lm = wrap(row.models[0], cw, 7.5), lo = wrap(row.omits[0], cw, 7.5), h = Math.max(lm.length, lo.length) * 9.4 + 8;
    b += line(0, y, W, y, { stroke: C.rule, sw: 0.4 }) + text(0, y + 11, row.area, { size: 8, bold: true });
    b += rect(70, y + 2, cw + 4, h - 4, { fill: "#dcecf7" }) + lines(73, y + 11, lm, { size: 7.5 });
    b += rect(70 + cw + 12, y + 2, cw + 4, h - 4, { fill: C.faint }) + lines(70 + cw + 15, y + 11, lo, { size: 7.5 });
    y += h;
  }
  svg("sim-model", y + 2, b);
}
{
  const t = D.tiers, rec = read(t.source).split("\n");
  const cw = (W - 16) / 3;
  let b = "", hMax = 0;
  t.columns.forEach((c, i) => {
    const cell = rec.find((l) => l.startsWith(c.row));
    need(cell, `${t.source}: no tier row ${c.row}`);
    const x = i * (cw + 8);
    b += rect(x, 0, cw, 16, { fill: i === 0 ? C.blue : i === 1 ? C.sky : C.faint }) + text(x + 5, 11.5, c.tier, { size: 8.5, bold: true, fill: i === 0 ? "white" : C.ink });
    let y = 28;
    for (const [label, src] of c.items) {
      need(cell.includes(src), `${t.source} ${c.row} no longer contains: ${src}`);
      const planned = / \(planned\)$/.test(label), ls = wrap(label, cw - 14, 7.5);
      b += text(x + 3, y, "•", { size: 7.5, fill: planned ? C.muted : C.ink }) + lines(x + 10, y, ls, { size: 7.5, fill: planned ? C.muted : C.ink, italic: planned });
      y += ls.length * 9.4 + 2;
    }
    hMax = Math.max(hMax, y);
  });
  svg("tiers", hMax, b);
}
{
  const m = D.merge, n = m.steps.length, gap = 14, bw = (W - 2 - gap * (n - 1)) / n, bh = 74;
  let b = "";
  m.steps.forEach((s, i) => {
    quoted(m.source, s.src);
    const x = 1 + i * (bw + gap);
    b += box(x, 1, bw, bh, s.title, s.body, { fill: i === n - 1 ? "#dcecf7" : "white" });
    if (i > 0) b += line(x - gap + 1, 1 + bh / 2, x - 1, 1 + bh / 2, { arrow: true });
  });
  let y = bh + 16;
  b += text(0, y, "Exceptions", { size: 8, bold: true });
  for (const e of m.exceptions) {
    quoted(m.source, e.src);
    const ls = wrap(e.text, W - 30, 7.5);
    b += text(0, y + 12, "–", { size: 7.5 }) + lines(10, y + 12, ls, { size: 7.5 });
    y += ls.length * 9.4 + 2;
  }
  svg("merge", y + 6, b);
}

// ---- Write or check ----------------------------------------------------------------------
const check = process.argv.includes("--check");
const generatedDirs = ["svg", "excerpts", "data"];
const inputs = new Set([`${FIG}/data/reference-sections.json`]); // written by count-reference.mjs, read here
const stale = generatedDirs.flatMap((d) => (existsSync(`${FIG}/${d}`) ? readdirSync(`${FIG}/${d}`).map((f) => `${FIG}/${d}/${f}`) : [])).filter((f) => !out.has(f) && !inputs.has(f));
if (check) {
  const drift = [...out].filter(([p, t]) => !existsSync(p) || read(p) !== t).map(([p]) => p).concat(stale);
  if (drift.length) { console.error(`figures drifted from their records:\n  ${drift.join("\n  ")}\nrun: node ${FIG}/build.mjs`); process.exit(1); }
  console.log(`figures match their records (${out.size} files)`);
} else {
  need(stale.length === 0, `stale generated files, delete them: ${stale.join(", ")}`);
  for (const [p, t] of out) { mkdirSync(p.slice(0, p.lastIndexOf("/")), { recursive: true }); writeFileSync(p, t); }
  console.log(`wrote ${out.size} files`);
}
