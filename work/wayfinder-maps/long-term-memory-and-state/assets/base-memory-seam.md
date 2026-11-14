# The seam of the base story memory

Read on 2026-11-11, on the branch `long-term-memory-2` at `4baba0a4`. Every line
reference is to that commit. **Known** means I read it in the code or the doc and
quote it. **Inferred** means the code makes it follow but no line states it.
**Guessed** means I did not check it.

## 1. The cut today

- **The cut is a pure function of one list of token sizes and one space (known).**
  `HistoryCut::of(&entry_tokens, space)` in `src/domain/model/history_cut.rs:23`
  takes the size of each history entry and the history space in tokens. It walks
  the entries from the first one. Each time the kept entries pass the space, it
  drops the oldest kept entries until the kept part is at most 75 % of the space
  (`KEPT_AFTER_CUT_PERCENT`, `history_cut.rs:7`), as one block. So one call can
  produce several blocks.
- **The cut runs inside the prompt assembly, and nowhere else (known).**
  `src/application/prompting/assembler.rs:188` `render_cut_prompt` builds
  `entry_tokens` (`:195`), computes the space (`:196`,
  `budget.history_space(system_tokens, self.fixed_tokens())`) and calls
  `HistoryCut::of` (`:197`). `rg -n "HistoryCut" src` finds the type in
  `assembler.rs`, `history_cut.rs` and `history_cut_tests.rs` only. No other
  module reads a cut.
- **The cut is not stored (known).** `HistoryCut` holds `len`, `kept_from` and
  `blocks` in memory only (`history_cut.rs:16`). Nothing writes it. No column,
  no JSON key and no debug record holds it.
- **The cut is decided at the Narrator call of a Job (known).** `Job::narrate`
  (`src/application/pipeline/job.rs:239`) loads the whole story
  (`:253`, `self.window_before(..., STORY_LIMIT)`), and `STORY_LIMIT` is
  `i32::MAX` (`src/adapters/driven/storage/stories.rs:17`). It then calls
  `self.narrator.narrate` (`narrator.rs:107`), which calls
  `self.assembler.assemble` (`narrator.rs:100`). The cut exists only inside that
  call, as a local value.
- **The Narrator holds no storage, by design (known).**
  `src/application/narration/narrator.rs:27`: "The values one narration reads.
  The narrate step loads them; the narrator holds no storage." The assembler is
  the same: `PromptAssembler` is `Default` and takes no `Storage`.
- **Which calls get a cut (known).** Every call that goes through
  `Narrator::narrate` renders the history through the assembler: a Narration, an
  Impersonate, a Retry of a typed Input and an event Narration
  (`narrator.rs:107`, and `Generation::preset_type`/`is_analysed` in
  `src/domain/model/generation.rs:52`). The Quantifier and the Options calls do
  not: they read `AGENT_HISTORY_WINDOW` = 4 entries through their own agents
  (`src/domain/model/agent.rs:82`, `job.rs:305`, `job.rs:349`).
- **A second drop can happen after the first (known).** The overflow guard
  (`assembler.rs:198-213`) drops more whole entries, as one block, when the
  prompt still passes the input limit. The comment says "a part outside the
  space (a long Player Input) can still pass the input limit". So a cut can grow
  at the very moment of assembly, after any earlier estimate.
- **What the cut records about a block (known).** Each `HistoryBlock` holds only
  `dropped: Range<usize>` (`history_cut.rs:10`). The index is the position in
  the `entries` vector, which is the story order (`entries_of` sorts by id,
  `stories.rs:183-208`). So a block names a range of `StoryRow`s, and each
  `StoryRow` has `id` (`src/domain/model/state/message_types.rs:32`). No text is
  copied.

**Consequence for the base (inferred).** At the moment the engine knows which
Messages leave, it is inside a pure function of the prompt renderer, which holds
no storage and no writer. A model call cannot run there. The Job must compute
the same cut itself, before the Narrator call, and then run the writer as its
own step.

## 2. The writer call

### Where it can run

- **In the Job, before the Narrator call (inferred, from the code above).** The
  Job is the only part of this path that holds storage, the preset, the
  connection, the value budget and the gate write. It has the whole story
  (`job.rs:253`), the start state (`job.rs:456`) and the input text. So it can
  plan the cut, run the writer and then call the Narrator.
