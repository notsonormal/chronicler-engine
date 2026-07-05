// Built-in default rules. Seeded into `.pi/permissions.json` on first run
// when the file is absent. Patterns match the documented defaults in
// docs/superpowers/specs/2026-07-06-pi-permissions-design.md.

export interface DefaultRule {
	readonly name: string;
	readonly pattern: string;
}

export const DEFAULT_RULES: readonly DefaultRule[] = [
	{ name: "git-commit", pattern: "\\bgit\\s+commit\\b" },
	{ name: "git-push", pattern: "\\bgit\\s+push\\b" },
	{ name: "git-tag", pattern: "\\bgit\\s+tag\\b" },
	{ name: "git-merge", pattern: "\\bgit\\s+merge\\b" },
	{ name: "git-rebase", pattern: "\\bgit\\s+rebase\\b" },
	{ name: "git-reset", pattern: "\\bgit\\s+reset\\b" },
	{ name: "git-pull", pattern: "\\bgit\\s+pull\\b" },
	{ name: "mnt-access", pattern: "(^|\\s)/mnt/" },
];

export function defaultRulesAsMap(): Record<string, string> {
	const out: Record<string, string> = {};
	for (const r of DEFAULT_RULES) out[r.name] = r.pattern;
	return out;
}