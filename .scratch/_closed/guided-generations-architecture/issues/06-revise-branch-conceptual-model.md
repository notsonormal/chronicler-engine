# 06 — Revise the branch's conceptual model before deepening the steering and regeneration candidates

Type: grilling
Status: resolved
Blocked by: (none)

## Question

Is the branch-introduced conceptual model the right frame for the steering
and regeneration features, or must it be revised before candidates 1, 2, and
4 are deepened?

## Background

The `guided-generations` branch introduced the terms **Steering**, **replay
blob** (`GenerationReplay`), **Impersonate**, **Guided Generation**, and
**Narrator Action** into `CONTEXT.md`. At the merge-base with main
(`ba19fdb`), none of these terms appear. They were added in two branch
commits (`915b748`, `5523248`).

Three doubts surfaced while grilling candidate 2 (ticket 01), and they
cross-cut candidates 1, 2, and 4:

**Doubt A — "retry" is the wrong umbrella.** The three `RetryMode` variants
(`ReNarrate`, `ReImpersonate`, `UserRegen`) are grouped in `retry.rs` as one
operation. They are not one kind of thing:

- `ReNarrate` regenerates the AI Narration of the last input.
- `ReImpersonate` regenerates an impersonated Input line. Its flow is much
  closer to the impersonate generation flow than to the narration retry
  flow.
- `UserRegen` regenerates the player's plain Input as an alternate Swipe.

`CONTEXT.md` defines **Swipe** as "an alternate version of an AI-generated
Message." All three produce an alternate Swipe. The unifying concept is
Swipe, not retry. Grouping them in one module causes the duplication that
candidate 2 set out to fix.

**Doubt B — is "Steering" wanted as the umbrella?** The branch unifies
Guided Generation, Narrator Action, and Impersonate under one concept,
Steering. The user is not certain this unification is wanted, or whether
the three surfaces are better treated as separate things.

**Doubt C — is "replay blob" wanted as a concept?** The branch carries
transient steering on a replay blob (`GenerationReplay`) attached to the
Swipe, re-applied on retry. The user is not certain this carrier is wanted,
or whether it adds terminology without earning its place.

**Doubt D — new terminology from the architecture review.** The review
proposes further terms (`NarrationTurn`, `ReplaySteering`,
`SteeringPromptPolicy`). If the branch's own terms are in question, these
additions should not layer on top until the model settles.

## What to decide

- Keep, revise, or retire **Steering** as the umbrella for Guided
  Generation, Narrator Action, and Impersonate.
- Keep, revise, or retire **replay blob** (`GenerationReplay`) as the
  carrier for transient steering.
- Re-organize regeneration: is "retry" one operation or several? Where does
  impersonate retry belong — with impersonate generation, or with retry?
  Where does UserRegen belong?
- Whether the architecture review's proposed terms (`NarrationTurn`,
  `ReplaySteering`, `SteeringPromptPolicy`) should be avoided until the
  model settles, or are safe to adopt.

## Research asset

`research/marinara-terminology-cross-reference.md` — cross-references the
branch's terms against Marinara Engine (the SillyTavern-lineage project the
branch modeled on). Key findings that feed the grilling:

- **"Retry" is not the umbrella in Marinara.** Marinara uses "Regenerate"
  for all swipe-creation; "retry" is a narrow empty-send behavior. The
  branch's `RetryMode` grouping is a mis-inheritance.
- **"Steering" is not a Marinara concept.** Marinara presents Guided
  Generation and Impersonate as separate features; "steer" is a verb only.
  The branch's Steering umbrella is invented.
- **"Replay blob" has prior art and better names.** Marinara code calls it
  `generationReplay`; the UI calls it "Stored guidance." The branch ported
  the concept but named it worse.
- **Impersonate regeneration uses one path in Marinara** (the replay blob
  carries `impersonate: true`), confirming impersonate retry belongs with
  impersonate generation, not grouped with narration retry.
