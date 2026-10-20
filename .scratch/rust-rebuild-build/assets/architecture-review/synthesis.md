# Synthesis of the three architecture reviews (ticket 17)

- **Inputs:** [opus.md](opus.md) (8 candidates), [glm.md](glm.md) (5), [qwen.md](qwen.md) (5), [the review findings](../whole-rebuild-review/FINDINGS.md) (29 rows).
- **Tree:** `wf/rebuild-core` at `d7cf3eb0`. `git diff 2885df2c HEAD -- src tests` is empty, so the code is the code of `2885df2c`. No dirty file under `src/` or `tests/`.
- **Method:** I read each cited site myself. Read-only; no build, no server.
- **Labels:** *known* = I read it in the code at the cited line. *inferred* = it follows from the code, but no test or run confirms it.
- **Result:** the 18 candidates merge into 6 merged candidates (13 run candidates) and 5 single-model candidates. I reject no whole candidate. I reject 5 sub-claims (see [Rejected](#rejected)).

## 1. Merged candidates

Strength: S = Strong, W = Worth exploring, Sp = Speculative. "Verified" = the friction that the candidate names is in the code.

| # | Candidate | Opus | GLM | Qwen | Verified | Review findings it touches |
|---|---|---|---|---|---|---|
| M1 | **Deepen the Game rules: one room lookup, the event rules and the event record in one pure module** | 1 · S | C1 · S (room lookup only) | — | yes | 4 (event rules, opening room report, room lookup, one-shot mark), 12, 25 (`NpcEvents`, `GameRulesOutcome` fields) |
| M2 | **The in-memory Storage adapter is half an adapter** | 2 · S (delete it) | C2 · W (split the seam) | 2 · W (split the seam; `TxRead` key) | yes | 7; 28 (no storage read-failure case) through the Qwen variant only |
| M3 | **The narration prompt rules move from the Job behind the narrator** | 3 · S | — | 4 · W | yes | 19, 24, 15 (in part), 25 (`Narrator::assemble` pub) |
| M4 | **One analysis output type from the hook to the Game rules** | 6 · W | — | 1 · S | yes | 17 (the `QuantifierResult`↔`StatePatch` round trip) |
| M5 | **One story entry instead of three projections** | — | C5 · Sp | 3 · W | yes | 17 (`StoryRow` ≈ `MessageEntry`) |
| M6 | **The shallow catalogues (`PersonaCatalogue`, `WorldCatalogue`)** | — | C4 · W (fold reads into `GameViewQuery`) | 5 · Sp (one "what can a new Game bind to" module) | yes | none |

### M1 — Game rules (Opus 1, GLM C1)

GLM C1 is the room lookup only. Opus 1 contains it and adds the event rules, the opening room report and the event record.

- The room lookup ("map room, else added room") is in four places: `game_rules.rs:184`, `job.rs:701`, `view_query.rs:61-63`, `gate.rs:806` (known). `gate.rs:806` uses the map only and drops its read errors with `.ok()` (known). The name lookup `job.rs:711` also uses the map only (known).
- The opening room report is in three places: `opening.rs:29`, `gate.rs:711` (`tail.count == 1`), `job.rs:504` (`is_the_opening`) (known). A Retry of the opening writes the room report twice, first in `save_text` and then in `save_state_update` (known).
- For an event generation, `Job::apply_rules` (`job.rs:442-485`) removes the movement, calls the rules twice and clears `trigger_context` by hand (known).
- Two sites mark a one-shot Trigger as fired: `swipe_state.rs:56` (`pre_analysis`) and `game_rules.rs:219` (known).
- Two rules build the event record. A Trigger that fires renders its direction (`game_rules.rs:119-120`). A Retrigger copies the raw `narration_prompt` (`gate.rs:587`) (known).
- `GameRuleRequest::NpcEvents` has no production caller. Only `game_rules_tests.rs` calls it (known).
- **Constraint (known):** the `arch-lint.toml` rule "model → storage" (`arch-lint.toml:99-103`) applies to `domain/model/`. `WorldBundle` is in `adapters/driven/storage/worlds.rs:26`. So the Game rules cannot take a `WorldBundle`. Opus 1 says they take "the World". Ticket 13 must state the World as domain types.

### M2 — Storage (Opus 2, GLM C2, Qwen 2)

The three runs agree on the friction. They disagree on the fix: Opus deletes the in-memory adapter, and GLM and Qwen split the seam into a catalogue half and a record half. FINDINGS row 7 leaves this as an owner call.

- Non-test storage files hold 39 `Backend::InMemory` occurrences. Of these, 9 arms refuse with `no_in_memory`: 6 in `stories.rs`, 1 in `docks.rs`, 2 in `transaction.rs` (known). The runs give 40, 41, 7 and 9. The thermo reviewer gave 14. The count here is mine.
- `touch_game_updated_at` is a silent no-op on the in-memory adapter (`core.rs:225`). `add_failure` swaps in a placeholder in-memory backend (`core.rs:105`) (known).
- A failed read is keyed on a `&'static str` method name (`core.rs:97`, `core.rs:166`). A failed write is keyed on the `TxWrite` enum (`transaction.rs:19`) (known). This is in Qwen 2 only.
- 32 files call `new_in_memory(` (known). The `tests/STRATEGY.md:15` unit-tier row names "in-memory `Storage`" (known). A delete changes that row and those 32 files. That is the churn that GLM and Qwen give as their reason for W.

### M3 — Narration prompt rules (Opus 3, Qwen 4)

- `Narrator::narrate` takes role, `PromptContext`, preset, response length and budget (`narrator.rs:77-84`). The Job picks the preset (`job.rs:372-375`), builds the user message per `Kind` (`job.rs:576-603`), holds the English continuation text (`job.rs:605-615`) and sets where the window ends (`job.rs:639`) (known).
- One generation reads the settings 4 times (`preset`, `budget`, `preset_text` ×2), the Game row 2 times (`narrate_step`, `options_enabled`), and the Tail up to 4 times (known).
- `GameCatalogue::switch_mode` writes the Game row through `storage.update_game_config` (`catalogue.rs:162`), not through the writer. So a mode switch during a Job can give one Job two Game rows (inferred, as Opus says).
- **Constraint (known):** the narrator is in the `agent-edge` scope (`arch-lint.toml:73-78`, `108-111`), which must not import storage. Opus 3 puts the "World bundle" into the narrator input, but `WorldBundle` is a storage type. Ticket 13 must state that input without it.

### M4 — Analysis output type (Opus 6, Qwen 1)

- The analysis goes `QuantifierResult` → `StatePatch` (`quantifier/agent.rs:64-68`) → merge → a new `QuantifierResult` (`runner.rs:74-84`) (known).
- `Confidence` (`agent.rs:217`) and `QuantifierConfidence` (`quantifier.rs:8`) have the same three values and two `From` impls (`quantifier.rs:32`, `quantifier.rs:42`) (known).
- The round trip drops `movement_type` and the movement confidence. Only the parser and a log line read them (`orchestration.rs:115`) (known). Qwen only.
- `SceneState.quantifier_confidence: Option<String>` (`scene_state.rs:11`) has no production writer. Only the debug view copies it (`view_query.rs:160`) (known). Qwen only.

### M5 — Story projections (GLM C5, Qwen 3)

- `StoryRow` (`message_types.rs:32`) and `MessageEntry` (`message_types.rs:60`) hold the same fields, except `kind`. One `From` copies them (`message_types.rs:44`). `StoryEntry` (`agent.rs:91`) has two `From` impls (known).
- The analysis window converts twice: `StoryRow` → `MessageEntry` (`job.rs:274-279`) → `StoryEntry` (`job.rs:419`). The Options window converts once (`job.rs:528-532`) (known).
- `MessageEntry` has `Serialize` for `DebugStateView`. Both runs keep a type where serde needs one.

### M6 — Shallow catalogues (GLM C4, Qwen 5)

- `PersonaCatalogue` has one method, a pass-through (`persona_catalogue.rs:24`). `WorldCatalogue` has five methods. Only `create_world` maps an error (`world_catalogue.rs:39-50`) (known).
- `list_games_fragment` calls `world_catalogue`, `persona_catalogue` and `game_catalogue` (`games.rs:44`, `48`, `78`) (known).
- The guardrail `check_http_storage_leak` (`tests/infrastructure/guardrails/layers.rs:203`) keeps HTTP away from `Storage`. Both runs keep that rule.
- The two runs agree on the friction and disagree on where the reads go.

## 2. Single-model candidates

| # | Candidate | Model · strength | Verified | Review findings it touches |
|---|---|---|---|---|
| S1 | Move the Job plan rules from the writer to the `Tail` | Opus 4 · W | yes | 5; 1 (`require_job_data`, `gate.rs:546`); 3 (inferred: the plan is where the kind/data split shows) |
| S2 | One preset module: the preset of a Game for a hook, and the edit rules | Opus 5 · W | yes | 15; a new inferred gap (see below) |
| S3 | One story-window read for the story log, with the Retrigger rule from the `Tail` | Opus 7 · W | yes | the TN-L4 row (rejected in FINDINGS as not a regression); a new gap (Retrigger button, §4) |
| S4 | One composition root; the LLM port is the only test input | Opus 8 · Sp | in part | 14 |
| S5 | One hook-input assembly for the analysis hook and the Options hook | GLM C3 · W | yes | 19 (the World bundle is read in `generate` and again in `options_step`) |

- **S1 (known):** `GameWriter::plan_for`, `generation_plan`, `require_start`, `require_job_data`, `event_record` and `trigger_direction` are in `gate.rs:427-589`. `Tail::redo` and `Tail::retrigger` are in `tail.rs:50-80`. The bare `/impersonate` 500 of finding 1 is the check at `gate.rs:546`.
- **S2 (known):** there are three policies for a missing preset. A missing narration preset is a failure line (`job.rs:754-758`). A missing Quantifier or Options preset gives `None` (`job.rs:800`). `preset_id_or` falls back to the registry on any Game read error (`games.rs:237`, `.ok().flatten()`). The edit and delete rules are in the HTTP handler (`prompt_presets.rs:193`, `211`, `252-272`). `PromptPresetService::delete_preset` is a pass-through (`prompt_preset_service.rs:46`). **New, inferred:** the delete checks the registry and the default Options preset, but not the preset slots of each Game. So a Game can point at a deleted preset. No FINDINGS row names this.
- **S3 (known):** see §4, possible bugs 2 and 3. The window sizes are `STORY_LIMIT = i32::MAX` (`stories.rs:19`), `STORY_WINDOW = 4` (`agent.rs:114`) and `MAX_LOG_DISPLAY = 50` (`app_state.rs:100`).
- **S4 (in part):** two graphs exist. `build_app_graph_for_tests` uses one recorder for the narrator and the Quantifier and replaces `JobDeps` (`wiring.rs:180-219`) (known). Two HTTP sites start a Job with `state.job_deps.clone()` (`actions.rs:40`, `chat_window.rs:52`) (known). The Opus claim "tier-1 tests do not run the per-role connection resolution" is only partly true; see [Rejected](#rejected).
- **S5 (known):** `analyse_step` (`job.rs:410-437`) and `options_step` (`job.rs:533-547`) build the same `WorldView` and `HookInput`. Their windows differ (before the new Message, or the whole story). The idea overlaps M3: Opus 3 also wants one read of the generation inputs.

## Rejected

I reject no whole candidate: the code shows the friction of each one. I reject these sub-claims:

| Claim | Run | Why rejected (known) |
|---|---|---|
| "Tier-1 tests do not run the per-role connection resolution of production" | Opus 8 | `app_with_production_graph` (`tests/http/support/app_wiring.rs:76-91`) calls `build_app_graph`. 7 tier-1 sites use it. `src/bootstrap/wiring_tests.rs:25` also calls it. The default `TestAppBuilder` path does not use it. |
| `list_games_fragment` "then [calls] `GameViewQuery` for the rest" | GLM C4 | `games.rs` makes no `game_view_query` call. The handler stitches three catalogues, not four modules. |
| "Every window read converts twice" | GLM C5 | Only the analysis window converts twice. The Options window converts once (`job.rs:528-532`). |
| `quantifier/types.rs` "is mostly re-exports" | Qwen 1 | The file also defines `RoomInfo` and `QuantifierPromptContext`, which the prompt uses. The re-export line is real. |
| `WorldHasGames` is a `WorldCatalogue` rule | GLM C4 | Storage raises it (`worlds.rs:393`, `403`). `WorldCatalogue::delete_world` is a pass-through (`world_catalogue.rs:63`). |

## 3. Review findings no candidate touches

2, 3 (no candidate names it; S1 touches it only by inference), 6, 8, 9, 10, 11, 13, 16, 18, 20, 21, 22, 23, 26, 27, 29.

FINDINGS says that findings 3 to 6 are one refactor. The candidates cover 4 (M1) and 5 (S1) only. No run names finding 3 (the data-carrying kind) or finding 6 (`Job::run`/`failed` hold one fact in three booleans; `failed` skips the Options run after `Stopped::SaveFailed`, `job.rs:217-219`, known). Finding 2 (the failure lines, `impl From<EngineError> for Failure` at `job.rs:87`) is also untouched. Ticket 13 must cover these three findings separately.

## 4. The three possible bugs of the Opus run

| # | Bug | In the code | FINDINGS row |
|---|---|---|---|
| 1 | The Retrigger direction is not rendered | **Known.** `gate.rs:587` copies `trigger.narration.narration_prompt` raw. `game_rules.rs:119-120` renders it through `TemplateVars`. | **Yes, row 12** (AP-2, TN-M2). Latent: no shipped data uses a placeholder. |
| 2 | The story log keeps the 50 oldest Messages | **Known.** `get_story_log_entries` reads `STORY_LIMIT` (`view_query.rs:174`), which is `i32::MAX` (`stories.rs:19`). The read is `ORDER BY m.id DESC`, then `entries.reverse()` (`stories.rs:172`, `214`). So the list is the whole story, oldest first. `.take(50)` (`app_state.rs:107`) keeps the oldest 50. **Inferred:** past 50 Messages the newest Messages do not show, and the `loop.last` controls (delete, Retrigger, swipe) land on Message 50. No tier-1 test pins the log end of a long story (known from a search of `tests/http`). | **Not a numbered row.** "Rejected or downgraded" lists it as TN-L4 with the reason "not a regression" (the code at `bc7d6cd4` did the same). It is still a defect. |
| 3 | The Retrigger button disagrees with `Tail::retrigger_allowed` for an Impersonate Input | **Known.** `Tail::retrigger_allowed` (`tail.rs:98-107`) allows `Narration` and `ImpersonateInput`. An Impersonate Input has `MessageType::Input` (`generation_record.rs:24`), so `log_type` is `"input"` (`view_models.rs:135-139`), and `NarrativeLogTemplate::new` sets `show_retrigger` to false (`templates.rs:36-38`). **Inferred, the reachable state:** an Impersonate generation runs the analysis (`job.rs:265`: only a user regen skips it) and can set `trigger_context` (`job.rs:501`). If the event generation that follows fails, the last Message is the Impersonate Input with a Trigger context. The server then accepts a Retrigger, but the log shows no button. **Which side is right is open (inferred):** spec scenario 14.3 (`docs/specs/retrigger.md:45`) says a Retrigger on "an Input message (not a Narration)" answers 400, which agrees with the template. `tail_tests.rs` pins "allowed" (ticket 07, row 11). | **No.** No FINDINGS row and no full-review text names it. |

Ticket 18 tests bugs 2 and 3 in the running UI.

## 5. What each run saw that the others did not

These are facts about the three texts, not a ranking. The **Opus** run gave 8 candidates. It is the only run with the three possible bugs of §4, the counts of reads per generation, and the plan rules in `gate.rs`, the preset policies and the composition root (S1, S2, S4). It is the only run that proposed to delete the in-memory adapter instead of splitting it. The **GLM** run gave 5 candidates. It is the only run with the hook-input assembly (S5). It named the room lookup as a candidate of its own and said that the `gate.rs` copy already drops the added-room branch. It put an `arch-lint.toml` note on each card that touches a rule. The **Qwen** run gave 5 candidates. It is the only run that named the `&'static str` keys of read failures against the `TxWrite` enum, the fields the analysis merge drops, the unwritten `quantifier_confidence` field, the English continuation text in the Job, and the guardrail (`check_http_storage_leak`) behind the catalogues. No run named findings 2, 3 or 6. No run noted that `WorldBundle` is a storage type that the "model" and `agent-edge` rules keep out of M1 and M3.

## 6. Recommended order

| Order | Candidate | Reason |
|---|---|---|
| 1 | M4 analysis output type | It is the smallest change, in-process, and it deletes code. It also fixes the input type that M1 takes, so M1 does not change twice. |
| 2 | M1 Game rules | It is pure `domain/model/` code, so it gets unit tests. It removes the core of finding 4 and finding 12. M3 and S1 build on it. |
| 3 | S1 plan rules to the `Tail`, with finding 3 | It removes finding 5 and the cause of finding 1. The data-carrying kind (finding 3) lands at the same place. |
| 4 | M3 narration prompt rules (with S5) | It needs the event record of M1 and the order of S1. It removes finding 19 and gives the narrator contract its user-message `CASE`s. |
| 5 | S3 story window | It closes two UI defects (§4, bugs 2 and 3) once ticket 18 confirms them. It does not depend on 1 to 4, so it can go earlier as a bug fix. |
| 6 | S2 preset module | It removes finding 15 and the inferred dangling-preset gap. It is independent of 1 to 5. |
| 7 | M2 storage | It needs the owner call of finding 7 (delete or split) first. It touches 32 test files and a `tests/STRATEGY.md` row. |
| 8 | M5 story projections | It is low risk but a low gain, and serde and `agent-edge` may keep a type. |
| 9 | M6 catalogues | The guardrail decides most of their shape. Both runs rate the gain low or unsure. |
| 10 | S4 composition root | It is partly in place already (`app_with_production_graph`). What remains is finding 14. |

Findings 2 and 6 have no candidate. Ticket 13 must place them in this order itself.
