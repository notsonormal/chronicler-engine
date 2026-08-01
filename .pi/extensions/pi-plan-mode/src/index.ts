import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join, resolve, sep } from "node:path";
import type {
	ExtensionAPI,
	ExtensionContext,
	ToolInfo,
} from "@earendil-works/pi-coding-agent";

const STATE_ENTRY_TYPE = "plan-mode-state";
const STATUS_KEY = "plan-mode";
const PLAN_WIDGET_KEY = "plan-mode-plan";
const PLAN_CONTEXT_MESSAGE_TYPE = "plan-mode-context";
const PROPOSED_PLAN_MESSAGE_TYPE = "proposed-plan";
const PLAN_MODE_QUESTION_TOOL_NAME = "plan_mode_question";
const PLAN_CONTEXT_MARKER = "[CODEX-LIKE PLAN MODE ACTIVE]";
const PLAN_MODE_TRANSITION_MESSAGE_TYPE = "pi-plan";
const SAFE_BUILTIN_PLAN_TOOLS = new Set(["read", "bash", "grep", "find", "ls"]);
const BLOCKED_BUILTIN_TOOLS = new Set<string>();
const WRITE_EDIT_TOOL_NAMES = ["write", "edit"] as const;
const DEFAULT_TOOLS = ["read", "bash", "edit", "write"];
const TOOL_SELECTOR_PAGE_SIZE = 10;
const PROPOSED_PLAN_PATTERN =
	/<proposed_plan>\s*([\s\S]*?)\s*<\/proposed_plan>/i;
const PROPOSED_PLAN_BLOCK_PATTERN =
	/<proposed_plan>\s*[\s\S]*?\s*<\/proposed_plan>/gi;

interface CommandArgumentCompletion {
	value: string;
	label: string;
	description?: string;
}

interface PlanModeState {
	enabled: boolean;
	latestPlan?: string;
	awaitingAction: boolean;
	selectedToolNames?: string[];
	selectedToolKeys?: string[];
}

interface PlanModeDefaultsConfig {
	defaultTools?: string[];
	planFolder?: string;
	scratchFolders?: string[];
}

export function normalizeRelativePath(value: string): string | undefined {
	const trimmed = value.trim();
	if (trimmed.length === 0) return undefined;
	if (trimmed.startsWith("/")) {
		console.warn(
			`Plan mode: ignoring config path "${trimmed}" (must be cwd-relative).`,
		);
		return undefined;
	}
	const segments = trimmed.split("/");
	if (segments.includes("..")) {
		console.warn(
			`Plan mode: ignoring config path "${trimmed}" (must not contain ".." segment).`,
		);
		return undefined;
	}
	return trimmed;
}

type SessionEntry = {
	type?: string;
	customType?: string;
	data?: Partial<PlanModeState>;
	message?: SessionMessage;
};

type SessionMessage = {
	role?: string;
	content?: unknown;
};

type TextBlock = {
	type?: string;
	text?: string;
};

type PlanModeQuestionOption = {
	label: string;
	description?: string;
};

type PlanModeQuestion = {
	id: string;
	header: string;
	question: string;
	options: PlanModeQuestionOption[];
};

type PlanModeQuestionParams = {
	questions: PlanModeQuestion[];
};

type PlanModeQuestionAnswer = {
	id: string;
	header: string;
	question: string;
	answer: string;
	wasCustom: boolean;
	optionIndex?: number;
};

type PlanModeQuestionReason =
	| "cancelled"
	| "ui_unavailable"
	| "plan_mode_inactive"
	| "invalid_input";

type PlanModeQuestionDetails = {
	cancelled: boolean;
	reason?: PlanModeQuestionReason;
	questions: PlanModeQuestion[];
	answers?: PlanModeQuestionAnswer[];
};

const PLAN_COMMAND_COMPLETIONS: readonly CommandArgumentCompletion[] = [
	{ value: "exit", label: "exit", description: "Leave Plan mode" },
	{ value: "off", label: "off", description: "Leave Plan mode" },
	{
		value: "tools",
		label: "tools",
		description: "Select tools allowed in Plan mode",
	},
	{
		value: "grill",
		label: "grill",
		description: "Stress-test the plan with /grill-with-docs",
	},
];

const PLAN_MODE_QUESTION_PARAMS = {
	type: "object",
	additionalProperties: false,
	required: ["questions"],
	properties: {
		questions: {
			type: "array",
			minItems: 1,
			maxItems: 3,
			description: "Questions to show the user. Prefer 1 and do not exceed 3.",
			items: {
				type: "object",
				additionalProperties: false,
				required: ["id", "header", "question", "options"],
				properties: {
					id: {
						type: "string",
						description: "Stable identifier for mapping answers (snake_case).",
					},
					header: {
						type: "string",
						description:
							"Short header label shown in the UI (12 or fewer chars).",
					},
					question: {
						type: "string",
						description: "Single-sentence prompt shown to the user.",
					},
					options: {
						type: "array",
						minItems: 2,
						maxItems: 4,
						description:
							"Provide 2-4 mutually exclusive choices. Put the recommended option first when there is a clear default.",
						items: {
							type: "object",
							additionalProperties: false,
							required: ["label", "description"],
							properties: {
								label: {
									type: "string",
									description: "User-facing label (1-5 words).",
								},
								description: {
									type: "string",
									description:
										"One short sentence explaining impact/tradeoff if selected.",
								},
							},
						},
					},
				},
			},
		},
	},
} as const;

const MUTATING_BASH_PATTERNS = [
	/\brm\b/i,
	/\brmdir\b/i,
	/\bmv\b/i,
	/\bcp\b/i,
	/\bmkdir\b/i,
	/\btouch\b/i,
	/\bchmod\b/i,
	/\bchown\b/i,
	/\bchgrp\b/i,
	/\bln\b/i,
	/\btee\b/i,
	/\btruncate\b/i,
	/\bdd\b/i,
	/(^|[^<])>(?!>)/,
	/>>/,
	/\bnpm\s+(install|uninstall|update|ci|link|publish|version)\b/i,
	/\byarn\s+(add|remove|install|publish|upgrade)\b/i,
	/\bpnpm\s+(add|remove|install|publish|update)\b/i,
	/\bbun\s+(add|remove|install|update|publish)\b/i,
	/\bpip\s+(install|uninstall)\b/i,
	/\buv\s+(add|remove|sync|lock|pip\s+install)\b/i,
	/\bgit\s+(add|commit|push|pull|merge|rebase|reset|checkout|switch|stash|cherry-pick|revert|tag|init|clone)\b/i,
	/\bsudo\b/i,
	/\bsu\b/i,
	/\bkill\b/i,
	/\bpkill\b/i,
	/\bkillall\b/i,
	/\breboot\b/i,
	/\bshutdown\b/i,
	/\bsystemctl\s+(start|stop|restart|enable|disable)\b/i,
	/\bservice\s+\S+\s+(start|stop|restart)\b/i,
	/\b(vim?|nano|emacs|code|subl)\b/i,
];

