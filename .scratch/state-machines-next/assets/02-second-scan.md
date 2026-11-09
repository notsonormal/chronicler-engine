# Ticket 02: the second scan compared with the catalogue

Made on 2026-11-09. Scan 1 is the catalogue ([`catalogue.md`](catalogue.md), five DeepSeek scouts, 2026-11-08). Scan 2 is the five reports `scan2-*.md` (five Claude Sonnet scouts, 2026-11-09, the same [brief](scan-brief.md), no access to scan 1). Both scans cite commit `8b23151c`. The working tree is at `8eb0b057`. Every file:line below is checked in the working tree.

Epistemic labels: **known** = the code path was read. **inferred** = reasoned from the code, not run. **unsure** = not settled.

## Summary

| Count | Value |
|---|---|
| Machines in scan 1 (catalogue rows A1–G15) | 54 rows. Row A7 is a part of B1 and B2, so 53 machines. 42 are in scope. |
| Sections in scan 2 | 86: domain 15, application 20 (5 "already modeled" + 15 new), driven 17, driving 18, frontend 16. After the overlaps merge, they fill 63 lines of the match table. |
| Lines of the match table | 64 |
| `both` | 53 (every catalogue row except B8) |
| `only scan 1` | 1 (B8) |
| `only scan 2` | 10: 1 missed, 4 duplicates, 5 not a machine |
| Real machines that scan 1 missed | 1: the Game config state, now row B12 |
| Real machines that scan 2 missed | 1: B8 Swipe text against Swipe state (inferred: a weak machine) |
| Scan-1 claims refuted | 2: E3 World seeding (found by the parent session), G8 "the Test button becomes the opener" |
| Scan-2 claims refuted | 5 (one in part): the overflow guard ends as `ContextOverflow`; the edit dialog sends twice; a missing state row makes the Job report `MissingData`; Impersonate health under the Narrator contradicts the glossary; the boot touches `updated_at` a second time (true only on the create path, and harmless) |
| Bug tickets filed | 5: `state-machines-work` 03 to 07 |
| Behaviour questions added | 10: ticket 03 five, ticket 04 three, ticket 05 two |
| "To check" items added | 15: ticket 03 six, ticket 04 four, ticket 05 five |
| Doc mismatches added to ticket 05 | 5 (scan 2 D1–D5) |

The result: the catalogue is nearly complete. The two scans agree on 53 of 54 rows. Scan 2 adds one machine and many risks. Most new risks are behaviour questions, not defects.

## 1. Match table

Two machines match when they share their state variables. A scan-2 section that covers two rows shows in both lines.

