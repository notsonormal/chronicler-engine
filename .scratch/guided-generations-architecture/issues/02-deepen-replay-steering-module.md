# 02 — Does the stored-generation-inputs flow need an owning module? (architecture candidate 1)

Type: grilling
Status: resolved
Blocked by: (none)

> Re-framed 2026-08-29 after ticket 06 resolved: the name **ReplaySteering**
> is avoided, and the candidate's premise was cut — the replay blob is not
> a concept. `GenerationReplay` is data stored on the Swipe; scattered field
> access to plain data is normal data flow, not lost locality. This ticket
> must re-justify the module on other grounds or be rejected.
>
> Re-framed again 2026-08-30 after ticket 01 resolved: 01 committed a
> narration-generation core whose **callers** resolve inputs (reading the
> Swipe's stored inputs on redo) and confirmed there is no redo-policy
> seam. The stored-inputs flow is now settled as plain data flow: Swipe
> stores inputs → caller reads them → core consumes them. The case for an
> owning module is weaker still; re-justify on other grounds or reject.

## Question

Does any module shape still earn its place for the stored-inputs data flow
(stage → attach-to-swipe → read-back-for-generation → read-back-for-new-swipe)
— for example the `GameState::push_message` attachment behaviour, or the
guide/impersonate mutual-exclusion rule — or is candidate 1 rejected?

## Background

**Friction (from `architecture-review.html`, candidate 1, Strong):**
The stored-inputs lifecycle is enforced by no single module. Six files
jointly maintain it:

- `src/domain/model/message.rs` — blob definition (`GenerationReplay`).
- `src/domain/model/state/narrative_state.rs` — `pending_replay` transient
  field.
- `src/domain/model/state/game_state.rs` — `push_message` copies
  `pending_replay` onto the active swipe when a message is created.
- `src/application/pipeline/action_pipeline/action.rs` — builds the blob and
  stages it as `pending_replay`.
- `src/application/pipeline/pipeline_run.rs` — `resolve_guide` /
  `resolve_impersonate` read the blob back from both fresh `PipelineInputs`
  *and* `retry_target.replay()`.
- `src/application/pipeline/action_pipeline/retry.rs` — reconstructs the
  new-swipe mode from `old_target.replay()`.

Understanding the lifecycle requires bouncing between all six. Under the
settled model this is ordinary data flow — the grilling decides whether any
of it rises to behaviour worth an owner.

**Relationship to ticket 01 (narration-generation module).** The report
noted the narration-generation module gives this flow "a single consumer to
call against." The grilling should account for where the stored inputs are
consumed (the deepened module, if 01 is accepted; the current scattered
pipeline reads, if not).

## What to decide

- Re-justify or reject the deepening under the settled model.
- If a module survives: its **interface** (the report sketched
  `stage() · attach_to_swipe() · resolve_for_turn() · resolve_for_retry()` —
  re-skin against the settled model), its **seam** (domain `GameState`,
  application pipeline, or between?), what stays as the value type in
  `message.rs` vs what moves behind the interface, and what **tests** cover
  it at the new seam.
- Whether `GameState::push_message`'s attachment behaviour (copying
  `pending_replay` onto the active swipe) moves behind this module or stays
  on `GameState`.
- Whether the guide/impersonate mutual-exclusion rule lives here or in
  ticket 04's prompt policy — moot if this ticket is rejected (defaults to
  04's grilling).

## Answer

Resolved 2026-08-30 by grilling (rounds 1–2: Q1–Q2). Candidate 1 is
**rejected**. No ADR — offered per the map's Notes, declined; the
reasoning lives here and in the map's index, where a future review of
this branch would look.

**Rejection reason.** Evidence was re-verified against the tree at
`67c7824` before grilling. After ticket 06 (the stored inputs are
unnamed data on the Swipe — CONTEXT.md "Replay blob: don't use") and
ticket 01 (caller-resolved inputs, no redo seam), the stored-inputs
flow is plain data flow: entry builds inputs → caller resolves → core
consumes → Swipe stores. Six touch points, all thin: the data struct
(`message.rs:9`), two struct-literal construction sites
(`action.rs:29`, `action.rs:140`), one staging assignment
(`action.rs:78`), one attachment inside `push_message`
(`game_state.rs:145`) plus redo-swipe inheritance (`game_state.rs:129`),
two read-backs (`pipeline_run.rs:436`, `retry.rs:240`), one
classification read (`retry.rs:133`). The deletion test: remove the
hypothetical owner module and no complexity reappears across callers —
the module's interface would match its implementation in size, a
shallow module.

**Settled decisions:**

1. **Reject** (Q1→A). No module shape earns its place for the
   stored-inputs data flow.
2. **Attachment stays on `GameState`.** The `push_message` attachment
   and redo-swipe inheritance are inseparable from that method's
   swipe-vs-message decision; extracting only the stored-inputs part
   would split one cohesive method to save three lines.
3. **Mutual exclusion passes to ticket 04.** The guide/impersonate
   rule is enforced today by construction shape — the two entry sites
   each set only their own fields, and `Action::parse` makes them
   distinct commands — plus one defensive line (`core.rs:232`). The
   question of where the rule lives is now wholly ticket 04's, per
   this ticket's terms. The enum-reshape idea (make the violation
   unrepresentable by type) was considered and dropped (Q2→B);
   convention plus the defensive line is enough, and 04's grilling can
   surface the idea on its own if it matters.

**Execution observation (inferred, not a decision).** With no owner
module, removal of the `pending_replay` staging buffer belongs to
ticket 01's execution: the core holds `GenerationInputs` at
message-add time and can write the stored inputs onto the Swipe
directly, instead of transporting them through a transient state
field.

**Consequences for the map:** ticket 04 owns the exclusion question.
No fog graduates — both "Not yet specified" items wait on the full
accept/reject split, which stands at 1 accept (01), 1 reject (02),
03/04/05 open.
