# Pass 3 Recheck: Verification of Claims

## C1: Options Parser Accepts Exactly Three Output Shapes
**HOLDS**

Evidence:
- `src/application/agents/options/utils/parser.rs:17-27` defines three regex patterns: `<suggestion>` tags, numbered list lines, and `Suggestion N:` prefix.
- `parser.rs:36-43` shows the parser tries all three strategies in priority order and returns empty Vec if none match.
- `src/application/agents/options/utils/orchestration.rs:11` defines `MAX_OPTIONS_ATTEMPTS: u32 = 2`.
- `orchestration.rs:29-48` shows the retry loop: attempts up to 2 times, returns error on failure.
- `src/application/pipeline/job.rs:358-367` shows when Options generation fails, it creates `OptionsResult::Unavailable` with reason "Options generation failed" and the raw error string.
- `docs/specs/options.md:37,72,148` show the player sees "the dock shows the Options failure" - a generic failure message, not a format-specific error.

Limit: The claim is accurate — the parser accepts exactly three shapes, retries once, and shows a generic failure to the player.

---

## C2: Character Schema Allows Missing ID but Rust Type Requires It
**HOLDS**

Evidence:
- `data/schemas/character.schema.json:29` shows `"required": ["name", "description", "personality", "scenario"]` — no "id" field.
- `src/domain/model/character.rs:54-56` shows `CharacterCard` has `pub id: String` (non-optional, no `#[serde(default)]`).
- `src/bootstrap/load.rs:123-128` shows when `read_json_file::<CharacterCard>` fails, it logs a warning and continues: `tracing::warn!("Failed to parse Character {}: {e}", path.display())`.

Limit: A character file without id passes JSON schema validation but fails at boot with one warning.

---

## C3: Map Schema Accepts Any Exit Direction but Rust Type Is Fixed
**PARTLY**

Evidence:
- `data/schemas/map.schema.json:24` shows `"exits": { "type": "object", "additionalProperties": { "type": "string" } }` — accepts any key.
- `src/domain/model/map.rs:52-61` shows `Direction` enum with fixed variants: North, South, East, West, Up, Down.
- `src/bootstrap/load.rs:59-71` shows `seed_worlds` loops through world directories and calls `process_world_dir` with `?` operator.
- `load.rs:85-89` shows `read_json_file(&map_path)?` — error propagates, stopping subsequent worlds in the loop.
- `src/bootstrap/run.rs:53-55` shows `if let Err(e) = seed_game_data(...)` catches the error with a warning and continues boot.

Does it stop the seeding of Worlds after it? Yes, within the `seed_worlds` loop, one world's map failure stops processing subsequent worlds. But the boot itself continues after catching the error in run.rs.

Guardrail path: `src/bootstrap/load.rs:59-71` (seed_worlds loop with early exit), `run.rs:53-55` (error caught, boot continues).

---

## C4: Room Name Shown Above Story-Log Entry Resolved at Read Time
**HOLDS**

Evidence:
- `src/application/games/view_query.rs:150-165` shows `story_window` method reads current world map: `let world = self.storage.require_world(&game.world_key)?`.
- `view_query.rs:156-159` shows room names resolved from current map: `read.entries.iter().map(|(_, state)| state.room(&world.map).map(|room| room.name.clone()))`.
- `src/domain/model/story_window.rs:1-25` shows `StoryWindowEntry` has `room_header: Option<String>` computed at read time.
- The room name is not stored with the entry; it's resolved from the current world map and dynamic rooms.

Limit: Editing a room name in the World changes the header of old entries.

---

## C5: Status Display Data-Status Values Not Tested for Synchronization
**WRONG**

Evidence:
- `tests/http/dashboard.rs:31,47,72,91,337,453,506` contains multiple tests checking for `data-status="idle"`, `data-status="thinking"`, `data-status="failed"`.
- `tests/browser/stub/story_log_edit.rs:630,651,677` contains tests checking for `data-status="thinking"`, `data-status="idle"`, `data-status="failed"`.
- `src/adapters/driving/http/layout/handlers/endpoints.rs:66,88,95` shows Rust code uses string literals "idle", "thinking", "autoplay".
- `assets/dashboard-action-area.js:43-50` shows JS code queries `[data-status="thinking"]`, `[data-status="autoplay"]`.

Limit: Tests DO exist that would fail if only one side were renamed. The claim is incorrect.

---

## C6: Default Prompt Preset IDs Written in Multiple Places Without Cross-Check
**HOLDS**

Evidence:
- `src/domain/model/utils/settings_defaults.rs:6-10` defines constants: `DEFAULT_SYSTEM_PROMPT_PRESET_ID = "system_default"`, etc.
- `data/prompt_presets/system/default.json`, `quantifier/default.json`, etc. contain `"id": "system_default"`, etc.
- `data/settings.json:3-5,8-9` contains `"system_prompt_preset_id": "system_default"`, etc.
- `src/adapters/driven/storage/utils/plumbing.rs:8-12` contains migration literals `DEFAULT 'system_default'`, etc.
- No test found that compares the seed file id with the Rust constant.

