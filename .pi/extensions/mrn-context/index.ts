// mrn-context: inject a compact context block (Model, Build Log) into each
// pi turn via before_agent_start. Three handlers: session_start resets dedup
// state; before_agent_start collects two sources, per-source dedup, returns
// one <pi-note> message when something changed; context re-injects the
// build-log pointer non-persistently on every LLM call while the newest log
// has not been persisted yet — before_agent_start only fires when a prompt
// is submitted, so without this a build run inside the agent loop stayed
// invisible until the next prompt.
//
// Build logs are session-attributed: build.py stamps each log's first line
// with "Session-Id: <uuid>" (from PI_SESSION_ID, injected into tool
// subprocesses only — not this process's env). The buildLog source surfaces
// the newest log only when it is unstamped (manual terminal runs) or stamped
// with THIS session's id; logs from other pi sessions are suppressed.
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

function readSessionStamp(logPath: string): string | undefined {
  return safeCall(() => {
    const fd = fs.openSync(logPath, "r");
    try {
      const buf = Buffer.alloc(1024);
      const bytes = fs.readSync(fd, buf, 0, 1024, 0);
      const head = buf.toString("utf8", 0, bytes);
      const match = head.match(/^Session-Id:\s*(\S+)\s*$/m);
      return match ? match[1] : undefined;
    } finally {
      fs.closeSync(fd);
    }
  });
}

function getBuildLogNote(ctx: any): SourceResult | undefined {
  const cwd = ctx?.cwd;
  if (!cwd) return undefined;
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

  // Loose fallback: unstamped logs (manual terminal runs) and an unknown
  // own session both keep the pointer visible.
  const stamp = readSessionStamp(newestPath);
  const ownSession = safeCall(() => ctx?.sessionManager?.getSessionId?.());
  if (stamp !== undefined && ownSession && stamp !== ownSession) return undefined;

  const minutes = Math.floor(ageSeconds / 60);
  const ageStr = minutes < 60 ? `${minutes}m old` : `${Math.floor(minutes / 60)}h old`;
  const prefix = cwd.endsWith(path.sep) ? cwd : cwd + path.sep;
  const emitPath = newestPath.startsWith(prefix) ? newestPath.substring(prefix.length) : newestPath;
  return { key: `${newestPath}:${Math.floor(newestMtime)}`, line: `Build Log: ${emitPath} (${ageStr})` };
}

const SOURCES = [
  { name: "model", get: (ctx: any) => getModelNote(ctx) },
  { name: "buildLog", get: (ctx: any) => getBuildLogNote(ctx) },
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

  // Within-turn freshness: append the build-log pointer to the outgoing
  // context (non-persistent) when the newest log differs from the last
  // persisted key. Once before_agent_start persists it at the next prompt,
  // lastKeys catches up and this handler goes quiet again.
  pi.on("context", (event: any, ctx: any) => {
    const result = safeCall(() => getBuildLogNote(ctx));
    if (result === undefined) return undefined;
    if (result.key === lastKeys.buildLog) return undefined;
    return {
      messages: [
        ...event.messages,
        {
          role: "custom",
          customType: "pi-note",
          content: `<pi-note>\n${result.line}\n</pi-note>`,
          display: false,
          timestamp: Date.now(),
        },
      ],
    };
  });
}