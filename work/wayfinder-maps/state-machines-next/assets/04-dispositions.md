# Ticket 04: dispositions for the prompt, the settings and forms, and the storage and boot

Plan-only research. Rule: ticket 01 Answer. Evidence: code in the working tree, read on 2026-11-09. Each claim is labelled **known** (code read, file:line), **inferred** (reasoned from code, not run) or **unsure**.

Scope: C3, C4, C5, D1 to D6, E1 to E4 (13 machines).

## Summary

| Disposition | Machines | Count |
|---|---|---|
| Leave it | C3, C4, C5, D1, D5, D6, E2, E3, E4 | 9 |
| Explicit in the code | D2, D3, D4, E1 | 4 |
| Model-based test | none | 0 |
| TLA+ model plus verify skill | none | 0 |
| Generated diagram | none | 0 |

Notes on the counts:

- D3 and E1 are "explicit in the code" only if the owner answers behaviour questions 9 and 5 with "refuse". D2 and D4 hold whatever the owner answers (D4 also has question 3, which changes the form, not the disposition).
- D5 holds a possible lost update between two routes. The race is **known** in structure and **inferred** in effect. The rule says an inferred path gives only "leave it". So D5 is "leave it", with a plain defect ticket (20) and a To-check item.
- E3 and D3 become explicit or model-based-test candidates if the owner answers question 1 so that the seed rules change (the precedence becomes a table). That is a conditional, not a disposition now.
- No machine here shares a variable with `VerifyGate.tla`. No machine here needs a new `verify-<area>` skill.
- `04-sketches.md` holds one review sketch for each machine that is not "leave it" (D2, D3, D4, E1).

## The table

Column "Same time": parts that run at the same time, and a client-visible order difference.
Column "Prose": the prose in `docs/diataxis/` (found with `rg`).

### C3 Prompt call mode (`PromptContext.guide`, `.impersonate`)

| Column | Finding |
|---|---|
| Invalid state and path | "Impersonate call with a guide" or "Narration call with a direction". Both fields are `pub` (`src/application/prompting/assembler.rs:45-46`). **Known**: the only non-test writer is `src/application/narration/narrator.rs:128-129`. It reads one `Generation` variant at a time: `Impersonate` carries a direction, `Narration` carries a guide. `with_impersonate(true)` clears the guide (`assembler.rs:135-138`). No request sequence reaches the pair. Only a test can. |
| Places that make the moves | 1 non-test place (`narrator.rs:128-129`). |
| Same time | None. The context is built and used inside one Job step. No order difference. |
| Prose | `reference/narrative/narration_system.md:34-38`, `reference/narrative/prompt_system.md`. They describe the two kinds of generation, not the fields. |
| Disposition and form | **Leave it.** The machine is open (`pub` fields), not reachable. The map allows a small fix in its own module with no disposition: make the two fields private and keep the builder calls. Optional. |
| Behaviour question | None. |

### C4 History cut and overflow guard (`HistoryCut`, `excess_chars`)

| Column | Finding |
|---|---|
| Invalid state and path | None in the cut. `src/domain/model/history_cut.rs` is a pure fold: `HistoryCut::of` and `drop_to_free` compute `kept_from` and the blocks that cover `0..kept_from`. It holds no stored state, so it is not a state machine (**known**). The defect is in the guard: `response_tokens` clamps to `.max(1)` and asks for a one-token answer instead of raising `ContextOverflow` (`src/application/prompting/token_budget.rs:67`, `assembler.rs:188`). This confirms scan 1 over scan 2. It is bug 02 of `state-machines-work`. Not re-filed. |
| Places that make the moves | 1 (the assembler). |
| Same time | None. |
| Prose | `reference/narrative/prompt_system.md:59`, `explanation/prompt_system_design.md:22`. |
| Disposition and form | **Leave it.** Bug 02 fixes the guard. |
| Behaviour question | None new. |

### C5 Agent registration for each Hook (`AgentRunner`)