- **The next call must already carry the memory (known, from the decision).**
  Ticket 05: "When Messages leave the history, a writer call first summarizes
  them into the story memory." A writer that runs at the end of the Job is too
  late: the Swipe that the story memory belongs to is already saved by then
  (`job.rs:193`), and the cut of the next call depends on the next call's
  preset, preset type and history (`job.rs:470` `call_plan`). So the writer runs
  in the same Job, before `narrate`.
- **The result cannot be written to the Swipe at that moment (known).** The
  target Swipe does not exist yet. `save_text` (`job.rs:288`) is the step that
  creates it (`gate.rs:793`, `write_new_swipe` at `gate.rs:878`). So the writer
  returns text to the Job, the Job puts it in the prompt, and `save_text` writes
  it into the new Swipe's state.
- **Order of the steps today (known).** `Job::generate`
  (`job.rs:176-204`): room check, `save_input`, `narrate`, `save_text`,
  `analyse`, `update_state`. There is no step before `narrate`. A memory step is
  a new step between `save_input` and `narrate`.
- **The gate allows it (known).** A Job holds the Game from accept to drop
  (`gate.rs:341`, holders; `JobWriter::hold` at `gate.rs:753`). The writer is a
  read plus a model call, so no record write is needed before `save_text`. The
  only write is the `llm_messages` row, which every model call already does
  outside the gate (`src/application/llm_recorder.rs:85`,
  `(*self.save_fn)(&message)?`).
- **A new `Step` name is cheap (known).** `Step` lives in memory only
  (`generation_status.rs:227`; the busy state is memory only, `gate.rs:175`).
  Its words are one `match` in
  `src/adapters/driving/http/layout/handlers/endpoints.rs:79-82`, and it also
  appears in the `/debug/state` answer (`src/application/debug/dto.rs:16`).

### Which seam fits

