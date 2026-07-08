import { test } from "node:test";
import assert from "node:assert/strict";
import { checkTaskSpec } from "./task-veto.ts";

test("Feature 1: empty task is blocked", () => {
	const r = checkTaskSpec({ task: "" });
	assert.equal(r.block, true);
	assert.match(r.reason, /worker minimum 500/);
});

test("Feature 1: 50-char task is blocked on delegate floor", () => {
	const r = checkTaskSpec({ task: "x".repeat(50) });
	assert.equal(r.block, true);
	assert.match(r.reason, /length 50/);
});

test("Feature 1: 70-char task is blocked (just under delegate floor)", () => {
	const r = checkTaskSpec({ task: "x".repeat(70) });
	assert.equal(r.block, true);
	assert.match(r.reason, /length 70/);
});

test("Feature 1: 100-char task with no header passes for delegate", () => {
	const r = checkTaskSpec({ task: "x".repeat(100), agent: "delegate" });
	assert.equal(r.block, false);
});

test("Feature 1: 100-char task with no header is blocked for worker", () => {
	const r = checkTaskSpec({ task: "x".repeat(100), agent: "worker" });
	assert.equal(r.block, true);
	assert.match(r.reason, /worker minimum 500/);
});

test("Feature 1: 400-char task with no header is blocked for worker", () => {
	const r = checkTaskSpec({ task: "x".repeat(400), agent: "worker" });
	assert.equal(r.block, true);
	assert.match(r.reason, /worker minimum 500/);
});

test("Feature 1: 600-char task with no header passes for worker", () => {
	const r = checkTaskSpec({ task: "x".repeat(600), agent: "worker" });
	assert.equal(r.block, false);
});

test("Feature 1: undefined agent defaults to worker floor", () => {
	const r = checkTaskSpec({ task: "x".repeat(400) });
	assert.equal(r.block, true);
	assert.match(r.reason, /worker minimum 500/);
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

test("Feature 1: scout agent uses delegate floor (not worker floor)", () => {
	const r = checkTaskSpec({ task: "x".repeat(100), agent: "scout" });
	assert.equal(r.block, false);
});

test("Feature 1: agent matching is case-insensitive", () => {
	const r = checkTaskSpec({ task: "x".repeat(100), agent: "WORKER" });
	assert.equal(r.block, true);
	assert.match(r.reason, /worker minimum 500/);
});

test("Feature 1: whitespace-only task is treated as empty", () => {
	const r = checkTaskSpec({ task: "   \n  \t  " });
	assert.equal(r.block, true);
	assert.match(r.reason, /length 0/);
});

test("Feature 1: leading/trailing whitespace is trimmed before length check", () => {
	const r = checkTaskSpec({
		task: "   " + "x".repeat(100) + "\n\n",
		agent: "delegate",
	});
	assert.equal(r.block, false);
});

test("Feature 1: short worker task gets worker-specific message (not delegate)", () => {
	// 70 chars < delegate floor (80) AND < worker floor (500). Must surface
	// the binding constraint for the agent being dispatched, not the generic
	// delegate floor message.
	const r = checkTaskSpec({ task: "x".repeat(70), agent: "worker" });
	assert.equal(r.block, true);
	assert.match(r.reason, /worker minimum 500/);
	assert.doesNotMatch(r.reason, /below minimum 80/);
});