const SAFE_BASH_PATTERNS = [
	/^\s*(cat|head|tail|less|more|grep|find|ls|pwd|echo|printf|wc|sort|uniq|diff|file|stat|du|df|tree|which|whereis|type|env|printenv|uname|whoami|id|date|uptime|ps|jq|awk|rg|fd|bat|eza)\b/i,
	/^\s*sed\s+-n\b/i,
	/^\s*git\s+(status|log|diff|show|branch|remote|config\s+--get|ls-files|grep)\b/i,
	/^\s*npm\s+(list|ls|view|info|search|outdated|audit)\b/i,
	/^\s*(node|python|python3|npm|tsc|biome|ruff|ty)\s+--version\b/i,
];

export default function planMode(pi: ExtensionAPI) {
	let state: PlanModeState = { enabled: false, awaitingAction: false };
	let previousTools: string[] | undefined;
	let planModeDefaultsConfig: PlanModeDefaultsConfig | undefined;

	function resolvedPlanFolder(): string | undefined {
		return planModeDefaultsConfig?.planFolder;
	}

	function resolvedScratchFolders(): string[] {
		return planModeDefaultsConfig?.scratchFolders ?? [];
	}

	function sessionAllowedFolders(): string[] {
		const planFolder = resolvedPlanFolder();
		return planFolder
			? [planFolder, ...resolvedScratchFolders()]
			: [...resolvedScratchFolders()];
	}

	type SubcommandHandler = (
		prompt: string,
		ctx: ExtensionContext,
	) => void | Promise<void>;

	function buildPlanSubcommands() {
		const exitPlanSub: SubcommandHandler = (_prompt, ctx) => {
			exitPlanMode(ctx);
		};
		const toolsSub: SubcommandHandler = async (_prompt, ctx) => {
			if (!state.enabled) enterPlanMode(ctx);
			await showToolSelector(ctx);
		};
		const grillSub: SubcommandHandler = (_prompt, ctx) => {
			if (!state.enabled) {
				ctx.ui.notify("Enter Plan mode first before grilling.", "warning");
				return;
			}
			sendPlanModeUserMessage("/grill-with-docs", ctx);
		};
		const record: Record<string, SubcommandHandler> = {
			exit: exitPlanSub,
			off: exitPlanSub,
			tools: toolsSub,
			grill: grillSub,
		};
		return record;
	}

	pi.registerFlag("plan", {
		description: "Start in Codex-like Plan mode",
		type: "boolean",
		default: false,
	});

	pi.registerFlag("plan-tools", {
		description:
			"Comma-separated tool names to enable by default when entering Plan mode (e.g. read,bash,grep,find,ls,describe_image). Overrides .pi/plan-mode.json for the current session.",
		type: "string",
	});

	pi.registerTool({
		name: PLAN_MODE_QUESTION_TOOL_NAME,
		label: "Plan question",
		description:
			"Ask the user one to three Plan-mode clarification questions with meaningful options, then wait for the answer. Only available while Plan mode is active.",
		promptSnippet: "Ask user decision questions while Plan mode is active",
		promptGuidelines: [
			"In Plan mode, use plan_mode_question for important preferences, tradeoffs, or assumptions that cannot be discovered from read-only exploration.",
		],
		parameters: PLAN_MODE_QUESTION_PARAMS,
		async execute(_toolCallId, params: unknown, _signal, _onUpdate, ctx) {
			if (!state.enabled) {
				return planModeQuestionCancelled(
					[],
					"plan_mode_inactive",
					"Error: plan_mode_question is only available while Plan mode is active.",
				);
			}

			const parsed = normalizePlanModeQuestionParams(params);
			if (!parsed.ok) {
				return planModeQuestionCancelled(
					[],
					"invalid_input",
					`Error: ${parsed.error}`,
				);
			}

			if (!ctx.hasUI) {
				return planModeQuestionCancelled(
					parsed.questions,
					"ui_unavailable",
					"Unable to ask Plan-mode questions because interactive UI is not available.",
				);
			}

			const answers = await askPlanModeQuestions(parsed.questions, ctx);
			if (!answers) {
				return planModeQuestionCancelled(
					parsed.questions,
					"cancelled",
					"User cancelled the Plan-mode question prompt.",
				);
			}

			return planModeQuestionAnswered(parsed.questions, answers);
		},
	});

	pi.registerCommand("plan", {
		description: "Enter or manage Codex-like Plan mode",
		getArgumentCompletions: completePlanArguments,
		handler: async (args, ctx) => {
			const prompt = args.trim();
			const command = prompt.toLowerCase();
			const subcommand = buildPlanSubcommands()[command];
			if (subcommand) {
				await subcommand(prompt, ctx);
				return;
			}
			if (prompt) {
				enterPlanModeWithPrompt(prompt, ctx);
				return;
			}
			if (!state.enabled) {
				enterPlanMode(ctx);
				ctx.ui.notify(
					"Plan mode enabled. I will explore and plan, but not modify files.",
					"info",
				);
				return;
			}
			await showPlanMenu(ctx);
		},
	});

	pi.on("session_start", (_event, ctx) => {
		planModeDefaultsConfig = resolveDefaultToolsConfig(
			ctx.cwd,
			planToolsFlagOverride(),
		);
		restoreState(ctx);
		if (pi.getFlag("plan") === true) state.enabled = true;
		if (state.enabled) {
			activatePlanModeTools();
			pi.sendMessage(
				{
					customType: PLAN_MODE_TRANSITION_MESSAGE_TYPE,
					content: "<pi-plan>Entering plan mode</pi-plan>",
					display: false,
				},
				{ triggerTurn: false },
			);
		} else {
			deactivatePlanModeQuestionTool();
		}
		updateUi(ctx);
	});

	pi.on("session_shutdown", (_event, ctx) => {
		persistState();
		clearUi(ctx);
	});

	pi.on("tool_call", async (event, ctx) => {
		if (!state.enabled) return;
		if (isBlockedBuiltinToolName(event.toolName)) {
			return {
				block: true,
				reason: `Plan mode blocks built-in mutating tool '${event.toolName}'. Use /plan and choose implementation when the plan is ready.`,
			};
		}
		if (event.toolName === "write" || event.toolName === "edit") {
			const allowed = sessionAllowedFolders();
			const decision = evaluateWriteEdit(event.input, allowed, ctx.cwd);
			if (!decision.allowed) {
				return { block: true, reason: decision.reason };
			}
			return;
		}
		if (event.toolName !== "bash" || !isBuiltinToolName(event.toolName)) return;

		const command = readCommand(event.input);
		if (!isSafeCommand(command)) {
			return {
				block: true,
				reason: `Plan mode blocks mutating or non-allowlisted bash commands.\nCommand: ${command}`,
			};
		}
	});

	pi.on("context", async (event) => {
		const messagesWithoutLegacyPlanContext = event.messages.filter(
			(message: unknown) =>
				!messageContainsLegacyPlanModeContextArtifact(message),
		);
		if (state.enabled) return { messages: messagesWithoutLegacyPlanContext };
		return {
			messages: messagesWithoutLegacyPlanContext
				.filter(
					(message: unknown) =>
						!messageContainsInactivePlanModeArtifact(message),
				)
				.map(stripProposedPlanBlocksFromMessage),
		};
	});

	pi.on("before_agent_start", (event, ctx) => {
		if (!state.enabled) return;
		if (state.latestPlan || state.awaitingAction) {
			state = { ...state, latestPlan: undefined, awaitingAction: false };
			persistState();
			updateUi(ctx);
		}
		applyPlanModeTools();
		return {
			systemPrompt: `${event.systemPrompt}\n\n${buildPlanModePrompt(resolvedPlanFolder(), resolvedScratchFolders())}`,
		};
	});

	pi.on("agent_end", async (event, ctx) => {
		if (!state.enabled) return;

		const text = latestAssistantText(event.messages);
		const proposedPlan = extractProposedPlan(text);
		if (!proposedPlan) {
			persistState();
			updateUi(ctx);
			return;
		}

		state = { ...state, latestPlan: proposedPlan, awaitingAction: true };
		persistState();
		updateUi(ctx);

		scheduleAfterCurrentAgentRun(async () => {
			if (!state.enabled || state.latestPlan !== proposedPlan) return;
			if (ctx.hasUI) await showPlanReadyMenu(ctx);
			if (!state.enabled || state.latestPlan !== proposedPlan) return;

			pi.sendMessage(
				{
					customType: PROPOSED_PLAN_MESSAGE_TYPE,
					content: `**Proposed Plan**\n\n${proposedPlan}`,
					display: true,
				},
				{ triggerTurn: false },
			);
		});
	});

	function enterPlanMode(ctx: ExtensionContext) {
		if (!state.enabled)
			previousTools = withoutPlanModeQuestionTool(safeGetActiveTools());
		state = { ...state, enabled: true, awaitingAction: false };
		activatePlanModeTools();
		persistState();
		pi.sendMessage(
			{
				customType: PLAN_MODE_TRANSITION_MESSAGE_TYPE,
				content: "<pi-plan>Entering plan mode</pi-plan>",
				display: false,
			},
			{ triggerTurn: false },
		);
		updateUi(ctx);
	}

	function enterPlanModeWithPrompt(prompt: string, ctx: ExtensionContext) {
		const wasEnabled = state.enabled;
		enterPlanMode(ctx);
		if (!wasEnabled) {
			ctx.ui.notify(
				"Plan mode enabled. I will explore and plan, but not modify files.",
				"info",
			);
		}
		sendPlanModeUserMessage(prompt, ctx);
	}

	function persistPlanToFile(ctx: ExtensionContext): string | undefined {
		const plan = state.latestPlan?.trim();
		if (!plan) return undefined;
		const planFolder = resolvedPlanFolder();
		if (!planFolder) {
			ctx.ui.notify(
				"Plan mode: no planFolder configured; skipping persistence. Set planFolder in .pi/plan-mode.json to enable.",
				"warning",
			);
			return undefined;
		}
		try {
			return writePlanFile(plan, ctx.cwd, planFolder, `${plan}\n`);
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			ctx.ui.notify(
				`Plan mode: failed to persist plan to ${planFolder} (${message}).`,
				"error",
			);
			return undefined;
		}
	}

	function exitPlanMode(ctx: ExtensionContext) {
		const wasEnabled = state.enabled;
		if (!wasEnabled) return;
		const persistedPath = persistPlanToFile(ctx);
		state = {
			...state,
			enabled: false,
			latestPlan: undefined,
			awaitingAction: false,
		};
		restoreTools();
		persistState();
		pi.sendMessage(
			{
				customType: PLAN_MODE_TRANSITION_MESSAGE_TYPE,
				content: "<pi-plan>Exiting plan mode</pi-plan>",
				display: false,
			},
			{ triggerTurn: false },
		);
		updateUi(ctx);
		if (persistedPath) {
			ctx.ui.notify(`Plan written to: ${persistedPath}`, "info");
		} else {
			ctx.ui.notify("Plan mode disabled.", "info");
		}
	}

	function sendPlanModeUserMessage(message: string, ctx: ExtensionContext) {
		if (ctx.isIdle()) pi.sendUserMessage(message);
		else pi.sendUserMessage(message, { deliverAs: "followUp" });
	}

	function scheduleAfterCurrentAgentRun(task: () => Promise<void> | void) {
		setTimeout(() => {
			void Promise.resolve(task()).catch((error: unknown) => {
				const message = error instanceof Error ? error.message : String(error);
				console.error(`Plan mode follow-up failed: ${message}`);
			});
		}, 0);
	}

	function startImplementation(ctx: ExtensionContext) {
		const plan = state.latestPlan?.trim();
		exitPlanMode(ctx);

		if (!plan) {
			ctx.ui.notify(
				"Plan mode disabled. No proposed plan is available to implement.",
				"warning",
			);
			return;
		}

		sendPlanModeUserMessage(
			`Plan mode is now disabled. Full tool access is restored. Implement this proposed plan now:\n\n${plan}`,
			ctx,
		);
	}

	async function showPlanMenu(ctx: ExtensionContext) {
		if (!ctx.hasUI) {
			ctx.ui.notify(planStatusText(), "info");
			return;
		}

		const choices = state.latestPlan
			? [
					"Show latest proposed plan",
					"Implement this plan",
					"Configure Plan-mode tools",
					"Stay in Plan mode",
					"Exit Plan mode",
				]
			: ["Configure Plan-mode tools", "Stay in Plan mode", "Exit Plan mode"];
		const choice = await ctx.ui.select(planStatusText(), choices);
		if (choice === "Show latest proposed plan") {
			ctx.ui.notify(state.latestPlan ?? "No proposed plan yet.", "info");
			return;
		}
		if (choice === "Implement this plan") {
			startImplementation(ctx);
			return;
		}
		if (choice === "Configure Plan-mode tools") {
			await showToolSelector(ctx);
			return;
		}
		if (choice === "Exit Plan mode") {
			exitPlanMode(ctx);
			return;
		}
		updateUi(ctx);
	}

	async function showPlanReadyMenu(ctx: ExtensionContext) {
		const choice = await ctx.ui.select("Proposed plan ready. What next?", [
			"Implement this plan",
			"Stay in Plan mode",
			"Exit Plan mode",
		]);
		if (choice === "Implement this plan") {
			startImplementation(ctx);
			return;
		}
		if (choice === "Exit Plan mode") {
			exitPlanMode(ctx);
		}
	}

	async function showToolSelector(ctx: ExtensionContext) {
		if (!ctx.hasUI) {
			ctx.ui.notify(formatToolSummary(), "info");
			return;
		}

		let pageIndex = 0;
		while (true) {
			const tools = selectableTools();
			const pageCount = toolSelectorPageCount(tools);
			pageIndex = Math.min(pageIndex, pageCount - 1);
			const pageStart = pageIndex * TOOL_SELECTOR_PAGE_SIZE;
			const pageTools = tools.slice(
				pageStart,
				pageStart + TOOL_SELECTOR_PAGE_SIZE,
			);
			const selectedNames = planModeSelectedNames(tools);
			const choices = pageTools.map((tool, index) =>
				formatToolChoice(tool, selectedNames.has(tool.name), pageStart + index),
			);
			const previousChoice = "Previous page";
			const nextChoice = "Next page";
			const doneChoice = "Done";
			const navigationChoices = [
				...(pageIndex > 0 ? [previousChoice] : []),
				...(pageIndex < pageCount - 1 ? [nextChoice] : []),
				doneChoice,
			];
			const choice = await ctx.ui.select(
				`Plan-mode tools (${pageIndex + 1}/${pageCount}). Non-built-in tools run at user risk.`,
				[...choices, ...navigationChoices],
			);
			if (!choice || choice === doneChoice) break;
			if (choice === previousChoice) {
				pageIndex = Math.max(0, pageIndex - 1);
				continue;
			}
			if (choice === nextChoice) {
				pageIndex = Math.min(pageCount - 1, pageIndex + 1);
				continue;
			}

			const selectedIndex = choices.indexOf(choice);
			const tool = pageTools[selectedIndex];
			if (!tool) continue;
			if (!canSelectToolInPlanMode(tool, sessionAllowedFolders())) {
				ctx.ui.notify(`${tool.name} is blocked in Plan mode.`, "warning");
				continue;
			}

			const nextSelectedNames = planModeSelectedNames(tools);
			if (nextSelectedNames.has(tool.name)) nextSelectedNames.delete(tool.name);
			else nextSelectedNames.add(tool.name);

			state = {
				...state,
				selectedToolNames: filterAvailableSelectedNames(
					Array.from(nextSelectedNames),
					tools,
					sessionAllowedFolders(),
				),
			};
			applyPlanModeTools();
			persistState();
			updateUi(ctx);
		}

		applyPlanModeTools();
		persistState();
		updateUi(ctx);
	}

	function activatePlanModeTools() {
		previousTools ??= withoutPlanModeQuestionTool(safeGetActiveTools());
		applyPlanModeTools();
	}

	function applyPlanModeTools() {
		pi.setActiveTools(planModeToolNames());
	}

	function planModeToolNames() {
		const tools = selectableTools();
		if (tools.length === 0)
			return ["read", "bash", PLAN_MODE_QUESTION_TOOL_NAME];

		const allowed = sessionAllowedFolders();
		const selectedNames = planModeSelectedNames(tools);
		return withRequiredPlanModeTools(
			tools
				.filter(
					(tool) =>
						selectedNames.has(tool.name) &&
						canSelectToolInPlanMode(tool, allowed),
				)
				.map((tool) => tool.name),
		);
	}

	function planModeSelectedNames(tools: ToolInfo[]) {
		const selectedToolNames =
			state.selectedToolNames ?? migrateSelectedToolKeys(tools);
		if (selectedToolNames === undefined)
			return new Set(defaultPlanModeToolNames(tools));

		const allowed = sessionAllowedFolders();
		state = {
			...state,
			selectedToolNames: filterAvailableSelectedNames(
				selectedToolNames,
				tools,
				allowed,
			),
			selectedToolKeys: undefined,
		};
		return new Set(state.selectedToolNames);
	}

	function defaultPlanModeToolNames(tools: ToolInfo[]) {
		return resolveInitialPlanModeToolNames(tools, planModeDefaultsConfig);
	}

	function migrateSelectedToolKeys(tools: ToolInfo[]) {
		if (state.selectedToolKeys === undefined) return undefined;
		return state.selectedToolKeys
			.map((key) => toolNameFromLegacyKey(key, tools))
			.filter((name): name is string => name !== undefined);
	}

	function filterAvailableSelectedNames(
		names: string[],
		tools: ToolInfo[],
		allowed: string[],
	) {
		const availableNames = new Set(
			tools
				.filter((tool) => canSelectToolInPlanMode(tool, allowed))
				.map((tool) => tool.name),
		);
		return unique(names.filter((name) => availableNames.has(name)));
	}

	function selectableTools() {
		return safeGetAllTools()
			.filter((tool) => tool.name !== PLAN_MODE_QUESTION_TOOL_NAME)
			.sort(compareTools);
	}

	function toolSelectorPageCount(tools: ToolInfo[]) {
		return Math.max(1, Math.ceil(tools.length / TOOL_SELECTOR_PAGE_SIZE));
	}

	function safeGetAllTools() {
		try {
			return pi.getAllTools();
		} catch {
			return [];
		}
	}

	function restoreTools() {
		const restoredTools =
			previousTools && previousTools.length > 0 ? previousTools : DEFAULT_TOOLS;
		pi.setActiveTools(withoutPlanModeQuestionTool(restoredTools));
		previousTools = undefined;
	}

	function deactivatePlanModeQuestionTool() {
		const activeTools = safeGetActiveTools();
		const filteredTools = withoutPlanModeQuestionTool(activeTools);
		if (filteredTools.length !== activeTools.length) {
			pi.setActiveTools(filteredTools);
		}
	}

	function safeGetActiveTools() {
		try {
			return pi.getActiveTools();
		} catch {
			return DEFAULT_TOOLS;
		}
	}

	function planToolsFlagOverride(): string[] | undefined {
		const flagValue = pi.getFlag("plan-tools");
		if (typeof flagValue !== "string" || flagValue.trim() === "")
			return undefined;
		return flagValue
			.split(",")
			.map((name) => name.trim())
			.filter((name) => name.length > 0);
	}

	function persistState() {
		pi.appendEntry<PlanModeState>(STATE_ENTRY_TYPE, state);
	}

	function restoreState(ctx: ExtensionContext) {
		const entries = ctx.sessionManager.getEntries() as SessionEntry[];
		const entry = entries
			.filter(
				(candidate) =>
					candidate.type === "custom" &&
					candidate.customType === STATE_ENTRY_TYPE,
			)
			.pop();
		if (!entry?.data) return;
		const enabled = entry.data.enabled ?? false;
		state = {
			enabled,
			latestPlan: enabled ? entry.data.latestPlan : undefined,
			awaitingAction: enabled ? (entry.data.awaitingAction ?? false) : false,
			selectedToolNames: entry.data.selectedToolNames,
			selectedToolKeys: entry.data.selectedToolKeys,
		};
	}

	function updateUi(ctx: ExtensionContext) {
		ctx.ui.setStatus(STATUS_KEY, formatStatus());
		if (state.enabled && state.latestPlan) {
			ctx.ui.setWidget(PLAN_WIDGET_KEY, [
				"Proposed plan ready",
				"Use /plan to implement, revise, or exit Plan mode.",
			]);
		} else if (state.enabled) {
			ctx.ui.setWidget(PLAN_WIDGET_KEY, [
				"Plan mode: planning",
				formatToolSummary(),
				"Produce a <proposed_plan> block.",
			]);
		} else {
			ctx.ui.setWidget(PLAN_WIDGET_KEY, undefined);
		}
	}

	function formatStatus() {
		if (!state.enabled) return undefined;
		if (state.awaitingAction || state.latestPlan) return "plan ready";
		return "plan active";
	}

	function clearUi(ctx: ExtensionContext) {
		ctx.ui.setStatus(STATUS_KEY, undefined);
		ctx.ui.setWidget(PLAN_WIDGET_KEY, undefined);
	}

	function planStatusText() {
		if (!state.enabled) return "Plan mode is off.";
		if (state.latestPlan)
			return `Plan mode is active and a proposed plan is ready. ${formatToolSummary()}`;
		return `Plan mode is active. ${formatToolSummary()} Explore, ask, and produce a <proposed_plan> block.`;
	}

	function formatToolSummary() {
		const names = planModeToolNames();
		return `Tools: ${names.length > 0 ? names.join(", ") : "none"}`;
	}

	function isBlockedBuiltinToolName(toolName: string) {
		if (!BLOCKED_BUILTIN_TOOLS.has(toolName)) return false;
		const tool = toolByName(toolName);
		return tool ? isBuiltinTool(tool) : true;
	}

	function isBuiltinToolName(toolName: string) {
		const tool = toolByName(toolName);
		return tool ? isBuiltinTool(tool) : toolName === "bash";
	}

	function toolByName(toolName: string) {
		return safeGetAllTools().find((candidate) => candidate.name === toolName);
	}
}