| Column | Finding |
|---|---|
| Invalid state and path | None. `src/application/agents/runner.rs` is a build-once configuration with private fields. `from_settings` runs at boot. A second config of one hook replaces the first with a `tracing` warning (**known**). A list that has only disabled Agents gives a runner with no Agents. That is valid: `reference/narrative/agent_system.md:10` says the engine works with zero Agents. |
| Places that make the moves | 1 (`runner.rs`, called from `src/bootstrap/wiring.rs:147-151`). |
| Same time | None after boot. |
| Prose | `reference/narrative/agent_system.md:10`, `:24`, `:31`; `reference/architecture_system.md:39`. |
| Disposition and form | **Leave it.** |
| Behaviour question | **Q8** (new): `AgentRunner::from_configs(...).unwrap_or_default()` at `wiring.rs:147-151` (also `:183`, `:186` in `build_app_graph_for_tests`) swallows `EngineError::Config("Unknown agent type: ...")`. A typo in `agent_type` in `data/settings.json` then boots with zero Agents and no log. Boot already refuses a dangling Connection slot (`wiring.rs:141`, `load.rs:69`). Must an unknown agent type also refuse boot, or at least log? (Known: the swallow. Inferred: that no other log line names it.) |

### D1 Connection slots

| Column | Finding |
|---|---|
| Invalid state and path | "A slot names a Connection that is not in the list." **Unreachable (known).** The four slots are one table `CONNECTION_SLOTS` (`src/domain/model/settings.rs`), each with a `Required(Role)` or `Optional` kind. `check_connection_slots` runs at boot (`wiring.rs:141`) and in the seed (`load.rs:69`, before the write). `remove_connection` refuses `NotFound`, `LastConnection` and `InUse(roles)` before any change, and clears optional slots that name the removed Connection. All writes go through `update_settings`, which does the read, the change and the write under one backend lock (`src/adapters/driven/storage/settings.rs:104-116`, **known**). |
| Places that make the moves | 3: `remove_connection`, `clear_optional_slots_naming`, the role handler (`settings/handlers/settings.rs`, `RoleForm`). All in `settings.rs` or one handler. |
| Same time | Settings writes are serial (one lock). No order difference. |
| Prose | `reference/narrative/narration_system.md:10`, `explanation/agent_system_design.md:16`. |
| Disposition and form | **Leave it.** The table plus the checks already make the invalid state unwritable. |
| Behaviour question | **Q4** (known, kept): must the Connection form show the difference between an empty slot (follows Narration) and a slot that names the same Connection? The model already holds the difference: an empty optional slot is `None` (`settings.rs`, `slot_reference`). The UI decision is the owner's. |

To-check resolved (D1, D6): `pin_provider_order = true` with an empty `provider_order` is accepted (`settings.rs:362-364`) but has **no effect** (**known**). `src/adapters/driven/llm/providers/chat_completions.rs:100-103` builds `provider_routing` only when `provider_order` is non-empty. So `allow_fallbacks = !pin_provider_order` is never sent. No state to prevent. No ticket.

### D2 Text check (`TextCheckMode` plus `enable_auto_check`)

| Column | Finding |
|---|---|
| Invalid state and path | The pair `mode = Disabled` with `enable_auto_check = true`. **Known, reachable by stored data**: the shipped `data/settings.json:70-71` holds exactly this pair, and `seed_settings` writes it without normalising (`src/bootstrap/load.rs:64-72`). The default is also this pair: `TextCheckMode::default()` is `Disabled` (`src/domain/model/settings.rs:15-19`) and `default_enable_auto_check()` is `true` (`src/domain/model/utils/settings_defaults.rs:4`). Both fields are `pub` (`settings.rs:299-304`). A normaliser repairs the pair on read (`effective_enable_auto_check`, `settings.rs:322`). **Known**: the two readers use it (`settings/templates/settings.rs:355`, `action/handlers/actions.rs:69`). So no client sees a wrong result today. The state exists only inside the stored row. |
| Places that make the moves | 3: the form handler `settings/handlers/settings.rs:196` (`set_mode_and_auto_check`), the seed (`load.rs:64`), the default (`settings.rs:308-314`). Raw readers: 0 outside the domain module (**known**, `rg` on `.enable_auto_check`). |
| Same time | None. |
| Prose | `reference/frontend/dashboard.md:63` (the Checking state of the action area). No prose states the pair rule. |
| Disposition and form | **Explicit in the code.** Form: private fields in `TextCheckSettings` with a checked constructor `TextCheckSettings::new(mode, requested_auto_check)` that normalises, getters, and `#[serde(from = "RawTextCheckSettings")]` so a stored or seeded pair is repaired on read. The JSON stays as it is, so no migration. The alternative (an enum `Disabled | Enabled { mode, auto_check }`) changes the stored JSON and needs a migration step. It costs more and is not needed. |
| Behaviour question | None. The rule already exists in the normaliser. |