| Catalogue row | Scan 2 report and section | Result |
|---|---|---|
| A1 Busy registry | application *A1. Busy registry and holder* | both |
| A2 Job life | application *A2. Job life* | both |
| A3 Autoplay run | application *A3. One Autoplay run* | both |
| A4 Active-Game pointer | application *A4. Active-Game pointer* | both |
| A5 Two writes of a Swipe state | application *A5. The two writes of a new Swipe state* | both |
| A6 Plan of one request | domain *Generation order and plan* | both |
| A7 Failure line + Options failure | domain *Generation status and the failure line*; application *1.* | both |
| B1 Game status | application *1. Game failure line plus dock record*, *2. Status rule*; driven *Failure line (`games.failure`) and derived Status*; domain *Generation status and the failure line* | both |
| B2 Dock record | domain *Dock record*; application *1.*; driven *Dock record (`game_docks` row)*; driving *10. Options dock content* | both |
| B3 Trigger fire-once | domain *Trigger lifecycle* | both |
| B4 Encounters | domain *Encounter* | both |
| B5 Story end (`Tail`) | domain *The story end (Tail) and the opening*; driven *Story tail read (`read_tail`)* | both |
| B6 Message type × kind × Swipe flags | domain *Message kind and Swipe flags*; application *13. Story shape* | both |
| B7 Active Swipe | domain *Message hydration and the active Swipe index*; driven *Message active Swipe* | both |
| B8 Swipe text against Swipe state | — | only scan 1 |
| B9 Movement, added Rooms | domain *Dynamic Rooms* | both |
| B10 Role health | application *9. Role health*; driven *LLM message log and role health*; driving *11. Role health* | both |
| B11 Change counters | domain *Counters of the changes*; driven *Change counters* | both |
| C1 Quantifier reading | application *3. Quantifier retry loop* | both |
| C2 Options attempt loop | application *4. Options agent retry* | both |
| C3 Prompt call mode | application *12. `PromptContext`* | both |
| C4 History cut, overflow guard | domain *History cut*; application *11. Prompt assembly* | both |
| C5 Agent registration | application *5. `AgentRunner` slots* | both |
| D1 Connection slot | domain *Connection slots* | both |
| D2 Text check pair | domain *Text-check settings pair*; driving *14. Form modes* (Text check) | both |
| D3 Mode preset registry | domain *Game config write kinds and mode-bound presets* (`ModePresetRegistry`) | both |
| D4 Preset Allowed Modes | driving *14. Form modes* (Preset) | both |
| D5 World form | driving *14. Form modes* (World, Game posture) | both |
| D6 Connection form | driving *14. Form modes* (Connection) | both |
| E1 Migration ladder | driven *Schema version ladder* | both |
| E2 Transaction (`Tx`) | driven *Unit of record writes* | both |
| E3 Boot seeding precedence | driven *Seed files vs database*, *Settings row* | both |
| E4 Boot resume or create | driven *Boot sequence and what a restart leaves* | both |
| E5 LLM message retention | driven *LLM message log and role health* | both |
| F1 Test storage | driven *Storage test failure layer* | both |
| F2 `MockBackend` | driven *Mock LLM provider* | both |
| F3 `AutoplayPause` | application *6. Pause point* | both |
| F4 `JobTasks` | application *7. `JobTasks` watch* | both |
| F5 Harper cache | driven *Harper dictionary cache* | both |
| G1 Action area | driving *3. Action area*; frontend *Action area* | both |
| G2 Screen stamp and waiters | driving *1. Screen sync protocol*; frontend *The ask loop and the screen stamp* | both |
| G3 Ask and load loop | driving *1.*; frontend *The ask loop and the screen stamp* | both |
| G4 What a load keeps | driving *2. Request-in-flight registry*; frontend *Request in-flight tracking* | both |
| G5 The Game a request names | driving *4. Draft Game pin*; frontend *Command draft pin* | both |
| G6 Story-log edit | driving *6. Story-log entry edit mode*; frontend *Story-log edit mode and its lock* | both |
| G7 Story-log scroll follow | driving *18. Story-log follow-scroll*; frontend *Story-log follow-the-bottom scroll* | both |
| G8 Edit dialog session | driving *5. Edit dialog lifecycle*; frontend *Edit dialog* | both |
| G9 Notice place | driving *8. Notice place, …, inline slots*; frontend *The notice place*, *Inline slots and error popovers* | both |
| G10 Failure banner | driving *8.*; frontend *Failure banner* | both |
| G11 Saved-on-change control | driving *13. Autosave controls*; frontend *Saving controls* | both |
| G12 Tab, sub-tab, Games page | driving *12. Saved-games pager*, *15. Tabs*; frontend *Tabs, Settings sub-tab, Games page*, *Pager of saved Games* | both |
| G13 Slash menu | driving *16. Slash menu*; frontend *Slash palette* | both |
| G14 Error detail popover | driving *8.*; frontend *Inline slots and error popovers* | both |
| G15 Live regions | driving *17. Live-region announcement baselines*; frontend *Live regions* | both |
| — (new B12) | domain *Game config write kinds and mode-bound presets*; application *8. Game config writes* | only scan 2 |
| — | application *14. Autoplay `Budget` and `AutoplayProgress` counters* | only scan 2 |
| — | application *15. Catalogue create / reset* | only scan 2 |
| — | driven *"Newest Game" ordering: two definitions* | only scan 2 |
| — | driving *9. Busy state to controls*; frontend *Generation gate, busy state, Job, Autoplay run (as shown in the browser)* | only scan 2 |
| — | domain *Story window room headers* | only scan 2 |
| — | application *10. Provider resolution on each call* | only scan 2 |
| — | driven *LLM transport and providers* | only scan 2 |
| — | driven *Storage calls that write in two statements with no transaction* | only scan 2 |
| — | driving *7. Answer outcome classification* | only scan 2 |

