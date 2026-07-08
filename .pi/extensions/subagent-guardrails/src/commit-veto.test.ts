import { test } from "node:test";
import assert from "node:assert/strict";
import { checkGitVeto } from "./commit-veto.ts";

const BLOCKED = ["commit", "push", "tag", "merge", "rebase", "reset", "rm", "checkout", "restore"];

for (const verb of BLOCKED) {
	test(`Feature 4: git ${verb} is blocked`, () => {
		const r = checkGitVeto(`git ${verb} -x`);
		if (!r.block) throw new Error("expected block");
		assert.match(r.reason, new RegExp(`git ${verb} blocked`));
	});
}

test("Feature 4: hub commit is also blocked", () => {
	const r = checkGitVeto("hub commit -m x");
	assert.equal(r.block, true);
});

test("Feature 4: `git commit-tree` is blocked (history mutation)", () => {
	const r = checkGitVeto("git commit-tree abc");
	assert.equal(r.block, true);
});

test("Feature 4: `git reset --hard` is blocked", () => {
	const r = checkGitVeto("git reset --hard");
	assert.equal(r.block, true);
	assert.match(r.reason, /git reset blocked/);
});

test("Feature 4: `git reset --soft HEAD~1` is blocked", () => {
	const r = checkGitVeto("git reset --soft HEAD~1");
	assert.equal(r.block, true);
	assert.match(r.reason, /git reset blocked/);
});

test("Feature 4: bare `git reset` is blocked", () => {
	const r = checkGitVeto("git reset");
	assert.equal(r.block, true);
});

test("Feature 4: `git reset HEAD~2` is blocked", () => {
	const r = checkGitVeto("git reset HEAD~2");
	assert.equal(r.block, true);
});

test("Feature 4: `hub reset --hard` is blocked", () => {
	const r = checkGitVeto("hub reset --hard");
	assert.equal(r.block, true);
});

test("Feature 4: `git rm path/to/file` is blocked", () => {
	const r = checkGitVeto("git rm path/to/file");
	assert.equal(r.block, true);
	assert.match(r.reason, /git rm blocked/);
});

test("Feature 4: `git rm --cached file` is blocked", () => {
	const r = checkGitVeto("git rm --cached file.txt");
	assert.equal(r.block, true);
});

test("Feature 4: `hub rm file` is blocked", () => {
	const r = checkGitVeto("hub rm file.txt");
	assert.equal(r.block, true);
	assert.match(r.reason, /git rm blocked/);
});

test("Feature 4: bare `git stash` is blocked (defaults to push)", () => {
	const r = checkGitVeto("git stash");
	assert.equal(r.block, true);
	assert.match(r.reason, /git stash blocked/);
});

test("Feature 4: `git stash push -m x` is blocked", () => {
	const r = checkGitVeto("git stash push -m x");
	assert.equal(r.block, true);
});

test("Feature 4: `git stash pop` is blocked", () => {
	const r = checkGitVeto("git stash pop");
	assert.equal(r.block, true);
});

test("Feature 4: `git stash apply` is blocked", () => {
	const r = checkGitVeto("git stash apply");
	assert.equal(r.block, true);
});

test("Feature 4: `git stash drop` is blocked", () => {
	const r = checkGitVeto("git stash drop");
	assert.equal(r.block, true);
});

test("Feature 4: `git stash clear` is blocked", () => {
	const r = checkGitVeto("git stash clear");
	assert.equal(r.block, true);
});

test("Feature 4: `git stash save` is blocked", () => {
	const r = checkGitVeto("git stash save wip");
	assert.equal(r.block, true);
});

test("Feature 4: `git stash list` is NOT blocked (read-only)", () => {
	const r = checkGitVeto("git stash list");
	assert.equal(r.block, false);
});

test("Feature 4: `hub stash list` is NOT blocked", () => {
	const r = checkGitVeto("hub stash list");
	assert.equal(r.block, false);
});

test("Feature 4: `hub stash` (bare) is blocked", () => {
	const r = checkGitVeto("hub stash");
	assert.equal(r.block, true);
});

const ALLOWED = [
	"git add -A",
	"git status",
	"git diff",
	"git log",
	"git stash list",
	"git fetch",
	"git pull",
];
for (const cmd of ALLOWED) {
	test(`Feature 4: \`${cmd}\` is NOT blocked`, () => {
		assert.equal(checkGitVeto(cmd).block, false);
	});
}

test("Feature 4: non-git command is not blocked", () => {
	assert.equal(checkGitVeto("ls -la").block, false);
});

test("Feature 4: reason routes worker to commit-and-push skill", () => {
	const r = checkGitVeto("git commit -m wip");
	if (!r.block) throw new Error("expected block");
	assert.match(r.reason, /commit-and-push/);
});

test("Feature 4: stash reason also routes worker to commit-and-push skill", () => {
	const r = checkGitVeto("git stash pop");
	if (!r.block) throw new Error("expected block");
	assert.match(r.reason, /commit-and-push/);
});

test("Feature 4: rm reason also routes worker to commit-and-push skill", () => {
	const r = checkGitVeto("git rm file.txt");
	if (!r.block) throw new Error("expected block");
	assert.match(r.reason, /commit-and-push/);
});

// --- git checkout: every variant blocked (no carve-outs) ---