### D3 Mode preset registry (`ModePresetRegistry`, `ModePresetBundle`)

| Column | Finding |
|---|---|
| Invalid state and path | Two paths, both **known** by code read. (a) A bundle names a preset that does not allow the bundle's mode. `update_preset_handler` calls `edit_preset` (`src/application/prompt_preset_service.rs:94-107`), which sets `allowed_modes` and does not read the registry. `delete_preset` does read it (`prompt_preset_service.rs:120`). So: activate preset P for Novel (the activate handler checks `allows`, `prompt_presets.rs:283`), then edit P with only Interactive Fiction checked. The Novel bundle now names a preset that does not allow Novel. (b) A bundle names a preset that does not exist: `seed_settings` checks Connection slots only (`load.rs:69`), not preset ids; presets seed before settings (`run.rs:44`). The effect of (a) and (b) is **inferred**: `new_game_request` (`src/application/games/catalogue.rs:271-290`) and `set_mode` (`catalogue.rs:165-185`) copy the bundle ids with no check of `allows` or existence. `chosen_preset` then returns `PresetNotFound` at the first call for (b) (`prompt_preset_service.rs:70`). (c) Duplicate mode entries in the list are possible in stored data (the type is a `Vec`); `stored_bundle` reads the first, `set_bundle` replaces the first (`settings.rs:200-212`). Effect **inferred**, low. |
| Places that make the moves | 1 writer of a bundle: `set_bundle` (`settings.rs:206`), 1 caller: `set_active_preset` (`settings.rs:592`), 1 caller: the activate handler (`prompt_presets.rs:292`). Plus the whole-row seed. 3 readers: `catalogue.rs:166`, `:275`, `prompt_preset_service.rs:120`. The table is not a machine over time; it is a record with an invariant that crosses two records (bundle and preset). |
| Same time | The edit of a preset and the activation of the same preset are two requests. Settings writes are atomic (one lock), but the preset check in the activate handler and the settings write are two calls (`get_preset`, then `update_settings`). **Inferred**: a preset edit between the two calls reaches the same state as (a). Low. |
| Prose | `reference/narrative/narrator_mode.md:31-34`: "Allowed modes gate selection surfaces only". The doc says that the narrate path uses the ids a Game already holds. So (a) is not a bug by the doc. It is a rule gap. |
| Disposition and form | **Explicit in the code**, conditional on **Q9**. Form: one check `AppSettings::check_preset_slots(&[PromptPreset])` (or the same check in `PromptPresetService`) that refuses (1) at seed, a bundle that names a missing preset, (2) at edit, a change of `allowed_modes` that removes a mode from a preset that a bundle of that mode names. This is the same shape as `check_connection_slots`. It is a refusal at the two write sites, not a new type. |
| Behaviour question | **Q9** (new): must an edit of a preset be refused when it removes a mode that the registry uses that preset for (as `delete_preset` refuses a delete)? Must the seed refuse a `mode_preset_registry` that names a missing preset (as it refuses a dangling Connection)? If the owner answers "no", the doc line at `narrator_mode.md:34` is the rule and D3 becomes "leave it". |

### D4 Allowed Modes of a preset (two checkboxes to `Option<Vec<NarratorMode>>`)

