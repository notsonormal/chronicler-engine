import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import type { ToolInfo } from "@earendil-works/pi-coding-agent";
import {
	canSelectToolInPlanMode,
	completePlanArguments,
	derivePlanSlug,
	evaluateWriteEdit,
	extractProposedPlan,
	isPathInsideFolder,
	isSafeCommand,
	latestAssistantText,
	loadDefaultToolsConfigFromPath,
	normalizePlanModeQuestionParams,
	normalizeRelativePath,
	resolvePlanFilePath,
	stripProposedPlanBlocks,
	stripProposedPlanBlocksFromMessage,
	writePlanFile,
} from "../src/index.ts";

test("completePlanArguments suggests management tokens only", () => {
	assert.deepEqual(
		completePlanArguments("")?.map((item) => item.label),
		["exit", "off", "tools", "grill"],
	);
	assert.deepEqual(
		completePlanArguments("to")?.map((item) => item.value),
		["tools"],
	);
	assert.equal(completePlanArguments("tools "), null);
	assert.equal(completePlanArguments("write a plan"), null);
	assert.equal(completePlanArguments("unknown"), null);
});

test("isSafeCommand permits read-only commands and blocks mutating commands", () => {
	assert.equal(isSafeCommand("git status --short"), true);
	assert.equal(isSafeCommand("sed -n '1,20p' file.ts"), true);
	assert.equal(isSafeCommand("rm -rf build"), false);
	assert.equal(isSafeCommand("npm install"), false);
	assert.equal(isSafeCommand(""), false);
});

test("normalizePlanModeQuestionParams validates question shape", () => {
	const result = normalizePlanModeQuestionParams({
		questions: [
			{
				id: "scope",
				header: "Scope",
				question: "How broad?",
				options: [
					{ label: "Small", description: "Only the bug." },
					{ label: "Broad", description: "Include nearby cleanup." },
				],
			},
		],
	});

	assert.equal(result.ok, true);
	if (result.ok) assert.equal(result.questions[0]?.options[1]?.label, "Broad");
	assert.deepEqual(normalizePlanModeQuestionParams({ questions: [] }), {
		ok: false,
		error: "questions must contain 1-3 items",
	});
});

test("proposed-plan helpers extract and remove plan blocks", () => {
	assert.equal(extractProposedPlan("Intro\n<proposed_plan>\n# Plan\n</proposed_plan>"), "# Plan");
	assert.equal(stripProposedPlanBlocks("A<proposed_plan>secret</proposed_plan>B"), "AB");
	assert.deepEqual(
		stripProposedPlanBlocksFromMessage({
			role: "assistant",
			content: [{ type: "text", text: "Keep\n<proposed_plan>remove</proposed_plan>" }],
		}),
		{ role: "assistant", content: [{ type: "text", text: "Keep\n" }] },
	);
	assert.equal(
		latestAssistantText([
			{ role: "user", content: "ignore" },
			{ message: { role: "assistant", content: [{ type: "text", text: "answer" }] } },
		]),
		"answer",
	);
});

test("normalizeRelativePath keeps cwd-relative, rejects absolute and '..' escape", () => {
	assert.equal(normalizeRelativePath("docs/plans"), "docs/plans");
	assert.equal(normalizeRelativePath("  tmp/inner  "), "tmp/inner");
	assert.equal(normalizeRelativePath("/etc/passwd"), undefined);
	assert.equal(normalizeRelativePath(""), undefined);
	assert.equal(normalizeRelativePath("../escape"), undefined);
	assert.equal(normalizeRelativePath("a/../../b"), undefined);
});

test("isPathInsideFolder is strict (equal paths = false) and walks subdirectories", () => {
	const root = "/root";
	assert.equal(isPathInsideFolder("/root/child.md", "/root"), true);
	assert.equal(isPathInsideFolder("/root/sub/deep.md", "/root"), true);
	assert.equal(isPathInsideFolder("/root", "/root"), false);
	assert.equal(isPathInsideFolder("/other/x.md", "/root"), false);
	assert.equal(isPathInsideFolder("/rootish/x.md", "/root"), false);
});

test("evaluateWriteEdit allows input with no path (pass-through)", () => {
	const decision = evaluateWriteEdit({ content: "no path" }, ["docs"], "/cwd");
	assert.deepEqual(decision, { allowed: true });
});

test("evaluateWriteEdit blocks when no folders are configured", () => {
	const decision = evaluateWriteEdit(
		{ path: "anything.md" },
		[],
		"/cwd",
	);
	assert.equal(decision.allowed, false);
	if (!decision.allowed) {
		assert.match(decision.reason, /no planFolder or scratchFolders/);
	}
});