- **`/narrator` changed meaning in Marinara** — older versions had it as a
  permanent command; current versions alias it to `/guided` (transient). The
  branch's permanent Narrator Action corresponds to older Marinara.

## Scope

This ticket grills the conceptual model only. It does not decide module
shapes — those return to candidates 1, 2, and 4 once this resolves.
Candidate 3 (steering entry dispatcher) and candidate 5 (narrative-voice
injection) may also relate; the grilling clarifies whether they unblock
independently.

## Answer

Resolved 2026-08-29 by grilling (Q1–Q8). The branch's conceptual model is
revised: its invented umbrella terms are retired, Narrator Action is retired
entirely, and the architecture review's proposed terms are avoided.

**Settled decisions:**

1. **Steering — retired** (Q1→C, Q4→A). The umbrella did not group its
   members so much as set them apart from "normal" narration. The engine
   treats free actions, Guided Generation, and Impersonate as the same
   thing — assemble a prompt, generate text, save the output. Nothing
   replaces it; each feature stands alone with its own properties. Marinara
   prior art: "steer" is a verb only there, never a concept name.
2. **Replay blob — not a concept** (Q5→C). `GenerationReplay` is data stored
   on the Swipe, transferred through the system like any other payload; the
   data sent for a first generation and for a new swipe differ only in
   values. The name "replay blob" drops from prose and CONTEXT.md; the Swipe
   definition carries the property "a Swipe stores the inputs that produced
   its generation." The data flow is unchanged — this retires the
   terminology, not the mechanism. Marinara prior art for the mechanism:
   `generationReplay` (code), "Stored guidance" (UI).
3. **Retry — not a concept** (Q6→C). The umbrella's work is internal
   plumbing (classify the last message, share anchor/snapshot/truncation
   code, dispatch) plus a vocabulary word the UI already owns: the operation
   is a **new swipe** (`/swipe/new` → `retry_handler`). `RetryMode` stays
   `pub(crate)` with no glossary entry. What regeneration runs falls out of
   what the last message *is* (Narration / impersonated Input / plain
   Input). Where impersonate-redo's *code* lives — folded into the
   impersonate flow, per Marinara's single-path prior art — is ticket 01's
   module-shape question.
4. **Review's terms — all avoided** (Q7→A). `NarrationTurn` collides with
   the deprecated term Turn; `ReplaySteering` and `SteeringPromptPolicy` are
   named on retired concepts. The module-shape questions return to tickets
   01, 02, 04 under re-framed names.
5. **Narrator Action — retired entirely** (Q8→A). Marinara demoted the
   permanent player command (`/narrator` → alias of `/guided`) and kept the
   narrator role only for engine-generated content (scene summaries #3641,
   session recaps, merged narrator replies #5282/#5336, TTS voice routing,
   narrator bubbles). Chronicler has no engine-generated Narrator messages —
   `action.rs:120` is the only producer of `MessageType::Narrator` — so
   Marinara's kept case does not exist here and its demoted case is what the
   branch built. Removal scope (execution effort, not this map): the
   `/narrator` slash command, `Action::Narrator`,
   `narrator_action`/`process_action_with_narrator`, the handler,
   `MessageType::Narrator` and its render branches (`assembler.rs:354`,
   quantifier prompt, view models), and the two HTTP tests
   (`tests/http/actions.rs:748,773`). If engine-generated recaps arrive
   later, Marinara's usage is the prior art for re-introducing a narrator
   message type.

**Resulting model:** the engine generates text. Player inputs are free
actions, Guided Generation, or Impersonate — all slash-command inputs are
transient. Every Message has Swipes; a Swipe stores the inputs that produced
it; making a new swipe redoes the last generation, re-applying the stored
inputs.

**Consequences for the map:** tickets 01, 02, 04 unblocked and re-framed;
03 re-framed (loses the narrator entry); 05 untouched. CONTEXT.md updated:
Steering and Narrator Action entries removed, Guided Generation rewritten,
Swipe gained the stored-inputs clause, retired terms moved to Deprecated.
