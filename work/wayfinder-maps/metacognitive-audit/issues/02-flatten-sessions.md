# Flatten sessions to a clean transcript

Type: task
Status: resolved
Blocked by:

## Question

Build a throwaway flattener that turns each chronicler-engine session jsonl into a clean markdown transcript: keep user messages and assistant text blocks verbatim, keep compaction summaries and session/model metadata, truncate tool results (head+tail, errors flagged) and toolCall/custom markers, drop thinking and image blocks. Preserve the ISO timestamp on every kept entry — Interruption-trap assessability depends on it. The flattened transcripts are the audit's single source; downstream tickets (03, 04, 05) all read them, and verification (04) checks quotes against the flattened files. Safe because user/assistant text is kept byte-identical.

## Answer

Built `tmp/flatten_sessions.py`. It reads a session jsonl (or a directory) and emits one markdown transcript per session with `--out-dir`.

Strip spec, grounded in a schema scan of all 82 chronicler-engine sessions (39.43 MB raw):
- Keep verbatim: `user` messages (1.79 MB), `assistant` `text` blocks (1.24 MB), `compaction` `summary` (0.46 MB), `session`/`model_change` metadata (0.10 MB).
- Keep truncated: `toolResult` (head 1000 + tail 1000, `[ERROR]` flag preserved) — 7.01 MB from 15.72; `toolCall` markers (name + 150-char args) — 1.13 MB from 3.05; `custom`/`custom_message` markers (type + 200-char head) — 0.65 MB from 1.22.
- Drop: `thinking` content blocks (14.24 MB), `image` blocks (0.25 MB), `thinking_level_change`.
- Non-negotiable: every kept entry carries its `timestamp` (millisecond ISO).

Result: kept size 12.39 MB (31.4% of raw), a 3.2x cut. Dropping `thinking` (never visible to the user, so not a stimulus they reacted to) is what earns the cut; keeping it would drop the benefit to 1.5x.

Integrity: the flattener keeps user and assistant text byte-identical, so the flattened transcripts faithfully preserve the quoted content (confirmed by the demo integrity check: zero artifacts leaked). Downstream verification (ticket 04) checks quotes against these flattened files; the byte-identical fidelity is the precondition that makes flattened-only verification safe.

The agent side of the conversation is kept (assistant text blocks), per the user's correction: a user response is uninterpretable without the code or suggestion it responds to.

Asset: `tmp/flatten_sessions.py`.
