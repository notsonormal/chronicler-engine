// delegate: run a task in a fresh Pi agent in its own herdr pane, and send the
// agent's last message back to the parent session as a message that starts a turn.
//
// One process, two modes:
// - Parent (PI_DELEGATE_JOB unset, inside herdr): registers `delegate` and
//   `delegate_cancel`, opens panes in an "agents" tab, and watches job folders.
// - Child (PI_DELEGATE_JOB set by the parent): registers no tools. When its run
//   settles it writes result.json and shuts down; the shell then exits and
//   herdr closes the pane.
//
// Job folders live in ~/.pi/agent/delegate/<id>/: job.json, task.md,
// result.json, exit (the child's exit code) and status.json (a blocking prompt).
// Roles come from ~/.pi/agent/agents/*.md and <cwd>/.pi/agents/*.md; only
// name, model, thinking and tools are read.
// @ts-nocheck

import { execFile } from "node:child_process";
import { randomUUID } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";

import { StringEnum, Type } from "@earendil-works/pi-ai";

import {
	childCommand,
	formatResult,
	type JobStatus,
	mergeRoles,
	modelLabel,
	pickSplit,
	resultStatus,
	summarizeEntries,
	taskText,
} from "./lib.ts";

const execFileAsync = promisify(execFile);
const JOBS_ROOT = path.join(os.homedir(), ".pi", "agent", "delegate");
const TAB_LABEL = "agents";
const POLL_MS = 2000;
const PANE_CHECK_MS = 15000;
const BLOCKED_SCREEN_LINES = 15;

const readJson = (file) => {
	try {
		return JSON.parse(fs.readFileSync(file, "utf8"));
	} catch {
		return undefined;
	}
};

// The other process polls these files, so it must never see a partial write.
const writeJson = (file, value) => {
	fs.writeFileSync(`${file}.tmp`, `${JSON.stringify(value, null, 2)}\n`);
	fs.renameSync(`${file}.tmp`, file);
};

export default function delegate(pi) {
	const jobDir = process.env.PI_DELEGATE_JOB;
	if (jobDir) return childMode(pi, jobDir);
	if (process.env.HERDR_ENV !== "1") return;
	parentMode(pi);
}

function childMode(pi, jobDir) {
	let done = false;
	// A resumed session already holds the entries of earlier runs.
	let firstEntryOfRun = 0;
	pi.on("session_start", (_event, ctx) => {
		firstEntryOfRun = ctx.sessionManager.getEntries().length;
	});
	pi.on("ui_prompt_start", (event) => {
		writeJson(path.join(jobDir, "status.json"), { state: "blocked", kind: event.kind, title: event.title, at: Date.now() });
	});
	pi.on("ui_prompt_end", () => {
		writeJson(path.join(jobDir, "status.json"), { state: "running", at: Date.now() });
	});
	pi.on("agent_settled", (event, ctx) => {
		// Escape in the pane means a person took over; leave the session open.
		if (done || event.aborted) return;
		done = true;
		const summary = summarizeEntries(ctx.sessionManager.getEntries(), firstEntryOfRun);
		writeJson(path.join(jobDir, "result.json"), {
			...summary,
			sessionFile: ctx.sessionManager.getSessionFile(),
			finishedAt: Date.now(),
		});
		ctx.shutdown();
	});
}

function loadRoles(cwd) {
	const read = (dir) => {
		try {
			return fs
				.readdirSync(dir)
				.filter((f) => f.endsWith(".md"))
				.map((f) => fs.readFileSync(path.join(dir, f), "utf8"));
		} catch {
			return [];
		}
	};
	return mergeRoles([read(path.join(os.homedir(), ".pi", "agent", "agents")), read(path.join(cwd, ".pi", "agents"))]);
}

async function herdr(...args) {
	const { stdout } = await execFileAsync(process.env.HERDR_BIN_PATH || "herdr", args, { timeout: 15000 });
	// `pane run` prints nothing on success, and `pane read` prints plain text.
	if (!stdout.trim().startsWith("{")) return { text: stdout };
	const out = JSON.parse(stdout);
	if (out.error) {
		const error = new Error(`herdr ${args.slice(0, 2).join(" ")}: ${out.error.message}`);
		error.code = out.error.code;
		throw error;
	}
	return out.result;
}

