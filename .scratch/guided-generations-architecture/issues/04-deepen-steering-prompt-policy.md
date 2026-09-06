# 04 — Deepen a prompt-policy module for slash commands (architecture candidate 4)

Type: grilling
Status: resolved
Blocked by: (none)

> Re-framed 2026-08-29 after ticket 06 resolved: the name
> **SteeringPromptPolicy** is avoided (Steering is retired). The
> prompt-policy question stands: impersonate's prompt shape is still split
> across two files and a builder method.
>
> Re-framed again 2026-08-30 after ticket 02 resolved (rejected, no
> module): the guide/impersonate mutual-exclusion question is now wholly
> owned here. Ticket 02's Answer records the enforcement as it stands
> (construction shape plus one defensive line at `core.rs:232`).
>
> Cross-note from ticket 03 (resolved 2026-08-30, committed): the
> impersonate preset pin now sits behind the `process_action(Action)`
> dispatcher's match — entry-time pinning, required by 06's
> swipe-stores-inputs rule. If this ticket commits a prompt-policy module
> that owns preset choice, whether that pin relocates into the policy is
> part of this ticket's decision.

## Question

Do we commit to a deep module that owns "what a slash command does to the
prompt" — returning the preset choice and layer set for a given command —
so impersonate's prompt shape stops being split between `assembler.rs` and
`pipeline_run.rs`, and if so, what is the shape of the deepened module?

## Background

**Friction (from `architecture-review.html`, candidate 4, Worth exploring):**
"What impersonate does to the prompt" is split across three places:

- `src/application/pipeline/pipeline_run.rs` (`phase_narrate`) — selects the
  impersonate preset (swaps the system preset).
- `src/application/prompting/assembler.rs` (`render_persona_layer`) — drops the
  player-character reference-card layer on impersonate.
- `PromptContext::with_impersonate` — clears `guide`, encoding the
  guide/impersonate mutual-exclusion rule inside the prompt-context builder.

Neither `assembler.rs` nor `pipeline_run.rs` is deep on its own for this
concern; they form a tightly coupled seam. A change to impersonate's prompt
shape touches both. `PromptContext` now carries both `guide` and `impersonate`
fields, exposing prompt-construction details to callers.

**Relationship to ticket 01 (narration-generation module).** The report
says this policy "layers on top of a stable narration-turn seam." This ticket
can be grilled independently — the policy's output (preset + layers) is
self-contained — but the grilling should confirm where the policy's output is
consumed (the deepened module, if 01 is accepted).

**Relationship to ticket 02 (stored-inputs flow).** The guide/impersonate
mutual-exclusion rule could live here or in ticket 02's module, if that
module survives its re-justification. Decide which owns it.

## What to decide

- Commit or reject the prompt-policy deepening.
- If committed: the module's **interface** (the report sketches
  `resolve(steering) → PresetChoice + LayerSet` — re-skin as
  `resolve(action)` against the settled model), its **seam** (between the
  pipeline and the assembler), what **implementation** moves behind it
  (preset swap, layer drop, mutual-exclusion), and what **tests** cover
  "what does slash command X do to the prompt" at one interface.
- Whether `PromptContext` drops its `guide` / `impersonate` fields (the
  policy feeds it pre-resolved layer/preset data instead).
- Where the guide/impersonate mutual-exclusion rule lives — here or in
  ticket 02's module, if it survives.

## Answer

Resolved 2026-08-30 by grilling (rounds 1–2: Q1–Q3). Candidate 4 is
**rejected**. No ADR (Q3→A) — consistent with ticket 02; the reasoning
lives here and in the map's index, where a future review of this branch
would look.

**Evidence re-verified** against the tree at `67c7824` (the review
artifact predates the narrator-modes work): all three splits still
exist — preset swap (`pipeline_run.rs:118-130`), layer drop
(`assembler.rs:297`), exclusion (`assembler.rs:152-157` plus
`core.rs:230-234`). New fact from the narrator-modes work: preset id
resolution is now mode-aware behind storage (`games.rs:158-180`, game
slot → per-mode bundle), and `allowed_modes` is enforced only at HTTP
preset-selection time (`prompt_presets.rs:265`), never at generation
time.

**Rejection reason.** Tickets 01, 03, and 06 absorbed the friction the
review saw at `1898a53`. Under 01's committed `narration_generation`
module the duplicate impersonate path in `retry.rs` dissolves, leaving
exactly one impersonate/system preset branch in the generate-and-save
prefix. The layer drop is two lines inside the assembler — the rightful
owner of which layers render for a given input kind. The deletion test:
remove the hypothetical policy module and one branch reappears in one
caller — a shallow module. `LayerSet` is a hypothetical seam: one
adapter (Impersonate drops the `<PlayerCharacter>` layer), and the
narrator-modes work adds mode-aware preset choice but no per-mode layer
variation. Moving layer knowledge out of the assembler would make the
assembler shallower. The one real residue — the near-duplicate
`load_preset_and_response_length` /
`load_impersonate_preset_and_response_length` — dies as plumbing inside
01's execution, without a new seam.

**Settled decisions:**

1. **Reject** (Q1→A). No prompt-policy module. The preset choice is a
   branch in `narration_generation`'s prefix — ticket 01's "consume
   the preset choice" step resolves to: the prefix reads the
   caller-resolved inputs and branches impersonate/system itself.
2. **Exclusion keeps ticket 02's enforcement** (Q2→A). Construction
   shape (the two entries set only their own fields; `Action::parse`
   makes them distinct commands) plus one defensive clear, which under
   01's execution moves to the caller-side input preparation — where
   the dispatcher and redo path build `GenerationInputs` from the
   Swipe's stored inputs. The enum-reshape of `GenerationInputs` was
   considered and declined: the Swipe stores flat fields, so the enum
   would trade one defensive line for a fallible reconstruction path.
3. **`PromptContext` keeps its `guide` and `impersonate` fields.**
   Callers state facts about the input; the assembler owns what those
   facts do to the prompt. The fields are input facts, not leaked
   construction detail.
4. **The entry-time impersonate preset pin stays behind the
   `process_action(Action)` dispatcher.** Ticket 03's committed shape
   stands; its relocation question is answered by the rejection.
5. **No ADR** (Q3→A). Caveat recorded: a re-review run before 01's
   execution lands would still see the split and could re-suggest this
   candidate; the rejection is contingent on 01's committed shape.

**Execution note for ticket 01.** Merge the two preset loaders into one
private helper when the prefix consolidates: the impersonate/system
branch produces the preset id (pinned id or active fallback), and one
loader fetches preset + response length. Plumbing, not a seam.

**Behavior observation (inferred gap, not a decision).** A preset's
`allowed_modes` is checked when a user selects the preset over HTTP,
not when a generation uses it. A pinned impersonate preset whose modes
later change still generates. This belongs to the narrator-modes effort
or to execution, not to this map.

**Consequences for the map:** ticket 03's pin question is answered
(stays). No fog graduates — both "Not yet specified" items wait on the
full accept/reject split, which stands at 1 accept (01), 2 reject
(02, 04), 05 open. Ticket 05 is now the whole frontier.
