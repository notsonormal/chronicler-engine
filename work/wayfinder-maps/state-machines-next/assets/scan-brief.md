# The brief of the first scan

The five scouts of 2026-11-08 got this brief. Each one also got one area line (end of this file) and `CONTEXT.md`. A second scan (ticket 02) uses the same brief, so the two lists can be compared. Do not give the second scan the catalogue.

## The brief

You are a read-only scout in the Rust repo /workspace/chronicler-engine (an interactive fiction engine: Rust server, HTMX dashboard, LLM calls, SQLite). Do NOT edit, create or delete any file. Use rg (not grep at the repo root; repo-root grep is denied because target/ is 10 GB). Read CONTEXT.md first for the domain terms (Game, Job, Message, Swipe, Swipe state, Autoplay run, Cycle, Busy state, Step, Options, Trigger, Connection, LLM message). Skip *_tests.rs files except to confirm a behaviour.

GOAL: find every IMPLICIT STATE MACHINE in your assigned area.

DEFINITION. An implicit state machine is a thing whose behaviour depends on a mode or phase that changes over time, where the states and/or the transitions are not written down as one explicit enum plus one transition function. Count it when ANY of these signals is present:

1. Two or more booleans / Option fields / nullable columns that together encode a mode (some combinations are invalid or meaningless).
2. An enum that IS explicit, but whose transitions are spread over several functions or modules with no single place that says which move is legal.
3. A string, integer or counter used as a status or phase (e.g. "pending", 0/1/2, retry counts, budget counters).
4. Presence/absence of a value (Option, nullable column, empty Vec, missing row, file exists) that means "we are in phase X".
5. A required call order (A must happen before B; B after C is a bug), lifecycles (created -> started -> ended), Drop-based ends, guards/holds/locks that are taken and released.
6. Retry / fallback / backoff loops, health tracking (healthy -> failing -> recovered), first-time vs. later behaviour.
7. UI modes: CSS classes, data-* attributes, hidden/disabled flags, module-level JS variables, open/closed dialogs, in-flight request flags.

Do NOT report plain data structs with no time-varying mode, or pure functions with no state.

ALREADY MODELED (still list them if they are in your area, but mark them "already modeled" and spend little time on them): the generation gate / busy registry (src/application/generation/gate.rs, GameWriter, JobWriter, BusyRegistry::hold), the life of one Job (src/application/pipeline/job.rs: steps, failure line, Stop, Options run), one Autoplay run (src/application/autoplay/driver.rs), the active-Game pointer (src/application/games/active.rs), and the two writes of a new Swipe state (pre-analysis write, analysed write). Look hardest for the ones NOT in this list.

METHOD. Read every non-test .rs/.js file in your area. Search for: enum, Option<, bool fields, "status", "state", "phase", "mode", "step", "pending", "busy", "active", "failed", "retry", "attempt", "count", "Mutex", "RwLock", "Atomic", "Drop", "take()", "replace(", "is_some", "is_none", classList, dataset, hidden, disabled, let/var at module level. Follow each candidate to every place that changes it.

OUTPUT (your final message; plain markdown, no preamble). For each finding:

```
### <short name>
- Where: file:line of the state variables (and the type name)
- State variables: the fields/flags/columns/values that hold the state
- States: the list of states, including states that are only implied (name them)
- Transitions: event -> from -> to, each with file:line of the code that does it
- Guards / invariants: what must be true; which combinations of the variables are invalid or should be impossible
- Explicitness: one of FULLY IMPLICIT (no enum, flags only) / EXPLICIT STATES, SCATTERED TRANSITIONS / EXPLICIT (enum + one transition place)
- Already modeled: yes/no
- Risks seen: concrete gaps you saw (an invalid combination the code can reach, a missing transition, a transition in two places that disagree, a Drop that can skip an end). Write "none seen" when none. Label each risk known (you read the code path) or inferred.
```

End with a one-line table of contents: name | file | explicitness. Be precise with file:line. Do not invent; if unsure, say "unsure".

## The five area lines

1. **Domain.** src/domain/ (all subfolders). Pay special attention to src/domain/model/state/, tail.rs, generation.rs, dock.rs, message.rs, trigger.rs, history_cut.rs, story_window.rs, utils/game_rules.rs, settings.rs, game.rs.
2. **Application.** src/application/ (all subfolders). The gate, job.rs, driver.rs and active.rs are already modeled: map them only briefly. Look hard at agents/ (runner, options, quantifier orchestration and parsers), narration/, prompting/ (assembler, token_budget), games/catalogue.rs, games/view_query.rs, generation/job_tasks.rs, autoplay/budget.rs and pause.rs, llm_recorder.rs, connection_test_service.rs, settings_service.rs, text_check_service.rs, and the Role health / Connection health if it lives here.
3. **Driven adapters and bootstrap.** src/adapters/driven/ (storage, llm, text_check, utils) and src/bootstrap/. In storage, read the schema/migrations in storage/utils/plumbing.rs and every table: find columns that act as status (nullable columns, flags, counters, 'active' markers such as the active Swipe, the dock record, the change counters in changes.rs, the write guard and injected failures in write_guard.rs, transaction.rs). In llm, look for retry/backoff, streaming, response parsing phases, mock provider scripted state. In bootstrap, look for startup phases, init_game resolution, seeding order, and what happens on restart (busy state left in the DB, half-done writes).
4. **Driving adapter.** src/adapters/driving/ (the HTTP adapter: handlers, app_state, request_layers, builders, view_models, templates). Look for: request-level state (CurrentGame, revision stamp, Game header check), which UI controls are enabled/disabled from the busy state (layout/handlers/action_controls.rs, endpoints.rs), the changes answer / polling protocol, failure banners and when they clear, the saved-games pages, settings and Role health cells, the screen parts loader, form states (edit mode of a preset or World), and any state held in AppState (Mutex, RwLock, Atomic). Also read the askama templates the handlers use for conditional rendering that encodes a mode.
5. **Browser.** assets/dashboard-*.js, assets/index.html, and the HTML templates the server renders. Skip htmx.min.js and idiomorph-ext.min.js. Look for: module-level variables, CSS classes toggled as modes, data-* attributes, hx-trigger polling and when it stops, dialogs open/closed, the slash menu, the story log scroll/follow mode, the action area, saved-games paging, panels, error banner shown/dismissed, in-flight request flags, retries after a failed fetch, and how the client reacts to the server's change counters / revision stamp. Also read docs/diataxis/reference/frontend/dashboard.md and report any mismatch between the doc and the code.
