// mrn-context: inject a compact context block (Model, Build Log, Anthropic
// usage) into each pi turn. Three handlers share one per-source dedup map:
// session_start resets it; before_agent_start collects every source and
// returns one persistent <pi-note> when a source's key is new at prompt time;
// context injects the midTurn sources (build log, usage) non-persistently
// when their key changes mid-turn — before_agent_start only fires when a prompt is submitted, so
// without this a build run inside the agent loop stayed invisible until
// the next prompt. Both handlers mark the key as seen on emit, so each
// note surfaces exactly once.
//
// The build-log key is the log path alone. build.py creates the file and
// writes the first lines at launch, so appends during a build change the
// mtime but not the key; only a new build (new file) re-triggers the note.
// The model note posts once per key: at session start (keys reset) and on
// every model change. No periodic re-anchor — after a compaction the
// attribution may drop out of the outgoing context until the next change.
//
// Build logs are session-attributed: build.py stamps each log's first line
// with "Session-Id: <uuid>" (from PI_SESSION_ID, injected into tool
// subprocesses only — not this process's env). The buildLog source surfaces
// the newest log only when it is unstamped (manual terminal runs) or stamped
// with THIS session's id; logs from other pi sessions are suppressed.
//
// The usage source reads the cache that @sreetej510/pi-usage writes
// (~/.pi/agent/usage-cache.json); it never calls the usage endpoint, which
// rate-limits. It shows only for an anthropic model. The key is the threshold
// bucket of the 5h and 7d windows, so a note posts when a window's bucket
// changes; a reset posts only when it lowers the bucket. A change between
// two anthropic models keeps the key, because the usage is per account.
// A non-anthropic model sets the key to "off" with no note, so a later
// change back to anthropic posts again. The cache stores display text
// ("5h: [...] 4% used (resets 03:40 on 21 Oct)"); pi-usage writes the month
// in the process locale. If any part of a window line fails to parse (a
// format change, a non-English month), no usage note posts.
// @ts-nocheck

import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

const BUILD_LOG_GLOB = "logs/build_*.log";
const MTIME_WINDOW_SECONDS = 300;
const USAGE_CACHE_PATH = path.join(os.homedir(), ".pi", "agent", "usage-cache.json");
const USAGE_THRESHOLDS = [0, 30, 60, 80, 90, 100];
const USAGE_WINDOWS = ["5h", "7d"];
const MONTH_ABBREVIATIONS = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];

// A result without a line records the key and posts no note.
type SourceResult = { key: string; line?: string };
type Source = { name: string; midTurn: boolean; get: (ctx: any) => SourceResult | undefined };

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

  const prefix = cwd.endsWith(path.sep) ? cwd : cwd + path.sep;
  const emitPath = newestPath.startsWith(prefix) ? newestPath.substring(prefix.length) : newestPath;
  return { key: newestPath, line: `Build Log: ${emitPath}` };
}

