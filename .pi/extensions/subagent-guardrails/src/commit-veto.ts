// Feature 4 (#13): Git commit/push veto.
// Forked subagent-only. Blocks `git commit|push|tag|merge|rebase` in `bash`
// tool calls. Workers must not mutate repository history or push; the parent
// session retains commit-and-push ownership per AGENTS.md.
//
// Read-only and staging ops (`add`, `status`, `diff`, `log`, `stash`,
// `fetch`, `pull`) are intentionally allowed.
//
// Out of scope by design: obfuscated invocations (`git c""mmit`, aliases,
// `git -c` indirection). This is a regex match, not a sandbox.

export interface GitVetoFailure {
	block: true;
	reason: string;
}

export interface GitVetoPass {
	block: false;
}

export type GitVetoResult = GitVetoFailure | GitVetoPass;

const BLOCKED_GIT = /\b(?:git|hub)\s+(commit|push|tag|merge|rebase)\b/;

export function checkGitVeto(command: string): GitVetoResult {
	const match = command.match(BLOCKED_GIT);
	if (!match) return { block: false };

	const verb = match[1];
	return {
		block: true,
		reason:
			`git ${verb} blocked in subagent context. Workers must not mutate repository history or push. ` +
			"Return a summary of staged or local changes and let the parent session commit " +
			"via the commit-and-push skill.",
	};
}