function isBuiltinTool(tool: ToolInfo) {
	return tool.sourceInfo.source === "builtin";
}

const PLAN_MODE_PROJECT_CONFIG_PATH = join(".pi", "plan-mode.json");
const PLAN_MODE_USER_CONFIG_PATH = join(".pi", "plan-mode.json");

export function resolveDefaultToolsConfig(
	cwd: string,
	override: string[] | undefined,
	env: NodeJS.ProcessEnv = process.env,
	home: string = homedir(),
): PlanModeDefaultsConfig {
	if (override !== undefined) {
		return override.length > 0 ? { defaultTools: override } : {};
	}
	const projectPath = join(cwd, PLAN_MODE_PROJECT_CONFIG_PATH);
	const userPath = env.PLAN_MODE_CONFIG
		? env.PLAN_MODE_CONFIG
		: join(home, PLAN_MODE_USER_CONFIG_PATH);
	const loaded =
		loadDefaultToolsConfigFromPath(projectPath) ??
		loadDefaultToolsConfigFromPath(userPath);
	return loaded ?? {};
}

export function loadDefaultToolsConfigFromPath(
	configPath: string,
): PlanModeDefaultsConfig | undefined {
	if (!existsSync(configPath)) return undefined;
	let parsed: unknown;
	try {
		const raw = readFileSync(configPath, "utf-8");
		parsed = JSON.parse(raw);
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		console.warn(`Plan mode: failed to read ${configPath} (${message}).`);
		return undefined;
	}
	if (!isRecord(parsed)) {
		console.warn(`Plan mode: ignoring config at ${configPath} (not an object).`);
		return undefined;
	}

	type FieldVerdict =
		| { ok: true; value: unknown }
		| { ok: false; fatal: boolean; reason: string };
	type FieldLoader = (raw: unknown) => FieldVerdict;

	const fields: Record<keyof PlanModeDefaultsConfig, FieldLoader> = {
		defaultTools: (raw) => {
			if (raw === undefined) return { ok: true, value: undefined };
			if (!Array.isArray(raw) || !raw.every((s) => typeof s === "string")) {
				return {
					ok: false,
					fatal: true,
					reason: "defaultTools must be a string array",
				};
			}
			const trimmed = raw.map((s) => s.trim()).filter((s) => s.length > 0);
			return { ok: true, value: trimmed.length > 0 ? trimmed : undefined };
		},
		planFolder: (raw) => {
			if (raw === undefined) return { ok: true, value: undefined };
			if (typeof raw !== "string") {
				return { ok: false, fatal: false, reason: "planFolder must be a string" };
			}
			return { ok: true, value: normalizeRelativePath(raw) };
		},
		scratchFolders: (raw) => {
			if (raw === undefined) return { ok: true, value: undefined };
			if (!Array.isArray(raw) || !raw.every((s) => typeof s === "string")) {
				return {
					ok: false,
					fatal: false,
					reason: "scratchFolders must be a string array",
				};
			}
			const normalized = raw
				.map((s) => normalizeRelativePath(s))
				.filter((s): s is string => s !== undefined);
			return {
				ok: true,
				value: normalized.length > 0 ? normalized : undefined,
			};
		},
	};

	const result: PlanModeDefaultsConfig = {};
	let fatal = false;
	for (const key of Object.keys(fields) as (keyof PlanModeDefaultsConfig)[]) {
		const verdict = fields[key](parsed[key]);
		if (!verdict.ok) {
			console.warn(
				`Plan mode: ignoring ${key} at ${configPath} (${verdict.reason}).`,
			);
			if (verdict.fatal) {
				fatal = true;
				break;
			}
			continue;
		}
		if (verdict.value !== undefined) {
			(result as Record<string, unknown>)[key] = verdict.value;
		}
	}
	if (fatal) return undefined;
	return result;
}

