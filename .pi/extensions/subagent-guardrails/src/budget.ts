// Feature 2 (#12): Time + turn budget.
// Forked subagent-only. Absolute thresholds, single tier for all sessions.
// Steers the worker when wall-time or turn-count crosses the threshold.
//
// State is held in a module-level closure scoped to the current forked
// session. `session_start` with reason "fork" resets it (the extension
// reloads on fork, so module state is fresh per child process).

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

const SOFT_MINUTES = 15;
const SOFT_TURNS = 50;
const HARD_MINUTES = 30;
const HARD_TURNS = 100;

interface BudgetState {
	startMs: number;
	turnCount: number;
	softFired: boolean;
	hardFired: boolean;
}

let state: BudgetState | null = null;

export function onSessionStart(): void {
	state = {
		startMs: Date.now(),
		turnCount: 0,
		softFired: false,
		hardFired: false,
	};
}

export function onTurnEnd(pi: ExtensionAPI): void {
	if (!state) return;
	state.turnCount += 1;

	const elapsedMin = (Date.now() - state.startMs) / 60000;
	const mins = Math.floor(elapsedMin);

	if (!state.hardFired && (elapsedMin >= HARD_MINUTES || state.turnCount >= HARD_TURNS)) {
		state.hardFired = true;
		state.softFired = true;
		pi.sendMessage(
			{
				customType: "subagent-guardrails:stop",
				content:
					`BUDGET EXCEEDED (${mins} min / ${state.turnCount} turns). ` +
					"Stop all work. Return a focused summary of what is complete and incomplete, " +
					"plus the next concrete step. Do not start a new subtask.",
				display: true,
			},
			{ deliverAs: "steer", triggerTurn: true },
		);
		return;
	}

	if (!state.softFired && (elapsedMin >= SOFT_MINUTES || state.turnCount >= SOFT_TURNS)) {
		state.softFired = true;
		pi.sendMessage(
			{
				customType: "subagent-guardrails:nudge",
				content:
					`You have been running for ${mins} min over ${state.turnCount} turns. ` +
					"Send a one-line progress summary to your supervisor via contact_supervisor " +
					"with reason 'progress_update', then continue unless the task is genuinely " +
					"larger than 1 story point.",
				display: true,
			},
			{ deliverAs: "steer", triggerTurn: true },
		);
	}
}