// The root pane of a new "agents" tab, or a split of the largest pane of the
// existing tab.
async function openPane(cwd) {
	const workspace = process.env.HERDR_WORKSPACE_ID;
	const { tabs } = await herdr("tab", "list");
	const tab = tabs.find((t) => t.label === TAB_LABEL && t.workspace_id === workspace);
	if (!tab) {
		const created = await herdr("tab", "create", "--workspace", workspace, "--label", TAB_LABEL, "--cwd", cwd, "--no-focus");
		return created.root_pane.pane_id;
	}
	const { panes } = await herdr("pane", "list", "--workspace", workspace);
	const anyPane = panes.find((p) => p.tab_id === tab.tab_id);
	const { layout } = await herdr("pane", "layout", "--pane", anyPane.pane_id);
	const split = pickSplit(layout.panes);
	const created = await herdr("pane", "split", split.paneId, "--direction", split.direction, "--cwd", cwd, "--no-focus");
	return created.pane.pane_id;
}

function parentMode(pi) {
	const roles = loadRoles(process.cwd());
	if (!roles.size) return;
	const jobs = new Map();
	let ctxRef;
	let timer;
	let polling = false;
	let lastPaneCheck = 0;
	let paneQueue = Promise.resolve();

	const save = (job) => writeJson(path.join(job.dir, "job.json"), job);

	const finish = (job, status: JobStatus, content) => {
		if (!jobs.has(job.id)) return;
		jobs.delete(job.id);
		job.status = status;
		save(job);
		pi.sendMessage(
			{ customType: "delegate-result", content, display: true, details: { id: job.id, status } },
			{ triggerTurn: true, deliverAs: "followUp" },
		);
	};

	// Finishes the job if its child left a result or an exit code.
	const settleFromFiles = (job) => {
		const resultFile = path.join(job.dir, "result.json");
		const result = readJson(resultFile);
		if (result) {
			finish(job, resultStatus(result), formatResult(job, result, resultFile));
			herdr("pane", "close", job.paneId).catch(() => {});
			return true;
		}
		const exitFile = path.join(job.dir, "exit");
		if (fs.existsSync(exitFile)) {
			const code = fs.readFileSync(exitFile, "utf8").trim();
			const where = code === "0" ? "Its pane is closed." : `Pane ${job.paneId} stays open.`;
			finish(job, "failed", `Delegate failed · ${job.label} (${job.id}): the agent exited with code ${code} and no result. ${where}\nSession ${job.sessionId}`);
			return true;
		}
		return false;
	};

	const reportBlocked = async (job, status) => {
		const prompt = status.title ? `${status.kind} prompt "${status.title}"` : `${status.kind} prompt`;
		const screen = await herdr("pane", "read", job.paneId, "--source", "visible", "--lines", String(BLOCKED_SCREEN_LINES))
			.then((out) => out.text.trimEnd())
			.catch(() => "(could not read the pane)");
		ctxRef?.ui?.notify?.(`Delegate ${job.label} waits on a ${prompt} in pane ${job.paneId}`, "warning");
		pi.sendMessage(
			{
				customType: "delegate-blocked",
				content: [
					`Delegate blocked · ${job.label} (${job.id}): the agent waits on a ${prompt} in pane ${job.paneId}.`,
					"It continues when someone answers there. `delegate_cancel` stops it.",
					"",
					"Last lines of the pane:",
					screen,
				].join("\n"),
				display: true,
				details: { id: job.id, paneId: job.paneId },
			},
			{ triggerTurn: true, deliverAs: "followUp" },
		);
	};

	const pollOnce = async () => {
		const checkPanes = Date.now() - lastPaneCheck > PANE_CHECK_MS;
		if (checkPanes) lastPaneCheck = Date.now();
		for (const job of [...jobs.values()]) {
			if (!jobs.has(job.id) || settleFromFiles(job)) continue;
			const status = readJson(path.join(job.dir, "status.json"));
			if (status?.state === "blocked" && job.blockedAt !== status.at) {
				job.blockedAt = status.at;
				save(job);
				await reportBlocked(job, status);
			}
			if (!checkPanes) continue;
			try {
				await herdr("pane", "get", job.paneId);
			} catch (error) {
				// A timeout or a herdr hiccup says nothing about the child.
				if (error.code !== "pane_not_found" || !jobs.has(job.id) || settleFromFiles(job)) continue;
				finish(job, "cancelled", `Delegate cancelled · ${job.label} (${job.id}): its pane closed before a result.\nSession ${job.sessionId}`);
			}
		}
	};

	const poll = async () => {
		if (polling) return;
		polling = true;
		try {
			await pollOnce();
		} catch (error) {
			ctxRef?.ui?.notify?.(`delegate: ${error.message}`, "error");
		} finally {
			polling = false;
		}
		if (!jobs.size && timer) {
			clearInterval(timer);
			timer = undefined;
		}
	};

	const watch = () => {
		if (timer) return;
		timer = setInterval(poll, POLL_MS);
		timer.unref?.();
	};

	pi.on("session_start", (_event, ctx) => {
		ctxRef = ctx;
		const parent = ctx.sessionManager.getSessionId();
		if (!fs.existsSync(JOBS_ROOT)) return;
		for (const id of fs.readdirSync(JOBS_ROOT)) {
			const job = readJson(path.join(JOBS_ROOT, id, "job.json"));
			if (job?.status === "running" && job.parentSessionId === parent) jobs.set(job.id, job);
		}
		if (jobs.size) watch();
	});

	pi.on("session_shutdown", () => {
		if (timer) clearInterval(timer);
		timer = undefined;
		jobs.clear();
	});

	const roleList = [...roles.values()]
		.map((r) => `- ${r.name}: ${modelLabel(r)}${r.tools ? ` (tools: ${r.tools.join(", ")})` : ""}`)
		.join("\n");

	pi.registerTool({
		name: "delegate",
		label: "Delegate",
		description: [
			"Start an agent in its own herdr pane on one task. The call returns at once. The agent's last message comes back later as a delegate-result message, which starts your next turn. Calls in one message run in parallel.",
			"The agent sees `task`, `files` and the project instructions, but not this conversation.",
			"Roles (model, thinking, tools):",
			roleList,
		].join("\n"),
		parameters: Type.Object({
			role: StringEnum([...roles.keys()]),
			task: Type.String({ description: "The task for the agent." }),
			label: Type.Optional(Type.String({ description: "Short name for the pane and the result." })),
			files: Type.Optional(Type.Array(Type.String(), { description: "Paths listed for the agent to read first." })),
			resume: Type.Optional(
				Type.String({
					description: "Session id from an earlier result. The agent continues that session with all its earlier context.",
				}),
			),
		}),
		async execute(_id, params, _signal, _onUpdate, ctx) {
			ctxRef = ctx;
			const role = roles.get(params.role);
			if (!role) throw new Error(`Unknown role ${params.role}`);
			const running = params.resume && [...jobs.values()].find((j) => j.sessionId === params.resume);
			if (running) throw new Error(`Session ${params.resume} is still running as ${running.label} (${running.id}).`);
			const id = randomUUID().slice(0, 8);
			const dir = path.join(JOBS_ROOT, id);
			fs.mkdirSync(dir, { recursive: true });
			const taskFile = path.join(dir, "task.md");
			fs.writeFileSync(taskFile, taskText(params.task, params.files));
			const label = params.label || `${role.name}-${id}`;
			const sessionId = params.resume || randomUUID();
			// Serialized, so that parallel calls never split the same pane.
			const opened = paneQueue.then(() => openPane(ctx.cwd));
			paneQueue = opened.catch(() => {});
			const paneId = await opened;
			const command = childCommand({
				jobDir: dir,
				role,
				sessionId,
				resume: Boolean(params.resume),
				sessionName: `delegate · ${role.name} · ${label}`,
				taskFile,
			});
			await herdr("pane", "rename", paneId, label).catch(() => {});
			await herdr("pane", "run", paneId, command);
			const job = {
				id,
				label,
				role,
				sessionId,
				paneId,
				dir,
				status: "running" as JobStatus,
				parentSessionId: ctx.sessionManager.getSessionId(),
				startedAt: Date.now(),
			};
			save(job);
			jobs.set(id, job);
			watch();
			return {
				content: [
					{
						type: "text",
						text: `Started ${label} (${id}) as ${role.name} on ${role.model} in pane ${paneId}. The result arrives as a message.`,
					},
				],
				details: { id, paneId, role: role.name, sessionId },
			};
		},
	});

	pi.registerTool({
		name: "delegate_cancel",
		label: "Delegate cancel",
		description: "Stop a running delegate by its id and close its pane. No result message follows.",
		parameters: Type.Object({ id: Type.String() }),
		async execute(_id, params) {
			const job = jobs.get(params.id);
			if (!job) throw new Error(`No running delegate ${params.id}`);
			jobs.delete(job.id);
			job.status = "cancelled";
			save(job);
			await herdr("pane", "close", job.paneId).catch(() => {});
			return { content: [{ type: "text", text: `Cancelled ${job.label} (${job.id}).` }], details: { id: job.id } };
		},
	});
}