## 2. Labels of the machines that only one scan has

| Machine | Label | Evidence |
|---|---|---|
| B8 Swipe text against Swipe state (only scan 1) | **missed** by scan 2 (inferred: a weak machine) | `Swipe` holds `text` and `state` side by side (`src/domain/model/message.rs:26-37`). `GameWriter::edit` writes the text only (`src/application/generation/gate.rs:533-545`, `tx.set_swipe_text`). So a Swipe moves from "the state matches the text" to "the text changed after the analysis", and no field records the move. Scan 2 saw `edit` only as a writer of the failure line. |
| Game config state (only scan 2) | **missed** by scan 1. Added as **B12**. | The mode is a phase that changes over time and decides which presets are valid. `ConfigChange` names the moves (`src/domain/model/game.rs`). The rule "the preset allows the mode" is checked in the catalogue, outside the hold (`src/application/games/catalogue.rs`, `set_preset_selection`). `save_config` refuses only an Autoplay holder. The writes are in `transaction.rs:252-292`. The catalogue had only D3, the registry of default bundles, not the mode of each Game. |
| Autoplay `Budget` and `AutoplayProgress` | **duplicate** of A3 | `AutoplayProgress { done, budget }` lives in the gate holder of an Autoplay run (`src/application/generation/gate.rs:108-117`), and the loop of A3 reads it (`src/application/autoplay/driver.rs`). It is the cycle count of A3. |
| Catalogue create / reset | **duplicate** of A1 and A4 | `replace` and `create_game` move the active-Game pointer (A4) under the hold of A1. The order of the two writes is the gate scope of `verify-gate`. |
| "Newest Game" ordering | **duplicate** of E4 | `newest_game_of_world` (`src/adapters/driven/storage/games.rs:135-160`) is the read of the E4 boot pick. The list order (`games.rs:18`, `:48`) is a sort key, not a state. |
| Busy state to controls (driving 9, frontend gate section) | **duplicate**: a view of A1 | The catalogue *Not counted* section already says this. The parts read the busy registry and render it. They hold no state of their own (`src/adapters/driving/http/layout/handlers/action_controls.rs`). |
| Story window room headers | **not a machine** | A fold over the entries of one read. It keeps no value between two reads (`src/domain/model/story_window.rs`). |
| Provider resolution on each call | **not a machine** | `LlmCallRecorder::provider` resolves the Connection from the settings row on each call (`src/application/llm_recorder.rs:41-43`). No value is kept. |
| LLM transport and providers | **not a machine** | A new client for each call, no retry, no cache (`src/adapters/driven/llm/transport/utils/client.rs`). |
| Two-statement writes with no transaction | **not a machine** | Two defects of single calls, not a mode. One is bug ticket 04 (`worlds.rs:297-335`). The other goes to question 12 of ticket 03 (`llm_messages.rs:14-46`). |
| Answer outcome classification | **not a machine** | A status-code mapping. The catalogue *Not counted* section already says this (`utils/answer_words.rs`). |

## 3. Risk checks

### 3.1 The three inferred risks with high impact

