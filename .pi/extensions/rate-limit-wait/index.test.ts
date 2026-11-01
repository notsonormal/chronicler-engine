// Unit tests for the wait decision. Run with:
// node --test .pi/extensions/rate-limit-wait/index.test.ts

import assert from "node:assert/strict";
import test from "node:test";

import { decide } from "./index.ts";

const NOW = 1_000_000;

test("decide stops when the 7-day limit is reached", () => {
	assert.deepEqual(decide({ capturedAt: NOW, sevenStatus: "rejected" }, NOW), {
		action: "stop",
		reason: "7-day limit reached (resets unknown); not waiting",
	});
	assert.equal(decide({ capturedAt: NOW, sevenUtil: 1 }, NOW).action, "stop");
});

test("decide retries in 5 minutes when the 5h reset time is unknown", () => {
	assert.deepEqual(decide({ capturedAt: NOW }, NOW), {
		action: "wait",
		untilS: NOW + 300,
		reason: "5h reset time unknown; retry in 5 min",
	});
});

test("decide retries in 1 minute when the 5h reset time has passed", () => {
	assert.deepEqual(decide({ capturedAt: NOW, fiveReset: NOW - 1, fiveUtil: 0.95 }, NOW), {
		action: "wait",
		untilS: NOW + 60,
		reason: "5h reset time has passed; retry in 1 min",
	});
});

test("decide stops when the 5h window is not near full", () => {
	assert.deepEqual(decide({ capturedAt: NOW, fiveReset: NOW + 600, fiveUtil: 0.5 }, NOW), {
		action: "stop",
		reason: "5h window only 50% used; this is not the 5h limit",
	});
});

test("decide waits for the 5h reset when the 5h limit is reached", () => {
	assert.deepEqual(decide({ capturedAt: NOW, fiveReset: NOW + 1000, fiveUtil: 0.95 }, NOW), {
		action: "wait",
		untilS: NOW + 1060,
		reason: "5h limit reached",
	});
	assert.equal(
		decide({ capturedAt: NOW, fiveReset: NOW + 100000, fiveStatus: "rejected" }, NOW).untilS,
		NOW + 5 * 3600 + 15 * 60,
	);
});
