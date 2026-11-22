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
	MAX_RUNNING_JOBS,
	mergeRoles,
	modelLabel,
	pickModel,
	pickSplit,
	queueNotice,
	resultStatus,
	startsNow,
	summarizeEntries,
	taskText,
} from "./lib.ts";
import { readState } from "../rate-limit-wait/state.ts";

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
	// Tasks whose pane is opening: they hold a place, and are reachable by id.
	const starting = new Map();
	// Tasks that wait for a free place, oldest first.
	const pending = [];
	let ctxRef;
	let timer;
	let polling = false;
	let lastPaneCheck = 0;
	let paneQueue = Promise.resolve();

	const save = (job) => writeJson(path.join(job.dir, "job.json"), job);

	const heldJobs = () => [...pending, ...starting.values(), ...jobs.values()];
	const findJob = (id) => jobs.get(id) ?? starting.get(id) ?? pending.find((job) => job.id === id);
	const freePlace = () => startsNow(jobs.size + starting.size);

	const startJob = async (job) => {
		// Serialized, so that parallel calls never split the same pane.
		const opened = paneQueue.then(() => openPane(job.cwd));
		paneQueue = opened.catch(() => {});
		const paneId = await opened;
		// A cancel during the pane open leaves nothing to run.
		if (job.settled) {
			await herdr("pane", "close", paneId).catch(() => {});
			throw new Error("cancelled before it started");
		}
		const pick = pickModel(job.role, readState(), Date.now() / 1000);
		job.role = pick.spare ? { ...job.role, model: pick.model } : job.role;
		job.pick = pick;
		const command = childCommand({
			jobDir: job.dir,
			role: job.role,
			sessionId: job.sessionId,
			resume: job.resume,
			sessionName: `delegate · ${job.role.name} · ${job.label}`,
			taskFile: path.join(job.dir, "task.md"),
		});
		await herdr("pane", "rename", paneId, job.label).catch(() => {});
		// Recorded before the run, so that a restart mid-start cannot start it twice.
		job.paneId = paneId;
		job.status = "starting";
		save(job);
		await herdr("pane", "run", paneId, command);
		job.status = "running";
		job.startedAt = Date.now();
		save(job);
		jobs.set(job.id, job);
		watch();
		return paneId;
	};

	// Holds a place while a task starts, and gives the place back when it cannot.
	const startReserved = async (job) => {
		starting.set(job.id, job);
		try {
			return await startJob(job);
		} finally {
			if (starting.get(job.id) === job) starting.delete(job.id);
			startWaiting();
		}
	};

	const startWaiting = () => {
		while (pending.length && freePlace()) {
			const job = pending.shift();
			startReserved(job).catch((error) =>
				finish(job, "failed", `Delegate failed · ${job.label} (${job.id}): it could not start. ${error.message}`),
			);
		}
	};

	// Drops a task from every queue, so that nothing can start it again.
	const release = (job) => {
		job.settled = true;
		jobs.delete(job.id);
		starting.delete(job.id);
		const at = pending.indexOf(job);
		if (at >= 0) pending.splice(at, 1);
	};

	const finish = (job, status: JobStatus, content) => {
		if (job.settled) return;
		release(job);
		job.status = status;
		save(job);
		pi.sendMessage(
			{ customType: "delegate-result", content, display: true, details: { id: job.id, status } },
			{ triggerTurn: true, deliverAs: "followUp" },
		);
		startWaiting();
	};

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
			if (job?.parentSessionId !== parent) continue;
			if (job.status === "running") jobs.set(job.id, job);
			else if (job.status === "queued") pending.push(job);
			else if (job.status === "starting") {
				// The parent stopped while the pane opened, so the run was never recorded.
				finish(job, "failed", `Delegate failed · ${job.label} (${job.id}): the parent stopped while its pane was opening, so it was not restarted. Pane ${job.paneId} stays open.`);
			}
		}
		pending.sort((a, b) => a.queuedAt - b.queuedAt);
		if (jobs.size) watch();
		startWaiting();
	});

	pi.on("session_shutdown", () => {
		if (timer) clearInterval(timer);
		timer = undefined;
		jobs.clear();
		starting.clear();
		pending.length = 0;
	});

	const roleList = [...roles.values()]
		.map(
			(r) =>
				`- ${r.name}: ${modelLabel(r)}${r.spareModel ? ` (${r.spareModel} while the Anthropic sub has spare capacity)` : ""}${r.tools ? ` (tools: ${r.tools.join(", ")})` : ""}`,
		)
		.join("\n");

	pi.registerTool({
		name: "delegate",
		label: "Delegate",
		description: [
			`Start an agent in its own herdr pane on one task. The call returns at once. The agent's last message comes back later as a delegate-result message, which starts your next turn. Calls in one message start together, but at most ${MAX_RUNNING_JOBS} agents run at once; the rest wait in a queue and start in the order they were asked for, and the answer of such a task says it is queued.`,
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
			const roleDef = roles.get(params.role);
			if (!roleDef) throw new Error(`Unknown role ${params.role}`);
			const running = params.resume && heldJobs().find((j) => j.sessionId === params.resume);
			if (running) {
				throw new Error(`Session ${params.resume} is still running or queued as ${running.label} (${running.id}).`);
			}
			const id = randomUUID().slice(0, 8);
			const dir = path.join(JOBS_ROOT, id);
			fs.mkdirSync(dir, { recursive: true });
			fs.writeFileSync(path.join(dir, "task.md"), taskText(params.task, params.files));
			const label = params.label || `${roleDef.name}-${id}`;
			const job = {
				id,
				label,
				role: roleDef,
				sessionId: params.resume || randomUUID(),
				dir,
				cwd: ctx.cwd,
				resume: Boolean(params.resume),
				status: "queued" as JobStatus,
				parentSessionId: ctx.sessionManager.getSessionId(),
				queuedAt: Date.now(),
			};
			if (!freePlace()) {
				pending.push(job);
				save(job);
				return {
					content: [{ type: "text", text: queueNotice(label, id, pending.length) }],
					details: { id, role: roleDef.name, sessionId: job.sessionId, queued: true },
				};
			}
			let paneId;
			try {
				paneId = await startReserved(job);
			} catch (error) {
				// A cancel already recorded its own status.
				if (!job.settled) {
					job.status = "failed";
					save(job);
				}
				throw error;
			}
			return {
				content: [
					{
						type: "text",
						text: `Started ${label} (${id}) as ${roleDef.name} on ${job.role.model}${roleDef.spareModel ? ` (spare capacity ${job.pick.spare ? "used" : "not used"}: ${job.pick.reason})` : ""} in pane ${paneId}. The result arrives as a message.`,
					},
				],
				details: { id, paneId, role: roleDef.name, sessionId: job.sessionId },
			};
		},
	});

	pi.registerTool({
		name: "delegate_cancel",
		label: "Delegate cancel",
		description: "Stop a running or queued delegate by its id and close its pane. No result message follows.",
		parameters: Type.Object({ id: Type.String() }),
		async execute(_id, params) {
			const job = findJob(params.id);
			if (!job) throw new Error(`No running delegate ${params.id}`);
			release(job);
			job.status = "cancelled";
			save(job);
			if (job.paneId) await herdr("pane", "close", job.paneId).catch(() => {});
			startWaiting();
			const where = job.paneId ? "" : " It had not started.";
			return { content: [{ type: "text", text: `Cancelled ${job.label} (${job.id}).${where}` }], details: { id: job.id } };
		},
	});
}
