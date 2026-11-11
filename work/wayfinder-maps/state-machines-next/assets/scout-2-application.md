### Role health (the health of the four Roles)
- Where: `src/application/games/view_query.rs:24-29` (`RoleHealth`), `:249-270` (`role_health()`)
- State variables: the `llm_messages` table. `RoleHealth { role, backend_model: Option<String>, last_error: Option<String> }` is derived: for each `Role::ALL` member, the message with the largest `(created_at, id)` whose `agent_name == role.agent_name()`.
- States: *never called* (`backend_model` is `None`), *healthy* (newest row has `error_message` `None`), *failing* (newest row has `error_message` `Some`). Recovery is not a transition of its own: a healthy row simply becomes newest.
- Transitions: one row per model call. A success writes `error_message: None` (`src/application/ports/llm_provider.rs:51-65`, saved at `src/application/llm_recorder.rs:83-86`); a transport failure writes `error_message: Some(...)` (`src/application/llm_recorder.rs:61-78`, `:92-113`). A retried attempt writes its own row, so the newest row is the last attempt (`agents/quantifier/utils/orchestration.rs:97-160`, `agents/options/utils/orchestration.rs:53-67`).
- Guards / invariants: the health of a Role moves only through `LlmCallRecorder::complete`; `ConnectionTestService` must bypass it, else a probe would move the health (`src/application/connection_test_service.rs:1-24`, `:62-85`). `connection_test` and `dialogue` are not Role names, so their rows never take part (`src/application/ports/llm_provider.rs:10-14`).
- Explicitness: FULLY IMPLICIT (derived from a nullable column; no enum, no named state)
- Already modeled: no
- Risks seen:
  - known: the retention window is **50 rows for the whole table** (`src/adapters/driven/storage/llm_messages.rs:11`, `:40-46`), yet the read assumes it holds "every role's newest attempt however long ago it ran" (`view_query.rs:250-252`). A busy Narrator alone can push every Trigger row out, and the Trigger Role then reads as *never called*.
  - known: the Quantifier calls with the literal string `"quantifier"` instead of `Role::Quantifier.agent_name()` (`agents/quantifier/utils/orchestration.rs:98`). A rename of the role would orphan that row's health, silently.
  - inferred: `elapsed_ms` of a probe and the TokenBudgetSpent-as-success rule (`connection_test_service.rs:77-85`) mean a probe can pass where a real call fails, so "healthy" from a probe and the Role health are two different judgements.

### The Game status: idle, busy or failed
- Where: `src/application/games/view_query.rs:32-39` (`enum Status`), `:198-212` (`status()`), `:127-135` (`get_debug_state` mapping)
- State variables: `GameWriter::busy(game)` (memory, the holders map) and the nullable `games.failure` column, read as `game.failure` (`view_query.rs:203-210`).
- States: *Busy* (a Job runs or a run holds), *Failed* (nothing holds, but a failure line stands), *Idle* (nothing holds and no line stands). The three appear in the dashboard as `GenerationStatus::{Busy, Failed, Idle}` (`:127-135`).
- Transitions: the failure line is written by `not_accepted_with_failure` (`generation/gate.rs:684-694`) and by `JobWriter::save_failure` (`gate.rs:873-877`, called from `pipeline/job.rs:385-418`); it is cleared by `accept_in_tx` at the start of every accept (`gate.rs:737-742`, from `plan_and_start` `:437-453`), by `stop` (`:515-526`), by `switch_swipe` (`:554`) and by `delete_last` (`:591`).
- Guards / invariants: the status rule reads busy first, then the line, then idle (`view_query.rs:31`). A Game that nothing holds should not show a *Busy* step, and a *Busy* Game should not hold a stale line.
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (the enum is one place; five writers of the column are in `gate.rs` plus one in `job.rs`)
- Already modeled: the busy half, yes (gate). The failure-line half is modeled as part of the Job, but its transitions sit in four separate gate methods.
- Risks seen:
  - known: the failure line is written while the holder still stands, so between `Job::end` (`job.rs:385`) and `JobWriter::drop` (`gate.rs:944-966`) a status read sees *Busy* with a line set. The banner and the status disagree for that window.
  - known: `edit` (`gate.rs:560-573`) is the one state-changing request that writes neither `clear_failure_and_put_dock` nor a dock reset, unlike `switch_swipe` and `delete_last`. An edited text leaves the old failure line and the old offered set.
  - known: the Options failure is not part of this status at all. It lives in the dock (next finding), so a Game can be *Idle* while the dock shows a failure line.

