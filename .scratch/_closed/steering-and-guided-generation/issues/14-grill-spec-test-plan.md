# Grill: spec and integration-test plan for AI steering

Type: grilling
Status: closed
Blocked by: 11
Assignee: pi (wayfinder session 2026-08-23)

## Question

Review the research asset `research/11-specs-and-integration-tests.md` (audit findings + proposed `docs/specs/steering.md` + proposed `tests/http/steering.rs`) and resolve the open questions so the spec and tests can be committed as gating artifacts for the implementation tickets.

## Open questions

1. **Scenario ID range.** The proposed spec uses `22.x`–`25.x`. Confirm this allocation or pick a different range.
2. **Impersonate preset activation.** The tests assume a new `AppSettings::active_impersonate_prompt_preset_id` field. Is this the intended mechanism?
3. **Unknown slash commands.** Should `/unknown hello` fall back to a plain player action, or should unknown slash commands be rejected?
4. **Mutual-exclusivity enforcement.** Should the parser reject a combined command, or is exclusivity only a per-turn replay-blob property?
5. **Narrator message placement.** Is `/narrator` a permanent history note that can later be edited/deleted like other messages?
6. **Impersonate output type.** ~~Should impersonate output be `MessageType::Input` (as proposed) or a distinct type such as `Dialogue`?~~ **Resolved ahead of grilling (2026-08-15): `MessageType::Input`.** Impersonate's purpose is to produce text that reads as player input — to pretend to be the player. Distinguishing it with a separate type or metadata would defeat that purpose; the output must be indistinguishable from a typed `Input` by everything downstream. The replay blob (ticket 06) still holds `impersonate=true` for retry mechanics, but that is not a visible type distinction.
7. **Text check on recognized slash commands.** Surfaced by ticket 07: `action_check_handler` runs the player-input text check on the raw `command` string before dispatch, so a recognized command like `/guide make it tense` is checked as if it were player dialogue (slash prefix included). Should recognized slash commands bypass the text check, since steering text is not player dialogue? Or should the check run on the argument only?

## Resolution

Grilled 2026-08-23. Four of the original seven questions were already settled by the implementation tickets that landed after the research asset was written; only three were genuinely open. The asset's proposed greenfield `docs/specs/steering.md` (scenarios 22.x–25.x) and `tests/http/steering.rs` are **superseded** — not to be committed.

### Already-decided (no question to the user)

- **Q2 — Impersonate preset activation.** Decided by ticket 09: `AppSettings::active_impersonate_prompt_preset_id` was added (storage migration v16) with seed `data/prompt_presets/impersonate/default.json`. Confirmed in code.
- **Q3 — Unknown slash commands.** Decided by ticket 07: the parser falls unknown slash through to `Action::FreeAction` verbatim. Covered by `docs/specs/actions.md` scenarios 1.9–1.11 and `tests/http/actions.rs`.
- **Q4 — Mutual-exclusivity enforcement.** Decided by tickets 07 + 08 + 09: the single-command grammar makes combination impossible (`/guide x /impersonate y` parses as `Guide("x /impersonate y")`), and `PromptContext::with_guide`/`with_impersonate` enforce exclusivity at the prompt layer. No parser rejection needed.
- **Q6 — Impersonate output type.** Resolved ahead of grilling (2026-08-15) and carried by ticket 17: `MessageType::Input`. `Dialogue` variant removed entirely; `sender` deleted.

### Grilling results (three open questions)

**Q1 — Spec/test placement (ratify inline, drop the separate file).** No need for a dedicated `steering.md`/`steering.rs`. The committed inline coverage is the spec:
- `docs/specs/actions.md` 1.9 (`/impersonate`→Input), 1.10 (`/guide`→no Input), 1.11 (`/narrator`→Narrator) — dispatch-level.
- `docs/specs/swipe_new.md` 22.1 (re-impersonate retry), 22.2 (user-regen retry), 22.3 (steering record preserved on retry).
- Prompt-layer behavior (guide = final `<Guide>` layer, Narrator bare-render, impersonate preset + persona injection) is covered at the **unit tier** in `src/application/prompting/assembler_tests.rs` — the assembler is the actual unit of behavior, so unit tests there are the right tier and less brittle than a recorded-narrator HTTP fixture.
- Scenario ID collision noted: the research asset's proposed `22.x`–`25.x` is partly taken — `swipe_new.md` already uses `22.x`. Creating `steering.md` would force a reshuffle and duplicate unit-tier assertions at higher fixture cost. Ratifying inline avoids both.

**Q5 — Narrator message editability.** Narrator rows are editable and deletable like any other history row. The story-log template renders an edit button on every row and a delete button on the last row with no `MessageType` gate; `message_service.edit_history`/`delete_last` operate on any type. A Narrator-only lock would be inconsistent with Narration (also scene truth, already editable). If a lock is wanted, it should be a separate decision covering Narration too — out of scope here.

**Q7 — Text check on recognized slash commands.** Decision: **bypass the text check for recognized slash commands** (`/guide`/`/narrator`/`/impersonate`). The text check exists to catch errors in player-typed prose before it enters history; steering text is an instruction to the model, not player dialogue (guide is not persisted; narrator text is a directive; impersonate output is generated by the model). Checking it clutters the confirm flow with irrelevant flags on imperatives and fragments. Implementation: in `action_check_handler`, detect a recognized `Action::parse` variant and dispatch directly without calling `check_player_input`; unknown slash and plain input keep the check as today. This is a behavior change not yet in code — graduates a new implementation ticket (20).

### Graduated ticket

- **20 — Bypass text check for recognized slash commands** (`wayfinder:task`, AFK). Small change in `src/adapters/driving/http/action/handlers/actions.rs::action_check_handler` plus a test. Not blocked.

### Superseded assets

- `research/11-specs-and-integration-tests.md` proposed `docs/specs/steering.md` and `tests/http/steering.rs` — **superseded by the committed inline specs/tests**; do not create these files.