| Risk | Result | Evidence | Where it went |
|---|---|---|---|
| (a) The edit dialog sends a write twice before the first answer (scan 2 frontend, *Edit dialog*) | **refuted** (known for the script; inferred for htmx) | `assets/dashboard-dialog.js:107-113` cancels a dialog submit in `htmx:confirm` while the dialog holds `data-saving`. In htmx 1.9.10, `htmx:confirm` fires before the queue check, so a repeat submit that waits behind the first runs the check again after the first answer set `data-saving` (`:115-121`). A repeat after an answer that is not `done` sends again, which is the wanted retry. | nowhere: no risk |
| (b) `status` reads a busy Game as Idle between its two reads (scan 2 application, finding 2) | **confirmed** (known) | `ViewQuery::status` reads the registry (`src/application/games/view_query.rs:199-201`) and then the Game row (`:202-210`). An accept between the two clears the failure line, so a Game that had a failure line reads Idle, a state that never held. The accept raises the busy-state number, so the next load corrects it. | ticket 03, question 8 |
| (c) An element removed during its request never releases its in-flight hold (scan 2 driving, finding 2) | **confirmed** in mechanism (known for the script; inferred for htmx 1.9.10) | `endRequest` listens on `document.body` only (`assets/dashboard-screen.js:101-102`). htmx fires `htmx:afterRequest` on the element and re-fires it only on ancestors that are still in the body. The dialog close handler empties the body (`dashboard-dialog.js:87-93`), and `leaveSection` empties a part (`dashboard-screen.js:400-404`). The `requestsInFlight` entry of the detached node stays (`:72-82`). No later load meets that node, so the real cost is that the answer is not read: a save that fails after a close shows no report. | ticket 05, question 4 |

### 3.2 Conflicts between the two scans

| Conflict | Result | Evidence |
|---|---|---|
| C4: scan 2 says an incomplete overflow guard ends as `ContextOverflow`. Scan 1 says the answer budget clamps to 1. | scan 1 **confirmed**, scan 2 **refuted** (known) | `TokenBudget::response_tokens` (`src/application/prompting/token_budget.rs`) clamps with `.min` against `saturating_sub` and then `.max(1)`. `ContextOverflow` needs the system prompt alone to overflow. Bug ticket 02 of `state-machines-work` stands. |
| B5: scan 2 says a missing state row makes the Job report `MissingData`. Scan 1 says a Retry starts from the wrong state with no error. | scan 1 **confirmed**, scan 2 **refuted** (known) | `read_tail` maps a missing row to `state_before_last = None` (`src/adapters/driven/storage/stories.rs:80-89`). `Tail::start_at` then starts from Swipe 0 of the last Message (`src/domain/model/tail.rs:37-45`), and `redo` treats a plain Narration as the opening (`src/domain/model/generation.rs:330`). No `MissingData` is made on this path. Reachable only with a bad `active_swipe_index`. Ticket 03 "To check". |
| G8: scan 1 says the Test button becomes the dialog opener. Scan 2 says no dialog control targets the body today. | scan 2 **confirmed**, scan 1 **refuted** (known) | The Test button posts into `next .connection-test-slot` (`src/adapters/driving/http/settings/templates/settings.rs:199-202`). The card button `connection-test-open-{id}` (`settings.rs:60`) is a real opener. Catalogue G8 marked; ticket 05 question 3 annotated. |
| E3: scan 1 said the seed upserts a World. Scan 2 says the database wins. | scan 2 **confirmed** (by the parent session before this ticket) | `src/bootstrap/load.rs:115-125`. Catalogue E3 already marked. |
| E4: scan 2 driven says `GameCatalogue::new` touches `updated_at` a second time at boot. | **refuted in part** (known) | `resolve_boot_game` builds its catalogue with no initial Game, so it does not touch. On the create path, `create_game` uses `PointerHold::set_new`, which skips the touch, and the later `set` touches once more. Two writes on create, harmless. Catalogue E4 stays refuted. |
| B10: scan 2 says Impersonate health under the Narrator contradicts the glossary. | the fact is **confirmed**, the contradiction is **refuted** (known) | `narrator.rs:115`, `:136-140` use the Role of the generation kind (`src/domain/model/agent.rs:66-71`). `docs/diataxis/reference/narrative/narration_system.md:18` says: "The health of an Impersonate shows under the Narrator Role, because Impersonate has no Role." Intended. |
| B1: scan 1 (scout) and scan 2 (known) say the failure line survives a restart. | **confirmed**, intended (known) | The column is stored; the registry is memory only. `docs/diataxis/reference/game_flow.md:99` says a failure line "stays until the player acts". Nowhere. |

