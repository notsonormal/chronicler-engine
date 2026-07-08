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

const SUB_SUBAGENT_BLOCK_REASON =
	"Subagents cannot spawn sub-subagents. The subagent tool is parent-only. " +
	"Implement the task directly using your own tools, or report back to the " +
	"parent session if you are blocked on context or scope.";

export default function (pi: ExtensionAPI) {
	// Feature 1: parent-side task-spec veto.
	// Also enforces the role-anchor rule that subagents cannot spawn
	// sub-subagents — with an explicit, actionable error rather than the
	// generic "Tool subagent not found" that pi-agent-core returns when the
	// tool is not registered.
	pi.on("tool_call", async (event) => {
		if (!isToolCallEventType<"subagent", SubagentInput>("subagent", event)) return;
		if (isSubagent) {
			return { block: true, reason: SUB_SUBAGENT_BLOCK_REASON };
		}
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
