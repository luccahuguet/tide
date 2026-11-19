// Counts the core reference's tokens section by section, for the budget figure.
// Usage, from the repository root (needs a logged-in `claude` CLI, like the script it runs):
//   node paper/figures/count-reference.mjs
// Splits design/reference.md at its `## ` headings (outside code fences; the lines before the
// first heading are the "Preamble"), counts each part alone with scripts/count-tokens.sh, and
// writes paper/figures/data/reference-sections.json with the reference's git blob hash.
// build.mjs refuses that file once the reference changes, so rerun this after every change to
// the reference, as the companion's "Core count" line asks for the whole file.
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const source = "design/reference.md";
const lines = readFileSync(source, "utf8").split("\n");
const parts = [];
let fence = false;
let current = { heading: "Preamble", start: 1, body: [] };
lines.forEach((line, i) => {
  if (line.startsWith("```")) fence = !fence;
  if (!fence && line.startsWith("## ")) {
    parts.push(current);
    current = { heading: line.slice(3), start: i + 1, body: [] };
  }
  current.body.push(line);
});
parts.push(current);

// The parts go in a scratch directory under the figure tree, removed however the counts end.
mkdirSync("paper/figures/node_modules", { recursive: true });
const dir = mkdtempSync("paper/figures/node_modules/.count-reference-");
let sections;
try {
sections = parts.map((part, i) => {
  const file = join(dir, `part-${i}.md`);
  writeFileSync(file, part.body.join("\n"));
  const tokens = Number(execFileSync("scripts/count-tokens.sh", [file], { encoding: "utf8" }).trim());
  if (!Number.isInteger(tokens) || tokens <= 0) throw new Error(`bad count for ${part.heading}`);
  console.error(`${tokens}\t${part.heading}`);
  return { heading: part.heading, first_line: part.start, last_line: part.start + part.body.length - 1, tokens };
});
} finally {
  rmSync(dir, { recursive: true, force: true });
}

const blob = execFileSync("git", ["hash-object", source], { encoding: "utf8" }).trim();
const out = {
  source,
  blob,
  command: "node paper/figures/count-reference.mjs (scripts/count-tokens.sh on each part alone)",
  counted: new Date().toISOString().slice(0, 10),
  sections,
};
writeFileSync("paper/figures/data/reference-sections.json", JSON.stringify(out, null, 2) + "\n");
