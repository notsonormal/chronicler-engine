import { test } from "node:test";
import assert from "node:assert/strict";
import { onSessionStart, onTurnEnd } from "./budget.ts";

interface Sent {
	customType: string;
	content: string;
}

function fakePi(): { pi: any; sent: Sent[] } {
	const sent: Sent[] = [];
	const pi = {
		sendMessage(msg: Sent) {
			sent.push(msg);
		},
	};
	return { pi, sent };
}

test("Budget: 10 turns / fast session — no steer", () => {
	const { pi, sent } = fakePi();
	onSessionStart();
	for (let i = 0; i < 10; i++) onTurnEnd(pi);
	assert.equal(sent.length, 0);
});

test("Budget: soft nudge fires exactly once at turn 50", () => {
	const { pi, sent } = fakePi();
	onSessionStart();
	for (let i = 0; i < 51; i++) onTurnEnd(pi);
	assert.equal(sent.length, 1);
	assert.equal(sent[0].customType, "subagent-guardrails:nudge");
	assert.match(sent[0].content, /progress_update/);
});

test("Budget: hard steer fires at turn 100 after soft nudge at 50", () => {
	const { pi, sent } = fakePi();
	onSessionStart();
	for (let i = 0; i < 101; i++) onTurnEnd(pi);
	assert.equal(sent.length, 2);
	assert.equal(sent[0].customType, "subagent-guardrails:nudge");
	assert.equal(sent[1].customType, "subagent-guardrails:stop");
	assert.match(sent[1].content, /BUDGET EXCEEDED/);
});

test("Budget: after hard steer, additional turns do not re-fire", () => {
	const { pi, sent } = fakePi();
	onSessionStart();
	for (let i = 0; i < 101; i++) onTurnEnd(pi);
	const after = sent.length;
	for (let i = 0; i < 50; i++) onTurnEnd(pi);
	assert.equal(sent.length, after);
});
