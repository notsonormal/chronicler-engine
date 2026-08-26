# Slash-command parser for steering entry

Type: task
Status: resolved

## Question

Replace the `Action::FreeAction` stub with a slash-command parser recognizing `/narrator`, `/impersonate`, `/guide`.

Per the design synthesis (`../research/04-design-synthesis.md`, Q10):

1. Today `Action` (`src/domain/model/action.rs`) has one variant `FreeAction(String)` and `Action::parse` copies input verbatim. `FreeAction` is a leftover from removed non-LLM movement — it carries no weight and is replaced, not extended-by-habit.
2. Add a parser that recognizes `/narrator <text>`, `/impersonate [text]`, `/guide <text>` prefixes in the single `command` field of `ActionForm` (`src/adapters/driving/http/action/handlers/actions.rs`). One input box, one form field, one endpoint — the slash-command convention across ST, Marinara, and GG.
3. Dispatch each command to its feature's pipeline path. Plain input (no slash) keeps the existing `process_action` / `continue_narration` dispatch.
4. The parser is the entry point for tickets 07 (guide), 08-specs, 09 (impersonate), and the narrator generate path (ticket 10). Keep the dispatch targets decoupled so each feature ticket can land independently.

## Answer

Replaced the `Action::FreeAction` stub with a typed semantic-command enum and a slash-command parser; wired the single HTTP dispatch point to route by variant; added three decoupled pipeline seam methods for the feature tickets to fill.

**Parser (`src/domain/model/action.rs`).** `Action` is now `{ FreeAction(String), Guide(String), Narrator(String), Impersonate(Option<String>) }`. `Action::parse` recognizes a leading `/` (after `trim_start`), splits the command word from its argument at the first whitespace, matches the command case-insensitively, and trims the argument. Recognized: `/guide <text>`, `/narrator <text>`, `/impersonate [text]` (empty argument → `Impersonate(None)`). Plain input and unrecognized slash input (e.g. `/shrug`) fall through to `FreeAction(input)` verbatim, preserving the original string — so existing behavior for any non-steering input is unchanged. The provisional unknown-slash fallback (→ plain player action) is exactly ticket 14 Q3's open question and flips if the grill decides otherwise.

**Dispatch (`src/adapters/driving/http/action/handlers/actions.rs`).** `dispatch_action` now calls `Action::parse(&command)` and matches: `FreeAction` keeps the existing empty→`continue_narration` / non-empty→`process_action` split; `Guide`/`Narrator`/`Impersonate` route to the new seam methods. This is the only entry point — `action_handler`, `action_confirm_handler`, and `action_check_handler` all go through `dispatch_action`. The text-check in `action_check_handler` still runs on the raw `command` string (slash prefix included); whether recognized slash commands should bypass the player-input check is a new open question, logged as ticket 14 Q7.

**Decoupled seams (`src/application/pipeline/action_pipeline/action.rs`).** Added `guide_narration(gate, guide: String)`, `narrator_action(gate, text: String)`, `impersonate(gate, direction: Option<String>)` on `ActionPipeline`, each returning `Result<ProcessActionResult, EngineError>`. Each stub currently delegates to `continue_narration` (the shared no-player-input generation shape) and carries its steering text in the signature ready to consume. The one-line doc comment on each names where the steering actually applies (guide = final prompt layer during assembly; narrator = persisted history entry before generation; impersonate = preset + persona swap during assembly) — the WHY the entry stub delegates. Tickets 08/09/10 fill the bodies; because each owns a distinct method, they land independently without editing the shared dispatch match.

**Tests.** `action_tests.rs`: added cases for each command, empty-argument forms, case-insensitivity, internal-spacing preservation, unknown-slash verbatim fallback, and leading-whitespace-before-slash. Existing `FreeAction` tests pass unchanged (all non-slash inputs still map verbatim). `actions_tests.rs`: added HTTP dispatch tests asserting `/guide`, `/narrator`, `/impersonate` each return 200 + `Thinking` (exercising the seam's continue delegation).

**Verification.** `cargo fmt` clean; `cargo clippy --all-targets -- -D warnings` clean; `cargo test --lib` 998 passed; `python build.py` green (all 12 steps, full integration suite, 2 LLM tests skipped as standard — no `narrative_prompt/` or `driven/llm/` files touched, so `--llm-only` not required).

**Deferred.** (a) Text-check behavior for recognized slash commands → ticket 14 Q7. (b) Actual steering application (guide layer, narrator persist+generate, impersonate preset) → tickets 08/09/10, which unblock from this one.