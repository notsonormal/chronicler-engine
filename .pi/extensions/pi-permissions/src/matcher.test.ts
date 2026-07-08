import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { firstMatchingRule, loadFromDisk, configPath } from "./matcher.ts";
import { DEFAULT_RULES } from "./rules.ts";

// ---------- firstMatchingRule ----------

test("matcher: first matching rule wins (insertion order)", () => {
	const set = compile({
		"git-commit": "\\bgit\\s+commit\\b",
		"git-push": "\\bgit\\s+push\\b",
	});
	assert.equal(firstMatchingRule(set, "git commit && git push"), "git-commit");
	assert.equal(firstMatchingRule(set, "git push origin main"), "git-push");
});

test("matcher: returns null when no rule matches", () => {
	const set = compile({
		"git-push": "\\bgit\\s+push\\b",
	});
	assert.equal(firstMatchingRule(set, "ls -la"), null);
	assert.equal(firstMatchingRule(set, "git status"), null);
});

// ---------- default rule coverage ----------

test("matcher: each default rule matches its positive case", () => {
	const set = compile(Object.fromEntries(DEFAULT_RULES.map((r) => [r.name, r.pattern])));
	const positives: Array<[string, string]> = [
		["git-commit", "git commit -m wip"],
		["git-push", "git push origin main"],
		["git-tag", "git tag v1.0"],
		["git-merge", "git merge feature"],
		["git-rebase", "git rebase main"],
		["git-reset", "git reset --hard HEAD~1"],
		["git-pull", "git pull origin main"],
		["git-cherry-pick", "git cherry-pick abc123"],
		["git-revert", "git revert HEAD"],
		["git-rm", "git rm path/to/file"],
		["git-rm", "git rm --cached file.txt"],
		["git-stash", "git stash push -m wip"],
		["git-branch-force", "git branch -f main feature"],
		["git-branch-force", "git branch -D broken"],
		["git-branch-force", "git branch --force main feature"],
		["git-checkout", "git checkout feature"],
		["git-checkout", "git checkout -b new-branch"],
		["git-checkout", "git checkout -B main feature"],
		["git-checkout", "git checkout -- file.txt"],
		["git-checkout", "git checkout HEAD -- file.txt"],
		["git-restore", "git restore file.txt"],
		["git-restore", "git restore --staged file.txt"],
		["git-restore", "git restore --source=HEAD file.txt"],
		["git-clean-force", "git clean -f"],
		["git-clean-force", "git clean -fd"],
		["git-clean-force", "git clean -fdx"],
		["git-clean-force", "git clean --force"],
		["mnt-access", "ls /mnt/c/Users"],
	];
	for (const [rule, cmd] of positives) {
		assert.equal(firstMatchingRule(set, cmd), rule, `expected ${rule} to match: ${cmd}`);
	}
});

test("matcher: default rules do not match benign git commands", () => {
	const set = compile(Object.fromEntries(DEFAULT_RULES.map((r) => [r.name, r.pattern])));
	const benign = [
		"git status",
		"git diff",
		"git log --oneline -20",
		"git add -A",
		"git stash list",
		"git fetch origin",
		"git branch --list",
		"git branch -d merged-feature",
		"git clean -n",
	];
	for (const cmd of benign) {
		assert.equal(firstMatchingRule(set, cmd), null, `unexpected match: ${cmd}`);
	}
});

test("matcher: git-checkout blocks every variant (switch, create, force-create, discard, ref)", () => {
	const set = compile(Object.fromEntries(DEFAULT_RULES.map((r) => [r.name, r.pattern])));
	assert.equal(firstMatchingRule(set, "git checkout feature"), "git-checkout");
	assert.equal(firstMatchingRule(set, "git checkout -b new-branch"), "git-checkout");
	assert.equal(firstMatchingRule(set, "git checkout -B main feature"), "git-checkout");
	assert.equal(firstMatchingRule(set, "git checkout -- file.txt"), "git-checkout");
	assert.equal(firstMatchingRule(set, "git checkout HEAD -- file.txt"), "git-checkout");
});

test("matcher: git-restore blocks every variant (working tree, staged, source, dot)", () => {
	const set = compile(Object.fromEntries(DEFAULT_RULES.map((r) => [r.name, r.pattern])));
	assert.equal(firstMatchingRule(set, "git restore file.txt"), "git-restore");
	assert.equal(firstMatchingRule(set, "git restore --staged file.txt"), "git-restore");
	assert.equal(firstMatchingRule(set, "git restore --source=HEAD file.txt"), "git-restore");
	assert.equal(firstMatchingRule(set, "git restore ."), "git-restore");
});

test("matcher: git-stash carve-out permits `git stash list` only", () => {
	const set = compile(Object.fromEntries(DEFAULT_RULES.map((r) => [r.name, r.pattern])));
	assert.equal(firstMatchingRule(set, "git stash list"), null);
	assert.equal(firstMatchingRule(set, "git stash"), "git-stash");
	assert.equal(firstMatchingRule(set, "git stash pop"), "git-stash");
	assert.equal(firstMatchingRule(set, "git stash apply"), "git-stash");
	assert.equal(firstMatchingRule(set, "git stash drop"), "git-stash");
	assert.equal(firstMatchingRule(set, "git stash clear"), "git-stash");
	assert.equal(firstMatchingRule(set, "git stash push -m wip"), "git-stash");
});

