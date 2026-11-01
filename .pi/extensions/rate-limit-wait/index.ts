/**
 * rate-limit-wait: hold a run that hit the Anthropic 5-hour limit, then continue it.
 *
 * Every successful Anthropic response carries `anthropic-ratelimit-unified-*` headers.
 * The extension writes them to a state file that all pi processes share, so a delegate
 * that fails on its first request still knows the reset time.
 *
 * When a run settles with the Anthropic `rate_limit_error` (after pi's own short
 * retries), the extension waits in `agent_before_settle` until the 5-hour window resets,
 * then adds one message and continues the run. A 7-day limit is not waited for.
 *
 * `/rate-wait` shows the state, `/rate-wait now` ends the wait and retries at once,
 * `/rate-wait cancel` ends the wait and lets the run stop.
 */
import { type LimitState, readState, stateFromHeaders, writeState } from "./state.ts";

const STATUS_KEY = "rate-limit-wait";
const RATE_LIMIT_ERROR = /rate_limit_error|exceed your account's rate limit/i;
const NEAR_FULL = 0.9;
const RESET_MARGIN_S = 60;
const UNKNOWN_RETRY_S = 300;
const PASSED_RETRY_S = 60;
const MAX_WAIT_S = 5 * 3600 + 15 * 60;
const MAX_WAITS_PER_RUN = 8;

type Decision =
	| { action: "wait"; untilS: number; reason: string }
	| { action: "stop"; reason: string };

export function decide(state: LimitState | undefined, nowS: number): Decision {
	if (state?.sevenStatus === "rejected" || (state?.sevenUtil ?? 0) >= 1) {
		const when = state?.sevenReset ? new Date(state.sevenReset * 1000).toLocaleString() : "unknown";
		return { action: "stop", reason: `7-day limit reached (resets ${when}); not waiting` };
	}
	if (!state?.fiveReset) {
		return { action: "wait", untilS: nowS + UNKNOWN_RETRY_S, reason: "5h reset time unknown; retry in 5 min" };
	}
	if (state.fiveReset <= nowS) {
		return { action: "wait", untilS: nowS + PASSED_RETRY_S, reason: "5h reset time has passed; retry in 1 min" };
	}
	const nearFull = state.fiveStatus === "rejected" || (state.fiveUtil ?? 0) >= NEAR_FULL;
	if (!nearFull) {
		return {
			action: "stop",
			reason: `5h window only ${Math.round((state.fiveUtil ?? 0) * 100)}% used; this is not the 5h limit`,
		};
	}
	const untilS = Math.min(state.fiveReset + RESET_MARGIN_S, nowS + MAX_WAIT_S);
	return { action: "wait", untilS, reason: "5h limit reached" };
}

function lastAssistant(messages: any[]): any | undefined {
	for (let i = messages.length - 1; i >= 0; i--) {
		if (messages[i]?.role === "assistant") return messages[i];
	}
	return undefined;
}

function formatLeft(seconds: number): string {
	const s = Math.max(0, Math.round(seconds));
	const h = Math.floor(s / 3600);
	const m = Math.floor((s % 3600) / 60);
	const r = s % 60;
	return h > 0 ? `${h}h${String(m).padStart(2, "0")}m` : `${m}m${String(r).padStart(2, "0")}s`;
}

export default function (pi: any) {
	let waitsThisRun = 0;
	let command: "now" | "cancel" | undefined;
	let waiting = false;

	pi.on("after_provider_response", (event: { status: number; headers: Record<string, string> }) => {
		const state = stateFromHeaders(event.headers ?? {}, Date.now() / 1000);
		if (state) writeState(state);
	});

	pi.on("agent_settled", () => {
		waitsThisRun = 0;
	});

	pi.on("agent_before_settle", async (event: any, ctx: any) => {
		if (event.outcome !== "error") return;
		const msg = lastAssistant(event.context?.contextMessages ?? []);
		if (!msg || msg.stopReason !== "error" || msg.provider !== "anthropic") return;
		if (!RATE_LIMIT_ERROR.test(msg.errorMessage ?? "")) return;

		const nowS = Date.now() / 1000;
		const decision = decide(readState(), nowS);
		if (decision.action === "stop") {
			ctx.ui?.notify?.(`rate-limit-wait: ${decision.reason}`, "warning");
			return;
		}
		if (waitsThisRun >= MAX_WAITS_PER_RUN) {
			ctx.ui?.notify?.(`rate-limit-wait: gave up after ${MAX_WAITS_PER_RUN} waits`, "error");
			return;
		}
		waitsThisRun++;

		const until = new Date(decision.untilS * 1000);
		ctx.ui?.notify?.(
			`rate-limit-wait: ${decision.reason}. Waiting until ${until.toLocaleTimeString()} (/rate-wait now | cancel).`,
			"warning",
		);
		command = undefined;
		waiting = true;
		try {
			while (Date.now() / 1000 < decision.untilS) {
				if (command || ctx.signal?.aborted) break;
				const left = decision.untilS - Date.now() / 1000;
				ctx.ui?.setStatus?.(STATUS_KEY, `⏳ rate limit: ${formatLeft(left)} left`);
				await new Promise((resolve) => setTimeout(resolve, 1000));
			}
		} finally {
			waiting = false;
			ctx.ui?.setStatus?.(STATUS_KEY, undefined);
		}
		if (command === "cancel" || ctx.signal?.aborted) {
			ctx.ui?.notify?.("rate-limit-wait: wait cancelled", "info");
			return;
		}

		return {
			continue: true,
			entries: [
				{
					type: "custom_message",
					customType: STATUS_KEY,
					display: true,
					content:
						"The Anthropic rate limit stopped the previous request. The limit window has now reset. Continue the task from where you stopped.",
				},
			],
		};
	});

	pi.registerCommand("rate-wait", {
		description: "Show the Anthropic rate-limit state, or end a wait: /rate-wait [now|cancel]",
		handler: async (args: string, ctx: any) => {
			const arg = (args ?? "").trim();
			if (arg === "now" || arg === "cancel") {
				if (!waiting) {
					ctx.ui.notify("rate-limit-wait: no wait is running", "info");
					return;
				}
				command = arg;
				return;
			}
			const s = readState();
			if (!s) {
				ctx.ui.notify("rate-limit-wait: no Anthropic rate-limit headers recorded yet", "info");
				return;
			}
			const fmt = (t?: number) => (t ? new Date(t * 1000).toLocaleString() : "?");
			const pct = (u?: number) => (u === undefined ? "?" : `${Math.round(u * 100)}%`);
			ctx.ui.notify(
				`5h: ${pct(s.fiveUtil)} (${s.fiveStatus}), resets ${fmt(s.fiveReset)}\n` +
					`7d: ${pct(s.sevenUtil)} (${s.sevenStatus}), resets ${fmt(s.sevenReset)}\n` +
					`captured ${fmt(s.capturedAt)}${waiting ? "\nwait running" : ""}`,
				"info",
			);
		},
	});
}
