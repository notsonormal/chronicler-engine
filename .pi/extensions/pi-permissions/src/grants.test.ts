import { test } from "node:test";
import assert from "node:assert/strict";
import { GrantStore } from "./grants.ts";

test("grants: grant + has + list", () => {
	const g = new GrantStore();
	g.grant("git-push");
	g.grant("git-commit");
	assert.equal(g.has("git-push"), true);
	assert.equal(g.has("git-merge"), false);
	assert.deepEqual(g.list(), ["git-commit", "git-push"]);
});

test("grants: revoke returns true when present, false when absent", () => {
	const g = new GrantStore();
	g.grant("git-push");
	assert.equal(g.revoke("git-push"), true);
	assert.equal(g.revoke("git-push"), false);
	assert.equal(g.has("git-push"), false);
});

test("grants: clear empties everything", () => {
	const g = new GrantStore();
	g.grant("a");
	g.grant("b");
	g.clear();
	assert.equal(g.list().length, 0);
});

test("grants: duplicate grant is idempotent", () => {
	const g = new GrantStore();
	g.grant("a");
	g.grant("a");
	assert.deepEqual(g.list(), ["a"]);
});