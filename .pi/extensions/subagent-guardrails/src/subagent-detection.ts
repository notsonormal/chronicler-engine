// Layered subagent detection. Any-true wins:
// 1. PI_SUBAGENT_CHILD=1 env var (forward compat)
// 2. session name starts with "subagent-"
// 3. session header has parentSession field (covers /fork too)

interface SessionManagerLike {
	getSessionName(): string | undefined;
	getHeader(): { parentSession?: string } | null;
}

const SUBAGENT_NAME_PREFIX = "subagent-";

export function isSubagentSession(sessionManager: SessionManagerLike): boolean {
	if (process.env.PI_SUBAGENT_CHILD === "1") return true;

	try {
		const name = sessionManager.getSessionName();
		if (typeof name === "string" && name.startsWith(SUBAGENT_NAME_PREFIX)) {
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
