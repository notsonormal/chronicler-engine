// Feature 1 (#11): Task-spec veto.
// Parent-side tool_call on `subagent` tool. Blocks empty / thin task specs
// before any worker launches. Length-only check; pi-subagents does not
// naturally produce the `# Task for X` headers some parent templates emit,
// so header-presence checks were removed in favor of length floors.

export interface TaskSpecFailure {
	block: true;
	reason: string;
}

export interface TaskSpecPass {
	block: false;
}

export type TaskSpecResult = TaskSpecFailure | TaskSpecPass;

const WORKER_MIN_LENGTH = 500;
const DELEGATE_MIN_LENGTH = 80;

export function checkTaskSpec(input: {
	task?: unknown;
	agent?: unknown;
	action?: unknown;
}): TaskSpecResult {
	// Management/control actions (list, get, status, interrupt, resume,
	// append-step, doctor, create, update, delete) don't launch a subagent
	// and don't carry a task spec — nothing to validate.
	if (input.action !== undefined) return { block: false };

	const trimmed = (typeof input.task === "string" ? input.task : "").trim();
	const agent =
		typeof input.agent === "string" ? input.agent.trim().toLowerCase() : "";
	// Worker is the most common; undefined agent defaults to worker in pi-subagents.
	const workerOrDefault = agent === "" || agent === "worker";
	const floor = workerOrDefault ? WORKER_MIN_LENGTH : DELEGATE_MIN_LENGTH;

	if (trimmed.length < floor) {
		const msg = workerOrDefault
			? `worker task length ${trimmed.length} chars below worker minimum ${WORKER_MIN_LENGTH}. ` +
				"Workers need a tight, explicit contract; do not rely on forked context to carry scope."
			: `task length ${trimmed.length} chars below minimum ${DELEGATE_MIN_LENGTH}. ` +
				"Write a full task contract per the AGENTS.md worker/delegate template before delegating.";
		return fail(msg);
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