| Column | Finding |
|---|---|
| Invalid state and path | An empty `allowed_modes` list: a preset that no mode may select. **Known, reachable by stored data**: `process_preset_file` accepts `"allowed_modes": []` (`src/bootstrap/run.rs:217-227`, `serde_json::from_value::<Vec<NarratorMode>>` succeeds on `[]`), and `PromptPreset::validate` does not check the list (`src/domain/model/prompt_preset.rs:103-124`). The seed sets the list at insert only. No shipped file has the key (`rg` on `data/prompt_presets`: no hit). No request creates the empty list: create maps both-clear to the default (`prompt_presets.rs:171-176`), edit maps both-clear to `None` (`:207`). Duplicates (`[novel, novel]`) are also accepted by the seed (**known**; harmless, **inferred**). |
| Places that make the moves | 4: create handler, edit handler (`prompt_presets/handlers/prompt_presets.rs`), `edit_preset` (`prompt_preset_service.rs:101-104`), the seed (`run.rs`). |
| Same time | None. |
| Prose | `reference/narrative/narrator_mode.md:32-34`. `docs/specs/prompt_presets.md` scenarios 21.14 (edit keeps the stored modes when the form omits them), 21.25-21.27. |
| Disposition and form | **Explicit in the code.** Form: a private type `AllowedModes` that holds one or two modes, with a checked constructor and `TryFrom<Vec<NarratorMode>>`. The stored JSON stays a list. The seed falls back to both modes with a warning on `[]`, as it already does for an invalid value (`run.rs:217-227`). The two-boolean parse in the handlers stays, and the Q3 answer decides the both-clear case. |
| Behaviour question | **Q3** (known, kept): when both boxes are clear, create reads "both modes" and edit reads "no change". The cause is that urlencoded forms cannot tell "unchecked all" from "field omitted" (comment at `prompt_presets.rs:205-206`, **known**). Which meaning must the owner see? An error ("choose at least one mode") is the only answer that makes the form the same on both routes. |

### D5 World form and its two write routes

| Column | Finding |
|---|---|
| Invalid state and path | A lost update: a posture change or a detail change that the other route overwrites. **Known structure**: both handlers read the whole World (`get_world`) and write the whole row (`update_world`) as two separate `Storage` calls with no lock between them (`src/adapters/driving/http/worlds/handlers/worlds.rs:165-217` detail, `:236-297` posture; `src/application/world_catalogue.rs:59-64`). `update_world` writes every column (`src/adapters/driven/storage/worlds.rs:297-335`). Contrast: the settings read-modify-write is atomic (`storage/settings.rs:104-116`). **Inferred effect**: two requests interleaved in the window between the read and the write lose one change. Two posture requests can also finish out of order (each posts all three selects, so the last write wins and the screen can differ from the row). The window is milliseconds. No test holds a request between the two calls. The four form fields `is_edit`, `is_readonly`, `auto_save_key`, `form_action` all come from `world.is_some()` in one constructor (`worlds/templates/worlds.rs:161-206`), the only construction site (**known**). So the form itself has no invalid state. |
| Places that make the moves | 2 routes plus the create route: `update_world_handler`, `update_world_posture_handler`, `create_world_handler`. |
| Same time | Yes: two HTTP requests from the same dialog (the posture selects save on change; the detail form saves on submit). Client-visible difference: a posture value that reverts after a later reload. **Inferred.** |
| Prose | `docs/specs/worlds.md:30`, `:69-81` (posture patch). No `docs/diataxis/` prose states the rule. |
| Disposition and form | **Leave it** (the effect is inferred; the rule allows no other disposition). A plain defect ticket is filed: `20-bug-world-posture-and-detail-saves-can-overwrite-each-other.md`. The ticket proposes one fix: a posture write that updates only the three posture columns, and a detail write that leaves them out. Bug 04 (the map write outside a transaction) touches the same function. A TLA+ model is not worth its cost for one read-modify-write race. If the To-check below shows the race, the rule still gives no model: the fix removes the machine. |
| Behaviour question | None. |

To-check (D5): hold the first request between `get_world` and `update_world` (a barrier in a test with the storage fake), send the second, then read the row. The disposition holds if the row loses the second change.

