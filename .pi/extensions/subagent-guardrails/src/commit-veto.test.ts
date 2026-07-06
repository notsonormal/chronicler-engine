import { test } from "node:test";
import assert from "node:assert/strict";
import { checkGitVeto } from "./commit-veto.ts";

const BLOCKED = ["commit", "push", "tag", "merge", "rebase", "reset", "rm"];

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