test("Feature 4: `git checkout feature` (branch switch) is blocked", () => {
	const r = checkGitVeto("git checkout feature");
	assert.equal(r.block, true);
	assert.match(r.reason, /git checkout blocked/);
});

test("Feature 4: `git checkout -b new-branch` (branch create) is blocked", () => {
	const r = checkGitVeto("git checkout -b new-branch");
	assert.equal(r.block, true);
	assert.match(r.reason, /git checkout blocked/);
});

test("Feature 4: `git checkout -B main feature` (force-create) is blocked", () => {
	const r = checkGitVeto("git checkout -B main feature");
	assert.equal(r.block, true);
	assert.match(r.reason, /git checkout blocked/);
});

test("Feature 4: `git checkout -- file.txt` (working-tree discard) is blocked", () => {
	const r = checkGitVeto("git checkout -- file.txt");
	assert.equal(r.block, true);
	assert.match(r.reason, /git checkout blocked/);
});

// --- git restore: every variant blocked (no carve-outs) ---

test("Feature 4: `git restore file.txt` (working-tree discard) is blocked", () => {
	const r = checkGitVeto("git restore file.txt");
	assert.equal(r.block, true);
	assert.match(r.reason, /git restore blocked/);
});

test("Feature 4: `git restore --staged file.txt` is blocked (no carve-out)", () => {
	const r = checkGitVeto("git restore --staged file.txt");
	assert.equal(r.block, true);
	assert.match(r.reason, /git restore blocked/);
});

test("Feature 4: `git restore --source=HEAD file.txt` (source variant) is blocked", () => {
	const r = checkGitVeto("git restore --source=HEAD file.txt");
	assert.equal(r.block, true);
	assert.match(r.reason, /git restore blocked/);
});

// --- hub variants: parity with git ---

test("Feature 4: `hub checkout feature` is blocked", () => {
	const r = checkGitVeto("hub checkout feature");
	assert.equal(r.block, true);
	assert.match(r.reason, /git checkout blocked/);
});

test("Feature 4: `hub restore file.txt` is blocked", () => {
	const r = checkGitVeto("hub restore file.txt");
	assert.equal(r.block, true);
	assert.match(r.reason, /git restore blocked/);
});

// --- reason text covers new behavior ---

test("Feature 4: checkout + restore reasons mention branch change + working-tree restore", () => {
	const checkoutReason = checkGitVeto("git checkout feature");
	const restoreReason = checkGitVeto("git restore file.txt");
	if (!checkoutReason.block || !restoreReason.block) throw new Error("expected block");
	assert.match(checkoutReason.reason, /change branches/);
	assert.match(restoreReason.reason, /restore working tree/);
});

// --- bypass hardening: flag groups between git and verb ---

test("Feature 4: `git --no-pager commit` is blocked (bypass hardening)", () => {
	const r = checkGitVeto("git --no-pager commit -m x");
	assert.equal(r.block, true);
	assert.match(r.reason, /git commit blocked/);
});

test("Feature 4: `git --no-pager push` is blocked", () => {
	const r = checkGitVeto("git --no-pager push");
	assert.equal(r.block, true);
	assert.match(r.reason, /git push blocked/);
});

test("Feature 4: `git -c core.editor=true commit` is blocked (was a known bypass)", () => {
	const r = checkGitVeto("git -c core.editor=true commit -m x");
	assert.equal(r.block, true);
});

test("Feature 4: `git -C /tmp/repo commit` is blocked (space-separated flag value)", () => {
	const r = checkGitVeto("git -C /tmp/repo commit -m x");
	assert.equal(r.block, true);
});

test("Feature 4: `git --git-dir=/path commit` is blocked (=value flag form)", () => {
	const r = checkGitVeto("git --git-dir=/path commit");
	assert.equal(r.block, true);
});

test("Feature 4: `git --no-pager -c core.editor=true commit` (two flags) is blocked", () => {
	const r = checkGitVeto("git --no-pager -c core.editor=true commit -m x");
	assert.equal(r.block, true);
});

test("Feature 4: `hub --no-pager commit` is blocked (hub parity)", () => {
	const r = checkGitVeto("hub --no-pager commit");
	assert.equal(r.block, true);
});

test("Feature 4: `git --no-pager stash pop` is blocked (stash bypass hardening)", () => {
	const r = checkGitVeto("git --no-pager stash pop");
	assert.equal(r.block, true);
	assert.match(r.reason, /git stash blocked/);
});

test("Feature 4: `git --no-pager stash list` is NOT blocked (carve-out survives)", () => {
	const r = checkGitVeto("git --no-pager stash list");
	assert.equal(r.block, false);
});

// --- shell-expansion forms (word-boundary coverage) ---

test("Feature 4: command substitution `$(git --no-pager commit)` is blocked", () => {
	assert.equal(checkGitVeto("echo $(git --no-pager commit -m x)").block, true);
});

test("Feature 4: backtick `git --no-pager push` is blocked", () => {
	assert.equal(checkGitVeto("echo `git --no-pager push`").block, true);
});

// --- non-regression: read-only commands stay unblocked with flags ---

for (const cmd of [
	"git --no-pager log --oneline -10",
	"git --no-pager diff --stat",
	"git -C /tmp/repo status",
	"git -c color.ui=always log",
]) {
	test(`Feature 4: \`${cmd}\` is NOT blocked`, () => {
		assert.equal(checkGitVeto(cmd).block, false);
	});
}