### 3.3 New risks that scan 2 labels known

Only risks that the catalogue does not have. "Out of scope" means rows A1–A7 or group F: compared, not filed.

| Risk (scan 2 section) | Result | Evidence | Where it went |
|---|---|---|---|
| `times_met` increments on every entry; the doc comment says first encounter only (domain *Encounter*) | confirmed | `trigger.rs:39` against `game_rules.rs:147-155` | **bug 03** |
| `update_world` writes the World and the map with no transaction (driven *two statements*) | confirmed | `worlds.rs:297-335`; the rule at `:174` | **bug 04** |
| A World with a bad map file stops the seed loop (driven *Seed files vs database*) | confirmed, wider than scan 2 said: the Personas are skipped too | `load.rs:34`, `:87`, `:122`; `startup.md:26` | **bug 05** |
| The changes spec and a comment say the Games list orders by the newest Message (driven *"Newest Game"*) | confirmed | `changes.md:33`, `changes.rs:180-181` against `games.rs:18`, `games.md:325` | **bug 06**; the choice of rule: ticket 04, question 7 |
| A preview send names the screen Game after the input is emptied (frontend *Action area*) | confirmed | `dashboard-action-area.js:234`, `dashboard-screen.js:415-427` against `dashboard.md:37` | **bug 07** |
| Two matching Triggers: `HashMap` order decides (domain *Trigger lifecycle*) | confirmed | `game_rules.rs:87-90`, `:166-173` | ticket 03, question 9 |
| The Quantifier cannot name an added Room, so a return makes a new one (domain *Dynamic Rooms*) | confirmed | `orchestration.rs:35-47`, `game_rules.rs:124-145` | ticket 03, question 10 |
| An empty Narration and an unparseable Quantifier answer read as healthy (application *9*) | confirmed | `llm_recorder.rs:85`, `narrator.rs:151-156` | ticket 03, question 11 |
| A failed log save fails a good call, but not a failed call (application *9*; driven *two statements*) | confirmed | `llm_recorder.rs:72-76`, `:85`; `llm_messages.rs:46` | ticket 03, question 12 |
| No guard for a database from a newer binary (driven *Schema version ladder*) | confirmed | `plumbing.rs:17`, `:51-54` | ticket 04, question 5 |
| `--persona` is checked but a resumed Game keeps its own Persona (driven *Boot sequence*) | confirmed | `run.rs:86-88`, `init_game.rs:13-22` | ticket 04, question 6 |
| Live regions announce a whole story window after a Game switch (frontend *Live regions*) | confirmed | `dashboard-action-area.js:140-160`, `:171-187` | ticket 05, question 5 |
| The active Swipe index is read four ways (driven *Message active Swipe*) | confirmed; extends B7 | `stories.rs:18-24` (`COALESCE`), `:150-167` (`has_text`), `:231-237` (window fails), `mappers/message.rs:72` (repair to 0) | nowhere new: B7 in ticket 03 holds it |
| `accept` clears the failure line but not the Options failure (application *1*) | confirmed | `gate.rs:700-704` | out of scope: A7 (`verify-gate` `optFail`) |
| A failed failure write leaves a failed Job with no line (application *A2*) | confirmed by scan 2; not checked again | `job.rs` `end`, the `Err(refused)` arm | out of scope: A2. The parent may want to tell the `verify-gate` owner. |
| The Job loop has no counter; the Autoplay loop keeps a second copy of `done` (application *A2*, *A3*) | not checked | — | out of scope: A2, A3 |
| `replace` writes the new Game before the delete, in two transactions (application *15*) | not checked | — | out of scope: A1, A4 |
| Pause entry stays armed; `running()` reads 0 during a wait; `MockBackend` counter can wrap (application *6*, *7*; driven *Mock*) | not checked | — | out of scope: group F |
| `get_debug_state` reads in separate steps (application *2*) | confirmed | `view_query.rs:121-157` | nowhere: a debug view only |
| `max_attempts = 2` is a literal; `Ok(None)` has two meanings (application *3*, *4*) | confirmed | Quantifier orchestration; `runner.rs:13-17` | nowhere: code clarity, no effect |
| `Narrator::backend_info` resolves the provider again (application *10*) | confirmed | `narrator.rs:87-92` | nowhere: a label of the debug view |
| `render_persona_layer` drops the Persona for Impersonate (application *12*) | confirmed | `assembler.rs:415` | nowhere: intended by the Impersonate rule |
| `MessageType::System` is never made; `trigger_fired` keeps a `bool` that no code reads (domain) | confirmed | `rg MessageType::System`; `trigger.rs` | nowhere: dead values, no effect; ticket 03 sees them through B3 and B6 |
| `/action/check` answers a preview or a dispatch under one status (driving *3*) | confirmed | `actions.rs:36`, `dashboard-action-area.js:270-279` (`onCommandAfterRequest`) | nowhere: the doc states the two answers (`dashboard.md:16`) |
| An empty `data-game` sends no Game header (driving *4*) | confirmed | `dashboard-screen.js:420-427` | nowhere: the doc states it (`dashboard.md:135`) |
| A failed or unreachable Save reverts the entry in edit (driving *6*) | confirmed | `dashboard-story-log.js` `revertEdit` | nowhere: the doc states it (`dashboard.md:103`) |
| The Save of an entry in edit looks disabled but sends, and gets `409` (driving *6*) | confirmed | CSS `:has(...)` | nowhere: the doc states it (`dashboard.md:103`) |
| `editOption` does not check `actionIsGenerating()` (frontend *Action area*) | confirmed | `dashboard-action-area.js:326-334` | nowhere: it fills the input only and sends nothing |
| `/screen` has no row in the answer-word table (driving *7*) | confirmed by scan 2; not checked again | `utils/answer_words.rs` | nowhere: not a state machine |
| The settings save writes `created_at` again (driven *Settings row*) | confirmed | `settings.rs:69-80` | ticket 04 "To check": no reader found |