test("evaluateWriteEdit blocks paths outside the allowed folders", () => {
	const decision = evaluateWriteEdit(
		{ path: "src/file.ts" },
		["docs/plans", "tmp"],
		"/repo",
	);
	assert.equal(decision.allowed, false);
	if (!decision.allowed) {
		assert.match(decision.reason, /Allowed \(relative to \/repo\)/);
		assert.match(decision.reason, /docs\/plans/);
	}
});

test("evaluateWriteEdit allows paths inside the allowed folders", () => {
	const cases: Array<{ input: string; folders: string[] }> = [
		{ input: "docs/plans/foo.md", folders: ["docs/plans"] },
		{ input: "tmp/scratch.md", folders: ["docs/plans", "tmp"] },
		{ input: "./docs/plans/x.md", folders: ["docs/plans"] },
	];
	for (const { input, folders } of cases) {
		const decision = evaluateWriteEdit({ path: input }, folders, "/repo");
		assert.equal(decision.allowed, true, `expected allow for ${input}`);
	}
});

function builtinTool(name: string): ToolInfo {
	return {
		name,
		sourceInfo: {
			path: `<builtin>/${name}`,
			source: "builtin",
			scope: "user",
			origin: "top-level",
		},
	} as ToolInfo;
}

function extensionTool(name: string): ToolInfo {
	return {
		name,
		sourceInfo: {
			path: `extension/${name}`,
			source: "extension",
			scope: "project",
			origin: "package",
		},
	} as ToolInfo;
}

test("canSelectToolInPlanMode blocks write with empty allowed folders", () => {
	assert.equal(canSelectToolInPlanMode(builtinTool("write"), []), false);
});

test("canSelectToolInPlanMode allows write with at least one allowed folder", () => {
	assert.equal(
		canSelectToolInPlanMode(builtinTool("write"), ["docs/plans"]),
		true,
	);
	assert.equal(canSelectToolInPlanMode(builtinTool("write"), ["tmp"]), true);
});

test("canSelectToolInPlanMode blocks edit with empty allowed folders", () => {
	assert.equal(canSelectToolInPlanMode(builtinTool("edit"), []), false);
});

test("canSelectToolInPlanMode allows edit with at least one allowed folder", () => {
	assert.equal(
		canSelectToolInPlanMode(builtinTool("edit"), ["docs/plans"]),
		true,
	);
	assert.equal(canSelectToolInPlanMode(builtinTool("edit"), ["tmp"]), true);
});

test("canSelectToolInPlanMode keeps read/bash/grep/find/ls selectable regardless of folders", () => {
	for (const name of ["read", "bash", "grep", "find", "ls"]) {
		const tool = builtinTool(name);
		assert.equal(
			canSelectToolInPlanMode(tool, []),
			true,
			`expected ${name} with []`,
		);
		assert.equal(
			canSelectToolInPlanMode(tool, ["docs/plans"]),
			true,
			`expected ${name} with folders`,
		);
	}
});

test("canSelectToolInPlanMode preserves non-builtin tool opt-in regardless of folders", () => {
	assert.equal(
		canSelectToolInPlanMode(extensionTool("fetch_content"), []),
		true,
	);
	assert.equal(
		canSelectToolInPlanMode(extensionTool("fetch_content"), ["docs/plans"]),
		true,
	);
});

test("derivePlanSlug derives from first H1 title", () => {
	const slug = derivePlanSlug("\n\n# Hello World Plan\n\nbody");
	assert.equal(slug, "hello-world-plan");
});

test("derivePlanSlug strips non-alphanumeric and caps length", () => {
	const slug = derivePlanSlug(
		`# ${"a".repeat(80)}!@#${"b".repeat(80)} title`,
	);
	assert.match(slug ?? "", /^[a-z0-9-]+$/);
	assert.ok((slug ?? "").length <= 60);
});

test("derivePlanSlug falls back to a timestamped name when no title", () => {
	const slug = derivePlanSlug("no heading here\njust body");
	assert.match(slug ?? "", /^plan-\d{8}-\d{6}$/);
});

test("resolvePlanFilePath returns first slug candidate", () => {
	const dir = mkdtempSync(join(tmpdir(), "plan-mode-"));
	try {
		const path = resolvePlanFilePath("# Hello World\n", dir, "plans");
		assert.equal(path, join(dir, "plans", "hello-world.md"));
	} finally {
		rmSync(dir, { recursive: true, force: true });
	}
});

test("writePlanFile creates the plan file with derived slug", () => {
	const dir = mkdtempSync(join(tmpdir(), "plan-mode-"));
	try {
		const written = writePlanFile(
			"# My Plan\n\nbody",
			dir,
			"plans",
			"# My Plan\n\nbody\n",
		);
		assert.equal(written, join(dir, "plans", "my-plan.md"));
		assert.equal(readFileSync(written, "utf-8"), "# My Plan\n\nbody\n");
	} finally {
		rmSync(dir, { recursive: true, force: true });
	}
});