### D6 Connection form (`editing: bool`, `ConnectionTestSurface`)

| Column | Finding |
|---|---|
| Invalid state and path | None. `ConnectionFormTemplate::new` is the only construction site (**known**, `rg`); `editing` and `form_action` come from one `Option` (`settings/templates/settings.rs:224-262`). `ConnectionTestSurface` is already an enum with two variants (`settings/handlers/settings.rs:285-290`). The add, edit and form-test routes share one mapping, `ConnectionForm::into_config`, which refuses a non-positive token count and an unlisted reasoning effort. The edit keeps the stored `max_tokens_ceiling`, which the form does not own (`settings/handlers/settings.rs:271-273`). |
| Places that make the moves | 3 routes (add, edit, form test) plus the card test (`:345`). One mapping. |
| Same time | The Test button posts the form to `/connections/test` while a Save can run. The test records nothing. No order difference to a client. |
| Prose | `reference/frontend/dashboard.md:185`. |
| Disposition and form | **Leave it.** |
| Behaviour question | Q4 (see D1). |

### E1 Migration ladder (`user_version`, `due(step)`)

| Column | Finding |
|---|---|
| Invalid state and path | A database whose `user_version` is above `LATEST_VERSION` (34). **Known, reachable by stored data**: a database that a newer binary wrote, opened by an older binary. `migrate_up_to` reads the version (`src/adapters/driven/storage/utils/plumbing.rs:51-54`) and every `due(step)` is false, so nothing runs and no refusal is raised. The older code then reads and writes the newer schema. The path needs a downgrade of the binary. Other checks, all **known**: every block writes its own number as `user_version`, except `due(10)`, which writes 11 (there is no `due(11)` block); the last block `due(34)` writes 34, equal to `LATEST_VERSION` (`:17`). No test proves this for every future block. The whole ladder runs in one transaction (`:58-60`), so a failed step leaves the old version. |
| Places that make the moves | 1: `migrate_up_to` (`plumbing.rs:25-800`). |
| Same time | None. Boot is single-threaded before the server binds. |
| Prose | None in `docs/diataxis/` (`rg` finds `migration` only in test standards). The block comment at `plumbing.rs:20-24` is the rule. |
| Disposition and form | **Explicit in the code**, conditional on **Q5**. Form: one check at the top of `migrate_up_to`: if `version > LATEST_VERSION`, return `EngineError` that names both numbers. About 5 lines. No new type. |
| Behaviour question | **Q5** (known, kept): must the boot refuse a database whose `user_version` is above `LATEST_VERSION`? If the owner answers "no", E1 is "leave it". |

To-check resolved (E1): the `column_exists` guards. Some steps guard and some do not (**known**: for example `due(13)` adds columns with no guard; `due(12)`, `due(15)`, `due(16)`, `due(18)`, `due(19)` and later guard). The version number decides which steps run, and the ladder is one transaction, so an unguarded `ALTER TABLE` runs once on a database at the right version. A guard matters only for a database whose schema is ahead of its `user_version` (for example after a hand edit). No request or stored data reaches this by normal use (**inferred**). The test `test_v34_runs_over_a_database_that_already_has_the_columns` (`plumbing_tests.rs`) shows the guard idea for step 34. No state to prevent. No ticket.

### E2 Transaction (`Tx`, `WriteGuard`)

| Column | Finding |
|---|---|
| Invalid state and path | (a) A closure that calls `Storage` inside a `Tx`: it waits for the backend mutex that the unit holds, so it deadlocks (comment at `src/adapters/driven/storage/transaction.rs:46-47`). (b) A closure that swallows a failed write and returns `Ok`: the other writes commit. **Both unreachable (known).** `TxToken` is made only by `BusyRegistry::hold` (`gate.rs:224-225`). Every closure is in `src/application/generation/gate.rs` (the 10 call sites at `:480`, `:497`, `:525`, `:542`, `:562`, `:581`, `:590`, `:601`, `:772`, `:823-893`). **Known**: none calls `Storage` or swallows an error. The `unwrap_or_default` at `gate.rs:616` and the `if let Err` at `:654` sit outside any closure. `Tx` fields are private, so a write cannot reach another Game (`transaction.rs:51-55`). |
| Places that make the moves | `Tx` has 12 write methods (`transaction.rs:128-321`); 10 closure sites, all in `gate.rs`. |
| Same time | The busy registry serialises them. This is in the TLA+ models of the gate (out of scope). |
| Prose | `explanation/storage_design.md:14` (the two points where the change counters rise). |
| Disposition and form | **Leave it.** The comment is the only rule; the token already confines every caller to one file. |
| Behaviour question | None. |