The other risks that scan 2 labels inferred or unsure went to the "To check (from scan 2)" sections of tickets 03, 04 and 05, without a proof.

## 4. Doc mismatches

Scan 2 frontend D1 to D5 are not in the catalogue's list. All five went to "Also check" of ticket 05, not verified. D6 and D7 report agreement.

## Answer

Draft for ticket 02:

> The second scan (five Claude Sonnet scouts, 2026-11-09) agrees with the catalogue on 53 of its 54 rows. It found one machine that the catalogue missed: the Game config state (narrator mode and the mode-bound preset slots), now row B12 in ticket 03. It missed one catalogue row, B8 (Swipe text against Swipe state), which is a weak machine. Its other nine extra sections are duplicates (4) or not machines (5).
> Ticket 02 refuted two scan-1 claims (E3 World seeding, the G8 Test-button opener) and five scan-2 claims (the overflow guard ends as `ContextOverflow`, the edit dialog sends twice, a missing state row reports `MissingData`, Impersonate health contradicts the glossary, a second `updated_at` touch at boot in part).
> Of the three high-impact inferred risks, the double send is refuted, and the status read and the detached in-flight hold are confirmed as behaviour questions.
> Five plain defects went to `state-machines-work` (03 to 07). Ten behaviour questions and fifteen "to check" items went to the apply tickets 03, 04 and 05. Five doc mismatches went to "Also check" of ticket 05.
> Details: [`assets/02-second-scan.md`](../assets/02-second-scan.md).