- **Not `AfterNarrationAgent` (known).** It runs after the text save
  (`docs/diataxis/reference/narrative/agent_system.md`, "Hooks": "After
  narration: runs after the text save and before the state update"). Its input
  is `HookInput { text, story, world }` (`src/domain/model/agent.rs:93`), which
  holds no previous memory and no dropped entries. `story` is
  `AGENT_HISTORY_WINDOW` = 4 entries (`job.rs:305`).
- **A new hook trait is the shape that fits (inferred).** The agent system says
  "A hook is a fixed point in a Job where the Job runs the Agent registered for
  it, named by its position. Each Agent gives one kind of result." The new hook
  is "before narration", with the result "a story memory text". Its input needs
  the previous memory text and the leaving `StoryRow`s, so it needs its own
  input type next to `HookInput`. That is a new vocabulary item in the agent
  system, not a field added to `HookInput` for one agent.
- **The hook must not be gated by the agent config (known).** The agent system
  promises "The engine works with **zero agents** — all agent execution is
  optional." The base memory is not optional. And the config list reaches an
  existing DB only once: `Storage::seed_settings` writes `OnlyIfAbsent`
  (`src/adapters/driven/storage/settings.rs:120-124`), and the stored `agents`
  column is JSON (`src/adapters/driven/storage/models/settings.rs:16`). So an
  existing DB keeps the two agent entries in `data/settings.json:74` and would
  never see a third one. The writer must be built unconditionally in
  `src/bootstrap/wiring.rs`, next to the Narrator, and not from
  `AgentConfig`.
- **How the three calls share one memory (known).** All three go through
  `Narrator::narrate` and one `NarrationValues` (`narrator.rs:29`). One field
  added to `NarrationValues` and to `PromptContext` (`assembler.rs:32`) reaches
  the Narration, the Impersonate and the event Narration in one place. The Retry
  of a typed Input goes the same way.
- **Autoplay runs ordinary Jobs (known).** `Job::run_accepted` (`job.rs:112`) is
  the run's unit (`src/application/autoplay/driver.rs:100`). A memory step
  inside the Job runs once for each Cycle, inside the run's hold, with no change
  to the budget or the loop.

### The rewrite rule

- **The writer's input is the previous memory plus the entries that left since
  (inferred).** With the same space, the cut is monotonic in the story:
  `HistoryCut::of` walks the
  prefixes in order, so a longer story keeps the earlier blocks and adds blocks
  at the end. So the new block is the part of `0..kept_from` that the previous
  memory does not cover. Without a marker the writer would re-read every dropped
  entry at each cut, and its prompt would grow with the story.
- **The marker is a Message id (inferred).** Each dropped entry has an id
  (`StoryRow.id`), and ids only grow. So "covered up to id N" is a stable
  statement, and a delete of a newer Message cannot invalidate it.

### What each action does to the memory

- **Retry (known).** `JobPlan::redo` (`generation.rs:326`) makes the order
  `Place::NewSwipeOnLast` with `start: tail.start_at()` (`generation.rs:345`,
  `tail.rs:37`). `write_new_swipe` inserts a new Swipe at index `len` and sets
  it active (`gate.rs:878-899`). The old Swipe and its state stay. So a Retry
  starts the memory from the state before the last Message and writes a new
  memory for the new Swipe. A switch back gives the old memory back, with no
  extra code.
- **A swipe switch (known).** `switch_swipe` only calls `tx.set_active_swipe(at)`
  (`gate.rs:501-531`). Every state read joins the active swipe
  (`stories.rs:24`, `LEFT JOIN message_swipes s ON ... s.swipe_index =
  m.active_swipe_index`). So the memory follows the active Swipe for free.
- **Delete of the last Message (known).** `delete_last` calls
  `tx.delete_message` (`gate.rs:547-567`), and the Swipe rows go with it
  (`message_swipes.message_id ... ON DELETE CASCADE`, `plumbing.rs:110`). The
  memory of that Swipe goes too. The next call starts from the state of the
  Message before it, whose memory is the older one, and the cut is recomputed.
  No extra code.
- **Edit of an older Message (known).** `edit` calls `tx.set_swipe_text(at, text)`
  only (`gate.rs:533-545`, `transaction.rs:205`). The state is untouched. So the
  memory keeps the old text, and the prompt then holds both the new text (if it
  is in the history) and the old summary. This is the open item "An edit of an
  older Message" of the map, and the base does not fix it.
- **A delete that shrinks the cut (inferred).** The cut is a size rule, so a
  shorter story can keep entries that the memory already covers. The prompt then
  shows them twice, in the memory and in the history. Nothing is lost. A summary
  cannot be un-summarised.

## 3. The storage

- **A `SwipeState` key needs no migration (known).** `state` is a JSON text
  column (`plumbing.rs:639`, `ALTER TABLE message_swipes ADD COLUMN state
  TEXT`), written by `serde_json::to_string(&swipe.state)`
  (`transaction.rs:197`) and read by `serde_json::from_str` (`stories.rs:343`).
  `SwipeState` has the pattern already: `event_record` is
  `#[serde(default, rename = "trigger_context", skip_serializing_if =
  "Option::is_none")]` (`swipe_state.rs:23-30`). A new key of the same shape
  reads as `None` in every old row.
- **What it stores (inferred).** `Option<StoryMemory>` with
  `{ text: String, covers_up_to: u64 }`. One text, as ticket 05 decides, plus
  the id marker that section 2 needs.
- **Where it is written and read (known).** Written: `save_text` puts it into
  the state that `GenerationOrder::write` clones into the new Swipe
  (`generation.rs:94`, `Swipe::new(text, maker, start.clone())`). Read: the
  start state (`job.rs:456` → `stories().swipe_state`), which is the last
  Message's active Swipe for a new Message (`tail.rs:27`) and the Message before
  the last for a Retry (`tail.rs:37`).
- **The analysed state keeps it (known).** `update_state` reads the saved
  state back (`job.rs:326`, `swipe_state_of(at, Record::SavedState)`) and
  `game_rules::apply` takes it by value and returns it (`game_rules.rs:45`), so
  a field it does not touch survives the update.
- **A table would cost a migration (known, from the standing rule).** A new
  table or column must add the next free step, guard it with `column_exists` and
  raise `LATEST_VERSION` (`plumbing.rs:17`, `LATEST_VERSION: i32 = 34`), with a
  test in `plumbing_tests.rs`. A `SwipeState` key costs none of that.
- **Recommendation: the `SwipeState` key for the base.** The lifecycle rules of
  section 2 then need no code: clone at the write, cascade at the delete, and
  the active swipe at the read. The key is the first field of `SwipeState` that
  the game rules never read, so if the memory grows into a list of scene entries
  it should move to its own table keyed by Game and Swipe.

## 4. The prompt

- **The block goes at the end of the first user part (known).** The first user
  part holds `before_layers()` = Known Characters, Persona, World Lore
  (`assembler.rs:276-282`), joined by blank lines (`join_layers`,
  `assembler.rs:326-333`). The history
  entries follow it as their own parts. So a `<StoryMemory>` layer pushed after
  World Lore in `before_layers` is the end of the first user message, before the
  history. That is the position ticket 05 asks for.
- **The block takes room from the history space (known).**
  `budget.history_space(system_tokens, self.fixed_tokens())`
  (`assembler.rs:196`) subtracts the fixed part from the input limit, and
  `fixed_tokens` measures `before_layers` (`:237-245`). So every token of the
  memory shrinks the history space, and can move the cut earlier. Ticket 05
  notes the same effect.
- **The block must be a constant size in the cut arithmetic (inferred).** The
  writer result is not known when the Job must know the cut. `fixed_tokens`
  measures the real text (`:239`), and the overflow guard measures the real
  characters (`:204-213`). So a text of unknown size gives two different cuts.
  The base must reserve a fixed size for the block in both `fixed_tokens` and
  the guard, and cap the writer's text at that size. Then one plan serves the
  writer step and the render, and the render cannot drop an entry that the
  writer did not see.
- **The cache cost is one extra break at the cut (inferred).**
  `docs/diataxis/explanation/prompt_system_design.md:16` says "The blocks that
  stay the same during a story come before the history." The memory changes
  only when a cut happens. On that call the cache already ends at the history
  start (`prompt_system_design.md:22`). A memory change moves the first
  difference back to the first user part, so the cost is the blocks before the
  history on that one call. The system message stays cached.
- **The player's story is untouched (known).** The assembler only renders
  (`assembler.rs:247`), and the story is read whole (`stories.rs:17`). Nothing
  of this writes a Message.

## 5. The failure rule

- **The existing rule (known).** `AgentRunner::after_narration` logs a failed
  agent and returns the state that keeps the Room and the Characters present
  (`src/application/agents/runner.rs:70-83`), and
  `docs/diataxis/reference/narrative/agent_system.md` says "the runner logs a
  failed Agent, and the Job continues".
- **The same rule for this writer (inferred).** A failed writer keeps the
  previous memory as the prompt's block, and the Job continues. That is the
  "a failed writer call loses nothing" of ticket 05.
- **The leaving Messages are not lost, they wait (inferred).** The marker does
  not advance when the writer fails. The entries that left are still in the
  story, because the engine never deletes history for a cut (`stories.rs` reads
  the whole story). At the next successful cut, the writer's input is "dropped
  entries with id > marker", so it summarises them then. The model misses them
  for the cycles in between.
- **No failure line and no banner (inferred).** The Job continues, so no
  `GenerationFailure` is made and the banner stays clear. The failed call is
  still in the record, with `error_message` set (`llm_recorder.rs:74-90`).
- **One cost of the rule (known).** The reader of the story memory cannot tell
  how old it is. A memory whose writer failed for several cuts is older than
  the cut. The record of the call is the only evidence.

## 6. The connection

- **A connection is reached through a `PresetType` slot (known).**
  `recorder_with_storage(storage, preset_type)` resolves the connection per call
  (`src/bootstrap/wiring.rs:46-78`), and `AppSettings::connection_for` reads the
  slot (`src/domain/model/settings.rs:612`). Wiring builds one recorder for each
  slot (`wiring.rs:143-150`).
- **A new `PresetType` is a large change (known).** `PresetType` has four
  variants (`src/domain/model/prompt_preset.rs:15-21`). Each one needs a field
  on `Game` and a stored preset id (`src/domain/model/game.rs:37-54`), so a new
  one needs a column and a migration. `CONNECTION_SLOTS` lists four
  (`settings.rs:511-536`).
- **The agent name and the connection are separate (known).** The Impersonate
  records the `narrator` agent name with its own recorder (`narrator.rs:79-86`,
  `Kind::role` at `agent.rs:66`). So the writer can take its connection from an
  existing slot and still record under its own name.
- **A role is not free either (known).** `GameViewQuery::role_health` counts
  the newest row of each `Role::ALL` by agent name
  (`src/application/games/view_query.rs:249-270`), and each role fills one cell
  (`src/adapters/driving/http/builders/headers.rs:111`). A writer recorded under
  the Narrator's name would raise "Narrator failed — no narration was written"
  (`headers.rs:27-34`) for a failed summary, which is false. So the writer needs
  its own `Role`, or its name is absent from the health table.
- **Recommendation for the base (inferred).** A new `Role::Memory` with the
  sentence "keeping the previous story memory" (`headers.rs:27`), and its
  recorder built from the Narration slot (`PresetType::System`). No new preset
  type, no new settings field, no migration. `AgentSettings.preset` stays
  `None` and the writer's instructions are code, because no preset type holds
  memory text. Its `max_tokens` is a constant, not the narration preset's.
- **The test fake needs the name (known).** `ScriptedModel` maps an agent name to
  a `Role` and treats an unknown name as a stray call
  (`src/test_support/scripted_model.rs:32-44`), so the new role must be added
  there with `AGENT_MEMORY` beside the others
  (`src/application/ports/llm_provider.rs:10-14`).

## 7. The records

- **The call record (known).** Every model call saves one `LlmMessage` with
  `system_prompt`, `user_prompt` and the raw request and answer
  (`llm_recorder.rs:85`, table at `plumbing.rs:121-135`). So the writer's own
  answer and the Narration's `<StoryMemory>` block are both in the record with
  no extra work. The dashboard dialog already shows these rows
  (`docs/specs/llm_messages.md`).
- **The trial pack copies the rows (known).** `trial.py:610` copies each new
  `llm_messages` row to `calls.jsonl`. `write_call_files` writes one file for
  each call with its messages and its answer (`trial.py:1094-1116`). So the
  Narration call file shows the memory text that the call carried.
- **The story is also in the pack (known).** `story_rows` selects `s.state` for
  the active Swipe (`trial.py:264-275`), and `story.json` holds it. But
  `state_line` reads only the room and the Characters present
  (`trial.py:965-971`). So the memory is in the pack data and not in the pack
  text today.
- **What the pack needs (inferred).** One row in `CALL_KINDS` for the new call
  kind (`trial.py:87-93`, `CallKind("memory", False)`), so the summary table
  counts it and `is_story_call` does not treat a failed memory call as a story
  failure (`trial.py:665-667`). And a line in `state_line` or `story_lines`
  that prints the memory of the active Swipe of each entry. The cut line
  (`failure_and_cut_lines`, `trial.py:1234-1259`) already reads the cut from the
  prompt, so it needs no change.

## 8. The tests

- **The cut function has a plain unit test (known).**
  `src/domain/model/history_cut_tests.rs` tests `HistoryCut::of` and
  `drop_to_free` directly. It is a pure function in `domain/model/`, which
  `tests/STRATEGY.md` ("The one rule", item 2) allows as a plain unit test.
- **The prompt side is the Narrator's seam contract (known).**
  `src/application/narration/narrator_tests.rs` carries the cut cases:
  `narrator-history-small-budget` (`:585`), `narrator-history-large-budget`
  (`:614`), `narrator-history-default-size` (`:633`),
  `narrator-history-size-field` (`:654`) and
  `narrator-history-cut-drops-whole-messages` (`:688`). The layer-order case is
  `narrator-layer-order` (`:421`). So the `<StoryMemory>` block and the plan of
  the cut get cases here.
- **The writer is an Agent, so its test file is a seam contract (known).**
  `tests/STRATEGY.md` lists "each Agent" among the boxes, and the Quantifier's
  `agent_tests.rs` shows the shape: one `// CASE <id>: <what it pins>` above each
  test. `scripts/validate_seam_cases.py` fails a test in a declaring file
  without a tag.
- **No spec covers the cut today (known).** `rg -ln "cut" docs/specs` finds no
  spec about the history cut. A behaviour a player can see needs a scenario
  (`tests/STRATEGY.md`, "A behaviour a player can see is not a unit test").
  The base adds a visible behaviour: the prompt carries the memory, and a failed
  writer keeps the old one. So it needs a new spec, `docs/specs/story_memory.md`,
  with scenarios numbered from 46 (the highest number today is 45), and a test
  file `tests/http/story_memory.rs` whose tests carry
  `// [story_memory] SCENARIO: 46.x`.
- **Tier 1 can assert on the prompt (known).** `ScriptedModel` records each call's
  `parts` and its `ModelCall::user_prompt()` (`scripted_model.rs:53-73`). So a
  scenario can check that the first user part holds `<StoryMemory>` before the
  first history entry, and that the second call after a writer failure holds the
  old text.

## 9. The open choices

1. **Where the memory lives.** A `SwipeState` key, or a table keyed by Game and
   Swipe. *Recommendation: the `SwipeState` key.* Reasons in section 3: no
   migration and the lifecycle for free. It is the room the map names. This is a
   design call the owner may take.
2. **The marker.** `covers_up_to: u64` beside the text, or no marker and a full
   rewrite at each cut. *Recommendation: the marker.* Without it the writer's
   prompt grows with the story, and a 100-Cycle trial pays that at every cut.
   Only the owner can decide whether a simpler form is worth that cost.
3. **The cap of the memory text.** A constant in tokens, and the writer is asked
   for that size and cut at that size. *Recommendation: one constant, in tokens,
   used for the reserve, the writer's answer limit and the save.* Only the owner
   can set the number. Ticket 05 leaves the cap to the design tickets.
4. **The connection and the role.** A new `Role::Memory` on the Narration slot,
   or the Quantifier slot and role. *Recommendation: a new `Role::Memory` on the
   Narration connection.* The Quantifier's slot would report a summary failure as
   a Quantifier failure, and the Narration's name would report it as a narration
   failure. The owner may want a separate, cheaper connection, which needs a new
   slot and a migration.
5. **Whether the writer is on by default.** *Recommendation: on, always, outside
   the agent config.* The base is not optional, and an existing DB never sees a
   new agent entry (section 2). Only the owner can decide whether the base may
   ship a switch at all.
6. **Where the writer step sits for an Input.** Today `save_input` runs before
   `narrate` (`job.rs:183-193`), so an Input Message's Swipe state carries the
   memory from before the cut. *Recommendation: leave it.* The memory belongs to
   the Swipe that the cut's call wrote. Only a person reading a stored Input's
   state sees the older text.
7. **The block's tag and lead.** `<StoryMemory>` with a lead line, or none.
   *Recommendation: a tag and no lead*, like `<WorldLore>`. The engine's other
   data blocks state a fact in place of an empty body
   (`prompt_system_design.md:40`), so an empty memory renders no block at all.
8. **The first cut's input.** All entries of the first block in one call, or the
   block split into several calls. *Recommendation: one call.* At 16,000 tokens
   one block is about 21 entries, which is one cheap call.

## 10. The refactor, if one is needed

The base does not fit the code as it stands. Two small refactors make it clean.

1. **Expose the cut plan of one call (small, about one function).** Extract the
   measure-and-cut part of `render_cut_prompt` (`assembler.rs:188-213`) into a
   function that returns the `HistoryCut` for one context, preset and budget, and
   let both `assemble` and a new `Narrator::plan_cut` call it. The cut stays a
   pure function of rendered sizes. Because the memory block has a fixed
   reserved size (section 4), the plan and the render give the same cut, and the
   writer sees exactly the entries the render drops. This is a move of about 30
   lines inside `assembler.rs` plus one wrapper in `narrator.rs`. No behaviour
   changes for any call that has no memory.
2. **Give the writer its own hook and step (small, one new module).** A new
   `BeforeNarrationAgent`-shaped trait with its own input and result, one
   implementation in `application/agents/memory/`, built in `wiring.rs` with its
   own recorder, and one `Step` variant. The Job calls it from a new
   `write_memory` step between `save_input` and `narrate`, and `narrate` returns
   the memory with the text so `save_text` can put it into the new Swipe.

Nothing else is needed: no migration, no new preset type, no new settings
field, and no change to the gate, the Autoplay loop or the story reads. The
dashboard needs one status label for the new step.

If the owner wants the memory and the writer to be visible in the dashboard and
the health header at once, the two pieces above still hold, and the extra work
is one `Role` arm, one health cell and one line in the trial pack.

## What I did not find

- **No stored record of a cut.** I looked in `src/domain/model/history_cut.rs`,
  `src/application/prompting/`, `src/adapters/driven/storage/` and the schema in
  `plumbing.rs`. Nothing stores `kept_from` or a block.
- **No existing writer that runs before a narration call.** The two hooks are
  after the text save and at the end of the Job
  (`src/application/agents/trait_def.rs:10-20`). I found no third hook.
- **No spec and no browser test for the history cut.** I searched
  `docs/specs/` for "cut" and "history" and `tests/browser/` for the same.
- **I did not run the engine, a build or a test.** The ticket forbids it. So the
  token effect of the block, and the exact cut shift that it causes, are not
  measured here.
