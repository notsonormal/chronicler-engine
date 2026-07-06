import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { isToolCallEventType } from "@earendil-works/pi-coding-agent";
import { checkTaskSpec } from "./task-veto";
import { onSessionStart, onTurnEnd } from "./budget";
import { ROLE_ANCHOR } from "./role-anchor";
import { checkGitVeto } from "./commit-veto";

interface SubagentInput extends Record<string, unknown> {
	task?: unknown;
	agent?: unknown;
	action?: unknown;
}

// Module-scoped flag: true when running inside a subagent session spawned
// by the pi-subagents extension. Set on `session_start` from the
// `PI_SUBAGENT_CHILD === "1"` env var (set unconditionally in every spawned
// child — sync, async, chain, parallel, fanout, fresh or forked context).
// Top-level `pi` and `/fork` `/` `clone` slash commands don't spawn a child
// and never set it. Used by Features 2, 3, 4. Feature 1 is parent-side and
// skips when `isSubagent` is true.
let isSubagent = false;

export default function (pi: ExtensionAPI) {
	// Feature 1 (#11): parent-side task-spec veto on `subagent` tool calls.
	// In a subagent we are not the parent — skip.
	pi.on("tool_call", async (event) => {
		if (!isToolCallEventType<"subagent", SubagentInput>("subagent", event)) return;
		if (isSubagent) return;
		const result = checkTaskSpec({
			task: event.input.task,
			agent: event.input.agent,
			action: event.input.action,
		});
		if (result.block) return result;
		return undefined;
	});

	// Feature 2 (#12): budget tracker. Reset state on subagent session start.
	// Detection from `PI_SUBAGENT_CHILD` env var (see module header).
	pi.on("session_start", () => {
		isSubagent = process.env.PI_SUBAGENT_CHILD === "1";
		if (!isSubagent) return;
		onSessionStart();
	});

	pi.on("turn_end", () => {
		if (!isSubagent) return;
		onTurnEnd(pi);
	});

	// Feature 3 (#6): role anchor appended to subagent system prompt.
	pi.on("before_agent_start", async (event) => {
		if (!isSubagent) return;
		return { systemPrompt: event.systemPrompt + ROLE_ANCHOR };
	});

	// Feature 4 (#13): git commit/push/tag/merge/rebase/reset/stash veto in
	// subagent `bash` calls only. Parent retains commit/push ownership.
	// `git stash list` is exempt (read-only).
	pi.on("tool_call", async (event) => {
		if (!isToolCallEventType("bash", event)) return;
		if (!isSubagent) return;
		const result = checkGitVeto(event.input.command);
		if (result.block) return result;
		return undefined;
	});
}
