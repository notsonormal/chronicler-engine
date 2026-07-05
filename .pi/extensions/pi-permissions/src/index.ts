import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { isToolCallEventType } from "@earendil-works/pi-coding-agent";
import { DEFAULT_RULES } from "./rules.ts";
import { GrantStore } from "./grants.ts";
import { firstMatchingRule, loadFromDisk, type RuleSet } from "./matcher.ts";

interface BlockResult {
	block: true;
	reason: string;
}

// Module-scoped cache. Reloaded on session_start and via /permissions reload.
let rules: RuleSet | null = null;
const grants = new GrantStore();

// In-session ring buffer of recent blocks for /permissions display.
interface BlockRecord {
	readonly rule: string;
	readonly command: string;
	readonly outcome: "denied" | "no-ui";
	readonly at: number;
}
const RECENT_LIMIT = 20;
const recent: BlockRecord[] = [];

function buildDefaults(): RuleSet {
	const compiled = new Map<string, RegExp>();
	for (const r of DEFAULT_RULES) {
		compiled.set(r.name, new RegExp(r.pattern));
	}
	return { rules: compiled, source: "defaults" };
}

export default function (pi: ExtensionAPI) {
	pi.on("session_start", async (_event, ctx) => {
		recent.length = 0;
		grants.clear();
		const result = await loadFromDisk(ctx.cwd);
		if (result.ok && result.set) {
			rules = result.set;
			if (result.seeded) {
				ctx.ui.notify(
					"pi-permissions: seeded default rules into .pi/permissions.json",
					"info",
				);
			}
			return;
		}
		// Fallback: in-memory defaults. Do not overwrite a malformed file.
		rules = buildDefaults();
		ctx.ui.notify(
			`pi-permissions: config error — using in-memory defaults. ${result.error ?? "unknown"}`,
			"warning",
		);
	});

	pi.on("tool_call", async (event, ctx) => {
		if (!isToolCallEventType("bash", event)) return;
		if (!rules) return;

		const command = event.input.command;
		const ruleName = firstMatchingRule(rules, command);
		if (!ruleName) return;

		if (grants.has(ruleName)) return;

		if (!ctx.hasUI) {
			recordBlock(ruleName, command, "no-ui");
			return {
				block: true,
				reason: `pi-permissions: rule '${ruleName}' blocked (no UI to confirm in non-interactive mode)`,
			} satisfies BlockResult;
		}

		const ok = await ctx.ui.confirm(
			"pi-permissions",
			`Allow \`${truncate(command, 120)}\`?\n\nRule: ${ruleName}`,
		);
		if (ok) return;

		recordBlock(ruleName, command, "denied");
		return {
			block: true,
			reason: `pi-permissions: rule '${ruleName}' denied by user`,
		} satisfies BlockResult;
	});

	pi.registerCommand("permissions", {
		description: "Manage pi-permissions rules and session grants",
		handler: async (args, ctx) => {
			const parts = (args ?? "").trim().split(/\s+/);
			const sub = parts[0] ?? "";

			switch (sub) {
				case "":
				case "list":
					showStatus(ctx);
					return;
				case "grant":
					doGrant(parts[1], ctx);
					return;
				case "revoke":
					doRevoke(parts[1], ctx);
					return;
				case "reload":
					await doReload(ctx);
					return;
				default:
					ctx.ui.notify(`pi-permissions: unknown subcommand '${sub}'`, "warning");
					return;
			}
		},
	});
}

function recordBlock(rule: string, command: string, outcome: BlockRecord["outcome"]): void {
	recent.push({ rule, command, outcome, at: Date.now() });
	while (recent.length > RECENT_LIMIT) recent.shift();
}

function doGrant(
	name: string | undefined,
	ctx: { ui: { notify: (m: string, l: "info" | "warning" | "error") => void } },
): void {
	if (!name) {
		ctx.ui.notify("pi-permissions: grant requires a rule name", "warning");
		return;
	}
	if (!rules?.rules.has(name)) {
		ctx.ui.notify(`pi-permissions: rule '${name}' not found in current rules`, "warning");
		return;
	}
	grants.grant(name);
	ctx.ui.notify(`pi-permissions: granted '${name}' for this session`, "info");
}

function doRevoke(
	name: string | undefined,
	ctx: { ui: { notify: (m: string, l: "info" | "warning" | "error") => void } },
): void {
	if (!name) {
		ctx.ui.notify("pi-permissions: revoke requires a rule name", "warning");
		return;
	}
	const removed = grants.revoke(name);
	if (removed) ctx.ui.notify(`pi-permissions: revoked '${name}'`, "info");
	else ctx.ui.notify(`pi-permissions: '${name}' was not granted`, "warning");
}

async function doReload(ctx: {
	cwd: string;
	ui: { notify: (m: string, l: "info" | "warning" | "error") => void };
}): Promise<void> {
	const result = await loadFromDisk(ctx.cwd);
	if (!result.ok || !result.set) {
		ctx.ui.notify(`pi-permissions: reload failed — ${result.error ?? "unknown"}`, "warning");
		return;
	}
	rules = result.set;
	// Drop any grants whose rule names no longer exist.
	const validNames = new Set(result.set.rules.keys());
	for (const name of grants.list()) {
		if (!validNames.has(name)) grants.revoke(name);
	}
	ctx.ui.notify(
		`pi-permissions: reloaded ${result.set.rules.size} rule(s) from ${result.set.source}`,
		"info",
	);
}

function showStatus(ctx: {
	ui: { notify: (m: string, l: "info" | "warning" | "error") => void };
}): void {
	const lines: string[] = [];
	lines.push(`pi-permissions status (source: ${rules?.source ?? "unloaded"})`);
	if (rules) {
		lines.push(`  rules: ${rules.rules.size}`);
		for (const name of rules.rules.keys()) {
			const g = grants.has(name) ? " [granted]" : "";
			lines.push(`    - ${name}${g}`);
		}
	}
	const g = grants.list();
	lines.push(`  session grants: ${g.length === 0 ? "(none)" : g.join(", ")}`);
	lines.push(`  recent blocks: ${recent.length}`);
	for (const r of recent.slice(-5)) {
		lines.push(
			`    - [${new Date(r.at).toISOString()}] ${r.rule} (${r.outcome}): ${truncate(r.command, 80)}`,
		);
	}
	ctx.ui.notify(lines.join("\n"), "info");
}

function truncate(s: string, n: number): string {
	return s.length <= n ? s : s.slice(0, n - 1) + "…";
}