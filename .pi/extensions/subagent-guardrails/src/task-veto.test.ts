import { test } from "node:test";
import assert from "node:assert/strict";
import { checkTaskSpec } from "./task-veto.ts";

test("Feature 1: empty task is blocked", () => {
	const r = checkTaskSpec({ task: "" });
	assert.equal(r.block, true);
	assert.match(r.reason, /200/);
});

test("Feature 1: 100-char task without header is blocked on length", () => {
	const r = checkTaskSpec({ task: "x".repeat(100) });
	assert.equal(r.block, true);
	assert.match(r.reason, /length 100/);
});

test("Feature 1: 250-char task without header marker is blocked on header", () => {
	const r = checkTaskSpec({ task: "y".repeat(250), agent: "delegate" });
	assert.equal(r.block, true);
	assert.match(r.reason, /missing a recognized header/);
});

test("Feature 1: worker task under 800 chars with header is blocked on worker floor", () => {
	// 18 (header) + 1 + 600 = 619 chars: passes 200 floor, fails worker 800.
	const r = checkTaskSpec({ task: "# Task for worker\n" + "x".repeat(600) });
	assert.equal(r.block, true);
	assert.match(r.reason, /worker minimum 800/);
});

test("Feature 1: worker task with header + 900 chars passes", () => {
	const r = checkTaskSpec({ task: "# Task for worker\n" + "x".repeat(900) });
	assert.equal(r.block, false);
});

test("Feature 1: undefined agent defaults to worker floor", () => {
	const r = checkTaskSpec({ task: "# Task for worker\n" + "x".repeat(500) });
	assert.equal(r.block, true);
	assert.match(r.reason, /worker minimum 800/);
});

test("Feature 1: delegate task with header + 250 chars passes", () => {
	const r = checkTaskSpec({
		task: "# Task for delegate\n" + "x".repeat(250),
		agent: "delegate",
	});
	assert.equal(r.block, false);
});

test("Feature 1: bare `Task:` marker is accepted", () => {
	const r = checkTaskSpec({
		task: "Task:\n" + "x".repeat(250),
		agent: "delegate",
	});
	assert.equal(r.block, false);
});

test("Feature 1: `# Task for scout` header is accepted", () => {
	const r = checkTaskSpec({
		task: "# Task for scout\n" + "x".repeat(250),
		agent: "scout",
	});
	assert.equal(r.block, false);
});

test("Feature 1: non-string task is treated as empty and blocked", () => {
	const r = checkTaskSpec({ task: undefined });
	assert.equal(r.block, true);
});

test("Feature 1: all veto reasons route back to AGENTS.md", () => {
	const r = checkTaskSpec({ task: "" });
	if (!r.block) throw new Error("expected block");
	assert.match(r.reason, /AGENTS\.md/);
});

test("Feature 1: management action bypasses task validation", () => {
	const r = checkTaskSpec({ action: "list" });
	assert.equal(r.block, false);
});

test("Feature 1: `action: 'status'` bypasses task validation", () => {
	const r = checkTaskSpec({ action: "status" });
	assert.equal(r.block, false);
});

test("Feature 1: `action: 'interrupt'` bypasses even with empty task", () => {
	const r = checkTaskSpec({ action: "interrupt", task: "" });
	assert.equal(r.block, false);
});

test("Feature 1: execution mode (no action) still validates task", () => {
	const r = checkTaskSpec({ task: "", agent: "worker" });
	assert.equal(r.block, true);
});