export function completePlanArguments(
	argumentPrefix: string,
): CommandArgumentCompletion[] | null {
	const prefix = argumentPrefix.trimStart().toLowerCase();
	if (prefix === "") return [...PLAN_COMMAND_COMPLETIONS];
	if (/\s/.test(prefix)) return null;

	const matches = PLAN_COMMAND_COMPLETIONS.filter((item) =>
		item.value.startsWith(prefix),
	);
	return matches.length > 0 ? [...matches] : null;
}

export function canSelectToolInPlanMode(
	tool: ToolInfo,
	allowedFolders: string[],
) {
	if (isBuiltinTool(tool)) {
		if (tool.name === "write" || tool.name === "edit") {
			return allowedFolders.length > 0;
		}
		return SAFE_BUILTIN_PLAN_TOOLS.has(tool.name);
	}
	return true;
}

/**
 * Resolve the initial tool set for a fresh Plan-mode session.
 *
 * When `planFolder` or any `scratchFolders` are configured, `write`/`edit` are
 * auto-appended (unless already explicitly listed in `defaultTools`). This
 * honours the README contract that writes are allowed inside those folders.
 * Without allowed folders, write/edit are excluded so read-only planning stays
 * strict.
 */
export function resolveInitialPlanModeToolNames(
	tools: ToolInfo[],
	config: PlanModeDefaultsConfig | undefined,
): string[] {
	const allowed = [
		...(config?.planFolder ? [config.planFolder] : []),
		...(config?.scratchFolders ?? []),
	];
	const byName = new Map(tools.map((tool) => [tool.name, tool]));

	const applyWriteEditDefault = (names: Set<string>): void => {
		if (allowed.length === 0) return;
		for (const name of WRITE_EDIT_TOOL_NAMES) {
			if (names.has(name)) continue;
			const tool = byName.get(name);
			if (tool !== undefined && canSelectToolInPlanMode(tool, allowed)) {
				names.add(name);
			}
		}
	};

	const configured = config?.defaultTools;
	if (configured && configured.length > 0) {
		const explicit = new Set(
			configured.filter((name) => {
				const tool = byName.get(name);
				return tool !== undefined && canSelectToolInPlanMode(tool, allowed);
			}),
		);
		// When defaultTools lists neither write nor edit, treat that as
		// "no opinion" and auto-add both. If the user mentions either by name
		// we trust the explicit list and skip auto-add for both.
		const userPickedWriteOrEdit = WRITE_EDIT_TOOL_NAMES.some((name) =>
			explicit.has(name),
		);
		if (!userPickedWriteOrEdit) {
			applyWriteEditDefault(explicit);
		}
		return [...explicit];
	}

	const names = new Set(
		tools
			.filter(
				(tool) =>
					isBuiltinTool(tool) && SAFE_BUILTIN_PLAN_TOOLS.has(tool.name),
			)
			.map((tool) => tool.name),
	);
	applyWriteEditDefault(names);
	return [...names];
}

