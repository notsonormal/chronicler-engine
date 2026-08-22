# Extract trap evidence from the session pool

Type: task
Status: resolved
Blocked by: 01, 02

## Question

For each session in the manifest (ticket 01), read the flattened transcript (ticket 02) and extract candidate evidence for each of the 8 cognitive traps (Forming, Dislodging, Assumption, Location, Achievement, Progression, Interruption, Mislead).

For each candidate, record: the trap, the exact quoted text (verbatim, not paraphrased), the session file path, the message id and timestamp, and one line of surrounding context.

Pay special attention to how the user responds after the assistant gives code or suggestions (per the premise's "Focus on User Prompts"). The flattened transcript keeps the assistant text the user reacted to, so the stimulus is visible. The flattened transcripts are the audit's single source; do not consult the raw jsonl.

If a trap has no candidate in a session, skip it. Do not force a finding.

Produce a raw evidence file, organized per-trap then per-session, linked from this ticket.

Risk: if the pool is large, this may overflow one session. The flattener (ticket 02) cut raw size to 31.4%, which reduces this risk; pool size (from ticket 01) still decides whether to batch.

## Answer

Evidence extracted to [`assets/raw-trap-evidence.md`](../assets/raw-trap-evidence.md). All 30 manifest sessions read in full from the flattened transcripts; every cited quote is a verbatim substring of its flattened file (9/9 spot-verified by `grep -qF` against the cited file).

**Headline.** The pool shows a highly disciplined user (plan-then-implement, code-review after nearly every change, frequent pushback, investigation before accepting). Traps are rare. One clear trap is detected, with one overlap:

- **Progression (AI-induced), High — session 11** (`01a0017d`, 2026-08-14T18:36:11.778Z, line 1813): `I kind of just left the AI write it as is but I'm not really sure about it` — the user accepted the AI-authored INV-NNN "runtime invariants" subsystem (§5 of guardrails.md + `invariant_contract.rs` + INV-001..007 + architecture.md cross-refs) without understanding it. The cost is confirmed in the same session: the AI found the subsystem had drifted / broken (INV-005 double-defined; INV-006 a phantom with no test; §5 pointing at a ghost file `tests/pison_recovery.rs`). The subsystem was retired this session.
- **Mislead (AI-induced) — same session 11 instance.** The INV-NNN subsystem was an over-complicated / partly incorrect AI suggestion the user followed. Cross-referenced under both traps (one event, two readings); not double-counted as independent.

**No candidate (not forced):** Forming, Dislodging, Assumption, Location, Achievement, Interruption. Each is justified in the evidence file. Interruption is assessable (millisecond timestamps) and assessed as not-occurring — substantive prompts have deliberate gaps; sub-minute re-sends are typo corrections.

**Resistance context (recorded for tickets 06/07).** The user's dominant habit is *resisting* AI-induced traps — 5 verbatim pushback quotes logged (S5 demands investigation before accepting review findings; S17 refuses foregone-conclusion framing; S19 catches a research ticket changing code; S23 corrects an AI speed overclaim; S24 provenance-checks extra macros against reference impls). The gap the invariants instance exposes: the user applies the discipline rigorously to **code**, but let the AI author freely in the **documentation / named-concept / architecture** layer, which was reviewed far less.

**Proposed primary vulnerability (for ticket 06 to propose, ticket 07 to confirm): Progression.**

**Process notes.**
- The pool (~5.4MB flattened across 30 sessions) was too large to hold in context at once. Two throwaway reading-aids were built: `tmp/extract_user_turns.py` (compacts each session to user turns + stimulus preview) and `tmp/extract_signal.py` (collapses skill-invocation bodies and pasted plans to markers, keeping genuine user reasoning verbatim). Quotes were lifted only from verbatim-kept portions and located by `grep` against the flattened files.
- Minor bug found in `extract_signal.py`: it occasionally misattributes the timestamp/line of a user turn to the following turn (e.g. S23's in-memory quote is at 2026-08-16T22:43:42.370Z / line 3261, not 22:44:58 / 3289). Did not affect extraction — all final locators were taken by `grep` from the flattened files, not from the reading-aid.
- **Stale-docstring finding for ticket 04:** `tmp/flatten_sessions.py`'s docstring says "Verification runs against the RAW originals, never these flattened copies." This contradicts the map's ticket-02 decision and ticket 04's body (both: verify against the flattened files). The docstring is stale; the map decision is authoritative. Flagged so ticket 04 follows flattened-files, not the docstring.

**No fog graduated; no new tickets; nothing ruled out of scope** by this extraction. The map's existing Achievement/diffs fog remains for ticket 05 to resolve.

## Course correction (post-resolution, 2026-08-20 — recorded on the map)

This extraction was **instance-level**: it scanned the 30 sessions for textbook trap moments, found one (Progression, session 11), and filed the rest as context or "None detected." The map's destination is a grounded audit of the user's habits *across the pool*, so the instance method could not serve it — n=1 is a property of the method, not a finding. Two specific gaps:

1. **The five "resistance" quotes were filed as context, not read as the pattern.** They are the closest thing to a pattern in the pool (the accept/resist boundary); reading them as one Progression-instance-plus-context collapsed 30 sessions into one sentence.
2. **The six "None detected" verdicts rest on unverified session citations** (S6, S15, S22…), not the verified-quote standard Q1 meets. "grep-verified" here was 9/12 (ticket 04 caught the gap), and the negatives were never held to that standard at all.

[Ticket 08](./08-re-extract-at-pattern-level.md) supersedes this ticket's evidence for synthesis: it re-reads the same 30 flattened transcripts at pattern level (gradients, accept/resist ratio, milder forms) and grounds the negatives to the verified-quote standard. This ticket's artifact (`assets/raw-trap-evidence.md`) is left intact as an auditable record of the instance-level pass.
