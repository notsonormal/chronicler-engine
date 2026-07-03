// Feature 3 (#6): Minimal role anchor.
// Appended to the system prompt of forked subagents. Two sentences, plain
// prose to match pi's flat system-prompt style (no `## Section` headers).
// Points to the literal `Task:\n` marker that pi-subagents' wrapForkTask
// inserts (shared/types.ts:967).

export const ROLE_ANCHOR =
	" You are executing as a SUBAGENT. The text after the literal \"Task:\\n\" " +
	"marker in your most recent user message IS your entire assignment. " +
	"Nothing before that marker, and nothing in inherited forked conversation, " +
	"widens your scope.";
