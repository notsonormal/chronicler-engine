// mrn-context: inject a compact context block (Model, Build Log) into each
// pi turn via before_agent_start. Two handlers: session_start resets dedup
// state; before_agent_start collects two sources, per-source dedup, returns
// one <pi-note> message when something changed.
// @ts-nocheck

import * as fs from "node:fs";
import * as path from "node:path";

const BUILD_LOG_GLOB = "logs/build_*.log";
const MTIME_WINDOW_SECONDS = 300;

type SourceResult = { key: string; line: string };

function safeCall<T>(fn: () => T | undefined): T | undefined {
  try {
    return fn();
  } catch {
    return undefined;
  }
}

function getModelNote(ctx: any): SourceResult | undefined {
  const m = ctx?.model;
  if (!m || typeof m.provider !== "string" || typeof m.id !== "string") return undefined;
  if (!m.provider || !m.id) return undefined;
  const key = `${m.provider}/${m.id}`;
  return { key, line: `Model: ${key}` };
}

function getBuildLogNote(cwd: string): SourceResult | undefined {
  const matches = fs.globSync([BUILD_LOG_GLOB], { cwd });
  if (matches.length === 0) return undefined;

  let newestPath: string | undefined;
  let newestMtime = -Infinity;
  for (const m of matches) {
    const abs = path.isAbsolute(m) ? m : path.join(cwd, m);
    try {
      const stat = fs.statSync(abs);
      if (!stat.isFile()) continue;
      if (stat.mtimeMs > newestMtime) {
        newestMtime = stat.mtimeMs;
        newestPath = abs;
      }
    } catch {
      continue;
    }
  }
  if (newestPath === undefined) return undefined;

  const ageSeconds = Math.max(0, (Date.now() - newestMtime) / 1000);
  if (ageSeconds > MTIME_WINDOW_SECONDS) return undefined;

  const minutes = Math.floor(ageSeconds / 60);
  const ageStr = minutes < 60 ? `${minutes}m old` : `${Math.floor(minutes / 60)}h old`;
  const prefix = cwd.endsWith(path.sep) ? cwd : cwd + path.sep;
  const emitPath = newestPath.startsWith(prefix) ? newestPath.substring(prefix.length) : newestPath;
  return { key: `${newestPath}:${Math.floor(newestMtime)}`, line: `Build Log: ${emitPath} (${ageStr})` };
}

const SOURCES = [
  { name: "model", get: (ctx: any) => getModelNote(ctx) },
  { name: "buildLog", get: (ctx: any) => getBuildLogNote(ctx.cwd) },
] as const;

const lastKeys: Record<string, string | undefined> = {};
let turnCount = 0;

export default function (pi: any): void {
  pi.on("session_start", () => {
    for (const k of Object.keys(lastKeys)) lastKeys[k] = undefined;
    turnCount = 0;
  });

  pi.on("before_agent_start", (_event: unknown, ctx: any) => {
    turnCount += 1;
    const forceRefresh = turnCount % 10 === 0;
    const lines: string[] = [];
    for (const src of SOURCES) {
      const result = safeCall(() => src.get(ctx));
      if (result === undefined) continue;
      if (!forceRefresh && result.key === lastKeys[src.name]) continue;
      lastKeys[src.name] = result.key;
      lines.push(result.line);
    }
    if (lines.length === 0) return undefined;
    return {
      message: {
        customType: "pi-note",
        content: `<pi-note>\n${lines.join("\n")}\n</pi-note>`,
        display: false,
      },
    };
  });
}