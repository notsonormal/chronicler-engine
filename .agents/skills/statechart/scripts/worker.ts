import { Buffer } from "node:buffer";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { inspect } from "node:util";
import { CHART_STATES, DUMP_STATES } from "./ir.ts";
import { checkView } from "./frontends/view.ts";
import { checkScenarios } from "./frontends/scenarios.ts";
import { compile, compileScenarios } from "./pipeline.ts";
import { check, messageOf, runScenarios } from "./run.ts";

const [stage, specFile, configFile, viewFile, scenariosFile] = process.argv.slice(2);

const load = async (file: string) => await import(pathToFileURL(path.resolve(file)).href);

try {
  const view = checkView(viewFile ? await load(viewFile) : {}, inspect);
  if (scenariosFile && !(view.lanes && view.message))
    throw new Error("--scenarios needs a view that exports lanes and message().");
  const { files, exit } = scenariosFile
    ? compileScenarios(
        await runScenarios(specFile, configFile, checkScenarios(await load(scenariosFile), inspect)),
        view,
      )
    : compile(await check(specFile, configFile, view.state ? DUMP_STATES : CHART_STATES), view);
  const draft = path.join(stage, "draft");
  fs.mkdirSync(draft);
  for (const [name, parts] of files) {
    const fd = fs.openSync(path.join(draft, name), "wx");
    try {
      for (const part of parts)
        fs.writeSync(fd, typeof part === "string" ? Buffer.from(part) : part);
      fs.writeSync(fd, "\n");
    } finally {
      fs.closeSync(fd);
    }
  }
  fs.renameSync(draft, path.join(stage, "done"));
  process.exit(exit);
} catch (error) {
  console.error(messageOf(error));
  process.exit(2);
}