function toolNameFromLegacyKey(key: string, tools: ToolInfo[]) {
	const directName = tools.find((tool) => tool.name === key)?.name;
	if (directName) return directName;
	const [name] = key.split("\u001f");
	return tools.find((tool) => tool.name === name) ? name : undefined;
}

function compareTools(left: ToolInfo, right: ToolInfo) {
	const leftBuiltin = isBuiltinTool(left);
	const rightBuiltin = isBuiltinTool(right);
	if (leftBuiltin !== rightBuiltin) return leftBuiltin ? -1 : 1;
	return left.name.localeCompare(right.name);
}

function formatToolChoice(tool: ToolInfo, selected: boolean, index: number) {
	const marker = selected ? "[x]" : "[ ]";
	return `${marker} ${index + 1}. ${tool.name} (${toolPolicyLabel(tool)})`;
}

function toolPolicyLabel(tool: ToolInfo) {
	if (isBuiltinTool(tool)) {
		if (!SAFE_BUILTIN_PLAN_TOOLS.has(tool.name)) return "built-in blocked";
		return tool.name === "bash" ? "built-in limited" : "built-in";
	}
	return `user risk: ${toolSourceLabel(tool)}`;
}

function toolSourceLabel(tool: ToolInfo) {
	const sourceInfo = tool.sourceInfo;
	const source = `${sourceInfo.scope}/${sourceInfo.source}`;
	return sourceInfo.path ? `${source} ${sourceInfo.path}` : source;
}

