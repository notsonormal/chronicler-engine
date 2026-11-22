// Unit tests for the pi-note sources. Run with:
// node --test .pi/extensions/mrn-context/sources.test.ts

import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, utimesSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import * as path from "node:path";
import test from "node:test";

import { type Source, collectNewLines, formatPiNote, getBuildLogNote, getModelNote } from "./sources.ts";

// A cwd that holds a logs/ folder, removed when the test ends.
function logsDir(t: test.TestContext): string {
  const dir = mkdtempSync(path.join(tmpdir(), "mrn-context-"));
  mkdirSync(path.join(dir, "logs"), { recursive: true });
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}

// A log with the first line build.py writes, and an age in seconds.
function writeLog(
  cwd: string,
  name: string,
  opts: { sessionId?: string; ageSeconds?: number } = {},
): string {
  const file = path.join(cwd, "logs", name);
  const stamp = opts.sessionId === undefined ? "" : `Session-Id: ${opts.sessionId}\n`;
  writeFileSync(file, `${stamp}Full build log: logs/${name}\n`);
  if (opts.ageSeconds !== undefined) {
    const at = (Date.now() - opts.ageSeconds * 1000) / 1000;
    utimesSync(file, at, at);
  }
  return file;
}

const ownSession = { getSessionId: () => "own-session" };

test("getModelNote keys the note by provider and model id", () => {
  assert.deepEqual(getModelNote({ model: { provider: "anthropic", id: "claude-opus-5-5" } }), {
    key: "anthropic/claude-opus-5-5",
    line: "Model: anthropic/claude-opus-5-5",
  });
  assert.equal(getModelNote({ model: { provider: "", id: "claude-opus-5-5" } }), undefined);
  assert.equal(getModelNote({ model: { provider: "anthropic" } }), undefined);
  assert.equal(getModelNote({}), undefined);
});

test("getBuildLogNote names the newest build log of this session", (t) => {
  const cwd = logsDir(t);
  writeLog(cwd, "build_1.log", { sessionId: "own-session", ageSeconds: 120 });
  const newest = writeLog(cwd, "build_2.log", { sessionId: "own-session" });

  assert.deepEqual(getBuildLogNote({ cwd, sessionManager: ownSession }), {
    key: newest,
    line: "Build Log: logs/build_2.log",
  });
});

test("getBuildLogNote drops a newer log that another session wrote", (t) => {
  const cwd = logsDir(t);
  writeLog(cwd, "build_1.log", { sessionId: "own-session", ageSeconds: 120 });
  writeLog(cwd, "build_2.log", { sessionId: "other-session" });

  assert.equal(getBuildLogNote({ cwd, sessionManager: ownSession }), undefined);
});

test("getBuildLogNote keeps an unstamped log visible", (t) => {
  const cwd = logsDir(t);
  const manual = writeLog(cwd, "build_1.log");

  assert.deepEqual(getBuildLogNote({ cwd, sessionManager: ownSession }), {
    key: manual,
    line: "Build Log: logs/build_1.log",
  });
  assert.deepEqual(getBuildLogNote({ cwd }), { key: manual, line: "Build Log: logs/build_1.log" });
});

test("getBuildLogNote keeps an unknown own session loose", (t) => {
  const cwd = logsDir(t);
  const stamped = writeLog(cwd, "build_1.log", { sessionId: "other-session" });

  assert.deepEqual(getBuildLogNote({ cwd, sessionManager: {} }), {
    key: stamped,
    line: "Build Log: logs/build_1.log",
  });
});

test("getBuildLogNote keeps the key when a build appends to the log", (t) => {
  const cwd = logsDir(t);
  const file = writeLog(cwd, "build_1.log", { sessionId: "own-session" });
  const ctx = { cwd, sessionManager: ownSession };
  const source: Source = { name: "buildLog", midTurn: true, get: getBuildLogNote };
  const keys = {};

  assert.deepEqual(collectNewLines([source], keys, ctx), ["Build Log: logs/build_1.log"]);

  // An append during the build changes the mtime but not the key, so nothing posts again.
  const first = getBuildLogNote(ctx);
  writeFileSync(file, "Session-Id: own-session\nmore output\n");
  assert.deepEqual(collectNewLines([source], keys, ctx), []);
  assert.equal(getBuildLogNote(ctx)?.key, first?.key);
});

test("getBuildLogNote ignores a log older than the freshness window", (t) => {
  const cwd = logsDir(t);
  writeLog(cwd, "build_1.log", { sessionId: "own-session", ageSeconds: 600 });

  assert.equal(getBuildLogNote({ cwd, sessionManager: ownSession }), undefined);
});

test("getBuildLogNote returns nothing without a log or a cwd", (t) => {
  const cwd = logsDir(t);

  assert.equal(getBuildLogNote({ cwd, sessionManager: ownSession }), undefined);
  assert.equal(getBuildLogNote({ sessionManager: ownSession }), undefined);
});

test("collectNewLines posts a line once per key, per source", () => {
  const keys = {};
  const sources: Source[] = [
    { name: "model", midTurn: false, get: () => ({ key: "m", line: "Model: m" }) },
    { name: "buildLog", midTurn: true, get: () => ({ key: "m", line: "Build Log: b" }) },
  ];

  assert.deepEqual(collectNewLines(sources, keys, {}), ["Model: m", "Build Log: b"]);
  assert.deepEqual(collectNewLines(sources, keys, {}), []);
  assert.deepEqual(keys, { model: "m", buildLog: "m" });
});

test("collectNewLines posts again when a key changes", () => {
  const keys = {};
  let key = "a";
  const source: Source = { name: "model", midTurn: false, get: () => ({ key, line: `Model: ${key}` }) };

  assert.deepEqual(collectNewLines([source], keys, {}), ["Model: a"]);
  key = "b";
  assert.deepEqual(collectNewLines([source], keys, {}), ["Model: b"]);
});

test("collectNewLines skips a source that throws", () => {
  const keys = {};
  const sources: Source[] = [
    {
      name: "bad",
      midTurn: false,
      get: () => {
        throw new Error("boom");
      },
    },
    { name: "model", midTurn: false, get: () => ({ key: "m", line: "Model: m" }) },
  ];

  assert.deepEqual(collectNewLines(sources, keys, {}), ["Model: m"]);
  assert.deepEqual(keys, { model: "m" });
});

test("formatPiNote puts the lines in one pi-note block", () => {
  assert.equal(formatPiNote(["a", "b"]), "<pi-note>\na\nb\n</pi-note>");
});
