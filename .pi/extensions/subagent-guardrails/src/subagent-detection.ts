// Layered subagent detection. Any-true wins:
// 1. PI_SUBAGENT_CHILD=1 env var (forward compat)
// 2. session name starts with "subagent-" but is NOT "subagent-chat-*"
//    (pi-subagents names spawned workers "subagent-{role}-{runId}-{index}",
//    but interactive parent sessions are also named "subagent-chat-{id}" —
//    those are primaries, not subagents, so must be excluded)
// 3. session header has parentSession field (covers /fork too)

interface SessionManagerLike {
	getSessionName(): string | undefined;
	getHeader(): { parentSession?: string } | null;
}

const SUBAGENT_NAME_PREFIX = "subagent-";
// Interactive parent session naming convention. Excluded from the subagent
// prefix match so primaries are not misclassified (issue: model called
// subagent tool, pi-agent-core returned cryptic "Tool subagent not found"
// because the tool was never registered for the wrongly-flagged parent).
const PARENT_CHAT_NAME_PREFIX = "subagent-chat-";

export function isSubagentSession(sessionManager: SessionManagerLike): boolean {
	if (process.env.PI_SUBAGENT_CHILD === "1") return true;

	try {
		const name = sessionManager.getSessionName();
		if (
			typeof name === "string" &&
			name.startsWith(SUBAGENT_NAME_PREFIX) &&
			!name.startsWith(PARENT_CHAT_NAME_PREFIX)
		) {
			return true;
		}
	} catch {}

	try {
		const header = sessionManager.getHeader();
		if (header && typeof header.parentSession === "string" && header.parentSession.length > 0) {
			return true;
		}
	} catch {}

	return false;
}
