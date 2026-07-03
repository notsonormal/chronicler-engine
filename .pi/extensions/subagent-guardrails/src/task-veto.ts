// Feature 1 (#11): Task-spec veto.
// Parent-side tool_call on `subagent` tool. Blocks empty / thin / missing-header
// task specs before any worker launches.

export interface TaskSpecFailure {
	block: true;
	reason: string;
}

export interface TaskSpecPass {
	block: false;
}

export type TaskSpecResult = TaskSpecFailure | TaskSpecPass;

const HEADER_MARKERS = [
	"# Task for worker",
	"# Task for delegate",
	"# Task for scout",
	"# Task for Explore",
	"# Task for reviewer",
	"Task:",
];

const WORKER_MIN_LENGTH = 800;
const DELEGATE_MIN_LENGTH = 200;

export function checkTaskSpec(input: {
	task?: unknown;
	agent?: unknown;
	action?: unknown;
}): TaskSpecResult {
	// Management/control actions (list, get, status, interrupt, resume,
	// append-step, doctor, create, update, delete) don't launch a subagent
	// and don't carry a task spec — nothing to validate.
	if (input.action !== undefined) return { block: false };

	const task = typeof input.task === "string" ? input.task : "";
	const trimmed = task.trim();

	if (trimmed.length < DELEGATE_MIN_LENGTH) {
		return fail(
			`task length ${trimmed.length} chars below minimum ${DELEGATE_MIN_LENGTH}. ` +
				"Write a full task contract per the AGENTS.md worker/delegate template before delegating.",
		);
	}

	const hasHeader = HEADER_MARKERS.some((m) => task.includes(m));
	if (!hasHeader) {
		return fail(
			"task is missing a recognized header marker. " +
				"Start the task with one of: " +
				HEADER_MARKERS.join(", ") +
				" (the pi-subagents fork-preamble marker is `Task:`).",
		);
	}

	const agent =
		typeof input.agent === "string" ? input.agent.trim().toLowerCase() : "";
	// Worker is the most common; undefined agent defaults to worker in pi-subagents.
	const isWorker = agent === "" || agent === "worker";

	if (isWorker && trimmed.length < WORKER_MIN_LENGTH) {
		return fail(
			`worker task length ${trimmed.length} chars below worker minimum ${WORKER_MIN_LENGTH}. ` +
				"Workers need a tight, explicit contract; do not rely on forked context to carry scope.",
		);
	}

	return { block: false };
}

function fail(reason: string): TaskSpecFailure {
	return {
		block: true,
		reason:
			"Subagent task spec failed validation: " +
			reason +
			" Re-write the task per the AGENTS.md worker/delegate template before delegating.",
	};
}
