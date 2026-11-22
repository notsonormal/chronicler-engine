// Tests of the pi wiring: the model note posts once and again after a session
// start, and only the midTurn sources post inside a turn. Run with:
// node --test .pi/extensions/mrn-context/index.test.ts

import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import * as path from "node:path";
import test from "node:test";

import register from "./index.ts";

function fakePi() {
  const handlers = new Map<string, Function>();
  return { handlers, on: (name: string, fn: Function) => handlers.set(name, fn) };
}

// A registered extension, with its cwd and model under the test's control.
function setup(t: test.TestContext) {
  const cwd = mkdtempSync(path.join(tmpdir(), "mrn-context-"));
  mkdirSync(path.join(cwd, "logs"), { recursive: true });
  t.after(() => rmSync(cwd, { recursive: true, force: true }));

  const pi = fakePi();
  register(pi);
  const ctx = {
    cwd,
    model: { provider: "anthropic", id: "claude-opus-5-5" },
    sessionManager: { getSessionId: () => "own-session" },
  };
  const prompt = () => pi.handlers.get("before_agent_start")?.({}, ctx);
  const afterSessionStart = () => pi.handlers.get("session_start")?.();
  const midTurn = (messages: unknown[] = []) => pi.handlers.get("context")?.({ messages }, ctx);

  // The note keys live for the process, so each test starts from a fresh session.
  afterSessionStart();
  return { cwd, ctx, prompt, afterSessionStart, midTurn };
}

function content(result: any): string {
  return result?.message?.content ?? "";
}

test("the model note posts once, and again after a session start", (t) => {
  const { prompt, afterSessionStart } = setup(t);

  assert.match(content(prompt()), /Model: anthropic\/claude-opus-5-5/);
  assert.equal(prompt(), undefined);

  afterSessionStart();
  assert.match(content(prompt()), /Model: anthropic\/claude-opus-5-5/);
});

test("the model change is not posted inside a turn", (t) => {
  const { ctx, prompt, midTurn } = setup(t);
  assert.match(content(prompt()), /Model: anthropic\/claude-opus-5-5/);

  ctx.model = { provider: "opencode-go", id: "deepseek-v4.1-flash" };
  assert.equal(midTurn(), undefined);
  assert.match(content(prompt()), /Model: opencode-go\/deepseek-v4.1-flash/);
});

test("a new build log is added to the messages inside a turn", (t) => {
  const { cwd, midTurn } = setup(t);

  writeFileSync(path.join(cwd, "logs", "build_1.log"), "Session-Id: own-session\n");
  const messages = [{ role: "user", content: "hello" }];
  const result = midTurn(messages);

  assert.equal(result.messages.length, 2);
  assert.equal(result.messages[0], messages[0]);
  assert.deepEqual(
    { role: result.messages[1].role, customType: result.messages[1].customType, display: result.messages[1].display },
    { role: "custom", customType: "pi-note", display: false },
  );
  assert.match(result.messages[1].content, /Build Log: logs\/build_1\.log/);
});

test("nothing is posted when no source has a new key", (t) => {
  const { prompt, midTurn } = setup(t);
  prompt();

  assert.equal(midTurn(), undefined);
});
