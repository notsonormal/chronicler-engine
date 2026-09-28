# 05 — Single injection point for narrative-voice setting (architecture candidate 5)

Type: grilling
Status: resolved
Blocked by: (none)

## Question

Do we commit to routing narrative-voice application through a single owner —
the prompt-construction module — so `set_narrative_voice` stops being called in
both `assembler.assemble` and `arrival_service`, and if so, what is the shape
of the deepened seam?

## Background

**Friction (from `architecture-review.html`, candidate 5, Worth exploring):**
`TemplateVars::set_narrative_voice` (in
`src/domain/model/template.rs`, +46 lines on the branch) is called in two
places:

- `src/application/prompting/assembler.rs` — `assemble` applies it from
  `AppSettings`.
- `src/application/arrival_service.rs` — `ArrivalTaskContext::run` applies it
  separately.

The setting is therefore not encapsulated by the assembler; any other caller
that builds a `PromptContext` must remember to apply it.
`TemplateVars::from_persona` defaults to third-person-past, so forgetting the
call silently produces wrong voice. A leaking seam.

**This candidate is independent and small** — the report calls it "a safe
parallel win." It has no relationship to tickets 01–04 and can be grilled
first or last without affecting them.

## What to decide

- Commit or reject the single-injection-point deepening.
- If committed: which module owns voice application (the assembler, or a
  shared `TemplateVars` builder both callers route through), the **interface**
  change (does `PromptContext` construction apply voice, or does the
  assembler's `assemble` remain the sole caller?), and what **tests** guard
  against the silent-wrong-voice failure mode.
- Whether `arrival_service` routes through the assembler for its prompt, or
  whether a thinner shared builder is warranted (avoiding the assembler's full
  fit/budget path for the arrival case).

## Answer

Resolved 2026-08-30 by grilling (rounds 1–2: Q1–Q3; the original
voice-source question was withdrawn — the narrator-modes map owns it). The
deepening is **committed**, scoped to what survives narrator-modes ticket 06.

Evidence re-verified against the tree at `67c7824` before grilling: the two
call sites hold (`assembler.rs:81`, `arrival_service.rs:152-156`), but the
friction had inverted since the review — `assemble` unconditionally overwrites
`template_vars` voice with world posture, so arrival's game-posture stamping
is dead code, and the doc comment on `TemplateVars::set_narrative_voice`
still named `AppSettings` as the source of truth after posture relocated to
World/Game.

**Voice source of truth (not decided here):** the narrator-modes map owns
this — its ticket 01 settled posture as per-game inherited from world; its
ticket 05 landed the data layer; its ticket 06 (pending) threads game posture
into the narration path, and its scope items 1–2 are these exact two call
sites. This ticket's shape is designed to survive that ticket, not compete
with it.

**Settled decisions:**

1. **Commit, scoped to the ownership contract** (Q1→A). The
   prompt-construction module is the sole owner of voice application;
   callers supply resolved posture and never stamp. The rejected alternative
   — close as subsumed by narrator-modes 06 — lost because 06 fixes *where
   posture comes from* but not *who applies it*; the contract and the
   cleanup survive 06 untouched, and only the stamp's source (world → game)
   churns, inside `assemble`.
2. **Shape: delete the method; the stamper becomes module-private** (Q2→A).
   `TemplateVars::set_narrative_voice` is deleted (with its stale doc
   comment). A module-private helper in `assembler.rs` (e.g.
   `apply_posture(&mut TemplateVars, perspective, tense)`) does the two
   field assignments; `assemble` is its only caller; arrival's posture
   resolution and stamping (`arrival_service.rs:152-156`) are deleted.
   Compile-time enforcement via `pub(in crate::application::prompting)` is
   impossible — Rust restricts `pub(in path)` to ancestor modules and
   `template.rs` lives in `crate::domain::model` — so enforcement is
   structural instead: no named stamp operation exists outside the owner
   module. The domain type sheds application policy.
3. **Tests** (Q3→accept). Extend `assembler_tests.rs:759`
   (`test_assemble_injects_narrative_voice_from_world_posture`) so the
   incoming `template_vars` deliberately carry conflicting voice and the
   assertion is that the owner's stamp wins — proving callers cannot inject
   voice, the review's failure mode in its current inverted form. Arrival's
   existing flow tests must pass unmodified (the deletion is
   behavior-neutral); execution adds one assertion only if a voice-coverage
   gap exists. No guardrails-style "nobody else stamps" test — the deleted
   method plus the module-private helper make that compile-time.

**Execution notes (pre-merge effort):**

- Delete `arrival_service.rs:152-156` (posture resolution + stamping);
  behavior-neutral today.
- Delete `TemplateVars::set_narrative_voice` (`template.rs:50-57`)
  including the stale `AppSettings` comment.
- Add the private `apply_posture` helper in `assembler.rs`; `assemble`
  calls it with world posture (source unchanged until narrator-modes 06).
- Extend the ownership test as above; verify the arrival suite passes
  unmodified.
- Adjacent paths are excluded by fact: voice macros appear only in prompt
  presets, so the scenario-log and trigger-narration `TemplateVars`
  producers (`message_service.rs:127`, `game_state.rs:174`,
  `game_state.rs:350`) never encounter them.

**Cross-map note:** appended to narrator-modes ticket 06 — thread resolved
posture (game, with world fallback in arrival) to the owner; arrival never
stamps. The `Game` posture fields, write-only today except arrival's dead
read, are vindicated by that ticket — no removal. The "posture" CONTEXT.md
term is owned by the narrator-modes documentation ticket; not duplicated
here.

**Consequences for the map:** all five candidates plus the conceptual model
are now decided (committed: 01, 03, 05; rejected: 02, 04). Both
Not-yet-specified patches graduate: the accept/reject split and the accepted
set are known, so the handoff questions are specifiable → new ticket 07
(pre-merge handoff: landing shape and merge strategy). The candidate work of
the destination is complete; the frontier is 07.
