// Smoke Break: nudges an agent that has gone down a rabbit hole.
//
// DeepSeek-class models keep taking defensible actions after the point where
// more work can no longer change the conclusion. Observed incidents in this
// repo were marathon-shaped (a 44-minute build chase interrupted manually
// with "What is the endgoal here?", and a 2.6 h / 233-tool-call run while the
// user was away). This extension appends one short, model-visible reflection
// prompt per elapsed interval — see docs/plans/pi-extension-smoke-break-nudge.md
// for the measured data and the decision history.
//
// Wiring:
// - before_agent_start anchors the turn (fires once per user-submitted prompt,
//   so any user input restarts the timer — parity with the original plugin's
//   UserPromptSubmit anchor). The interval is re-read here, so env edits apply
//   to the next turn without a restart.
// - tool_result checks the bucket and, on a new one, returns a content patch.
//   pi documents tool_result as "Can modify result": the patched result
//   becomes the toolResult message the model receives. Handler errors are
//   logged by pi while the agent continues, so no try/catch here — a throwing
//   handler simply leaves the tool result unchanged.
//
// Config: SMOKE_BREAK_INTERVAL_MS (milliseconds, default 30 min). A
// non-positive integer disables the extension; an unparseable value falls
// back to the default. See tracker.ts.
// @ts-nocheck

import { onToolComplete, readIntervalMs, startTurn, type TurnState } from "./tracker.ts";

export default function smokeBreak(pi) {
	let turn: TurnState | null = null;

	pi.on("before_agent_start", () => {
		const intervalMs = readIntervalMs(process.env.SMOKE_BREAK_INTERVAL_MS);
		turn = startTurn(Date.now(), intervalMs);
	});

	pi.on("tool_result", (event) => {
		const result = onToolComplete(turn, Date.now());
		turn = result.state;
		if (result.reminder === null) {
			return undefined;
		}
		return {
			content: [...event.content, { type: "text", text: result.reminder }],
		};
	});
}
