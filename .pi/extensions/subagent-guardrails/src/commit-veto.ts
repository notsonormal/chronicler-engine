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

// Consume zero+ git flag groups between the binary and the verb. Each flag
// is `-x` or `--name`, optionally followed by `=value` (no spaces) or a
// space-separated value. This catches `--no-pager`, `-c name=value`,
// `-C /path`, `--git-dir=/path`, and any combination thereof. Bypasses still
// possible: shell-quoted command names (`git c""mmit`, `g""it commit`), git
// aliases, `hub` shimmed with a different binary name.
const FLAG_GROUP = `(?:\\s+--?\\w[\\w-]*(?:=\\S+|\\s+\\S+)?)*`;

const BLOCKED_GIT = new RegExp(`\\b(?:git|hub)${FLAG_GROUP}\\s+(commit|push|tag|merge|rebase|reset|rm|checkout|restore)\\b`);
// Block any `git [flags] stash` invocation whose next token isn't `list`.
// Negative lookahead folds the carve-out into a single regex.
const STASH_BLOCK = new RegExp(`\\b(?:git|hub)${FLAG_GROUP}\\s+stash(?!\\s+list\\b)\\b`);

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

	if (STASH_BLOCK.test(command)) {
		return { block: true, reason: `git stash ${REASON_SUFFIX}` };
	}

	return { block: false };
}