function formatDuration(ms: number): string {
  const minutes = Math.floor(ms / 60000);
  if (minutes < 1) return "<1m";
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h${(minutes % 60).toString().padStart(2, "0")}m`;
  return `${Math.floor(hours / 24)}d ${hours % 24}h`;
}

// pi-usage formats a reset as "HH:MM" (same day as the capture) or
// "HH:MM on D Mon", in local time with no year.
function parseResetTime(text: string, capturedAt: number): number | undefined {
  const match = text.match(/^(\d{1,2}):(\d{2})(?: on (\d{1,2}) (\w{3}))?$/);
  if (!match) return undefined;
  const base = new Date(capturedAt);
  let year = base.getFullYear();
  let month = base.getMonth();
  let day = base.getDate();
  if (match[3] !== undefined) {
    month = MONTH_ABBREVIATIONS.indexOf(match[4].toLowerCase());
    if (month < 0) return undefined;
    day = Number(match[3]);
    if (new Date(year, month, day).getTime() < capturedAt - 86400000) year += 1;
  }
  return new Date(year, month, day, Number(match[1]), Number(match[2])).getTime();
}

function usageBucket(percent: number): number {
  let bucket = USAGE_THRESHOLDS[0];
  for (const threshold of USAGE_THRESHOLDS) if (percent >= threshold) bucket = threshold;
  return bucket;
}

type WindowUsage = { percent: number; resetAt?: number };

// Returns null when the window line exists but does not parse, and
// undefined when the cache has no line for the window.
function parseWindowUsage(summaryLines: string[], window: string, capturedAt: number): WindowUsage | null | undefined {
  const pattern = new RegExp(`^\\s*${window}:.*?(\\d+)% used(?: \\(resets (.+)\\))?\\s*$`);
  const prefix = new RegExp(`^\\s*${window}:`);
  const line = summaryLines.find((l) => typeof l === "string" && prefix.test(l));
  if (line === undefined) return undefined;
  const match = line.match(pattern);
  if (!match) return null;
  if (match[2] === undefined) return { percent: Number(match[1]) };
  const resetAt = parseResetTime(match[2], capturedAt);
  if (resetAt === undefined) return null;
  return { percent: Number(match[1]), resetAt };
}

function getUsageNote(ctx: any): SourceResult | undefined {
  if (ctx?.model?.provider !== "anthropic") return { key: "off" };
  const cache = JSON.parse(fs.readFileSync(USAGE_CACHE_PATH, "utf8"));
  const report = cache?.entries?.anthropic?.report;
  const summaryLines: string[] = report?.summaryLines;
  const capturedAt: number = report?.capturedAt;
  if (!Array.isArray(summaryLines) || typeof capturedAt !== "number") return undefined;

  const now = Date.now();
  const windowBuckets: string[] = [];
  const windowTexts: string[] = [];
  for (const window of USAGE_WINDOWS) {
    const usage = parseWindowUsage(summaryLines, window, capturedAt);
    if (usage === null) return undefined;
    if (usage === undefined) continue;
    windowBuckets.push(`${window}:${usageBucket(usage.percent)}`);
    let text = `${window} ${usage.percent}% used`;
    if (usage.resetAt !== undefined) {
      text += usage.resetAt > now ? `, resets in ${formatDuration(usage.resetAt - now)}` : ", reset time has passed";
    }
    windowTexts.push(text);
  }
  if (windowTexts.length === 0) return undefined;

  const age = formatDuration(Math.max(0, now - capturedAt));
  return {
    key: windowBuckets.join("|"),
    line: `Anthropic usage (data ${age} old): ${windowTexts.join(". ")}. Source: ${USAGE_CACHE_PATH}`,
  };
}

const SOURCES: Source[] = [
  { name: "model", midTurn: false, get: getModelNote },
  { name: "buildLog", midTurn: true, get: getBuildLogNote },
  { name: "usage", midTurn: true, get: getUsageNote },
];

const lastKeys: Record<string, string | undefined> = {};

// Marks each new key as seen and returns the lines to post.
function collectNewLines(sources: Source[], ctx: any): string[] {
  const lines: string[] = [];
  for (const src of sources) {
    const result = safeCall(() => src.get(ctx));
    if (result === undefined) continue;
    if (result.key === lastKeys[src.name]) continue;
    lastKeys[src.name] = result.key;
    if (result.line) lines.push(result.line);
  }
  return lines;
}

function formatPiNote(lines: string[]): string {
  return `<pi-note>\n${lines.join("\n")}\n</pi-note>`;
}

export default function (pi: any): void {
  pi.on("session_start", () => {
    for (const k of Object.keys(lastKeys)) lastKeys[k] = undefined;
  });

  pi.on("before_agent_start", (_event: unknown, ctx: any) => {
    const lines = collectNewLines(SOURCES, ctx);
    if (lines.length === 0) return undefined;
    return {
      message: {
        customType: "pi-note",
        content: formatPiNote(lines),
        display: false,
      },
    };
  });

  // Within-turn freshness: append the midTurn sources to the outgoing
  // context (non-persistent) the first time their key changes mid-turn.
  // Marking lastKeys here also dedups against before_agent_start — whoever
  // sees the key first emits it, exactly once.
  pi.on("context", (event: any, ctx: any) => {
    const lines = collectNewLines(
      SOURCES.filter((src) => src.midTurn),
      ctx,
    );
    if (lines.length === 0) return undefined;
    return {
      messages: [
        ...event.messages,
        {
          role: "custom",
          customType: "pi-note",
          content: formatPiNote(lines),
          display: false,
          timestamp: Date.now(),
        },
      ],
    };
  });
}