function unique(values: string[]) {
	return Array.from(new Set(values));
}

export function withRequiredPlanModeTools(toolNames: string[]) {
	return unique([
		...withoutPlanModeQuestionTool(toolNames),
		PLAN_MODE_QUESTION_TOOL_NAME,
	]);
}

export function withoutPlanModeQuestionTool(toolNames: string[]) {
	return toolNames.filter(
		(toolName) => toolName !== PLAN_MODE_QUESTION_TOOL_NAME,
	);
}

type NormalizePlanModeQuestionParamsResult =
	| { ok: true; questions: PlanModeQuestion[] }
	| { ok: false; error: string };

export function normalizePlanModeQuestionParams(
	input: unknown,
): NormalizePlanModeQuestionParamsResult {
	if (!isRecord(input) || !Array.isArray(input.questions)) {
		return { ok: false, error: "questions must be an array" };
	}
	if (input.questions.length < 1 || input.questions.length > 3) {
		return { ok: false, error: "questions must contain 1-3 items" };
	}

	const questions: PlanModeQuestion[] = [];
	for (const [questionIndex, rawQuestion] of input.questions.entries()) {
		if (!isRecord(rawQuestion)) {
			return {
				ok: false,
				error: `question ${questionIndex + 1} must be an object`,
			};
		}

		const id = stringField(rawQuestion.id);
		const header = stringField(rawQuestion.header);
		const question = stringField(rawQuestion.question);
		if (!id || !header || !question) {
			return {
				ok: false,
				error: `question ${questionIndex + 1} requires non-empty id, header, and question`,
			};
		}

		if (!Array.isArray(rawQuestion.options)) {
			return {
				ok: false,
				error: `question ${questionIndex + 1} options must be an array`,
			};
		}
		if (rawQuestion.options.length < 2 || rawQuestion.options.length > 4) {
			return {
				ok: false,
				error: `question ${questionIndex + 1} options must contain 2-4 items`,
			};
		}

		const options: PlanModeQuestionOption[] = [];
		for (const [optionIndex, rawOption] of rawQuestion.options.entries()) {
			if (!isRecord(rawOption)) {
				return {
					ok: false,
					error: `question ${questionIndex + 1} option ${optionIndex + 1} must be an object`,
				};
			}

			const label = stringField(rawOption.label);
			if (!label) {
				return {
					ok: false,
					error: `question ${questionIndex + 1} option ${optionIndex + 1} requires a label`,
				};
			}
			const description = stringField(rawOption.description);
			if (!description) {
				return {
					ok: false,
					error: `question ${questionIndex + 1} option ${optionIndex + 1} requires a description`,
				};
			}
			options.push({ label, description });
		}

		questions.push({ id, header, question, options });
	}

	return { ok: true, questions };
}

async function askPlanModeQuestions(
	questions: PlanModeQuestion[],
	ctx: ExtensionContext,
): Promise<PlanModeQuestionAnswer[] | undefined> {
	const answers: PlanModeQuestionAnswer[] = [];
	for (const question of questions) {
		const choices = question.options.map(formatPlanModeQuestionChoice);
		const otherChoice = `${question.options.length + 1}. Other (free-form)`;
		const choice = await ctx.ui.select(
			`${question.header}: ${question.question}`,
			[...choices, otherChoice],
		);
		if (!choice) return undefined;

		if (choice === otherChoice) {
			const customAnswer = (await ctx.ui.editor(question.question, ""))?.trim();
			if (!customAnswer) return undefined;
			answers.push({
				id: question.id,
				header: question.header,
				question: question.question,
				answer: customAnswer,
				wasCustom: true,
			});
			continue;
		}

		const optionIndex = choices.indexOf(choice);
		const option = question.options[optionIndex];
		if (!option) return undefined;
		answers.push({
			id: question.id,
			header: question.header,
			question: question.question,
			answer: option.label,
			wasCustom: false,
			optionIndex: optionIndex + 1,
		});
	}
	return answers;
}

