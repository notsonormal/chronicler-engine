// Rule matching + config load/save.
//
// Rules are loaded once at session_start and cached as compiled RegExp in
// `RuleSet`. `/permissions reload` swaps the cache via `loadFromDisk`.
//
// Reload contract:
// - File missing       -> seed from defaults, write to disk, return ok
// - File malformed     -> return error, leave any existing cache untouched
// - File valid JSON    -> replace cache, return ok

import { readFile, writeFile, mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { defaultRulesAsMap } from "./rules.ts";

export interface RuleSet {
	readonly rules: ReadonlyMap<string, RegExp>;
	readonly source: "disk" | "defaults";
}

export interface LoadResult {
	readonly ok: boolean;
	readonly set?: RuleSet;
	readonly error?: string;
	readonly seeded?: boolean;
}

const CONFIG_RELATIVE = join(".pi", "permissions.json");

export function configPath(cwd: string): string {
	return join(cwd, CONFIG_RELATIVE);
}

export async function loadFromDisk(cwd: string): Promise<LoadResult> {
	const path = configPath(cwd);

	let raw: string;
	try {
		raw = await readFile(path, "utf8");
	} catch (err) {
		if ((err as NodeJS.ErrnoException).code === "ENOENT") {
			const seeded = await seedDefaults(path);
			if (!seeded.ok) return seeded;
			const built = build(defaultRulesAsMap(), "disk");
			return { ok: true, set: built, seeded: true };
		}
		return { ok: false, error: `read failed: ${(err as Error).message}` };
	}

	let parsed: unknown;
	try {
		parsed = JSON.parse(raw);
	} catch (err) {
		return { ok: false, error: `parse failed: ${(err as Error).message}` };
	}

	const result = parseConfig(parsed);
	if (!result.ok) return { ok: false, error: result.error };
	return { ok: true, set: build(result.rules, "disk") };
}

function parseConfig(value: unknown): { ok: true; rules: Record<string, string> } | { ok: false; error: string } {
	if (value === null || typeof value !== "object" || Array.isArray(value)) {
		return { ok: false, error: "root must be an object" };
	}
	const obj = value as Record<string, unknown>;
	const rulesRaw = obj.rules;
	if (rulesRaw === undefined) return { ok: true, rules: {} };
	if (rulesRaw === null || typeof rulesRaw !== "object" || Array.isArray(rulesRaw)) {
		return { ok: false, error: "rules must be an object" };
	}
	const rulesObj = rulesRaw as Record<string, unknown>;
	const rules: Record<string, string> = {};
	for (const [name, pattern] of Object.entries(rulesObj)) {
		if (typeof pattern !== "string") {
			return { ok: false, error: `rule '${name}' pattern must be a string` };
		}
		rules[name] = pattern;
	}
	return { ok: true, rules };
}

function build(rules: Record<string, string>, source: "disk" | "defaults"): RuleSet {
	const compiled = new Map<string, RegExp>();
	for (const [name, pattern] of Object.entries(rules)) {
		compiled.set(name, new RegExp(pattern));
	}
	return { rules: compiled, source };
}

async function seedDefaults(path: string): Promise<{ ok: boolean; error?: string }> {
	try {
		await mkdir(dirname(path), { recursive: true });
		const payload = JSON.stringify({ rules: defaultRulesAsMap() }, null, 2) + "\n";
		await writeFile(path, payload, "utf8");
		return { ok: true };
	} catch (err) {
		return { ok: false, error: `seed write failed: ${(err as Error).message}` };
	}
}

// Pure matcher: returns the first rule name whose regex matches `command`,
// or null. Caller decides what to do (grant bypass, confirm, block).
export function firstMatchingRule(set: RuleSet, command: string): string | null {
	for (const [name, regex] of set.rules) {
		if (regex.test(command)) return name;
	}
	return null;
}