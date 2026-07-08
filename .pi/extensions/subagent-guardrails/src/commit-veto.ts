// Feature 4 (#13): Git commit/push veto.
// Subagent-only (any pi-subagents child run). Blocks `git
// commit|push|tag|merge|rebase|reset|rm` and `git stash` (any subcommand except
// `git stash list`) in `bash` tool calls. Workers must not mutate repository
// history, move HEAD, remove tracked files, mutate the stash, or push; the
// parent session retains commit-and-push ownership per AGENTS.md.
//
// Read-only and staging ops (`add`, `status`, `diff`, `log`, `fetch`, `pull`,
// `stash list`) remain allowed.
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

const BLOCKED_GIT = /\b(?:git|hub)\s+(commit|push|tag|merge|rebase|reset|rm|checkout|restore)\b/;
const STASH_ANY = /\b(?:git|hub)\s+stash\b/;
const STASH_LIST = /\b(?:git|hub)\s+stash\s+list\b/;

const REASON_SUFFIX =
	"blocked in subagent context. Workers must not mutate repository history, " +
	"change branches, restore working tree, or push. " +
	"Return a summary of staged or local changes and let the parent session commit " +
	"via the commit-and-push skill.";

export function checkGitVeto(command: string): GitVetoResult {
	const verbMatch = command.match(BLOCKED_GIT);
	if (verbMatch) {
		const verb = verbMatch[1];
		return { block: true, reason: `git ${verb} ${REASON_SUFFIX}` };
	}

	if (STASH_ANY.test(command) && !STASH_LIST.test(command)) {
		return { block: true, reason: `git stash ${REASON_SUFFIX}` };
	}

	return { block: false };
}