test("matcher: mnt-access matches at start and after whitespace", () => {
	const set = compile(Object.fromEntries(DEFAULT_RULES.map((r) => [r.name, r.pattern])));
	assert.equal(firstMatchingRule(set, "/mnt/c/Users"), "mnt-access");
	assert.equal(firstMatchingRule(set, "ls /mnt/c"), "mnt-access");
	assert.equal(firstMatchingRule(set, "cat /mnt/data.txt"), "mnt-access");
});

test("matcher: mnt-access does not match /mntfoo or mount mentions", () => {
	const set = compile(Object.fromEntries(DEFAULT_RULES.map((r) => [r.name, r.pattern])));
	assert.equal(firstMatchingRule(set, "echo /mntfoo"), null);
});

// ---------- loadFromDisk ----------

test("loadFromDisk: seeds defaults when file missing", async () => {
	const cwd = mkdtempSync(join(tmpdir(), "pi-perm-"));
	try {
		const result = await loadFromDisk(cwd);
		assert.equal(result.ok, true);
		assert.equal(result.seeded, true);
		assert.ok(result.set);
		assert.equal(result.set!.rules.size, DEFAULT_RULES.length);
		assert.equal(result.set!.source, "disk");
		// File now exists
		assert.equal(existsSync(configPath(cwd)), true);
		const written = JSON.parse(readFileSync(configPath(cwd), "utf8"));
		assert.ok(written.rules);
		assert.equal(Object.keys(written.rules).length, DEFAULT_RULES.length);
	} finally {
		rmSync(cwd, { recursive: true, force: true });
	}
});

test("loadFromDisk: returns error on malformed JSON, leaves defaults untouched", async () => {
	const cwd = mkdtempSync(join(tmpdir(), "pi-perm-"));
	try {
		mkdirSync(dirname(configPath(cwd)), { recursive: true });
		writeFileSync(configPath(cwd), "{ this is not json", "utf8");
		const result = await loadFromDisk(cwd);
		assert.equal(result.ok, false);
		assert.match(result.error ?? "", /parse failed/);
		// File should not be overwritten
		assert.equal(readFileSync(configPath(cwd), "utf8"), "{ this is not json");
	} finally {
		rmSync(cwd, { recursive: true, force: true });
	}
});

test("loadFromDisk: returns error on root not an object", async () => {
	const cwd = mkdtempSync(join(tmpdir(), "pi-perm-"));
	try {
		mkdirSync(dirname(configPath(cwd)), { recursive: true });
		writeFileSync(configPath(cwd), "[]", "utf8");
		const result = await loadFromDisk(cwd);
		assert.equal(result.ok, false);
		assert.match(result.error ?? "", /root must be an object/);
	} finally {
		rmSync(cwd, { recursive: true, force: true });
	}
});

test("loadFromDisk: returns error on rules not an object", async () => {
	const cwd = mkdtempSync(join(tmpdir(), "pi-perm-"));
	try {
		mkdirSync(dirname(configPath(cwd)), { recursive: true });
		writeFileSync(configPath(cwd), JSON.stringify({ rules: "nope" }), "utf8");
		const result = await loadFromDisk(cwd);
		assert.equal(result.ok, false);
		assert.match(result.error ?? "", /rules must be an object/);
	} finally {
		rmSync(cwd, { recursive: true, force: true });
	}
});

test("loadFromDisk: returns error on rule pattern not a string", async () => {
	const cwd = mkdtempSync(join(tmpdir(), "pi-perm-"));
	try {
		mkdirSync(dirname(configPath(cwd)), { recursive: true });
		writeFileSync(configPath(cwd), JSON.stringify({ rules: { "git-push": 42 } }), "utf8");
		const result = await loadFromDisk(cwd);
		assert.equal(result.ok, false);
		assert.match(result.error ?? "", /pattern must be a string/);
	} finally {
		rmSync(cwd, { recursive: true, force: true });
	}
});

test("loadFromDisk: loads valid config", async () => {
	const cwd = mkdtempSync(join(tmpdir(), "pi-perm-"));
	try {
		mkdirSync(dirname(configPath(cwd)), { recursive: true });
		writeFileSync(
			configPath(cwd),
			JSON.stringify({ rules: { "custom-rule": "echo hello" } }),
			"utf8",
		);
		const result = await loadFromDisk(cwd);
		assert.equal(result.ok, true);
		assert.equal(result.set!.rules.size, 1);
		assert.equal(firstMatchingRule(result.set!, "echo hello"), "custom-rule");
	} finally {
		rmSync(cwd, { recursive: true, force: true });
	}
});

test("loadFromDisk: empty rules object is valid", async () => {
	const cwd = mkdtempSync(join(tmpdir(), "pi-perm-"));
	try {
		mkdirSync(dirname(configPath(cwd)), { recursive: true });
		writeFileSync(configPath(cwd), JSON.stringify({}), "utf8");
		const result = await loadFromDisk(cwd);
		assert.equal(result.ok, true);
		assert.equal(result.set!.rules.size, 0);
	} finally {
		rmSync(cwd, { recursive: true, force: true });
	}
});

// ---------- helpers ----------

function compile(rules: Record<string, string>) {
	const compiled = new Map<string, RegExp>();
	for (const [name, pattern] of Object.entries(rules)) {
		compiled.set(name, new RegExp(pattern));
	}
	return { rules: compiled, source: "disk" as const };
}