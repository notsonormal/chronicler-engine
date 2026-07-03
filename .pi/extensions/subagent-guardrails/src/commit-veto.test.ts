import { test } from "node:test";
import assert from "node:assert/strict";
import { checkGitVeto } from "./commit-veto.ts";

const BLOCKED = ["commit", "push", "tag", "merge", "rebase"];

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

const ALLOWED = ["git add -A", "git status", "git diff", "git log", "git stash", "git fetch", "git pull"];
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

test("Feature 4: `git commit-tree` is blocked (history mutation)", () => {
	const r = checkGitVeto("git commit-tree abc");
	assert.equal(r.block, true);
});