### E3 Seeding precedence

| Column | Finding |
|---|---|
| Invalid state and path | None reachable. The precedence differs by kind (**known**): World, Character, Persona files seed only when the row is absent (`src/bootstrap/load.rs:115-125`); preset rows are seeded when absent, and a row with no text is filled again (`run.rs:246-262`); `allowed_modes` seeds at insert only (`run.rs:215-216`); settings seed only when the row is absent (`storage/settings.rs:118-124`, `SettingsWrite::OnlyIfAbsent`). So after the first boot the database wins for every kind. `seed_world` says that it replaces, but boot calls it only for a World that is not stored (`load.rs:115-125`): a doc comment that does not match the use (**known**; catalogue row E3 and bugs 04 and 05 already record this). |
| Places that make the moves | 4: `ensure_presets` (`run.rs`), `seed_worlds`, `seed_personas` (+ characters) (`load.rs`), `seed_settings` (`load.rs:60-72`). |
| Same time | None. Seeding ends before the server binds (`startup.md:26`). |
| Prose | `reference/startup.md:23-26`, `explanation/storage_design.md:20-32`. The doc says "idempotent" and "seed JSON files once". It matches the code. |
| Disposition and form | **Leave it.** If the owner answers Q1 so that a changed file must reach a stored row, the precedence becomes an explicit table (one policy per kind) and the rule's "becomes explicit" test then gives a model-based test. That is a follow-up, not a disposition now. |
| Behaviour question | **Q1** (known, kept), **Q2** (known, kept). Q2 evidence: `seed_settings` reads and parses the file and runs `check_connection_slots` on it before the `OnlyIfAbsent` write (`load.rs:64-72`), so a malformed or dangling file stops the boot even when the row exists (**known**). The test `test_seed_settings_malformed_file_fails` pins this as intended (`load_tests.rs:187`). |

To-check resolved (E3): `INSERT OR REPLACE` writes `created_at` again on every `update_settings` (`storage/settings.rs:69-80`, **known**). **Known**: the only reader is `DbSettings::from_row` (`models/settings.rs:35`); `AppSettings` has no `created_at` field (`rg`: no hit in `src/domain/model/settings.rs`). So no behaviour depends on it. No ticket. Note for the owner: the column no longer means "created".

### E4 Boot resume-or-create

| Column | Finding |
|---|---|
| Invalid state and path | "A boot id with no Game row." **Unreachable (inferred)**: `resolve_boot_game` (`run.rs:97-115`) gets its id from `newest_game_of_world` (a row it just read) or from `create_game` (the create unit). Nothing else writes at boot. The branch `Ok(None) => tracing::warn!("Starting on game {game_id}, which has no row")` exists but I found no path. `resolve_game_id` is the only decision (`src/bootstrap/init_game.rs:9-25`): resume the newest Game of the World, or create one. |
| Places that make the moves | 1 (`init_game.rs`). |
| Same time | None. |
| Prose | `reference/storage.md:66-76` (the newest-Game rule), `reference/startup.md`. |
| Disposition and form | **Leave it.** |
| Behaviour question | **Q6** (known, kept): `--persona` is validated (`run.rs:86`) but boot resumes the newest Game of the World whatever its Persona (`run.rs:88`, `init_game.rs:13-22`). The argument acts only when a new Game is made. **Q7** (known, kept): boot picks the Game with the newest Message (`storage/games.rs:135-149`). The Games list orders by `updated_at` (`games.rs:18`, `:48`). A restart can resume a Game other than the top of the list. Bug 06 fixes only the false texts. |

