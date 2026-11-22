// The context sources behind the <pi-note> block: the model of the turn, and
// the newest build log of this pi session.
//
// A source returns a key and the line to post. The key names the values the
// reader cares about, so a note posts when one of them changes.
//
// The build-log key is the log path alone. build.py creates the file and
// writes its first lines at launch, so appends during a build change the mtime
// but not the key; only a new build (new file) re-triggers the note.
//
// Build logs are session-attributed: build.py stamps each log's first line
// with "Session-Id: <uuid>" (from PI_SESSION_ID, injected into tool
// subprocesses only — not this process's env). The build-log source surfaces
// the newest log only when it is unstamped (manual terminal runs) or stamped
// with THIS session's id; logs from other pi sessions are suppressed.

import * as fs from "node:fs";
import * as path from "node:path";

const BUILD_LOG_GLOB = "logs/build_*.log";
const MTIME_WINDOW_SECONDS = 300;

export type SourceResult = { key: string; line: string };

// midTurn sources also post when their key changes inside a turn, which
// before_agent_start alone cannot see.
export type Source = { name: string; midTurn: boolean; get: (ctx: any) => SourceResult | undefined };

// The key each source posted last, by source name. A session start clears it.
export type SourceKeys = Record<string, string | undefined>;

function safeCall<T>(fn: () => T | undefined): T | undefined {
  try {
    return fn();
  } catch {
    return undefined;
  }
}

export function getModelNote(ctx: any): SourceResult | undefined {
  const m = ctx?.model;
  if (!m || typeof m.provider !== "string" || typeof m.id !== "string") return undefined;
  if (!m.provider || !m.id) return undefined;
  const key = `${m.provider}/${m.id}`;
  return { key, line: `Model: ${key}` };
}

// The stamp line, or undefined for a log that build.py did not write.
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

// The newest build log of this session, when it was written recently.
export function getBuildLogNote(ctx: any): SourceResult | undefined {
  const cwd = ctx?.cwd;
  if (!cwd) return undefined;
  const matches = fs.globSync([BUILD_LOG_GLOB], { cwd });
  if (matches.length === 0) return undefined;

  let newestPath: string | undefined;
  let newestMtime = -Infinity;
  for (const m of matches) {
    const abs = path.resolve(cwd, m);
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

  // The note shows the log relative to the working directory, whether the
  // glob matched an absolute path or a relative one.
  const emitPath = path.relative(cwd, newestPath);
  return { key: newestPath, line: `Build Log: ${emitPath}` };
}

// Marks each new key as seen and returns the lines to post.
export function collectNewLines(sources: Source[], keys: SourceKeys, ctx: any): string[] {
  const lines: string[] = [];
  for (const src of sources) {
    const result = safeCall(() => src.get(ctx));
    if (result === undefined) continue;
    if (result.key === keys[src.name]) continue;
    keys[src.name] = result.key;
    lines.push(result.line);
  }
  return lines;
}

export function formatPiNote(lines: string[]): string {
  return `<pi-note>\n${lines.join("\n")}\n</pi-note>`;
}
