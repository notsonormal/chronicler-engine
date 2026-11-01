// Unit tests for the rate-limit state. Run with:
// node --test .pi/extensions/rate-limit-wait/state.test.ts

import assert from "node:assert/strict";
import test from "node:test";

import { stateFromHeaders } from "./state.ts";

const NOW = 1_000_000;

test("stateFromHeaders reads the unified rate-limit headers", () => {
	assert.deepEqual(
		stateFromHeaders(
			{
				"anthropic-ratelimit-unified-5h-reset": "1700000000",
				"anthropic-ratelimit-unified-5h-utilization": "0.42",
				"anthropic-ratelimit-unified-5h-status": "allowed",
				"anthropic-ratelimit-unified-7d-reset": "1700500000",
				"anthropic-ratelimit-unified-7d-utilization": "0.1",
				"anthropic-ratelimit-unified-7d-status": "allowed",
			},
			NOW + 0.7,
		),
		{
			capturedAt: NOW,
			fiveReset: 1700000000,
			fiveUtil: 0.42,
			fiveStatus: "allowed",
			sevenReset: 1700500000,
			sevenUtil: 0.1,
			sevenStatus: "allowed",
		},
	);
	assert.equal(stateFromHeaders({}, NOW), undefined);
});
