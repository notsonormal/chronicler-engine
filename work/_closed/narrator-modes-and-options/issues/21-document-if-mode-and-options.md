# Task: Document IF mode and options

Type: task
Status: resolved
Blocked by: 20

## Question

Last fog toward the destination: the documentation for the two shipped features. What does the doc tree need for Interactive Fiction narrator mode and options-autogeneration, and where does each piece live?

Seed facts from the map:

- **CONTEXT.md terms** — add/refresh the ubiquitous-language entries: Narrator Mode (Novel / InteractiveFiction, per-game posture inherited from the world, ticket 05/06/07), the per-mode preset bundle + `allowed_modes` selection gating (tickets 13/14), and Options (the offered set, `MessageType::Input` semantics, always-on toggle vs on-demand `/options`, tickets 04/09/10/11).
- **Diataxis reference** — a reference doc (or docs) for IF mode + options: what the narrator may do under the IF posture (the inverted Agency Rule, ticket 02), how the options dock + slash item behave, and how both interact with steering (guide/impersonate stay mode-agnostic, ticket 13's ruling). Cross-reference, don't restate, `docs/diataxis/reference/narrative/ai_steering.md` and the specs (`docs/specs/narrator_mode.md`, `docs/specs/options.md`, `docs/specs/browser_options.md`).
- **DOC anchors** — the doc-anchor standard applied to the new code surfaces (options agent, posture relocation, preset mode flags) per the structure guardrails.
- Known follow-the-docs items recorded by earlier tickets: the HTTP-tier `/options` empty-history rejection surfaces as a 500 (ticket 12, pinned as-is) — decide whether the reference doc mentions it or a fix ticket is warranted.

Scope the doc set first (grill if the split isn't obvious), write it in STE, keep the compass test honest (reference defers to source; no code-indexing), and land with `python build.py validate-docs` + full gate green.

## Answer

Resolved 2026-09-12 (this session). The split was obvious — no grill needed: two focused reference docs under `docs/diataxis/reference/narrative/`, one per shipped feature, matching the tree's existing granularity (`narration_system.md`, `agent_system.md`, and `ai_steering.md` are peers).

**Reference docs.**

- `narrator_mode.md` — the two modes and the Agency Rule inversion (novel: the narrator never voices the PC; IF: the narrator elaborates commanded actions, with the one hard line both modes share — no uncommanded actions); posture as the per-game triple inherited from the World, with the deliberate-perspective-survives rule; preset selection split into its two mechanisms (the per-mode registry answers "what is the default per mode", `allowed_modes` answers "what is selectable", activation is mode-targeted, the narrate path never re-validates stored ids); steering neutrality.
- `options.md` — options as pre-written player inputs whose use is a normal action (an Input message; the offered set never enters history); the options agent's dedicated phase and its two gated dispatch sites (always-on turn end, on-demand `/options`); the offered-set lifecycle (replace on success, keep on failure plus a System message, clear-and-regenerate under always-on, impersonate turns untouched); the dock's Use/Edit contract and self-polling; the empty-history 500 recorded as specified, pinned behaviour; the `options` connection id with narrator fallback; mode neutrality.

Both defer to source and specs per the compass rule: no module or method inventories — the only identifiers are domain-level (the seeded preset ids, the `options` connection id, the `allowed_modes` field).

**CONTEXT.md** — six new Language entries: Narrator Mode, Posture, Allowed Modes, Mode Preset Registry, Options, Offered Set.

**DOC anchors.** The options agent's six production modules retargeted from `agent_system.md` to `options.md`; `prompt_preset.rs` (the allowed-modes flags) retargeted from `game_flow.md` to `narrator_mode.md`. The posture-relocation domain surfaces (`settings.rs`, `game.rs`, `world.rs`) keep their existing anchors — those files are broader than narrator mode, and their current targets (`architecture_system.md`, `game_flow.md`) remain the better homes. Both new docs are now anchor targets, satisfying the structure guardrail.

**The `/options` empty-history 500.** Documented in `options.md` as the specified behaviour (spec scenario 24.2 pins the 500 and the named validation failure). No fix ticket: the rejection is an intentional validation surface, not an accident; changing it would be a UX decision, not a defect repair.

**Also landed (working-tree hygiene from this session).** Stale ticket cross-references stripped from `assets/index.html` and `assets/styles.css` comments; two guardrail violations fixed (`tests/http/prompt_presets.rs` import ordering; `tests/test_utils/html.rs` multi-line `//!` header condensed to one line); `docs/diataxis/reference/coding_standards/guardrails.md` regenerated for drifted line numbers; the docs index regenerated.

**Verification.** vale: 0 errors / 0 warnings across the two new docs + CONTEXT.md. `python build.py validate-docs`: PASS, 0 errors. Full gate green: formatting, data validation, clippy, structure/spec-coverage/docstring guardrails, python tests, http-routes + guardrails-doc freshness, docs validation, architecture tests (1 passed), guardrail tests (142 passed), integration tests (1610 passed, 2 LLM skipped), browser tests (20 passed).
