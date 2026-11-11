import fs from "node:fs";
import path from "node:path";
import { render } from "pele";

const SKILL = "/workspace/chronicler-engine/.agents/skills/statechart";
const OPTIONS = { limit: 1_000_000, outputLimit: 8_000_000, maxEdges: 10_000 };

const css = (() => {
  const md = fs.readFileSync(path.join(SKILL, "references/render.md"), "utf8");
  const block = /```css\n([\s\S]*?)```/.exec(md);
  if (!block) throw new Error("render.md has no css block");
  return block[1];
})();

for (const file of process.argv.slice(2)) {
  const source = fs.readFileSync(file, "utf8");
  try {
    const { svg } = render(source, OPTIONS);
    const end = svg.indexOf(">") + 1;
    const styled = `${svg.slice(0, end)}\n<style>\n${css}</style>${svg.slice(end)}`;
    const out = file.replace(/\.mmd$/, ".svg");
    fs.writeFileSync(out, styled);
    console.log(`${out}\t${fs.statSync(out).size} bytes`);
  } catch (error) {
    console.error(`${file}: ${error.name}: ${error.message}`);
    process.exitCode = 1;
  }
}
