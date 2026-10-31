import { check } from "tla-ts";
import { readFileSync, existsSync } from "node:fs";
const r = await check({
  source: readFileSync("GateSpecOnly.tla", "utf8"),
  config: readFileSync("probe-small.cfg", "utf8"),
  dump: { path: "dumps/api-probe.dot", dot: true, actionLabels: true },
  writeFile: (p, c) => { console.log("writeFile", p, c.length); import("node:fs").then(fs => fs.writeFileSync(p, c)); },
});
console.log("status", r.status, "distinct", r.distinctStates, "generated", r.generatedStates);