## To-check results (all items of the ticket)

| Item | Result |
|---|---|
| D1, D6 `pin_provider_order` with empty `provider_order` | No effect (**known**, `chat_completions.rs:100-103`). Nothing to prevent. |
| E1 `column_exists` in some steps only | Not reachable by normal use (**inferred**). Steps are version-gated in one transaction. The real gap is the check for `version > LATEST_VERSION` (Q5). |
| E2 a closure that swallows an error and returns `Ok` | Not present (**known**, 10 closures read, all in `gate.rs`). |
| E3 `created_at` rewritten on every settings save | True (**known**), no reader (**known**). No ticket. |
| D3 (new) edit narrows the Allowed Modes of an active preset | Path **known** by code read; effect **inferred**. Needs a test with requests to confirm the screen and the Game start. This is the check that D3 depends on. |
| D5 (new) lost update between the two routes | Structure **known**, effect **inferred**. See D5. |

## Behaviour questions for ticket 06

Each names its machine and its file:line.

1. **E3** After the first boot the database wins for every kind. An edit of a World, Character or Persona file never reaches the stored row (`load.rs:115-125`). Do we want this, or must a changed file reach a row that nobody edited? The doc comment of `seed_world` also needs the answer.
2. **E3** A malformed or dangling `data/settings.json` stops the boot when the row exists (`load.rs:64-72`, pinned by `load_tests.rs`). Do we want this?
3. **D4** Both Allowed Modes boxes clear: create reads "both modes", edit reads "no change" (`prompt_presets.rs:171-176`, `:207`). Which meaning, or an error?
4. **D1, D6** Must the Connection form show the difference between an empty slot and a slot that names the same Connection?
5. **E1** Must the boot refuse a database whose `user_version` is above `LATEST_VERSION` (`plumbing.rs:17`, `:51-54`)?
6. **E4** Must `--persona` govern the resumed Game (`run.rs:86-88`, `init_game.rs:13-22`)?
7. **E4** Must the boot resume order (`games.rs:135-149`) agree with the Games list order (`games.rs:18`, `:48`)?
8. **C5** (new) Must an unknown `agent_type` refuse the boot or be logged (`wiring.rs:147-151`, `:183`, `:186`)?
9. **D3** (new) Must an edit of a preset be refused when it removes a mode that the registry uses that preset for (`prompt_preset_service.rs:94-107`)? Must the seed refuse a registry that names a missing preset (`load.rs:60-72`)?
10. **D2** (new, small) The shipped `data/settings.json:70-71` holds `Disabled` with `enable_auto_check: true`, and the default is the same pair. After D2, the pair reads as auto-check off. Must the shipped file say `false`, to show the real value? (A data edit, no code.)

## Draft work tickets (not filed; ticket 06 files them after owner acceptance)

### Work ticket A: the text check settings cannot hold `Disabled` with auto-check on (D2)

- **Form.** Private fields in `TextCheckSettings`, `TextCheckSettings::new(mode, requested_auto_check)` that normalises, getters `mode()`, `auto_check()`, `ignored_words()`. `#[serde(from = ...)]` repairs a stored or seeded pair on read. Remove `effective_enable_auto_check` and `set_mode_and_auto_check` (their work moves into the constructor).
- **Files.** `src/domain/model/settings.rs` (`:298-333`), `src/adapters/driving/http/settings/templates/settings.rs:355`, `src/adapters/driving/http/action/handlers/actions.rs:69`, `src/adapters/driving/http/settings/handlers/settings.rs:196`, `src/domain/model/utils/settings_defaults.rs:4`, the unit tests of the settings module.
- **Done when.** (1) `TextCheckSettings` cannot be built or read with `Disabled` and auto-check on. (2) A unit test reads the JSON `{"mode":"Disabled","enable_auto_check":true}` and finds auto-check off. (3) The existing scenarios of `docs/specs/` for the text check still pass with no change. (4) No migration step is added.
- **Cost.** Small, one module and three call sites. **Prevents**: the pair that `data/settings.json` holds today. Behaviour change: none for a client.
- **Needs.** Q10 only for the data file.

