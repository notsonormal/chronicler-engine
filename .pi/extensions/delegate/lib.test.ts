// Unit tests for the delegate helpers. Run with:
// node --test .pi/extensions/delegate/lib.test.ts

import assert from "node:assert/strict";
import test from "node:test";

import {
	childCommand,
	formatResult,
	mergeRoles,
	parseRole,
	pickModel,
	pickSplit,
	resultStatus,
	summarizeEntries,
	taskText,
} from "./lib.ts";

test("parseRole reads model, thinking and a one-line tools array", () => {
	const role = parseRole(
		'---\nname: reviewer\nmodel: anthropic/claude-opus-5-5\nthinking: high\ntools: ["read", "ls", "bash"]\nextensions: ["/x"]\n---\n',
	);
	assert.deepEqual(role, {
		name: "reviewer",
		model: "anthropic/claude-opus-5-5",
		thinking: "high",
		tools: ["read", "ls", "bash"],
	});
});

test("parseRole reads a tools array over several lines", () => {
	const role = parseRole(
		'---\nname: researcher\nmodel: m/x\nthinking: high\ntools:\n  [\n    "read",\n    "web_search",\n  ]\n---\n',
	);
	assert.deepEqual(role?.tools, ["read", "web_search"]);
});

test("parseRole leaves tools unset when the file has none", () => {
	const role = parseRole("---\nname: implementer\nmodel: m/x\n---\nbody");
	assert.equal(role?.tools, undefined);
	assert.equal(role?.thinking, undefined);
});

test("parseRole reads a YAML block list and a comma list", () => {
	assert.deepEqual(parseRole("---\nname: a\nmodel: m/x\ntools:\n  - read\n  - 'bash'\nthinking: low\n---")?.tools, ["read", "bash"]);
	assert.deepEqual(parseRole("---\nname: a\nmodel: m/x\ntools: read, bash\n---")?.tools, ["read", "bash"]);
});

test("parseRole does not take the next key for a tool when tools is empty", () => {
	const role = parseRole("---\nname: a\nmodel: m/x\ntools:\nthinking: high\n---");
	assert.equal(role?.tools, undefined);
	assert.equal(role?.thinking, "high");
});

test("parseRole rejects a role whose tools array is not closed", () => {
	assert.equal(parseRole('---\nname: a\nmodel: m/x\ntools: ["read", "bash"\nextensions: ["/x"]\n---'), null);
});

test("parseRole rejects a file with no model", () => {
	assert.equal(parseRole("---\nname: scout\n---\n"), null);
	assert.equal(parseRole("no frontmatter"), null);
});

test("mergeRoles lets a later source override a role of the same name", () => {
	const roles = mergeRoles([["---\nname: a\nmodel: global/x\n---"], ["---\nname: a\nmodel: project/y\n---"]]);
	assert.equal(roles.get("a")?.model, "project/y");
});

