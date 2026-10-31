# Add the connection slots and remove the global response length

Type: task (AFK)
Status: open
Blocked by: 18

## Question

Replace the two role-connection settings and the magic `options` connection with four connection slots: Narration (also Event Narration and the Retry), Impersonate, Quantifier and Options. An empty Impersonate or Options slot falls back to the Narration Connection. Remove the dead `BackendSelector`. Remove the global `response_length`, and carry its words in the system presets. Source: [Decide the parameters of each call](07-call-parameters.md).

A **connection slot** is the setting that names the Connection of one kind of call. A slot is not a Role: Impersonate has a slot and no Role. The owner chose to add the term to `CONTEXT.md` (O6 in [Settle the open questions of the engine tickets](17-settle-engine-questions.md)).

Done when the four slots choose the Connections, `response_length`, `BackendSelector` and the `options` connection id do not exist, the `<output_format>` layer adds nothing to the preset text, and `python build.py` passes.

## Decisions

From [Decide the parameters of each call](07-call-parameters.md): the four slots and their fallback. The `options` id and `agents[].backend` go. `response_length` goes, and the length words are preset text (ME's `{{length}}`). This also fixes the defect that the global length reaches Impersonate.

Choices made when the work was ticketed ([Write the build plan for the engine changes](09-build-plan.md)), corrected by the plan review of 2026-10-31:

- **One lookup.** `AppSettings` gets `impersonate_connection_id` and `options_connection_id`, both `Option<String>`, with `#[serde(default, skip_serializing_if = "Option::is_none")]` as the other optional settings fields have.
  - `AppSettings::connection_for(PresetType) -> Result<LlmConnection>` gives the Connection of each call kind: System → narration; Impersonate → impersonate, else narration; Quantifier → quantifier; Options → options, else narration.
  - `narration_connection()` and `quantifier_connection()` stay as one-line calls of `connection_for`, so there is one lookup and one error path, and their 14 call sites do not change.
  - A future Agent kind adds one arm and one field ([Decide whether this map designs memory and state, or only keeps room for them](16-memory-and-state-scope.md), room 3).
- **A dangling slot fails early.** The two optional slots are checked like the two required ones: at boot (`build_app_graph`), at the settings seed, and in `scripts/validate_data.py`. A typo in `data/settings.json` then shows at once, not at the first Impersonate.
- **Delete a Connection.** An optional slot that names the deleted Connection is cleared, so it falls back to narration. The existing guard of the two required slots stays. Both happen in the one `update_settings` call, which writes nothing when the guard refuses.
- **Trade-off.** An Impersonate on its own Connection still reports its health under the Narrator Role, because `Role` has no Impersonate. `/debug/backend` also shows only the narration recorder. A Role for Impersonate belongs to the map's fog item "A UI for the new changeable fields".
- **Routes** (owner, 2026-10-31, O3 in [Settle the open questions of the engine tickets](17-settle-engine-questions.md)). No route sets the two new slots. The trial script reads them from the probe DB, and refuses a setup that asks for another value.
- **The response length migration.**
  - Why a migration: a default preset cannot be edited (`PromptPresetService::ensure_editable`), so without it an existing database could never get the words into its default system presets.
  - It reads the stored `response_length`. When the settings row is absent (a fresh database: migrations run in `DbPool::new`, before `seed_settings`) or the text is empty, it skips the append.
  - Otherwise it appends `"\n\nResponse Length:\n" + text`, with no trailing newline, to the `output_format` of every stored system preset that has one. Today's render appends the same string before `wrap_xml`, so the prompt stays byte for byte the same.
  - Then it drops the column. The `UPDATE` comes before the `DROP COLUMN`, in the one transaction of the step.
  - The two system preset files get the same text, so a fresh database ends in the same state.
  - Effects: an Impersonate prompt loses the line. A stored empty text no longer renders an empty `Response Length:` header. The default text has no placeholder, so the render of placeholders in the moved text changes nothing today.
- **The response length test** is one clause that cannot pass by its own fixture: the `<output_format>` layer holds exactly the preset's `output_format` text, for a Narration and for an Impersonate.
- **The `BackendSelector` removal** stays in this ticket. It is mechanical and independent. If the session runs short, it is the part to move to a follow-up ticket.

## Facts

Known on 2026-10-31.

Settings and wiring:

- `AppSettings` (`src/domain/model/settings.rs`) has `narration_connection_id` and `quantifier_connection_id`, as scalar columns of the `settings` table. The Connections are one JSON column. The columns are listed in `DbSettings` (`storage/models/settings.rs`: `from_row` by position, and `to_settings`), in the SELECT and the INSERT of `storage/settings.rs`, and in `AppSettings::default()`. A count mismatch between the columns and `params![]` fails at run time, not at compile time.
- `AppSettings` has 65 struct literals. Only one lists every field. The rest use `..Default::default()`.
- `recorder_with_storage` (`bootstrap/wiring.rs`) takes `pick: fn(&AppSettings) -> Result<LlmConnection>`. A closure that captures nothing, such as `|s| s.connection_for(PresetType::Impersonate)`, fits it.
- The Narrator has one recorder, built on `AppSettings::narration_connection`. Impersonate uses it. The Options recorder takes the Connection whose id is `options`, else the narration Connection (`options_recorder`, with a doc comment that says so).
- `Narrator::new` has 4 call sites: `wiring.rs`, `src/test_support/context.rs` (in `make_test_job_deps`, which has 4 callers), and `narrator_tests.rs` (2).
- `Job::budget` (`src/application/pipeline/job.rs`) reads the narration Connection only. `Generation::preset_type()` exists.
- The two required slots fail fast at boot (`build_app_graph`) and at the seed (`bootstrap/load.rs`). `validate_data.py` checks the two required ids only.
- `AgentConfig.backend` (`BackendSelector`) is dead: both `from_config` functions ignore their config argument. `BackendSelector` appears in six files: `domain/model/agent.rs` (with `AgentConfig::defaults()`), the domain `agent_tests.rs`, `runner_tests.rs`, both agent seam files, and the storage `settings_tests.rs`. There are 13 `AgentConfig` literals in them. A stored `agents` column with a `backend` key still parses after the removal, because `AgentConfig` has no `deny_unknown_fields`.
- `settings.schema.json` sets `additionalProperties: false` at the top level only. The object of one `agents` entry has none, so a leftover `backend` key would pass the data check.
- `delete_connection_handler` (`settings/handlers/settings.rs`) runs inside `SettingsService::update_settings`: it refuses an unknown id and the last Connection, then refuses a Connection that `roles_using` finds, then removes it. Its refusal test is scenario 20.15 in `tests/http/settings.rs`. No route renders the two new slots. A tier-1 test reads settings through `state.settings_service.get_settings()`, as a test in `tests/http/settings.rs` already does. The next free scenario number in `docs/specs/settings.md` is 20.53.

The response length:

- `response_length` has no form field and no route. It is a `settings` column, seeded from `data/settings.json`, and declared in `data/schemas/settings.schema.json`. `default_response_length()` (`domain/model/utils/settings_defaults.rs`) returns the same text as `data/settings.json`.
- `PromptPreset::render_field_parts` appends it inside `<output_format>`, only when the preset has an output format. The Narrator always passes it, so it also reaches Impersonate.
- The parameter runs through six functions: `Narrator::narrate`, `Narrator::assemble`, `PromptAssembler::assemble`, `build_post_history_prompt`, `PromptPreset::assemble_text` and `render_field_parts`. Other call sites lose an argument: `PromptPresetService::hook_text` (calls `assemble_text`), `build_system_prompt` (`prompting/builders/sections.rs`), and `narrate_with` in `narrator_tests.rs` (passes `"Short"`).
- The settings row is seeded only when it is absent (`INSERT OR IGNORE`). So a new settings field reaches an existing database only through a migration or a route.
- Tests that change: the storage `settings_tests.rs` (33 places, where `response_length` is a marker value), `prompt_preset_tests.rs` (2), `load_tests.rs` (5), `settings/templates/settings_tests.rs` (1), the domain `settings_tests.rs` (1). The Narrator seam cases `narrator-prompt-layers` and `narrator-preset-fields-present` read the length words. The fixture `preset()` in `narrator_tests.rs` has `output_format: "Write one paragraph."`.

Docs:

- `docs/diataxis/reference/narrative/narration_system.md` names the `options` id.
- `docs/diataxis/reference/narrative/agent_system.md`, "Per-Agent Connections", says each Agent's connection comes from its type. (`docs/diataxis/explanation/agent_system_design.md` has the same heading, and its text stays true.)
- `docs/diataxis/reference/architecture_system.md`, "Settings": a reference that names no stored Connection fails. This stays true: an empty optional slot is no reference.
- The Output Format row of `docs/diataxis/reference/narrative/prompt_system.md` says "Preset `output_format` field + response length".
- `CONTEXT.md`, **Connection**: "One or more Roles use each Connection." It has no term for a slot.

## What already exists

| Need | Reuse |
|---|---|
| One lookup for each call kind | The shape of `AppSettings::active_preset_id(preset_type, mode)` |
| Two of the four arms | `narration_connection()` and `quantifier_connection()`, which become calls of `connection_for` |
| A recorder for a slot | `recorder_with_storage` (`wiring.rs`) |
| A test of two recorders | `narrator-call-record` in `narrator_tests.rs` |
| A read of the settings in a tier-1 test | `state.settings_service.get_settings()` |
| Settings with a slot set in a test | Seed `AppSettings` before the app is built, as the test of scenario 20.15 does |
| The migration test | The pattern in `plumbing_tests.rs`: `migrate_up_to`, raw rows, then `run_migrations` |

## Steps

1. **The slots.**
   - Add the two fields to `AppSettings` and `AppSettings::default()`, to `DbSettings` (`from_row`, `to_settings`), to the SELECT and the INSERT of `storage/settings.rs`, and to the settings schema.
   - A migration adds the two columns (the map's migration rule).
   - Add `connection_for`, with tests in the domain `settings_tests.rs` for each arm and both fallbacks. Make `narration_connection()` and `quantifier_connection()` call it.
   - Check the two optional slots at boot, at the seed, and in `validate_data.py`, like the two required ones.
2. **The wiring.**
   - `options_recorder` resolves `connection_for(Options)`. The lookup of the id `options` and its doc comment go.
   - The Narrator gets a second recorder on `connection_for(Impersonate)`, and uses it for `Generation::Impersonate`. Add `narrator-impersonate-uses-its-own-recorder` to `narrator_tests.rs`, after the shape of `narrator-call-record`.
   - Update the 4 `Narrator::new` sites. In `test_support/context.rs`, pass `Arc::clone(&recorder)` as the second recorder, so `make_test_job_deps` and its 4 callers stay as they are.
   - `Job::budget` takes the generation, and reads `connection_for(generation.preset_type())`.
   - Update `wiring_tests.rs`.
3. **Delete a Connection.**
   - Add scenario 20.53 to `docs/specs/settings.md`, for example "Deleting the Connection of the Impersonate slot returns Impersonate to the narration Connection". Word it with glossary terms, including **Connection slot**.
   - Its test in `tests/http/settings.rs` seeds settings with the Impersonate slot on a second Connection, deletes that Connection, and reads the cleared slot with `state.settings_service.get_settings()`.
   - In `delete_connection_handler`, inside the `update_settings` call, clear each optional slot that names the id, then run the existing guard, then remove the Connection.
4. **Dead config.** Delete `BackendSelector` and `AgentConfig.backend` in the six files, the `backend` values in `AgentConfig::defaults()`, the `backend` keys in `data/settings.json`, and the `backend` property in the schema. Add `additionalProperties: false` to the schema of one `agents` entry, so a leftover key fails the data check.
5. **The response length.**
   - Rewrite `narrator-prompt-layers` and `narrator-preset-fields-present`: the fixture `preset()` gains the length words in its `output_format`, and the cases read them from there.
   - Add `narrator-output-format-is-the-preset-text`: with a marker text in `output_format`, the `<output_format>` layer holds exactly that text, for a Narration and for an Impersonate.
   - Remove `AppSettings.response_length` and `default_response_length()`, and the parameter in the six functions and the three call sites named in the Facts.
   - Remove `response_length` from `data/settings.json` and the schema together.
   - Append the default text to the `output_format` of `data/prompt_presets/system/default.json` and `system/if_default.json`, in the shape that the migration writes.
   - Write the migration (see Decisions). It can share one step with the slot columns. Add its test to `plumbing_tests.rs`: a version-31 database with a settings row and two system presets (one with an output format, one without), and a second case with no settings row.
   - Fix the other tests mechanically. In the storage `settings_tests.rs`, use a Connection name as the marker value.
6. **The trial script.**
   - The setup file gains optional `impersonate` and `options` connections. The script reads the stored slots from the probe DB, and refuses a setup that asks for another value, in the same form as its refusal of a missing Connection id.
   - The pack drops its `response_length` row.
   - Extend `scripts/tests/test_trial.py`.
7. **Docs.**
   - `narration_system.md`: the four slots, and the fallback of the two optional slots. Remove the `options` id.
   - `docs/diataxis/reference/narrative/agent_system.md`, "Per-Agent Connections": the slot of each Agent.
   - `architecture_system.md`, "Settings": one sentence: an empty Impersonate or Options slot is no reference, and uses the narration Connection.
   - `prompt_system.md`, the Output Format row: "Preset `output_format` field".
   - `CONTEXT.md`, with `/domain-modeling` (O6): add a **Connection slot** entry, and change the **Connection** entry so it no longer says that only Roles use a Connection.
8. **Check.** Copy `target/ui-investigator/debug/chronicler_3001.db` to `target/ui-investigator/debug/chronicler_3002.db`, and start the probe server from this branch on port 3002, so the migration runs on real rows. The stored system presets hold the length words, and the `settings` table has no `response_length` column. Play one Impersonate: its prompt has no `Response Length:`.
9. Run `python build.py`.

## Failure modes

| Code path | Failure | Result | Handled by |
|---|---|---|---|
| Migration | A fresh database has no settings row yet | The append is skipped. The seed writes the presets with the words | The migration rule and its second test |
| Migration | A stored system preset has no output format | It gets no length words, as today | Same as today's render rule |
| Migration | A stored system preset already holds the words | The header appears twice | Accepted. Only a hand edit can make that state |
| Migration | The `UPDATE` fails | The step rolls back with its transaction | Existing migration rule |
| Migration | A user copied the default system preset | The copy gets the words too. An Impersonate preset copy does not | Intended: the rule follows the preset kind |
| Optional slot | `data/settings.json` names a Connection that does not exist | The data check and the boot fail, and name the id | "A dangling slot fails early" |
| Optional slot | An empty slot, and the narration Connection is missing | The call fails and names the narration id | Same as today for the narration slot |
| Delete | The guard refuses after a slot was cleared | `update_settings` writes nothing | Existing rule |
| Impersonate health | An Impersonate on its own Connection fails | The banner says "Narrator failed" | Accepted (see Decisions) |
| New fields on an existing database | A changed `data/settings.json` | It does not reach the stored row | Accepted (O3). Use a new database |

## Not in scope

- Routes and Settings rows for the two new slots (O3, and the map's UI fog item).
- A Role for Impersonate, and a `/debug/backend` row for its recorder.
- Preset generation parameters and the Connection ceiling ([Put the call parameters on the presets](22-call-parameters-on-presets.md)).
- The wording of the length words. They move as they are.

## Output

Resolved when the check passes and `python build.py` passes. The answer records the migration step, the scenario id, and the check results.
