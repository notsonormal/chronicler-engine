import { test } from "node:test";
import assert from "node:assert/strict";
import { isSubagentSession } from "./subagent-detection.ts";

// Build a minimal SessionManagerLike stub with only the fields
// isSubagentSession reads. Tests override per-case.
function makeSm(opts: {
	name?: string | undefined;
	parentSession?: string | null | undefined;
}): Parameters<typeof isSubagentSession>[0] {
	const header = opts.parentSession === null
		? null
		: { type: "session" as const, id: "test-id", timestamp: "t", cwd: "/cwd", parentSession: opts.parentSession };
	return {
		getSessionName: () => opts.name,
		getHeader: () => header,
	};
}

test("Detection: PI_SUBAGENT_CHILD=1 env var alone triggers subagent", () => {
	process.env.PI_SUBAGENT_CHILD = "1";
	try {
		const sm = makeSm({ name: undefined, parentSession: undefined });
		assert.equal(isSubagentSession(sm), true);
	} finally {
		delete process.env.PI_SUBAGENT_CHILD;
	}
});

test("Detection: session name 'subagent-...' alone triggers subagent", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	const sm = makeSm({ name: "subagent-worker-abc-1", parentSession: undefined });
	assert.equal(isSubagentSession(sm), true);
});

test("Detection: session name 'subagent-' prefix matches delegate role too", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	const sm = makeSm({ name: "subagent-delegate-xyz-0" });
	assert.equal(isSubagentSession(sm), true);
});

test("Detection: parentSession header alone triggers subagent (covers /fork)", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	const sm = makeSm({ name: undefined, parentSession: "/path/to/parent.jsonl" });
	assert.equal(isSubagentSession(sm), true);
});

test("Detection: parentSession empty string does NOT trigger", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	const sm = makeSm({ name: undefined, parentSession: "" });
	assert.equal(isSubagentSession(sm), false);
});

test("Detection: no signals → not a subagent", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	const sm = makeSm({ name: undefined, parentSession: undefined });
	assert.equal(isSubagentSession(sm), false);
});

test("Detection: plain session name 'my-session' without parentSession does NOT trigger", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	const sm = makeSm({ name: "my-session", parentSession: undefined });
	assert.equal(isSubagentSession(sm), false);
});

test("Detection: getHeader() returning null does NOT crash and does NOT trigger", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	const sm = makeSm({ name: undefined, parentSession: null });
	assert.equal(isSubagentSession(sm), false);
});

test("Detection: getSessionName() throwing does NOT crash (falls through to header check)", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	const sm = {
		getSessionName: () => {
			throw new Error("not initialized");
		},
		getHeader: () => ({ type: "session", id: "x", timestamp: "t", cwd: "/c", parentSession: "/p.jsonl" }),
	};
	assert.equal(isSubagentSession(sm), true);
});

test("Detection: getHeader() throwing does NOT crash (returns false)", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	const sm = {
		getSessionName: () => undefined,
		getHeader: () => {
			throw new Error("not initialized");
		},
	};
	assert.equal(isSubagentSession(sm), false);
});

test("Detection: 'subagent-' prefix must be exact (no false positive for 'subagentfoo')", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	const sm = makeSm({ name: "subagentfoo-session", parentSession: undefined });
	assert.equal(isSubagentSession(sm), false);
});

test("Detection: 'subagent-chat-*' interactive parent name does NOT trigger (false positive guard)", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	// pi names interactive parents "subagent-chat-{id}" — these are primaries,
	// not subagents, despite the prefix. Without this carve-out, primaries
	// were misclassified and pi-agent-core returned cryptic
	// "Tool subagent not found" when they tried to fire a subagent.
	const sm = makeSm({ name: "subagent-chat-019f372d", parentSession: undefined });
	assert.equal(isSubagentSession(sm), false);
});

test("Detection: 'subagent-chat-*' still triggers if parentSession IS set (e.g. /fork of a chat session)", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	// If a chat session was forked, the parentSession signal is authoritative
	// and overrides the name carve-out.
	const sm = makeSm({ name: "subagent-chat-019f372d", parentSession: "/path/to/parent.jsonl" });
	assert.equal(isSubagentSession(sm), true);
});

test("Detection: other subagent-* types (scout, planner, etc.) still trigger", () => {
	delete process.env.PI_SUBAGENT_CHILD;
	// Future subagent roles added by pi-subagents must keep working.
	const sm = makeSm({ name: "subagent-scout-abc-1", parentSession: undefined });
	assert.equal(isSubagentSession(sm), true);
});
