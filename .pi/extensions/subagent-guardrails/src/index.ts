import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { isToolCallEventType } from "@earendil-works/pi-coding-agent";
import { checkTaskSpec } from "./task-veto";
import { onSessionStart, onTurnEnd } from "./budget";
import { ROLE_ANCHOR } from "./role-anchor";
import { checkGitVeto } from "./commit-veto";
import { isSubagentSession } from "./subagent-detection";

interface SubagentInput extends Record<string, unknown> {
	task?: unknown;
	agent?: unknown;
	action?: unknown;
}

let isSubagent = false;

export default function (pi: ExtensionAPI) {
	// Feature 1: parent-side task-spec veto.
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

	// Features 2/3/4: subagent-only. Layered detection via sessionManager.
	pi.on("session_start", (_event, ctx) => {
		isSubagent = isSubagentSession(ctx.sessionManager);
		if (!isSubagent) return;
		onSessionStart();
	});

	pi.on("turn_end", () => {
		if (!isSubagent) return;
		onTurnEnd(pi);
	});

	pi.on("before_agent_start", async (event) => {
		if (!isSubagent) return;
		return { systemPrompt: event.systemPrompt + ROLE_ANCHOR };
	});

	pi.on("tool_call", async (event) => {
		if (!isToolCallEventType("bash", event)) return;
		if (!isSubagent) return;
		const result = checkGitVeto(event.input.command);
		if (result.block) return result;
		return undefined;
	});
}