test("writePlanFile appends -N suffix on collision (no TOCTOU)", () => {
	const dir = mkdtempSync(join(tmpdir(), "plan-mode-"));
	try {
		mkdirSync(join(dir, "plans"), { recursive: true });
		const first = writePlanFile("# Title\n", dir, "plans", "first body");
		const second = writePlanFile("# Title\n", dir, "plans", "second body");
		const third = writePlanFile("# Title\n", dir, "plans", "third body");
		assert.equal(first, join(dir, "plans", "title.md"));
		assert.equal(second, join(dir, "plans", "title-2.md"));
		assert.equal(third, join(dir, "plans", "title-3.md"));
		assert.equal(readFileSync(first, "utf-8"), "first body");
		assert.equal(readFileSync(second, "utf-8"), "second body");
	} finally {
		rmSync(dir, { recursive: true, force: true });
	}
});

test("writePlanFile surfaces non-collision errors (read-only directory)", () => {
	if (process.platform === "win32") return;
	const dir = mkdtempSync(join(tmpdir(), "plan-mode-"));
	try {
		mkdirSync(join(dir, "ro"), { recursive: true, mode: 0o555 });
		assert.throws(
			() => writePlanFile("# T\n", dir, "ro", "x"),
			/EACCES|EPERM/,
		);
	} finally {
		rmSync(dir, { recursive: true, force: true });
	}
});

test("loadDefaultToolsConfigFromPath returns undefined for missing file", () => {
	const dir = mkdtempSync(join(tmpdir(), "plan-mode-"));
	try {
		const result = loadDefaultToolsConfigFromPath(
			join(dir, "does-not-exist.json"),
		);
		assert.equal(result, undefined);
	} finally {
		rmSync(dir, { recursive: true, force: true });
	}
});

test("loadDefaultToolsConfigFromPath loads valid config", () => {
	const dir = mkdtempSync(join(tmpdir(), "plan-mode-"));
	try {
		const path = join(dir, "plan-mode.json");
		writeFileSync(
			path,
			JSON.stringify({
				defaultTools: ["read", " bash ", ""],
				planFolder: "docs/plans",
				scratchFolders: ["tmp", "../escape", "ok/sub"],
			}),
		);
		const result = loadDefaultToolsConfigFromPath(path);
		assert.deepEqual(result, {
			defaultTools: ["read", "bash"],
			planFolder: "docs/plans",
			scratchFolders: ["tmp", "ok/sub"],
		});
	} finally {
		rmSync(dir, { recursive: true, force: true });
	}
});

test("loadDefaultToolsConfigFromPath rejects whole config on defaultTools type error", () => {
	const dir = mkdtempSync(join(tmpdir(), "plan-mode-"));
	try {
		const path = join(dir, "plan-mode.json");
		writeFileSync(
			path,
			JSON.stringify({
				defaultTools: ["read", 42],
				planFolder: "docs",
			}),
		);
		const warnings: string[] = [];
		const original = console.warn;
		console.warn = (msg: string) => warnings.push(msg);
		try {
			const result = loadDefaultToolsConfigFromPath(path);
			assert.equal(result, undefined);
			assert.ok(warnings.some((w) => w.includes("defaultTools")));
		} finally {
			console.warn = original;
		}
	} finally {
		rmSync(dir, { recursive: true, force: true });
	}
});

test("loadDefaultToolsConfigFromPath keeps soft fields and drops bad ones", () => {
	const dir = mkdtempSync(join(tmpdir(), "plan-mode-"));
	try {
		const path = join(dir, "plan-mode.json");
		writeFileSync(
			path,
			JSON.stringify({
				defaultTools: ["read"],
				planFolder: 42,
				scratchFolders: "not-an-array",
			}),
		);
		const original = console.warn;
		console.warn = () => {};
		let result: ReturnType<typeof loadDefaultToolsConfigFromPath>;
		try {
			result = loadDefaultToolsConfigFromPath(path);
		} finally {
			console.warn = original;
		}
		assert.deepEqual(result, { defaultTools: ["read"] });
	} finally {
		rmSync(dir, { recursive: true, force: true });
	}
});

test("loadDefaultToolsConfigFromPath returns undefined for non-object root", () => {
	const dir = mkdtempSync(join(tmpdir(), "plan-mode-"));
	try {
		const path = join(dir, "plan-mode.json");
		writeFileSync(path, JSON.stringify(["not", "an", "object"]));
		const original = console.warn;
		console.warn = () => {};
		try {
			assert.equal(loadDefaultToolsConfigFromPath(path), undefined);
		} finally {
			console.warn = original;
		}
	} finally {
		rmSync(dir, { recursive: true, force: true });
	}
});