function formatPlanModeQuestionChoice(
	option: PlanModeQuestionOption,
	index: number,
) {
	return `${index + 1}. ${option.label}${option.description ? ` — ${option.description}` : ""}`;
}

function planModeQuestionAnswered(
	questions: PlanModeQuestion[],
	answers: PlanModeQuestionAnswer[],
) {
	return {
		content: [
			{
				type: "text" as const,
				text: formatPlanModeQuestionPayload({ cancelled: false, answers }),
			},
		],
		details: {
			cancelled: false,
			questions,
			answers,
		} satisfies PlanModeQuestionDetails,
	};
}

function planModeQuestionCancelled(
	questions: PlanModeQuestion[],
	reason: PlanModeQuestionReason,
	message: string,
) {
	return {
		content: [
			{
				type: "text" as const,
				text: formatPlanModeQuestionPayload({
					cancelled: true,
					reason,
					message,
				}),
			},
		],
		details: {
			cancelled: true,
			reason,
			questions,
		} satisfies PlanModeQuestionDetails,
	};
}

function formatPlanModeQuestionPayload(payload: {
	cancelled: boolean;
	reason?: PlanModeQuestionReason;
	message?: string;
	answers?: PlanModeQuestionAnswer[];
}) {
	return JSON.stringify(payload, null, 2);
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringField(value: unknown) {
	return typeof value === "string" ? value.trim() : undefined;
}

function describeAllowedFolders(
	planFolder: string | undefined,
	scratchFolders: string[],
): string {
	if (!planFolder && scratchFolders.length === 0) {
		return "No planFolder or scratchFolders are configured in .pi/plan-mode.json; all writes are blocked by default. Set at least planFolder to enable writes during planning.";
	}
	const parts: string[] = [];
	if (planFolder) parts.push(`${planFolder} (plan folder)`);
	if (scratchFolders.length > 0) {
		parts.push(`${scratchFolders.join(", ")} (scratch folders)`);
	}
	return [
		`- Write/edit tools: allowed only inside ${parts.join(" and ")}.`,
		"- Bash: allowlisted to read-only / non-mutating commands regardless of folder (no `>`, `>>`, installs, `rm`, `mv`, etc.).",
		"- Final plan auto-persists to the plan folder when the user exits Plan Mode from the ready menu (implement/stay/exit); the extension writes it, not the agent.",
	].join("\n");
}

function buildPlanModePrompt(
	planFolder: string | undefined,
	scratchFolders: string[],
) {
	const allowedDescription = describeAllowedFolders(planFolder, scratchFolders);
	return `${PLAN_CONTEXT_MARKER}
# Plan Mode

You are in Plan Mode. Chat your way to a decision-complete implementation plan, then emit a <proposed_plan> block.

## Mode rules

- Stay in Plan Mode until a developer or extension explicitly exits it.
- Treat requests to implement as requests to plan the implementation.
- Do not perform mutating actions outside the plan folder and scratch folders.
- Do not use update_plan/TODO tooling in Plan Mode; Plan Mode is conversational planning.
- ${allowedDescription}
- Use /grill-me-with-docs to stress-test the plan before finalizing if the user wants grilling. Mention it; do not auto-invoke.

### Story points

- Sizes: 1, 3, 5, 8, 13
- 8 SP or larger → must break into subtasks
- 5 SP = single worker session; primary agent must verify output
- Sub tasks optional for atomic tasks ≤5 SP; required for tasks >5 SP
- SP mandatory on every Task line

### Subagents

When using subagents to execute tasks, you need to make sure that task has proper per task validation criteria. Running cargo check and cargo test at minimum for code changes, or running build.py for more complex tasks. It is extremely disruptive if the primary agent has to cleanup after the subagent.

## Phase 1 — Ground in the environment

- Explore first and ask second. Use non-mutating exploration to read files, search, inspect configuration, run read-only checks, and resolve discoverable facts.
- Before asking the user any question, perform at least one targeted non-mutating exploration pass unless no local environment or repository is available.
- Do not ask questions that can be answered from repository or system truth. Ask only when multiple plausible choices remain, a needed identifier/context is missing, or the ambiguity is product intent.

## Phase 2 — Intent chat

- Keep asking until you can clearly state the goal, success criteria, in/out of scope, constraints, current state, and key preferences/tradeoffs.
- Bias toward questions over guessing: if a high-impact ambiguity remains, do not produce a proposed plan yet.

## Phase 3 — Implementation chat

- Once intent is stable, keep asking until the spec is decision-complete: approach, interfaces, data flow, edge cases/failure modes, testing and acceptance criteria, and any migration or compatibility constraints.
- Use plan_mode_question for important preferences, tradeoffs, or assumption locks that cannot be discovered by non-mutating exploration. Ask 1-3 concise questions with 2-4 meaningful options. Do not include filler options.
- If plan_mode_question returns cancelled or ui_unavailable, do not jump straight to a final plan when the missing answer is high impact. Ask one concise plain-text question or proceed only with a clearly stated low-risk assumption.

## Finalization rule

Only output the final plan when it is decision-complete and leaves no decisions to the implementer. When presenting the official plan, output exactly one proposed plan block and keep the tags exactly as shown:

<proposed_plan>
# Title

## Summary
...

## Key Changes
...

## Implementation

Use phases when the work spans multiple distinct stages. Skip the Phase
heading entirely for single-stage work.

### Phase 1: [Stage Name]

- [ ] #### Task 1.1: [Title] (N SP)
  - [ ] ##### SubTask 1.1.1: [Title] (N SP)
  - [ ] ##### SubTask 1.1.2: [Title] (N SP)
- [ ] #### Task 1.2: [Title] (N SP)

### Phase 2: [Stage Name]
...

## Test Plan
...

## Per Task/Sub Task Validation Steps
...

## Assumptions
...
</proposed_plan>

Keep the proposed plan concise, human and agent digestible, and free of open decisions. Do not ask "should I proceed?" in the final output; the Plan-mode ready menu handles implementation, staying in Plan mode, or exit.`;
}

function readCommand(input: unknown) {
	const command = input as { command?: unknown } | undefined;
	return typeof command?.command === "string" ? command.command : "";
}

export function isSafeCommand(command: string) {
	const trimmed = command.trim();
	if (!trimmed) return false;
	if (MUTATING_BASH_PATTERNS.some((pattern) => pattern.test(trimmed)))
		return false;
	return SAFE_BASH_PATTERNS.some((pattern) => pattern.test(trimmed));
}

export type WriteDecision = { allowed: true } | { allowed: false; reason: string };

/**
 * Decide whether a write/edit tool call's target file falls inside one of the
 * allowed folders. Returns `allowed: true` when the input doesn't expose a
 * recognizable path (e.g. legacy tool form); the caller treats that as a
 * pass-through to other handlers.
 *
 * Pi built-in write/edit tools pass `{ path: string }` as input.
 * Schema source: https://github.com/earendil-works/pi (built-in tool defs).
 */
export function evaluateWriteEdit(
	input: unknown,
	allowedFolders: string[],
	cwd: string,
): WriteDecision {
	const candidate = input as { path?: unknown } | undefined;
	const filePath = typeof candidate?.path === "string" ? candidate.path : undefined;
	if (filePath === undefined) return { allowed: true };
	if (allowedFolders.length === 0) {
		return {
			allowed: false,
			reason: `Plan mode blocks write/edit because no planFolder or scratchFolders are configured in .pi/plan-mode.json. Set at least planFolder to enable writes during planning.`,
		};
	}
	const resolvedFile = resolve(cwd, filePath);
	const inside = allowedFolders.some((folder) =>
		isPathInsideFolder(resolvedFile, resolve(cwd, folder)),
	);
	if (!inside) {
		return {
			allowed: false,
			reason: `Plan mode blocks writes outside the plan folder and scratch folders. Allowed (relative to ${cwd}): ${allowedFolders.join(", ")}`,
		};
	}
	return { allowed: true };
}

/**
 * True iff `filePath` resolves to a location strictly inside `folderPath`.
 * Equal paths return false (you can't write to a path that equals a folder).
 * Both inputs are resolved so symlinks and `.`/`..` collapse before compare.
 */
export function isPathInsideFolder(
	filePath: string,
	folderPath: string,
): boolean {
	const normalizedFile = resolve(filePath);
	const normalizedFolder = resolve(folderPath);
	if (normalizedFile === normalizedFolder) return false;
	const prefix = normalizedFolder.endsWith(sep)
		? normalizedFolder
		: normalizedFolder + sep;
	return normalizedFile.startsWith(prefix);
}

export function extractProposedPlan(text: string) {
	const match = PROPOSED_PLAN_PATTERN.exec(text);
	return match?.[1]?.trim();
}

export function derivePlanSlug(planContent: string): string {
	const lines = planContent.split("\n");
	for (const line of lines) {
		const trimmed = line.trim();
		if (trimmed.startsWith("# ")) {
			const title = trimmed.slice(2).trim();
			if (title.length === 0) break;
			const slugified = title
				.toLowerCase()
				.replace(/[^a-z0-9]+/g, "-")
				.replace(/^-+|-+$/g, "")
				.slice(0, 60);
			if (slugified.length > 0) return slugified;
			break;
		}
	}
	const stamp = new Date()
		.toISOString()
		.replace(/[-:]/g, "")
		.replace(/T/, "-")
		.replace(/\..*$/, "");
	return `plan-${stamp}`;
}

/**
 * First candidate path for a given plan slug in the configured folder.
 * Collision resolution is handled atomically by `writePlanAtomically`
 * (or any caller using the `wx` flag) to avoid TOCTOU races between
 * `exists` and `write`.
 */
export function resolvePlanFilePath(
	planContent: string,
	cwd: string,
	planFolder: string,
): string {
	const slug = derivePlanSlug(planContent);
	return join(cwd, planFolder, `${slug}.md`);
}

/**
 * Write `content` to a fresh file under `folder` derived from `planContent`.
 * Uses `wx` flag (O_CREAT | O_EXCL) for atomic creation; on EEXIST, retries
 * with `-2`, `-3`, ... suffix. Returns the actual path written.
 */
export function writePlanFile(
	planContent: string,
	cwd: string,
	planFolder: string,
	content: string,
): string {
	mkdirSync(join(cwd, planFolder), { recursive: true });
	const folder = join(cwd, planFolder);
	const baseSlug = derivePlanSlug(planContent);
	let nextSuffix = 0;
	while (true) {
		const candidate =
			nextSuffix === 0
				? join(folder, `${baseSlug}.md`)
				: join(folder, `${baseSlug}-${nextSuffix}.md`);
		try {
			writeFileSync(candidate, content, { flag: "wx", encoding: "utf-8" });
			return candidate;
		} catch (error) {
			const code = (error as NodeJS.ErrnoException | undefined)?.code;
			if (code !== "EEXIST") throw error;
			nextSuffix = nextSuffix === 0 ? 2 : nextSuffix + 1;
		}
	}
}

export function latestAssistantText(messages: unknown) {
	if (!Array.isArray(messages)) return "";
	for (const entry of [...messages].reverse()) {
		const message =
			(entry as { message?: SessionMessage })?.message ??
			(entry as SessionMessage);
		if (message?.role !== "assistant") continue;
		const text = messageText(message);
		if (text) return text;
	}
	return "";
}

function messageContainsLegacyPlanModeContextArtifact(message: unknown) {
	const candidate = unwrapSessionMessage(message);
	return candidate.customType === PLAN_CONTEXT_MESSAGE_TYPE;
}

function messageContainsInactivePlanModeArtifact(message: unknown) {
	const candidate = unwrapSessionMessage(message);
	return candidate.customType === PROPOSED_PLAN_MESSAGE_TYPE;
}

export function stripProposedPlanBlocksFromMessage<T>(message: T): T {
	const candidate = unwrapSessionMessage(message);
	if (candidate.role !== "assistant") return message;

	const content = stripProposedPlanBlocksFromContent(candidate.content);
	if (content === candidate.content) return message;

	if (isSessionMessageEntry(message)) {
		return { ...message, message: { ...candidate, content } };
	}
	return { ...candidate, content } as T;
}

function unwrapSessionMessage(message: unknown) {
	const entry = message as { message?: unknown };
	return (entry.message ?? message) as {
		role?: string;
		customType?: string;
		content?: unknown;
	};
}

function isSessionMessageEntry<T>(
	message: T,
): message is T & { message: SessionMessage } {
	return (
		typeof message === "object" && message !== null && "message" in message
	);
}

function stripProposedPlanBlocksFromContent(content: unknown) {
	if (typeof content === "string") return stripProposedPlanBlocks(content);
	if (!Array.isArray(content)) return content;

	let changed = false;
	const nextContent = content.map((block) => {
		const textBlock = block as TextBlock;
		if (textBlock.type !== "text" || typeof textBlock.text !== "string")
			return block;

		const text = stripProposedPlanBlocks(textBlock.text);
		if (text === textBlock.text) return block;

		changed = true;
		return { ...textBlock, text };
	});
	return changed ? nextContent : content;
}

export function stripProposedPlanBlocks(text: string) {
	return text.replace(PROPOSED_PLAN_BLOCK_PATTERN, "");
}

function messageText(message: SessionMessage) {
	return contentText(message.content);
}

function contentText(content: unknown): string {
	if (typeof content === "string") return content;
	if (!Array.isArray(content)) return "";
	return content
		.map((block) => {
			const textBlock = block as TextBlock;
			return textBlock.type === "text" && typeof textBlock.text === "string"
				? textBlock.text
				: "";
		})
		.filter(Boolean)
		.join("\n");
}