### The dock record (the offered set and the Options failure)
- Where: `src/domain/model/dock.rs:8-22` (`DockRecord`, `OptionsFailure`); writers in `src/application/generation/gate.rs:879-893`, `:917-950`
- State variables: `offered: Vec<String>` and `failure: Option<OptionsFailure>`, stored as the `offered` and `notice` columns (`src/adapters/driven/storage/docks.rs:23-46`).
- States: *no set, no failure* (a missing row reads as `DockRecord::default()`, `view_query.rs:194-196`), *a set offered*, *a failure shown*, and *both at once*.
- Transitions: `save_options` with `Offered` writes the set and `failure: None` (`gate.rs:883-889`); with `Unavailable` it keeps `offered` and sets the failure (`gate.rs:890-892`). A Narration empties both (`write_new_message` / `write_new_swipe`, `gate.rs:917-950`, driven by `order.kind().saves_a_narration()`); `stop` keeps the set and clears the failure (`gate.rs:517-524`); `switch_swipe`/`delete_last` clear both (`:554`, `:591`); a new Game writes the default (`:629`).
- Guards / invariants: the record's own doc says the failure is present "when a reason line stands in for the offered set" (`dock.rs:12`), which implies the two fields are exclusive. The code does not enforce that.
- Explicitness: FULLY IMPLICIT (two fields, no enum)
- Already modeled: no
- Risks seen:
  - known: `save_options(Unavailable)` (`gate.rs:890-892`) writes the failure **without clearing `offered`**, so the reachable state is "a set offered *and* a failure". The template shows only the failure when one exists (`src/adapters/driving/http/templates.rs:64`, `{% if vm.failure.is_some() %}`), so the offered set is silently hidden, not cleared, and reappears the next time something reads `offered` alone (`view_query.rs:155`, `get_debug_state` shows `current_options` from `offered` while the dock shows a failure).
  - inferred: the rejected `/options` path leaves a stale set that a later reader can serve as still pickable.

### The Quantifier reading: confidence, retry and fallback
- Where: `src/application/agents/quantifier/types.rs:12-31` (`QuantifierConfidence`, `QuantifierParseResult`), `src/application/agents/quantifier/utils/orchestration.rs:63-160` (the attempt loop), `:162-206` (the fallback), `src/application/agents/quantifier/utils/parser.rs:27-60`, `:142-206`
- State variables: `QuantifierParseResult { character_ids, confidence }` and the loop's `attempt`, `last_error`.
- States: *High* (JSON parsed, ids valid), *Medium* (text scan found valid ids), *Low* (nothing usable). A reading is *pending retry* when it is Low and `attempt < 2`, and *final* when it is Low on the second attempt or is High/Medium.
- Transitions: parse outcome sets the confidence (`parser.rs:43`, `:52`, `:58`, `:182`); Low plus another attempt available loops (`orchestration.rs:129-132`); a transport error loops (`:135-141`); the loop exits with an all-Low reading after two failures (`:145-160`); the fallback turns Low into "keep the Characters present" (`:187-192`, `static_character_result` `:194-206`).
- Guards / invariants: a Low reading must never reach the Game rules as an empty set; only High/Medium carry model ids (`orchestration.rs:167-186`), and an unknown id is filtered out there.
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (the enum is one place; the retry, the fallback and the filtering are three others)
- Already modeled: no
- Risks seen:
  - known: a Low reading on the **second** attempt returns as a Low reading and falls through to the static fallback, so a model that keeps answering garbage gives no error and no signal to the Job. `AgentRunner::after_narration` (`agents/runner.rs:70-81`) cannot tell a fallback from a real answer, because `QuantifierResult` drops the confidence (`types.rs:56-62`, documented at `:9-11`).
  - known: `last_error` is overwritten by a later successful but Low-confidence attempt, so a real transport error on attempt 1 and garbage on attempt 2 report garbage, and the error is lost (`orchestration.rs:135`, `:143-151`).

### The movement of a reading: `movement_type` × `destination`
- Where: `src/application/agents/quantifier/types.rs:35-48` (`MovementType`, `MovementParseResult`), `src/application/agents/quantifier/utils/parser.rs:167-188`, `:198-201`, `types.rs:56-62`
- State variables: `movement_type: Option<MovementType>` and `destination: Option<String>`.
- States: *no movement*, *entering a room*, *leaving a room*, and *a destination with no recognised type*.
- Transitions: set together from the JSON (`parser.rs:167-177`); both `None` on the text fallback (`parser.rs:198-201`) and after two failed attempts (`orchestration.rs:151-158`).
- Guards / invariants: the pair reads as one mode. A destination with no movement type is meaningful to nothing, yet nothing rejects it.
- Explicitness: FULLY IMPLICIT for the pair (`MovementType` is explicit but is never read outside this module)
- Already modeled: no
- Risks seen:
  - known: `movement_type` is mapped only for the exact words `entering`/`leaving` (`parser.rs:171-175`); any other word — `move`, `exit`, `walks` — yields `None`, while `destination` is kept as it came (`parser.rs:176`). `into_result` then drops the type entirely (`types.rs:56-62`), and the Game rules move the Persona on the destination alone (`src/domain/model/utils/game_rules.rs:106-127`). So an unrecognised movement word still teleports the Persona, and only the log line ever showed the type.

