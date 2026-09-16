// Unit tests for the smoke-break tracker. Run with: node --test
// .pi/extensions/smoke-break/tracker.test.ts — Node 24 strips the types
// natively, so no build step or pi import is needed. The clock is faked by
// passing absolute millisecond values; nothing here reads the real clock.

import assert from "node:assert/strict";
import test from "node:test";

import {
	DEFAULT_INTERVAL_MS,
	formatReminder,
	onToolComplete,
	readIntervalMs,
	startTurn,
} from "./tracker.ts";

const THIRTY_MIN = 30 * 60 * 1000;

test("silent through bucket 0 (interval minus 1 ms)", () => {
	const state = startTurn(0, THIRTY_MIN);
	const result = onToolComplete(state, THIRTY_MIN - 1);
	assert.equal(result.reminder, null);
	assert.equal(result.state?.notifiedBucket, 0);
});

test("fires at exactly the interval, with the original ask and the blind disclaimer", () => {
	const state = startTurn(0, THIRTY_MIN);
	const result = onToolComplete(state, THIRTY_MIN);
	assert.match(result.reminder ?? "", /about 30 minutes\./);
	assert.match(result.reminder ?? "", /simply continue/);
	assert.match(result.reminder ?? "", /This is an automated message, posted every 30 minutes/);
	assert.match(result.reminder ?? "", /no direct knowledge of what is happening/);
	assert.equal(result.state?.notifiedBucket, 1);
});

test("one notification per bucket: silent after firing, fires again at the next interval", () => {
	let state = startTurn(0, THIRTY_MIN);

	assert.equal(onToolComplete(state, THIRTY_MIN).reminder !== null, true);
	const sameBucket = onToolComplete(state, THIRTY_MIN + 1);
	assert.equal(sameBucket.reminder, null);

	const secondBucket = onToolComplete(state, 2 * THIRTY_MIN);
	assert.match(secondBucket.reminder ?? "", /about 60 minutes/);

	const stillSameBucket = onToolComplete(state, 2 * THIRTY_MIN + 1);
	assert.equal(stillSameBucket.reminder, null);
});

test("a new turn resets elapsed time and the notified bucket", () => {
	let state = startTurn(0, THIRTY_MIN);
	onToolComplete(state, 2 * THIRTY_MIN);

	state = startTurn(2 * THIRTY_MIN, THIRTY_MIN);
	const result = onToolComplete(state, 2 * THIRTY_MIN + THIRTY_MIN - 1);
	assert.equal(result.reminder, null);

	const fired = onToolComplete(state, 2 * THIRTY_MIN + THIRTY_MIN);
	assert.match(fired.reminder ?? "", /about 30 minutes/);
});

test("elapsed label: singular 'minute' for a 60 s interval", () => {
	const state = startTurn(0, 60_000);
	const result = onToolComplete(state, 60_000);
	assert.match(result.reminder ?? "", /about 1 minute\./);
	assert.match(result.reminder ?? "", /posted every 1 minute /);
});

test("a custom interval is respected", () => {
	const state = startTurn(0, 120_000);
	assert.equal(onToolComplete(state, 119_999).reminder, null);
	const result = onToolComplete(state, 120_000);
	assert.match(result.reminder ?? "", /about 2 minutes/);
});

test("an interval change applies from the next turn; the run in progress keeps its interval", () => {
	let state = startTurn(0, 60_000);
	const first = onToolComplete(state, 60_000);
	assert.match(first.reminder ?? "", /about 1 minute/);

	// The run in progress keeps interval 60 s: still fires at +60 s multiples.
	const kept = onToolComplete(state, 120_000);
	assert.match(kept.reminder ?? "", /about 2 minutes/);

	// Next turn re-reads the interval: 120 s from the new anchor.
	state = startTurn(120_000, 120_000);
	assert.equal(onToolComplete(state, 120_000 + 60_000).reminder, null);
	const changed = onToolComplete(state, 120_000 + 120_000);
	assert.match(changed.reminder ?? "", /about 2 minutes/);
});

test("disabled (non-positive interval) yields no turn and stays silent", () => {
	assert.equal(startTurn(0, 0), null);
	assert.equal(startTurn(0, -5_000), null);
	assert.equal(startTurn(0, Number.NaN), null);
	assert.equal(onToolComplete(null, 123_456).reminder, null);
});

test("tool completions with no active turn are silent", () => {
	const result = onToolComplete(null, 0);
	assert.equal(result.reminder, null);
	assert.equal(result.state, null);
});

test("formatReminder label forms", () => {
	assert.match(formatReminder(1, 60_000), /about 1 minute\./);
	assert.match(formatReminder(2, 60_000), /about 2 minutes\./);
	assert.match(formatReminder(1, THIRTY_MIN), /about 30 minutes\./);
	assert.match(formatReminder(1, 60_000), /posted every 1 minute without user input/);
	assert.match(formatReminder(3, 90_000), /posted every 2 minutes without user input/);
});

test("readIntervalMs: unparseable falls back to default, non-positive disables", () => {
	assert.equal(readIntervalMs(undefined), DEFAULT_INTERVAL_MS);
	assert.equal(readIntervalMs(""), DEFAULT_INTERVAL_MS);
	assert.equal(readIntervalMs("   "), DEFAULT_INTERVAL_MS);
	assert.equal(readIntervalMs("abc"), DEFAULT_INTERVAL_MS);
	assert.equal(readIntervalMs("30.5"), DEFAULT_INTERVAL_MS);
	assert.equal(readIntervalMs("60000"), 60_000);
	assert.equal(readIntervalMs(" 60000 "), 60_000);
	assert.equal(readIntervalMs("0"), 0);
	assert.equal(readIntervalMs("-1000"), -1000);
});
