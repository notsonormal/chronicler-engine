# Per-trap transcript assessability — metacognitive audit

Built by [ticket 05](../issues/05-declare-per-trap-assessability.md). Feeds synthesis
([ticket 06](../issues/06-synthesize-audit-report.md)): ticket 06 attempts only the traps marked
assessable here, and writes the rest as "not assessable from transcript (reason)" — never as
"None detected" (the Q7=B standard).

## Method

**Question (Q7=B).** For each of the 8 traps, can a *chat transcript* ground a verdict, or is another
signal required? This is a question about the **medium's capability**, judged from what the
flattened view actually carries — not about whether the trap occurred in this pool (that was
ticket 03's job).

**Source judged.** The flattened transcripts in `tmp/flattened_pool/` (built by
[`tmp/flatten_sessions.py`](../../tmp/flatten_sessions.py) per [ticket 02](../issues/02-flatten-sessions.md)).
The audit works from the flattened files, so assessability is judged from the flattened view, not
the raw jsonl.

**What the flattened view carries** (verified by reading samples — session 11 around lines 1–60
and 1780–1819; session 17 around lines 1160–1190; timestamp scans on sessions 11 and 8):

| Signal | In flattened view? | Notes |
|---|---|---|
| User text (decisions, admissions, pushback, design reasoning) | Yes, verbatim | Primary signal for user-side traps |
| Assistant text (suggestions, investigation findings, summaries) | Yes, verbatim | Primary signal for AI-side traps; carries the "stimulus" the user reacted to |
| Inter-prompt timing | Yes — millisecond ISO timestamp on every `## @ <ts> — user` header | Grounds Interruption (gap between user prompts is exactly measurable) |
| Tool results (file reads, test runs, diffs) | **Truncated** — head 1000 + tail 1000 chars, errors flagged | Full code/test state is NOT fully visible; the assistant's *summary* of a tool result is in assistant text (preserved) |
| Compaction summaries | Yes, verbatim | Lets long sessions stay interpretable across compaction |
| Thinking blocks | **Dropped** | Not a signal available to the audit (traps are about the *user*, not the AI's private reasoning, so this is a non-loss) |
| Images | Dropped | Non-loss for these traps |

**Key limitation from flattening.** Tool-result truncation means a trap whose *only* evidence lives
inside a long tool output (a full test log, a full file body) may be partially hidden. In practice
the audit routes around this because the assistant *surfaces* the relevant part of a tool result in
its own text (preserved) — e.g. session 11's drift findings (INV-005 double-defined, phantom
INV-006, ghost file) appear in the assistant's "Finding." block, not only in the raw `grep` tool
output. So truncation is a *risk to watch*, not a blanket disqualifier.

## Assessability table

Verdicts are "Assessable from transcript" or "Not assessable from transcript (reason)." One trap
(Achievement) takes a split verdict, explained below.

| # | Trap | Verdict | Reason |
|---|---|---|---|
| 1 | Forming (right goal, wrong mental model) | **Assessable from transcript** | A wrong mental model shows up as the user's *verbalized* model (what they say X is/does) and the decisions it drives; the assistant's correction or the eventual outcome is in the transcript. Session 11 shows this directly: the user asks "What does invarient even mean?" — the not-understanding is verbalized. Limit: a *silent* wrong model (never stated, never manifesting in a decision) cannot be caught from transcript alone. This is a general limit of transcript audit, not pool-specific. |
| 2 | Dislodging (repeating a failing approach instead of pivoting) | **Assessable from transcript** | The failing approach and its repeats are the user's own prompts, in order, verbatim; the failure is visible in the assistant's responses (or the user's own "this still doesn't work"). The whole attempt sequence is in the transcript. |
| 3 | Assumption (solving a hardcoded/narrow case, not generalized) | **Assessable from transcript** | The user's *framing* — "just handle this one case", "hardcode X for now" — is the primary signal and is verbatim. Whether the resulting code is truly narrow (vs the framing) is secondary and is usually surfaced by the assistant commenting on the code's generality (assistant text, preserved). Confirming against the actual code would strengthen a positive finding but is not required to *attempt* the assessment. |
| 4 | Location (skipping core architecture, realizing too late) | **Assessable from transcript** | Both halves of the trap are verbalized: the skip (user decides to defer architecture) and the realization (user's later "I should have done X first", or the assistant surfacing it late). Session 22 shows the realization half ("the map is wrong"). |
| 5 | Achievement (endless band-aid fixes instead of refactoring) | **Assessable at instance level; NOT assessable at pattern level without diffs** | See the split verdict below. |
| 6 | Progression / AI (accepting generated code beyond understanding) | **Assessable from transcript** | The acceptance and the (lack of) understanding are both in user text — session 11, line 1813: "I kind of just left the AI write it as is but I'm not really sure about it". The AI's suggestion is in assistant text. No external signal needed. |
| 7 | Interruption / AI (prompting within seconds of confusion, not reasoning through the pause) | **Assessable from transcript** | Millisecond timestamps on every entry make the gap between user prompts exactly measurable (confirmed: session 11 has a 46s gap; session 8 a 38s gap). "Confusion" is inferred from the prompt content (verbatim), which separates confusion-prompting from trivial re-sends (typo corrections). The map's established fact (timestamps are millisecond-precise) settles this. |
| 8 | Mislead / AI (blindly following off-target/over-complicated/incorrect AI suggestions) | **Assessable from transcript** | The suggestion (assistant text) and the user's follow-vs-resist response (user text) are both verbatim. Whether the suggestion was "off-target/incorrect" is judgeable from the assistant's own later correction — session 11 surfaces the drift in the same transcript — or from the outcome. The "blindly" qualifier is the user's acceptance-without-verification, which is in user text. |

## The Achievement split verdict (resolves the map's open question)

The map's **Not yet specified** flagged Achievement as possibly needing code-diff history. The
ticket asked: can the transcript alone ground it, or must it be marked not-assessable (graduating a
diffs-research ticket)?

**The trap definition has two parts.** "Endless band-aid fixes *instead of* refactoring" — a
*single* band-aid-vs-refactor decision, and an *endless/repeated* pattern of them.

- **Instance level — assessable from transcript.** A single band-aid-vs-refactor decision is
  grounded when the user verbalizes it (e.g. "just patch it for now" vs "let's refactor this").
  Ticket 03 found the user verbalizes the *refactor* side in this pool — "Yeah, fix it now" to
  *remove* a band-aid (the `is_cfg_test` `ponytail:` shortcut, session 20), and the INV-NNN
  invariants *retirement* (session 11). Those are instance-level verdicts, fully grounded in the
  transcript.
- **Pattern level — NOT assessable from transcript alone.** The "endless/repeated" qualifier is a
  *frequency* claim over code changes. A transcript shows only *verbalized* decisions; unspoken
  small patches to the same area do not appear unless the user mentions them. Git diff history
  (the change log on the affected files) grounds the accumulation reliably regardless of
  verbalization. So a **positive** Achievement finding that asserts "the user repeatedly band-aided
  area X" would require diff history to confirm — that is the case which graduates the diffs-research
  ticket from the map's fog.

**Consequence for this pool.** Ticket 03 made a **negative** Achievement finding (the user favors
refactoring and *removes* band-aids), grounded in verbalized instance-level decisions. A negative
finding does not need the pattern-level confirmation, so **no diffs-research ticket is needed for
this pool.** The fog patch is therefore resolved *without* graduating a ticket: diffs are needed
only if ticket 06 (or 07) wants to assert a *positive* Achievement pattern, which the evidence does
not support.

## Summary for ticket 06

- **Attempt (assessable):** all 8 traps. None is fully unassessable from transcript.
- **Write as "not assessable from transcript (reason)":** none — but Achievement's pattern-level
  claim is not assessable without diffs; if ticket 06 finds no Achievement candidate (as ticket 03
  did), it writes Achievement as "None detected" at the instance level and notes the pattern-level
  caveat, with no diffs ticket triggered.
- **No new ticket graduates from this assessability pass.** The Achievement/diffs fog is resolved
  conditionally: diffs needed only for a positive pattern finding, which the pool does not contain.
- **Grounding reminder.** Ticket 06 must use only the exact-passing quotes from
  [ticket 04](./quote-verification.md) (or its corrected verbatim forms), not the as-written forms
  flagged there. Assessability does not change which quotes are usable.