### The Options attempt loop
- Where: `src/application/agents/options/utils/orchestration.rs:18` (`MAX_OPTIONS_ATTEMPTS = 2`), `:42-71` (the loop), `:73-94` (one attempt)
- State variables: `attempt`, `last_error`.
- States: *attempt 1*, *attempt 2*, *done with a set*, *failed*.
- Transitions: a non-empty parse returns at once (`:55`); an empty parse becomes a `Parse` error and retries (`:56-61`); a transport error retries (`:62`); both attempts spent returns `last_error` (`:69-70`).
- Guards / invariants: one retry, no backoff, and the same prompt every time (`prompt` built once, `:47`).
- Explicitness: FULLY IMPLICIT (a counter; the two outcomes of an attempt are encoded as `Ok(Some)` / `Ok(None)` in an inner function, `:73-94`)
- Already modeled: no
- Risks seen:
  - known: the loop retries every error kind, so a bad API key or a 401 makes two full calls; nothing classifies the error as permanent.
  - known: only the last attempt's error is reported, so the first cause can be lost (mirrors the quantifier risk).
  - inferred: the retry means one Options step can write two `llm_messages` rows, so the Options Role health shows the second attempt only.

### The AgentRunner registration (which Agents each Hook holds)
- Where: `src/application/agents/runner.rs:20-66` (`after_narration: Option<Box<dyn AfterNarrationAgent>>`, `end_of_job: Option<Box<dyn EndOfJobAgent>>`)
- State variables: the two `Option` fields, plus the enabled flag of each config.
- States: *no Agent*, *after-narration only*, *end-of-Job only*, *both*. Each state is a different behaviour of a Job.
- Transitions: built once at wiring, from the configs: defaults apply when the list is empty (`:29-34`), a disabled config is skipped (`:36-38`), a second config of a kind `replace`s the first with only a warn (`:44-49`, `:54-59`), an unknown type is a hard error (`:60-62`).
- Guards / invariants: at most one Agent per Hook. An absent end-of-Job Agent must surface as "not available" rather than as an empty set (`:83-99`, `job.rs:352-355`).
- Explicitness: FULLY IMPLICIT (`Option` fields as mode; no enum)
- Already modeled: no
- Risks seen:
  - known: a config list holding one *disabled* Agent takes the non-empty branch at `:29-34`, so the defaults never apply and the runner ends with **no** Agent at all. That is a first-time-vs-later difference a user cannot see in the settings file.
  - known: the warning-only `replace` (`:44-49`) means two Quantifier configs silently run one, and the losing Agent is not reported in the UI.
  - known: `AgentRunner::is_empty` (`:100-102`) has no caller in `src/`.

### The call mode of a prompt: `impersonate` and `guide`
- Where: `src/application/prompting/assembler.rs:45`, `:47` (`PromptContext { guide: Option<String>, impersonate: bool }`), `:130-142` (`with_guide`, `with_impersonate`)
- State variables: the two fields, both public.
- States: *plain Narration* (neither set), *guided Narration* (guide only), *Impersonate* (`impersonate`, guide cleared), and the invalid *Impersonate with a guide*.
- Transitions: set by the caller of the builder in `src/application/narration/narrator.rs:127-131` (`with_impersonate(true)` or `with_guide(guide)` per `Generation` variant); `with_impersonate(true)` clears the guide (`assembler.rs:135-141`). The pair is then read at `:231-237` (the persona layer drops out) and `:319-338` (the guide layer).
- Guards / invariants: the persona layer and the guide layer must not both be present; a Guide is transient and never persisted.
- Explicitness: FULLY IMPLICIT (two fields; the only rule is in one setter)
- Already modeled: no
- Risks seen:
  - known: the fields are `pub` on a `pub struct`, so `PromptContext::new(...)` then `context.guide = Some(..)` (or a struct literal) reaches `impersonate == true && guide.is_some()` and renders both a Persona-less prompt and a Guide. The setter is the only thing preventing it.
  - inferred: `Generation` already encodes the four cases explicitly (`narration/narrator.rs:127-131`); the mode is re-encoded a second time as two booleans, so a fifth `Generation` variant must be taught to both encodings.