Limit: The IDs are duplicated across Rust, JSON files, and migrations without a test ensuring they match.

---

## C7: Text Check Failure Logged But Not Reported to Player
**HOLDS**

Evidence:
- `src/adapters/driving/http/action/handlers/actions.rs:62-68` shows when text check fails:
  ```rust
  Err(e) => {
      tracing::error!("Text check failed: {e}");
      return dispatch_action(&state, game_id, command);
  }
  ```
- The error is logged but the input is dispatched unchanged; no message is sent to the player.

Limit: The player receives no notification that the text check failed.

---

## C8: Order of Busy Check vs Text Check in /action and /action/check
**PARTLY**

Evidence:
- `src/adapters/driving/http/action/handlers/actions.rs:48-56` shows `action_handler` calls `dispatch_action` which calls `Job::start`, which checks busy.
- `actions.rs:58-91` shows `action_check_handler` does text check first (lines 75-80), then calls `dispatch_action` if needed.
- So `/action` refuses busy first, while `/action/check` runs text check before busy refusal.
- `docs/diataxis/reference/frontend/dashboard.md:63` mentions "the text check sits between a typed send and the request to the engine" but doesn't specify the busy check order.
- No spec explicitly states the intended order.

Limit: The code shows different orderings, but no spec documents which is intended.

---

## C9: Guardrail Limits Not Stated in Documentation
**WRONG**

Evidence:
- `docs/diataxis/reference/coding_standards/guardrails.md:113` states: "file length | Enforces a maximum of 2000 non-blank lines per file."
- `guardrails.md:105` states: "nesting depth | Fails when function-body control-flow nesting exceeds `MAX_NESTING_DEPTH`."
- `guardrails.md:116` states: "long comment runs | Warns when five or more countable comment lines appear consecutively."
- `guardrails.md:110` states: "no std thread all | Exposes the internal no-std-thread check."

Guardrail paths and documentation status:
- 2000 non-blank-line limit: `tests/infrastructure/guardrails/structure.rs:308-321` — stated in guardrails.md
- Nesting-depth limit of 3: `tests/infrastructure/guardrails/nesting.rs:8` — stated in guardrails.md (but not the value 3, just the constant name)
- Cap on consecutive comment lines: `tests/infrastructure/guardrails/style.rs:124-148` — stated in guardrails.md
- Ban on std::thread: `tests/infrastructure/guardrails/structure.rs:180-189` — stated in guardrails.md

Limit: All four limits ARE stated in guardrails.md, though nesting depth doesn't specify the value.

---

## C10: implement-spec SKILL.md References Non-Existent Merger Agent
**HOLDS**

Evidence:
- `.agents/skills/implement-spec/SKILL.md` states: "Once an **implementer subagent** completes, merge its work to the integration branch with a **merger subagent**."
- `.pi/agents/` contains: implementer.md, judge-*.md, researcher.md, reviewer.md, scout.md, strong-implementer.md.
- No merger.md exists.

Limit: The skill references a "merger subagent" that doesn't exist.

---

## C11: thermo-nuclear SKILL.md Contradicts AGENTS.md on Code Changes During Reviews
**HOLDS**

Evidence:
- `.agents/skills/thermo-nuclear-code-quality-review/SKILL.md` states: "Be ambitious, if there is a clear path to improving the implementation that involves restructuring some of the codebase, go for it."
- `AGENTS.md` (project context) states: "However, during code reviews you should not make code changes to fix problems."

Limit: The two documents give contradictory guidance.

---

## C12: Trigger Evaluation Does Not Check Character Presence
**HOLDS**

Evidence:
- `src/domain/model/utils/game_rules.rs:173-210` shows `evaluate_triggers` function:
  - Line 183-190: checks `trigger.room_id` against current room
  - Line 192-199: checks `state.encounters.check_condition` (times_met comparison)
  - Line 201-208: checks `trigger.repeat` and `is_trigger_fired`
  - No check that the Character is in `state.scene.characters_present`
- No spec under `docs/specs/` or `docs/diataxis/` states that a Trigger may fire while its Character is absent.

Limit: A Trigger can fire even if its Character is not present, and no spec documents this behavior.

---

## C13: No Test Uses --no-default-features
**HOLDS**

Evidence:
- Searched all files for `no-default-features`, `no_default_features`, `default.features`, `default_features` — no results found.
- `build.py` has no step that uses `--no-default-features`.
- No test in `tests/` or `src/**/*_tests.rs` builds with this flag.

Limit: The codebase is never tested with `--no-default-features`, so feature-gated code may have undetected issues.
