// Pure helpers of the delegate extension: roles, the child command, the
// result text and the pane split. index.ts holds the Pi and herdr wiring.
// Tests: node --test .pi/extensions/delegate/lib.test.ts

export interface Role {
	name: string;
	model: string;
	thinking?: string;
	tools?: string[];
}

export type JobStatus = "running" | "done" | "failed" | "cancelled";

const unquote = (value: string) => value.trim().replace(/^["']|["']$/g, "");

// `tools` is a flow array (on one line or over several), a comma list on one
// line, or a YAML block list. Returns null for an unclosed flow array, so that
// a role meant to be read-only never starts with every tool.
function parseTools(lines: string[]): string[] | undefined | null {
	const at = lines.findIndex((line) => line.startsWith("tools:"));
	if (at < 0) return undefined;
	const inline = lines[at].slice("tools:".length).trim();
	const next = lines[at + 1]?.trim() ?? "";
	let items: string[] = [];
	if (inline.startsWith("[") || (!inline && next.startsWith("["))) {
		const rest = lines.slice(at + 1);
		const nextKey = rest.findIndex((line) => /^[A-Za-z_][\w-]*:/.test(line));
		const flow = [inline, ...(nextKey < 0 ? rest : rest.slice(0, nextKey))].join("\n");
		const close = flow.indexOf("]");
		if (close < 0) return null;
		items = flow.slice(flow.indexOf("[") + 1, close).split(",");
	} else if (inline) {
		items = inline.split(",");
	} else {
		for (const line of lines.slice(at + 1)) {
			const item = line.match(/^\s+-\s*(.+)$/);
			if (!item) break;
			items.push(item[1]);
		}
	}
	const tools = items.map(unquote).filter(Boolean);
	return tools.length ? tools : undefined;
}

// A role file is Markdown with YAML-like frontmatter. Only `name`, `model`,
// `thinking` and `tools` are read; other keys and the body are ignored, so the
// pi-herdsman definition files in the same folders stay valid.
export function parseRole(text: string): Role | null {
	const frontmatter = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
	if (!frontmatter) return null;
	const lines = frontmatter[1].split(/\r?\n/);
	const scalar = (key: string) => {
		const line = lines.find((l) => l.startsWith(`${key}:`));
		return line ? unquote(line.slice(key.length + 1)) || undefined : undefined;
	};
	const name = scalar("name");
	const model = scalar("model");
	const tools = parseTools(lines);
	if (!name || !model || tools === null) return null;
	const role: Role = { name, model };
	const thinking = scalar("thinking");
	if (thinking) role.thinking = thinking;
	if (tools) role.tools = tools;
	return role;
}

// Later sources win, so a project role overrides a global role of the same name.
export function mergeRoles(sources: string[][]): Map<string, Role> {
	const roles = new Map<string, Role>();
	for (const texts of sources) {
		for (const text of texts) {
			const role = parseRole(text);
			if (role) roles.set(role.name, role);
		}
	}
	return roles;
}

export function modelLabel(role: Role): string {
	return role.thinking ? `${role.model}:${role.thinking}` : role.model;
}

export function shellQuote(value: string): string {
	return `'${value.replace(/'/g, `'\\''`)}'`;
}

// Orchestration tools of other installed packages. A child must not delegate.
export const CHILD_EXCLUDED_TOOLS = [
	"list_agents",
	"delegate_agent",
	"continue_agent",
	"steer_agent",
	"interrupt_agent",
	"reply_agent",
	"close_agent",
	"inspect_agent",
	"read_agent_transcript",
	"message_supervisor",
	"list_peers",
	"message_peer",
];

export interface ChildLaunch {
	jobDir: string;
	role: Role;
	sessionId: string;
	resume: boolean;
	sessionName: string;
	taskFile: string;
}

// The shell line that runs in the child pane. A clean exit closes the shell
// and so the pane; a crash leaves the pane open for inspection.
export function childCommand(launch: ChildLaunch): string {
	const args = [
		"pi",
		"--model",
		shellQuote(launch.role.model),
		launch.resume ? "--session" : "--session-id",
		shellQuote(launch.sessionId),
		"--exclude-tools",
		shellQuote(CHILD_EXCLUDED_TOOLS.join(",")),
	];
	if (launch.role.thinking) args.push("--thinking", shellQuote(launch.role.thinking));
	if (launch.role.tools?.length) args.push("--tools", shellQuote(launch.role.tools.join(",")));
	if (!launch.resume) args.push("--name", shellQuote(launch.sessionName));
	args.push(shellQuote(`@${launch.taskFile}`));
	const exitFile = shellQuote(`${launch.jobDir}/exit`);
	return `PI_DELEGATE_JOB=${shellQuote(launch.jobDir)} ${args.join(" ")}; code=$?; echo $code > ${exitFile}; [ $code -eq 0 ] && exit`;
}

export function taskText(task: string, files: string[] | undefined): string {
	const parts = [task.trim()];
	if (files?.length) parts.push(`Files to read first:\n${files.map((f) => `- ${f}`).join("\n")}`);
	parts.push("Your last message is returned to the agent that delegated this task, and this session then ends.");
	return parts.join("\n\n");
}

export interface RunSummary {
	text: string;
	stopReason?: string;
	turns: number;
	peakContext: number;
	outputTokens: number;
	cost: number;
}

interface EntryLike {
	type?: string;
	message?: {
		role?: string;
		content?: unknown;
		stopReason?: string;
		usage?: { input?: number; output?: number; cacheRead?: number; cacheWrite?: number; cost?: { total?: number } };
	};
}

function assistantText(content: unknown): string {
	if (typeof content === "string") return content.trim();
	if (!Array.isArray(content)) return "";
	return content
		.filter((part) => part?.type === "text")
		.map((part) => part.text ?? "")
		.join("\n")
		.trim();
}

// Summarizes the entries from `fromIndex` on, so that a resumed session
// reports only its new run.
export function summarizeEntries(entries: EntryLike[], fromIndex = 0): RunSummary {
	const summary: RunSummary = { text: "", turns: 0, peakContext: 0, outputTokens: 0, cost: 0 };
	for (const entry of entries.slice(fromIndex)) {
		const message = entry.message;
		if (entry.type !== "message" || message?.role !== "assistant") continue;
		summary.turns++;
		const usage = message.usage ?? {};
		const context = (usage.input ?? 0) + (usage.cacheRead ?? 0) + (usage.cacheWrite ?? 0);
		summary.peakContext = Math.max(summary.peakContext, context);
		summary.outputTokens += usage.output ?? 0;
		summary.cost += usage.cost?.total ?? 0;
		const text = assistantText(message.content);
		if (text) summary.text = text;
		summary.stopReason = message.stopReason;
	}
	return summary;
}

export interface JobInfo {
	id: string;
	label: string;
	role: Role;
	sessionId: string;
	startedAt: number;
}

export interface ChildResult extends RunSummary {
	sessionFile?: string;
	finishedAt: number;
}

// A run that ended on a model error has no trustworthy last message.
export function resultStatus(result: ChildResult): JobStatus {
	return result.stopReason === "error" ? "failed" : "done";
}

export function formatResult(job: JobInfo, result: ChildResult, resultFile: string, maxChars = 20000): string {
	const minutes = ((result.finishedAt - job.startedAt) / 60000).toFixed(1);
	const heading = resultStatus(result) === "failed" ? "Delegate failed (model error)" : "Delegate result";
	let text = result.text || "(the agent ended without a text message)";
	if (text.length > maxChars) text = `${text.slice(0, maxChars)}\n\n[truncated; full result in ${resultFile}]`;
	return [
		`${heading} · ${job.label} (${job.id}) · role ${job.role.name} · ${modelLabel(job.role)}`,
		`${minutes} min · ${result.turns} turns · peak context ${Math.round(result.peakContext / 1000)}k · ` +
			`output ${Math.round(result.outputTokens / 1000)}k · $${result.cost.toFixed(3)} · ended: ${result.stopReason ?? "?"}`,
		`Session ${job.sessionId}`,
		"",
		text,
	].join("\n");
}

export interface PaneRect {
	pane_id: string;
	rect: { width: number; height: number };
}

// A terminal cell is about twice as tall as it is wide, so a pane at least
// twice as wide as it is tall splits to the right.
export function pickSplit(panes: PaneRect[]): { paneId: string; direction: "right" | "down" } | null {
	if (!panes.length) return null;
	const largest = panes.reduce((a, b) => (b.rect.width * b.rect.height > a.rect.width * a.rect.height ? b : a));
	return { paneId: largest.pane_id, direction: largest.rect.width >= 2 * largest.rect.height ? "right" : "down" };
}