### The two-pass prompt fit (the token cut and the overflow guard)
- Where: `src/application/prompting/assembler.rs:168-186` (`render_and_fit`), `:188-214` (`render_cut_prompt`), `src/application/prompting/token_budget.rs:67-89` (`response_tokens`)
- State variables: `cut` (`HistoryCut`), `parts` (rendered once, maybe again), `excess_chars`, `max_tokens`.
- States: *first cut fits*, *first cut overflows and is cut again*, and an implied third state, *the second cut still overflows*.
- Transitions: render once (`:203`); if `excess_chars > 0`, cut harder with `drop_to_free` (`:206-213`); re-render only when the cut actually moved (`:212-214`); then clamp the answer length to at least 1 (`token_budget.rs:83-86`).
- Guards / invariants: the prompt must fit `input_limit()`; the guard runs **once** by design (`:196-199`).
- Explicitness: FULLY IMPLICIT (a two-phase render with no state name)
- Already modeled: no
- Risks seen:
  - known: the guard does not loop, so a prompt whose history is already minimal and whose fixed layers overflow stays over the limit; `response_tokens` then returns a value clamped to `.max(1)` (`token_budget.rs:86`), so the call asks for a 1-token answer instead of failing with `ContextOverflow` (that error is only raised for the system prompt, `:76-81`).

### The Autoplay pause hold
- Where: `src/application/autoplay/pause.rs:11-14` (`PauseSync { paused, released }`), `:22-27` (`AutoplayPause { armed: Mutex<HashMap<GameId, Arc<PauseState>>> }`), `:30-45` (`point`), `:50-62` (`hold`), `:72-96` (`wait_paused`, `release`, `Drop`)
- State variables: `armed` (the map), `paused`, `released`.
- States: *not armed*, *armed and not reached*, *reached and paused*, *released*. `PauseSync` has four combinations of two booleans; *released without ever paused* is reachable.
- Transitions: `hold` inserts into `armed` (`:56-59`); `point` **removes** the entry and then sets `paused` (`:31-40`); `release` sets `released` and notifies (`:82-86`); `Drop for PauseHandle` calls `release` (`:90-95`).
- Guards / invariants: one hold per Game at a time (an `assert!` at `:52-56`), and a hold fires exactly once, because `point` consumes the entry.
- Explicitness: FULLY IMPLICIT (two booleans plus a map)
- Already modeled: no (test-only, but built into production)
- Risks seen:
  - known: `release` before the loop reaches `point` sets `released` first. `point` then sets `paused`, skips the wait loop at once (`:42-44`), and returns. `wait_paused` sees `paused == true` and reports a pause that the loop never took (`:72-80`). The test's ordering then rests on a race, not on an invariant.
  - known: the same `released`-first path means the "fires once" claim holds for the map, but not for the pause.
  - known: `hold` panics on a second hold of the same Game (`:52-56`), a production-reachable panic from a test-only entry point.

### The watch on the tasks a Job spawns
- Where: `src/application/generation/job_tasks.rs:9-11` (`JobTasks { handles: Mutex<Vec<JoinHandle<()>>> }`)
- State variables: the vector of handles, and each handle's `is_finished`.
- States: *a handle is pending*, *finished*; the set is pruned on the next `watch` (`:16`) and drained by `wait` (`:36-39`).
- Transitions: `watch` prunes then pushes (`:14-18`); `wait_for_each` drains the whole vector, awaits each, and loops while new handles arrive (`:34-51`).
- Guards / invariants: `running()` must count only unfinished tasks; `wait` must not miss a Job that a drained task spawned.
- Explicitness: FULLY IMPLICIT (a collection, no mode)
- Already modeled: no
- Risks seen:
  - known: `wait` drops the handles it awaited. A Job that panicked is only logged (`:46-48`), so `wait` cannot report it, and the panic is invisible to a test that only checks `wait == true`.
  - inferred: `running()` reads a snapshot; nothing stops a Job from being watched between the `retain` and the count of another thread, so the count is advisory only.