test("childCommand starts a new session and closes the pane only on a clean exit", () => {
	const cmd = childCommand({
		jobDir: "/j/1",
		role: { name: "scout", model: "m/x", thinking: "low", tools: ["read", "bash"] },
		sessionId: "abc",
		resume: false,
		sessionName: "delegate · scout · it's",
		taskFile: "/j/1/task.md",
	});
	assert.match(cmd, /^PI_DELEGATE_JOB='\/j\/1' PLANNOTATOR_INBOX_TOOL=0 pi --model 'm\/x' --session-id 'abc'/);
	assert.match(cmd, /--thinking 'low' --tools 'read,bash' --name 'delegate · scout · it'\\''s' '@\/j\/1\/task.md'/);
	assert.match(cmd, /; code=\$\?; echo \$code > '\/j\/1\/exit'; \[ \$code -eq 0 \] && exit$/);
});

test("childCommand resumes with --session and keeps the old name", () => {
	const cmd = childCommand({
		jobDir: "/j",
		role: { name: "i", model: "m/x" },
		sessionId: "abc",
		resume: true,
		sessionName: "n",
		taskFile: "/j/t.md",
	});
	assert.match(cmd, /--session 'abc'/);
	assert.doesNotMatch(cmd, /--name|--session-id|--thinking|--tools /);
});

test("taskText lists the files and says the last message is returned", () => {
	const text = taskText("  Do X.  ", ["a.md", "b.md"]);
	assert.match(text, /^Do X\.\n\nFiles to read first:\n- a\.md\n- b\.md\n\n/);
	assert.match(text, /\n\nYour last message is returned to the agent that delegated this task, and this session then ends\.$/);
});

test("summarizeEntries takes the last assistant text and the peak context", () => {
	const entry = (text: string, input: number, cacheRead: number) => ({
		type: "message",
		message: {
			role: "assistant",
			content: [{ type: "thinking" }, { type: "text", text }],
			stopReason: "stop",
			usage: { input, cacheRead, output: 10, cost: { total: 0.5 } },
		},
	});
	const summary = summarizeEntries([
		entry("first", 100, 900),
		{ type: "message", message: { role: "user", content: "x" } },
		entry("", 50, 2000),
		entry("last", 10, 1500),
	]);
	assert.equal(summary.text, "last");
	assert.equal(summary.turns, 3);
	assert.equal(summary.peakContext, 2050);
	assert.equal(summary.outputTokens, 30);
	assert.equal(summary.cost, 1.5);
});

test("summarizeEntries counts only the entries of the resumed run", () => {
	const entry = (text: string) => ({ type: "message", message: { role: "assistant", content: text, usage: { input: 5 } } });
	const summary = summarizeEntries([entry("old run"), entry("new run"), entry("")], 1);
	assert.equal(summary.text, "new run");
	assert.equal(summary.turns, 2);
	assert.equal(summarizeEntries([entry("old run"), entry("")], 1).text, "");
});

test("a run that ended on a model error is a failed result", () => {
	const result = { text: "mid-run note", turns: 3, peakContext: 0, outputTokens: 0, cost: 0, finishedAt: 0 };
	assert.equal(resultStatus({ ...result, stopReason: "error" }), "failed");
	assert.equal(resultStatus({ ...result, stopReason: "stop" }), "done");
	const job = { id: "j", label: "l", role: { name: "r", model: "m" }, sessionId: "s", startedAt: 0 };
	assert.match(formatResult(job, { ...result, stopReason: "error" }, "/r"), /^Delegate failed \(model error\)/);
});

test("formatResult truncates a long result and names the full file", () => {
	const text = formatResult(
		{ id: "j1", label: "c5", role: { name: "implementer", model: "m/x", thinking: "high" }, sessionId: "s1", startedAt: 0 },
		{ text: "y".repeat(50), turns: 4, peakContext: 41000, outputTokens: 2500, cost: 0.0123, stopReason: "stop", finishedAt: 90000 },
		"/j/result.json",
		20,
	);
	assert.match(text, /^Delegate result · c5 \(j1\) · role implementer · m\/x:high\n1\.5 min · 4 turns · peak context 41k · output 3k · \$0\.012 · ended: stop\nSession s1\n/);
	assert.match(text, /y{20}\n\n\[truncated; full result in \/j\/result\.json\]$/);
});

test("pickSplit splits the largest pane along its long side", () => {
	assert.deepEqual(
		pickSplit([
			{ pane_id: "a", rect: { width: 80, height: 40 } },
			{ pane_id: "b", rect: { width: 174, height: 43 } },
		]),
		{ paneId: "b", direction: "right" },
	);
	assert.deepEqual(pickSplit([{ pane_id: "a", rect: { width: 80, height: 43 } }]), { paneId: "a", direction: "down" });
	assert.equal(pickSplit([]), null);
});

test("parseRole reads spare_model", () => {
	const role = parseRole("---\nname: implementer\nmodel: go/x\nspare_model: anthropic/s\n---\n");
	assert.equal(role?.spareModel, "anthropic/s");
});

const spareRole = { name: "implementer", model: "go/x", spareModel: "anthropic/s" };
const NOW = 1_000_000;
// 5h window 80% passed, 7d window 90% passed.
const limits = (fiveUtil: number, sevenUtil: number, extra = {}) => ({
	capturedAt: NOW - 60,
	fiveReset: NOW + 3600,
	fiveUtil,
	fiveStatus: "allowed",
	sevenReset: NOW + 0.1 * 7 * 86400,
	sevenUtil,
	sevenStatus: "allowed",
	...extra,
});

test("pickModel uses the spare model when both windows are under pace", () => {
	const pick = pickModel(spareRole, limits(0.5, 0.6), NOW);
	assert.equal(pick.model, "anthropic/s");
	assert.equal(pick.spare, true);
});

test("pickModel keeps the base model inside the 5h margin", () => {
	// 80% passed - 20% margin = 60%.
	assert.equal(pickModel(spareRole, limits(0.61, 0.6), NOW).model, "go/x");
	assert.equal(pickModel(spareRole, limits(0.59, 0.6), NOW).model, "anthropic/s");
});

test("pickModel keeps the base model when the 7d window is over pace", () => {
	assert.equal(pickModel(spareRole, limits(0.1, 0.95), NOW).model, "go/x");
});

test("pickModel keeps the base model on rejected status", () => {
	assert.equal(pickModel(spareRole, limits(0.1, 0.5, { fiveStatus: "rejected" }), NOW).model, "go/x");
	assert.equal(pickModel(spareRole, limits(0.1, 0.5, { sevenStatus: "rejected" }), NOW).model, "go/x");
});

test("pickModel keeps the base model on stale data of a running 5h window", () => {
	assert.equal(pickModel(spareRole, limits(0.1, 0.5, { capturedAt: NOW - 3600 }), NOW).model, "go/x");
});

test("pickModel treats an ended 5h window as empty, even with old data", () => {
	const state = limits(0.99, 0.5, { capturedAt: NOW - 20000, fiveReset: NOW - 10 });
	assert.equal(pickModel(spareRole, state, NOW).model, "anthropic/s");
});

test("pickModel keeps the base model without data or without a spare model", () => {
	assert.equal(pickModel(spareRole, undefined, NOW).model, "go/x");
	assert.equal(pickModel({ name: "scout", model: "go/x" }, limits(0, 0), NOW).spare, false);
});

test("pickModel keeps the base model when the 5h reset time is unknown", () => {
	// A state with no 5h reset says nothing about the 5h window, so the spare
	// model has no proven room.
	assert.equal(pickModel(spareRole, limits(0.5, 0.6, { fiveReset: undefined }), NOW).model, "go/x");
});
