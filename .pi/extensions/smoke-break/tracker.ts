// Smoke Break tracker: pure turn-timing logic for the pi extension.
//
// Ported from the smoke-break Codex plugin (ElKornacio, MIT), src/turn-tracker.mjs.
// The pi port drops the session-keyed Map and the eviction cap: pi rebinds
// extension instances per session, so one closure record per instance is
// already session-scoped. The bucket algorithm itself is verbatim.
//
// Firing is anchored to tool completion on purpose: a reminder can only ever
// surface when the agent is actively working, never while a session sits idle
// or open overnight (multi-hour idles observed in real session data).

// 30 minutes, from measured session data: the confirmed rabbit-hole runs were
// marathon-shaped (44 min and 2.6 h); only ~6% of healthy runs reach 30 min.
export const DEFAULT_INTERVAL_MS = 30 * 60 * 1000;

export interface TurnState {
	readonly startedAt: number;
	notifiedBucket: number;
	readonly intervalMs: number;
}

export interface ToolCompleteResult {
	state: TurnState | null;
	reminder: string | null;
}

// One interval per turn: the value captured at turn start is kept for the
// whole turn, so an env edit applies from the next turn, not mid-run.
export function startTurn(now: number, intervalMs: number): TurnState | null {
	if (!Number.isSafeInteger(intervalMs) || intervalMs <= 0) {
		return null;
	}
	return { startedAt: now, notifiedBucket: 0, intervalMs };
}

export function onToolComplete(state: TurnState | null, now: number): ToolCompleteResult {
	if (state === null) {
		return { state, reminder: null };
	}

	const elapsedMs = Math.max(0, now - state.startedAt);
	const bucket = Math.floor(elapsedMs / state.intervalMs);
	if (bucket === 0 || bucket <= state.notifiedBucket) {
		return { state, reminder: null };
	}

	state.notifiedBucket = bucket;
	return { state, reminder: formatReminder(bucket, state.intervalMs) };
}

// Reminder wording: the original plugin's text verbatim (ElKornacio
// smoke-break, turn-tracker.mjs #finishTool), plus a blind-disclaimer suffix
// the original lacks. The disclaimer states the cadence and that the
// checkpoint has no knowledge of the session, so the model does not read the
// nudge as a signal of trouble.
export function formatReminder(bucket: number, intervalMs: number): string {
	const elapsedMinutes = Math.max(1, Math.round((bucket * intervalMs) / 60_000));
	const cadenceMinutes = Math.max(1, Math.round(intervalMs / 60_000));
	const elapsedLabel = elapsedMinutes === 1 ? "minute" : "minutes";
	const cadenceLabel = cadenceMinutes === 1 ? "minute" : "minutes";
	return (
		`Smoke break: this turn has been running for about ${elapsedMinutes} ${elapsedLabel}. ` +
		"This is only a gentle checkpoint. Consider whether the work is progressing reasonably and " +
		"roughly according to plan. If so, or if any deviation seems modest, simply continue. " +
		"Only if it appears substantially off course or the time spent feels disproportionate, " +
		"consider whether changing approach or asking the user would help. " +
		`This is an automated message, posted every ${cadenceMinutes} ${cadenceLabel} without user input. ` +
		"It has no direct knowledge of what is happening in the session."
	);
}

// SMOKE_BREAK_INTERVAL_MS semantics: unparseable (non-numeric, non-integer)
// falls back to the default, mirroring the original's invalid-value behaviour;
// a non-positive integer disables the extension (startTurn yields null).
export function readIntervalMs(raw: string | undefined): number {
	if (raw === undefined || raw.trim() === "") {
		return DEFAULT_INTERVAL_MS;
	}
	const value = Number(raw);
	if (!Number.isSafeInteger(value)) {
		return DEFAULT_INTERVAL_MS;
	}
	return value;
}
