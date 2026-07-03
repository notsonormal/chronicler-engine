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

// Module-scoped flag: true when this extension instance is running inside a
// forked subagent session. Set on `session_start` with reason "fork". Used by
// Features 2, 3, and 4. Single source of truth for "am I a forked subagent."
let isFork = false;

export default function (pi: ExtensionAPI) {
	// Feature 1 (#11): parent-side task-spec veto on `subagent` tool calls.
	// In a forked subagent we are not the parent — skip.
	pi.on("tool_call", async (event) => {
		if (!isToolCallEventType<"subagent", SubagentInput>("subagent", event)) return;
		if (isFork) return;
		const result = checkTaskSpec({
			task: event.input.task,
			agent: event.input.agent,
			action: event.input.action,
		});
		if (result.block) return result;
		return undefined;
	});

	// Feature 2 (#12): budget tracker. Reset state on fork session start.
	pi.on("session_start", (event) => {
		if (event.reason !== "fork") {
			isFork = false;
			return;
		}
		isFork = true;
		onSessionStart();
	});

	pi.on("turn_end", () => {
		if (!isFork) return;
		onTurnEnd(pi);
	});

	// Feature 3 (#6): role anchor appended to forked subagent system prompt.
	pi.on("before_agent_start", async (event) => {
		if (!isFork) return;
		return { systemPrompt: event.systemPrompt + ROLE_ANCHOR };
	});

	// Feature 4 (#13): git commit/push/tag/merge/rebase veto in forked
	// subagent `bash` calls only. Parent retains commit/push ownership.
	pi.on("tool_call", async (event) => {
		if (!isToolCallEventType("bash", event)) return;
		if (!isFork) return;
		const result = checkGitVeto(event.input.command);
		if (result.block) return result;
		return undefined;
	});
}
