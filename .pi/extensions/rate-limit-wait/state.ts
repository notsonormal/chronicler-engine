// The Anthropic rate-limit state file: the last `anthropic-ratelimit-unified-*`
// headers that any pi process saw. rate-limit-wait writes it; delegate reads it.
// Times are Unix epoch seconds; utilization is a fraction (1 = full).

import { mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";

export const STATE_FILE =
	process.env.PI_RATE_LIMIT_WAIT_STATE ?? join(homedir(), ".pi", "agent", "rate-limit-wait.json");

export const FIVE_HOUR_S = 5 * 3600;
export const SEVEN_DAY_S = 7 * 24 * 3600;

export interface LimitState {
	capturedAt: number;
	fiveReset?: number;
	fiveUtil?: number;
	fiveStatus?: string;
	sevenReset?: number;
	sevenUtil?: number;
	sevenStatus?: string;
}

export function readState(file = STATE_FILE): LimitState | undefined {
	try {
		return JSON.parse(readFileSync(file, "utf8")) as LimitState;
	} catch {
		return undefined;
	}
}

// Other processes read the file at any time, so a write must never be partial.
export function writeState(state: LimitState, file = STATE_FILE): void {
	try {
		mkdirSync(dirname(file), { recursive: true });
		const tmp = `${file}.${process.pid}.tmp`;
		writeFileSync(tmp, JSON.stringify(state, null, 2));
		renameSync(tmp, file);
	} catch {
		// The state is a hint. A failed write must not break the request.
	}
}

// A missing value is unknown, not zero.
export function pct(fraction: number | undefined): string {
	return fraction === undefined ? "?" : `${Math.round(fraction * 100)}%`;
}

// A reset with no time is not an ended window: it is no data.
export function windowEnded(reset: number | undefined, nowS: number): boolean {
	return reset !== undefined && reset <= nowS;
}

function num(value: string | undefined): number | undefined {
	if (value === undefined) return undefined;
	const n = Number(value);
	return Number.isFinite(n) ? n : undefined;
}

// Undefined when the response carries no unified rate-limit headers.
export function stateFromHeaders(headers: Record<string, string>, nowS: number): LimitState | undefined {
	const fiveReset = num(headers["anthropic-ratelimit-unified-5h-reset"]);
	if (fiveReset === undefined) return undefined;
	return {
		capturedAt: Math.floor(nowS),
		fiveReset,
		fiveUtil: num(headers["anthropic-ratelimit-unified-5h-utilization"]),
		fiveStatus: headers["anthropic-ratelimit-unified-5h-status"],
		sevenReset: num(headers["anthropic-ratelimit-unified-7d-reset"]),
		sevenUtil: num(headers["anthropic-ratelimit-unified-7d-utilization"]),
		sevenStatus: headers["anthropic-ratelimit-unified-7d-status"],
	};
}