### The Job: steps, failure line and the end of its entry  *(already modeled)*
- Where: `src/application/pipeline/job.rs:142-174` (`run`), `:215-223` (`run_step`), `:385-418` (`end`); `src/application/generation/gate.rs:104-109` (`JobEntry`), `:780-800` (`JobWriter`), `:944-966` (`Drop`)
- State variables: `JobEntry { id, step }`, the holder variant, `JobWriter.id`, `JobEnd`.
- States: *not accepted*, *running a step*, *all steps done*, *failed with a line written*, *entry moved on*.
- Transitions: accept writes the entry (`gate.rs:360-372`); each step is set before it runs (`job.rs:222`); a failed step stops the loop (`job.rs:159-162`); `end` writes the failure line (`:388-403`); `JobWriter::drop` frees the entry only if the id still matches (`gate.rs:951-965`).
- Explicitness: EXPLICIT (enum `JobEnd`, `JobFailure`) with the transitions in two places (job and gate)
- Already modeled: yes
- Risks seen: none seen beyond the Busy/Failed window noted above.

### The Autoplay holder: a run that holds between two Jobs  *(already modeled)*
- Where: `src/application/generation/gate.rs:128-139` (`Holder::Autoplay { run, progress, job: Option<JobEntry> }`), `:374-435` (`accept_for_autoplay`), `:467-499` (`reserve`, `end_autoplay`); `src/application/autoplay/driver.rs:73-121`
- State variables: `job: Option<JobEntry>` plus `progress { done, budget }`.
- States: *held, between two Jobs* (`job == None`), *held, a Job running* (`job == Some`), *ended*.
- Transitions: `reserve_for_autoplay` inserts with `job: None` (`gate.rs:475-483`); an accept sets `job: Some` (`:419-427`); `JobWriter::drop` sets it back to `None` (`:956-957`); `Decision::Finish`, a `NotAccepted` outcome and `end_autoplay` remove the holder (`:406-409`, `:429-431`, `:487-498`), and `Drop for AutoplayRun` is the backstop (`driver.rs:137-140`).
- Explicitness: EXPLICIT STATES, one guard (`GameWriter::hold`)
- Already modeled: yes
- Risks seen:
  - known: `progress` is duplicated. The loop keeps its own `done` (`driver.rs:74`) and the gate stores a copy (`gate.rs:116-126`). The loop's `after_job` flag is a third piece of the same mode, so a run's cycle count has two homes that agree only because the loop passes its value in on each accept.

### The active-Game pointer  *(already modeled)*
- Where: `src/application/games/active.rs:20-23`, `:44-49` (`lock`), `:65-79` (`set`, `set_new`); writers in `src/application/games/catalogue.rs:62-66`, `:71-79`, `:95-104`, `:280-283`
- State variables: `Mutex<Option<GameId>>`.
- States: *no active Game*, *an active Game that exists*. The invalid state is a pointer that names no Game.
- Transitions: `set` validates by touching `updated_at` first (`active.rs:65-72`); `set_new` skips that check and trusts the create/reset transaction (`:77-79`); `GameCatalogue::new` seeds it (`catalogue.rs:31-34`).
- Explicitness: EXPLICIT (one `Option` plus one guard type)
- Already modeled: yes
- Risks seen:
  - known: `set_new` has no existence check, so its correctness rests on the caller holding the transaction that just wrote the Game. The two callers do hold it (`catalogue.rs:62-66`, `:280-283`), so the risk is a future caller, not a present path.

Table of contents:

| name | file | explicitness |
|---|---|---|
| Role health | src/application/games/view_query.rs | FULLY IMPLICIT |
| The Game status | src/application/games/view_query.rs | EXPLICIT STATES, SCATTERED TRANSITIONS |
| The dock record | src/application/generation/gate.rs | FULLY IMPLICIT |
| The Quantifier reading | src/application/agents/quantifier/utils/orchestration.rs | EXPLICIT STATES, SCATTERED TRANSITIONS |
| The movement pair | src/application/agents/quantifier/utils/parser.rs | FULLY IMPLICIT |
| The Options attempt loop | src/application/agents/options/utils/orchestration.rs | FULLY IMPLICIT |
| The AgentRunner registration | src/application/agents/runner.rs | FULLY IMPLICIT |
| The call mode of a prompt | src/application/prompting/assembler.rs | FULLY IMPLICIT |
| The two-pass prompt fit | src/application/prompting/assembler.rs | FULLY IMPLICIT |
| The Autoplay pause hold | src/application/autoplay/pause.rs | FULLY IMPLICIT |
| The watch on Job tasks | src/application/generation/job_tasks.rs | FULLY IMPLICIT |
| The Job and its steps | src/application/pipeline/job.rs | EXPLICIT (already modeled) |
| The Autoplay holder | src/application/generation/gate.rs | EXPLICIT (already modeled) |
| The active-Game pointer | src/application/games/active.rs | EXPLICIT (already modeled) |