### Work ticket B: a mode default and a preset agree (D3), conditional on Q9

- **Form.** `AppSettings::check_preset_slots(&[PromptPreset]) -> Result<()>`: each bundle names stored presets, and each preset allows the bundle's mode. Called at seed (`load.rs`, after `ensure_presets`) and by `PromptPresetService::edit_preset` for the preset it saves. A refusal uses `ApplicationError::validation`, as `delete_preset` does.
- **Files.** `src/domain/model/settings.rs`, `src/application/prompt_preset_service.rs`, `src/bootstrap/load.rs`, specs `docs/specs/prompt_presets.md` (a scenario for the refused edit).
- **Done when.** (1) An edit that removes Novel from the preset of the Novel bundle answers 400 with a text that names the mode. (2) A `data/settings.json` with a missing preset id stops the boot with the field name. (3) The doc line `narrator_mode.md:34` still reads true, or it names the new rule.
- **Cost.** Small to medium: one function, two call sites, one spec scenario, three tests. **Prevents**: path (a) and (b) of D3.

### Work ticket C: a preset has at least one Allowed Mode (D4)

- **Form.** Private type `AllowedModes` (one or two distinct modes) with `TryFrom<Vec<NarratorMode>>`; `PromptPreset.allowed_modes` keeps the same JSON. The seed falls back to both modes with a warning on `[]`.
- **Files.** `src/domain/model/prompt_preset.rs`, `src/domain/model/utils/settings_defaults.rs` (`default_allowed_modes`), `src/bootstrap/run.rs`, `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs`, `src/application/prompt_preset_service.rs`, `src/adapters/driving/http/builders/presets.rs`, the storage mapper of presets.
- **Done when.** (1) No code path builds a preset with an empty list. (2) A seed file with `"allowed_modes": []` loads with both modes and logs a warning. (3) The Q3 answer is in the handlers and in `docs/specs/prompt_presets.md`.
- **Cost.** Medium: the type touches the storage mapper and every reader of `allowed_modes` (`allows`, `allows_novel`, `allows_interactive_fiction`). Do it with the Q3 change so the form changes once.

### Work ticket D: refuse a database from a newer binary (E1), conditional on Q5

- **Form.** At the top of `migrate_up_to`, `if version > LATEST_VERSION { return Err(...) }` with both numbers in the text. No type.
- **Files.** `src/adapters/driven/storage/utils/plumbing.rs`, `plumbing_tests.rs`.
- **Done when.** A test sets `user_version` to 35 and finds that `run_migrations` returns an error and changes nothing.
- **Cost.** Very small. **Prevents**: writes to a newer schema.

## Plain defect ticket filed

- `work/issues/pending/16-bug-world-posture-and-detail-saves-can-overwrite-each-other.md` (D5). Checked against tickets 01 to 09: ticket 04 covers the map write outside a transaction, not the two-route overwrite. No other defect is new: the C4 guard is bug 02, the E3 seed facts are bugs 04, 05, the list-order texts are bug 06.

## Draft `## Answer` for ticket 04

Of 13 machines, 9 get "leave it" and 4 get "explicit in the code" (D2, D3, D4, E1). None gets a model-based test, a TLA+ model or a generated diagram: no machine here has two or more places that move it while it is explicit, and none has a client-visible order difference that is more than inferred. D2 is the one clear case: the shipped `data/settings.json` and the default both hold the invalid pair `Disabled` with auto-check on, and a normaliser hides it on read. D4 holds an empty Allowed Modes list that only a seed file can write. D3 and E1 depend on the owner: Q9 (refuse an edit that breaks a mode default) and Q5 (refuse a newer database). D5 has a possible lost update between the posture and detail saves; the effect is inferred, so it is a plain defect (ticket 20) and a To-check. Ten behaviour questions go to ticket 06; Q8 (swallowed unknown agent type) and Q9 are new. Four draft work tickets (A to D) are ready, A the cheapest and the most certain.
