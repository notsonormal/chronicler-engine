# Plan: build the scene entries (ticket 17, step 1)

Read on `long-term-memory-2` at `4832b0fa`. Every `file:line` below was checked at that commit.
Epistemic marks: **known** (read in the code), **inferred**, **recommended** (a choice the ticket leaves to the build).

**Conflicts: none that block the build.** Section 10 lists three small mismatches between the ticket text and the code. None changes a decision.

---

## 1. Storage: the table, the write and the read

### The table (migration step 35)

- `LATEST_VERSION` goes from 34 to 35 (`src/adapters/driven/storage/utils/plumbing.rs:17`). The new `if due(35)` block goes after step 34 (`plumbing.rs:782-792`), before `tx.commit()` (`:794`). Per the map's standing rules (`map.md:69`), it adds a new table and edits no old `CREATE TABLE`.

```sql
CREATE TABLE IF NOT EXISTS scene_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    swipe_id INTEGER NOT NULL REFERENCES message_swipes(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,          -- order inside the cut that wrote them
    from_message INTEGER NOT NULL,      -- Message id, set by the engine
    to_message INTEGER NOT NULL,        -- Message id, set by the engine
    story_time TEXT NOT NULL,
    place TEXT NOT NULL,
    took_part TEXT NOT NULL,            -- JSON array of names
    others TEXT NOT NULL,               -- JSON array of names
    what_happened TEXT NOT NULL,
    UNIQUE(swipe_id, position)
)
```

- The tie to the Swipe: `message_swipes.id` exists (`plumbing.rs:685-698`). `message_swipes.message_id` cascades from `messages` (`:687`), `messages.game_id` cascades from `games` (`:676`), and `foreign_keys` is on (`src/adapters/driven/storage/db.rs:20`). So a delete of the last Message (`Tx::delete_message`, `transaction.rs:171`) and a Game delete remove the entries with no new code (**known**).
- A Retry appends a Swipe (`gate.rs:878-898`), and a switch changes `active_swipe_index` (`gate.rs:501-531`). The read below joins only the active Swipe, so both need no code either (**known**).
- The test: one case in `src/adapters/driven/storage/utils/plumbing_tests.rs`. It covers the step from 34 to 35, and a delete of the Message that removes its entries.
- `scripts/tests/test_trial.py` replays the `CREATE TABLE` literals of `plumbing.rs` (`engine_tables`, `:71`). So keep the statement in one string literal, as the other steps do.

### The write: `Tx::insert_scene_entries`

- Add `TxWrite::InsertSceneEntries` (`transaction.rs:19-33`). Map it in `ChangeCounters::kinds_of_write` (`changes.rs:183-199`) to `&[ChangeKind::STORY]`. That kind adds no new counter (ticket, Not in scope), and the write always lands in the unit of an `InsertSwipe`, which raises `STORY` already (**recommended**).
- `pub fn insert_scene_entries(&mut self, at: SwipeAt, entries: &[SceneEntry])` goes through `self.write(TxWrite::InsertSceneEntries, …)` and `require_message_of_game`, as `insert_swipe` does (`transaction.rs:183-204`). It finds the `swipe_id` with `(SELECT id FROM message_swipes WHERE message_id = ? AND swipe_index = ?)`, and gives `position` the slice index.
- The ticket says `GameWriter::save_text`. The method is `JobWriter::save_text` (`gate.rs:793-818`, `impl JobWriter` at `:744`). It takes a new parameter `entries: &[SceneEntry]` and passes it to `write_new_message` (`:857-876`) and `write_new_swipe` (`:878-898`). Each one calls `tx.insert_scene_entries(at, entries)` after `tx.insert_swipe` (`:870`, `:890`) when the slice is not empty. One transaction holds the text and the entries, so a failure loses both (**known**: `TxOpener` units commit whole).

### The read: `StoryReads::scene_entries`

- `pub fn scene_entries(&self, game: GameId, before: Option<u64>) -> Result<SceneEntries, EngineError>` goes in `src/adapters/driven/storage/stories.rs`, beside `entries` (`:171-182`). It adds a new `StorageCall::StoriesSceneEntries` (`core.rs:64` area). That call goes in the no-kind arm of `changes.rs:170-176`.

```sql
SELECT e.from_message, e.to_message, e.story_time, e.place, e.took_part, e.others, e.what_happened
FROM scene_entries e
JOIN message_swipes s ON s.id = e.swipe_id
JOIN messages m ON m.id = s.message_id AND s.swipe_index = m.active_swipe_index
WHERE m.game_id = ?1 AND m.id < ?2            -- before_bound(before), stories.rs:242
ORDER BY m.id, e.position
```

- The row mapping stays inline in `stories.rs`. No model file is needed (**recommended**).

---

## 2. Domain types

New file `src/domain/model/scene_entry.rs`. It replaces `src/domain/model/story_memory.rs` (25 lines) and its `story_memory_tests.rs`.

```rust
pub struct SceneEntry {            // Debug, Clone, PartialEq, Eq
    pub from: u64,                 // Message id, first Message of the scene (engine-set range)
    pub to: u64,                   // Message id, last Message of the scene (engine-set)
    pub time: String,
    pub place: String,
    pub took_part: Vec<String>,
    pub others: Vec<String>,
    pub what_happened: String,
}

/// The scene entries of the active Swipes before a place, oldest first.
pub struct SceneEntries(Vec<SceneEntry>);
impl SceneEntries {
    pub fn new(entries: Vec<SceneEntry>) -> Self;
    pub fn entries(&self) -> &[SceneEntry];
    pub fn last(&self) -> Option<&SceneEntry>;      // "the last scene entry written before"
    pub fn covers_up_to(&self) -> Option<u64>;      // to of the newest entry
    pub fn uncovered<'a>(&self, dropped: &'a [StoryRow]) -> &'a [StoryRow];
    pub fn with_written(&self, written: &[SceneEntry]) -> SceneEntries; // the list the narrator sees after the writer
}
```

- `uncovered` keeps the binary search of `StoryMemory::uncovered` (`story_memory.rs:18-24`). The mark is `covers_up_to()`, which replaces `covers_up_to` of the base (`:13`). The five plain cases in `story_memory_tests.rs` move to `scene_entry_tests.rs` (rule 2 of "The one rule": a pure domain function). Register the module in `src/domain/model/mod.rs` (`:24`, `:65`).
- The entry has no serde derive. Storage writes the two name lists as JSON, and the answer parser has its own wire struct (§4).
- The text of one entry, `render_scene_entry(&SceneEntry) -> String`, goes in `src/application/prompting/builders/sections.rs`, beside `render_known_character_entry`. The writer prompt (the previous entry) and the block (§5) share it. The form is that of `render_scene` (`memory_test.py:358-364`): `[#from-#to] time. place. Took part: …. Others: ….` then `what_happened` (**recommended**).

---

## 3. The Job step: the split, the calls, the failed call, the second try

### The flow that stays (`src/application/pipeline/job.rs`)

`generate` (`:183-219`) calls `narrate` and then `save_text`. `narrate` (`:253-324`) loads once, plans the cut (`:298-300`), runs the writer inside the step, and calls the narrator. `write_story_memory` (`:329-347`) and `story_memory_call` (`:352-359`) also stay.

### What changes in `narrate`

1. Replace `let previous = start.story_memory.as_ref();` (`:281`) with a read: `let memory = self.scene_entries(before)?`. Use the same `before` as the `history` read (`:271-277`): `None`, or `Some(tail.last.id)` for `Place::NewSwipeOnLast`. A failure gives `Record::Story` through `Self::read`.
2. `planned.story_memory` becomes `&memory` (§5).
3. The leaving range: `let first_uncovered = dropped.len() - memory.uncovered(dropped).len();` (`:302`), then `cut.dropped_within_one_space(first_uncovered)` (`:303-305`) as today. A cut still writes at most one history space (Scenario 46.13, the old-Game row of the failure table).
4. The split: `cut.writer_calls(range, SCENE_WRITER_CALL_TOKENS) -> Vec<Range<usize>>`, a new pure method on `HistoryCut` (`src/domain/model/history_cut.rs`). It reuses the loop of `dropped_within_one_space` (`:75-86`) with a token bound as a parameter. So both share one private `dropped_within(first, end, limit)`, and `dropped_within_one_space` stays a call to it with `history_space`. A single Message over the bound is a call alone, as at `:80` (`end > first`). The counts are the cut's own `entry_tokens` (`:17`), which are `part_tokens` (the estimate plus 4 per Message). Plain cases go in `history_cut_tests.rs` (rule 2).
5. The cast: the Persona name first, then the World Characters' names in id order (`CharacterCard::in_id_order`, as the assembler uses). The Characters present are not used. The memory tool orders the cast `[persona] + cast` (`memory_test.py:439`).

### `write_story_memory`: the call sequence

```text
written = []
for each range in calls:
    self.writer.step(Step::WritingStoryMemory)?      // the Job id check; a Stop between calls ends the Job
    previous_entry = written.last() or memory.last()
    input = StoryMemoryHookInput::new(previous_entry, &cast, persona, &history[range])
    match story_memory_call(settings).and_then(|call| writer.run(&input, &call)):
        Ok(entries) => written.extend(entries)
        Err(e)      => warn!; break                   // the failed-call rule: this call and the later ones wait
self.writer.step(kind.step())?
return written
```

- It returns `Vec<SceneEntry>`, the entries this Job wrote. `narrate` returns `(text, written)`, and the narrator gets `memory.with_written(&written)`. `save_text` (`:361-370`) loses `start.story_memory = memory` (`:368`). It passes `&written` to `JobWriter::save_text`.
- The loop must `break` at the first failed call. The entries must cover the Messages with no gap, because the cover mark is the `to` of the newest entry (decision 2).
- `story_memory_call` uses `connection.max_tokens_for(Some(SCENE_WRITER_ANSWER_TOKENS))` (`:355`). It keeps the Narration slot (`PresetType::System`) and `AGENT_STORY_MEMORY` (`Role::StoryMemory`, agent name `memory`; `ports/llm_provider.rs:15`, `domain/model/agent.rs:44`).
- The constants go in `src/application/prompting/token_budget.rs`, where `STORY_MEMORY_TOKENS` was (`:93-97`): `SCENE_WRITER_CALL_TOKENS: usize = 4_000` and `SCENE_WRITER_ANSWER_TOKENS: u32 = 2_000` (**recommended**).

### The hook input (`src/domain/model/agent.rs:95-133`)

```rust
pub struct StoryMemoryHookInput<'a> {
    pub previous_entry: Option<&'a SceneEntry>,
    pub cast: &'a [String],
    pub persona_name: &'a str,
    leaving: &'a [StoryRow], oldest: u64, newest: u64,
}
```

- `new` keeps its `Option` (empty leaving gives no input) and gets the new parameters. `src/domain/model/agent_tests.rs:42,52` change in a mechanical way.

### The trait (`src/application/agents/trait_def.rs:10-17`)

- `fn run(&self, input: &StoryMemoryHookInput, call: &CallSettings) -> Result<Vec<SceneEntry>, EngineError>`. On `Ok` the vector is never empty, and the entries already carry the ranges the engine set.
- The wiring stays: `bootstrap/wiring.rs:113`, `test_support/context.rs:144` and `JobDeps.story_memory_writer` (`job.rs:46`).

### The second try: inside the Agent (`src/application/agents/memory/agent.rs`)

- It follows the pattern of `quantify_room_with_llm_call` (`quantifier/utils/orchestration.rs:94-140`) and of `MAX_OPTIONS_ATTEMPTS` (`options/utils/orchestration.rs:53`). `const MAX_ATTEMPTS: u32 = 2`, and each try is `recorder.complete_checked(AGENT_STORY_MEMORY, &prompt, call, check)` with the same prompt. The second try runs when the first returns `Err`. The error of the second try is returned.
- The `check` closure does the whole judgement, so a refused answer is recorded as a failed call (`llm_recorder.rs:61-63`). It refuses an empty answer, `stopped_at_limit` (as `check_answer` does, `agent.rs:34-51`, without the char cap), an answer that does not parse, and an answer whose `from` numbers fail the check. It stores the parsed entries in a captured `Option`, so the answer is parsed once.
- The parse failures use `LlmFailure::ParseError { raw_response, expected_format: "scene entries" }` (`src/error.rs:47-51`), so no new variant is needed (**recommended**).
- A Stop during the tries takes effect at the next `writer.step` of the Job (§3 loop), as the ticket says.

### The prompt (`src/application/agents/memory/prompt.rs`)

- A full rewrite to the `writer_system` and `writer_user` of `candidates/scene_entries.json`, as Rust constants, with `{cast}`, `{persona}`, `{previous_entry}`, `{first}`, `{last}` and `{entries}` filled. When there is no previous entry, the text is `(none: these are the first Messages of the story)` (`memory_test.py:440`).
- The transcript form `[#id Label]\ntext` stays (`prompt.rs:74-81`). It numbers the Messages by Message id.
- `words_for` and the response-length parameter go (`prompt.rs:21-32`): the new prompt asks for 60 to 150 words per entry, whatever the ceiling.

---

## 4. The parse step and the check

- New file `src/application/agents/memory/parser.rs`, registered in `memory/mod.rs`.
- **The JSON body.** Trim the text. Take the slice from the first `{` to the last `}`. That accepts a whole-body Markdown fence (as `parse_scene_answer` does, `memory_test.py:336-339`) and text around the object (the ticket wording, see §10). Then run `serde_json::from_str` into a private wire struct.

```rust
#[derive(Deserialize)] struct Answer { entries: Vec<AnswerEntry> }
#[derive(Deserialize)] struct AnswerEntry {
    from: u64, time: String, place: String,
    took_part: Vec<String>, others: Vec<String>, what_happened: String,
    // `to` is not read: serde ignores unknown fields
}
```

- **Refused:** JSON that does not parse, a missing field or a field of the wrong kind (the tool's field check, `memory_test.py:347-354`, minus `to`), and an empty `entries`.
- **The `from` check (decision 2).** Each `from` must be the id of a Message of the call (`input.leaving()` ids; ids can have gaps after a delete). The `from` numbers must go strictly up. If not, the answer is refused. Then the engine sets the first `from` to `input.oldest()`, each `to` to the id of the leaving Message just before the next scene's `from`, and the last `to` to `input.newest()`.
- The result is `Vec<SceneEntry>`. The parser holds no prompt and no recorder, and only the writer seam cases test it (the ticket puts the cases on the Agent).

---

## 5. The reserve in the cut and the block render

All in `src/application/prompting/assembler.rs`.

### The context

- `PromptContext.story_memory: Option<&'a str>` (`:48`), `with_story_memory` (`:148`), `LayerRenderer.story_memory` (`:178`) and `NarrationValues.story_memory` (`narrator.rs:42`, passed at `:180`) become `&'a SceneEntries` (an empty list when there is none).
- The opening is `self.history.first()`. The Job reads the whole story before the place (`STORY_LIMIT`, `job.rs:271-277`), so the first entry is the first Message of the story, and no new read is needed (**known**: `stories.rs:17`, `job.rs:276`).
- A caller of the assembler other than the Job (the debug prompt view, if any) passes an empty list. Check with `rg with_story_memory` in phase 2.

### The reserve: `cut_of` (`:247-260`)

```rust
let unreserved_space = budget.history_space(system_tokens, self.fixed_tokens());
let unreserved = self.cut_with_reserve(system, entries, budget, system_tokens, 0);
if self.story_memory.entries().is_empty() && unreserved.blocks().is_empty() {
    return (unreserved, 0);                 // no block before the first cut
}
let memory_space = unreserved_space / 2;    // decision 1
(self.cut_with_reserve(system, entries, budget, system_tokens, memory_space), memory_space)
```

- The second pass gives the history `min(room − memory_space, max_history_tokens)` (`token_budget.rs:56-62`). With room ≥ 1.5 × the history size, the history keeps its full size and the list gets half of it. With room ≤ the history size, the split is even (`room − room/2` and `room/2`). Between the two, the list keeps its half of the history size and the history gets the rest. See §10 note 3.
- At the trial size (32,768-token window for OpenCodeGo, `llm_backend.rs:72`, answer 2,048, history 16,000), the reference trial's last narrator call had a system part of about 1,600 tokens and fixed blocks of about 1,700. That leaves room of about 26,000, so the history keeps 16,000 and the list gets 8,000 (**inferred** from `tmp/ltm09/redmist_julian/trial-1/calls.jsonl`).
- The condition is the base condition (`:255`) with "no entries" in place of "no memory". The cut that the Job plans with the old list and the cut of the call with the new list therefore reserve the same space. The case `assembler-plan-cut-matches-the-assembly` keeps that true.
- `story_memory_layout_tokens` (`:264-274`) goes. The space now holds the whole block, tags included. `STORY_MEMORY_TOKENS` and `STORY_MEMORY_CHARS` go from `token_budget.rs:93-97`.
- `CutPrompt` (`:193-198`) gets `memory_space: usize`. `render_and_fit` (`:201`) passes it to `render_prompt`. The overflow guard keeps rendering with no block and subtracting the reserve (`:291-296`).

### The block render: `render_story_memory_layer` (`:550-557`)

```text
<StoryMemory>
<Opening>
{first Message text, word for word}
</Opening>

<SceneEntries>
Past scenes, oldest first. Each entry … may have changed since.   (list_head of the candidate)

{render_scene_entry(e)} joined by blank lines, oldest first
</SceneEntries>
</StoryMemory>
```

- The block renders when `memory_space > 0`, which is the same rule as the reserve. `<SceneEntries>` is left out when no entry fits or none exists, for example after a first cut whose writer failed. That matches `render_scene_memory` (`memory_test.py:367-380`).
- **The fit.** Drop the oldest entry while `estimate_tokens(block) + separator > memory_space`. Use the measured separator cost of `join_layers`, as `story_memory_layout_tokens` did. Dropped entries stay stored.
- The block stays the last layer before the history (`render_prompt`, `:343`).
- An opening larger than the space is kept whole with no entries (**recommended**; this is rare, and no decision covers it).

---

## 6. What of the base goes, and what stays

**Goes:**
- `SwipeState.story_memory` (`swipe_state.rs:11,30-32,49`). Old saved Games keep loading, because `SwipeState` has no `deny_unknown_fields`. Also the field literals in `generation_tests.rs:27`, `tail_tests.rs:21`, `game_rules_tests.rs:55` and `tests/storage/story_storage.rs:41`.
- `src/domain/model/story_memory.rs` and `story_memory_tests.rs`, replaced by `scene_entry.rs` and `scene_entry_tests.rs`.
- The base prompt (`prompt.rs` whole), `StoryMemoryPrompt::words_for`, and `check_answer`'s char cap (`agent.rs:43`).
- `STORY_MEMORY_TOKENS` and `STORY_MEMORY_CHARS` (`token_budget.rs:93-97`), and `story_memory_layout_tokens` (`assembler.rs:264-274`).
- `start.story_memory` reads and writes in `job.rs:281,368`.

**Stays** (with the new types): `BeforeNarrationAgent`, `StoryMemoryHookInput`, `StoryMemoryWriter` (the name stays: it writes the story memory), the wiring (`wiring.rs:113`), `Step::WritingStoryMemory` (`generation_status.rs:233`) and its status text (`endpoints.rs:80`), `Role::StoryMemory`, and `story_memory_call`.

**Copy that changes:** the failure-banner tail `"keeping the previous story memory"` (`headers.rs:34`). With scene entries, the earlier entries stay and the missed Messages wait, for example `"keeping the earlier scene entries"`. The spec rewrite (phase 4) fixes the words, and phase 3 changes the code (**recommended**).

---

## 7. `scripts/trial.py`

- `story_rows` (`trial.py:262-272`) also selects `s.id as swipe_id`.
- New `scene_entry_rows(conn, game_id)` reads `scene_entries` joined to the active Swipes of the Game, ordered by `m.id, position`. `story_rows` callers that build the pack (`:755`) attach `entry["scene_entries"]`, the entries written with that Swipe, so `story.json` carries them.
- `story_memory_of(entry)` (`:978-979`) returns `entry.get("scene_entries") or []`. `memory_lines` (`:982-989`) shows the entries written at that cut, in the `render_scene` form. It also shows the list size after the cut, as a count and in tokens of all active entries up to that entry, which the Check needs. An entry with none shows nothing.
- `scripts/tests/test_trial.py`: add `"scene_entries"` to `READ_TABLES` (`:30-37`). Rewrite `entry_with_memory` and `StoryMemoryLineTests` (`:580-620`) for entry lists. Add one case that inserts two Swipes with entries and checks that only the active Swipe's entries reach the story rows.

---

## 8. Tests per tier (test first in each phase)

### Seam cases (each with one `// CASE` tag)

**The writer Agent:** `src/application/agents/memory/agent_tests.rs`, a full rewrite. The current 13 cases (`:93-331`) go.
- `memory-result`: a good answer gives its entries, and the ranges cover first..=last with no gap.
- `memory-ranges-from-starts`: the model's wrong `to` (a gap and an overlap) is ignored, and the ranges come from the scene starts.
- `memory-first-from-set`: a first `from` after the first Message is set to the first Message.
- `memory-fenced-answer`: a fenced answer, and text around the object, both parse.
- `memory-prompt`: the cast, the Persona line, the previous entry in the entry form, and each leaving Message with its id and label.
- `memory-prompt-first-cut`: with no previous entry, the prompt shows the `(none: …)` text.
- `memory-call-settings`: the call keeps the call settings it gets.
- `memory-stopped-at-limit [failure]`: a cut-off answer, on both tries, fails.
- `memory-unparseable [failure]`: a non-JSON answer, a missing field, or an empty list, on both tries, fails.
- `memory-from-outside-call [failure]` and `memory-from-not-rising [failure]`.
- `memory-transport-failure [failure]`: two failed calls fail the writer.
- `memory-second-try`: a first try that fails and a second that succeeds give the second try's entries. The two prompts are equal.
- `memory-call-record`, `memory-call-record-failure`: the two call-record cases, which also cover both records of a failed then good pair.
- `ScriptedModel` already scripts `cut_off`, `failed` and replies per role (`test_support/scripted_model.rs:137,159,165`). Add a JSON answer helper in the test file.

**The `PromptAssembler`:** `src/application/prompting/assembler_tests.rs`. Rewrite the six story-memory cases (`:1528-1660`):
- `assembler-story-memory-before-history`: the block is `<Opening>` with the first Message word for word, then `<SceneEntries>` with the head line and the entries oldest first, as the last block before the history.
- `assembler-story-memory-absent`: no entries and a history that fits give no block.
- `assembler-story-memory-opening-only`: a cut with no entries gives `<Opening>` and no `<SceneEntries>`.
- `assembler-story-memory-drops-oldest`: a list over its space drops its oldest entries, and the block fits the space.
- `assembler-story-memory-space-large-window`: the history keeps its full size, and the list gets half of it.
- `assembler-story-memory-space-small-window`: the history and the list split the room evenly. The ticket names this case.
- Keep and adapt `assembler-plan-cut-matches-the-assembly`, `…-size-does-not-move-the-cut` (a short list and a full list leave one history start), `…-fits-the-limit` and `assembler-history-cut-reserves-the-story-memory`.

**The `Narrator`:** `narrator_tests.rs:800-850`. `narrator-story-memory-every-kind` and `narrator-plan-cut-matches-the-call` change only in a mechanical way (the new `NarrationValues` type).

**The storage contract:** `tests/storage/story_storage.rs`.
- `story-scene-entries-with-swipe`: entries written in the unit of the Swipe read back in order, with every field.
- `story-scene-entries-active-swipes`: the entries of an inactive Swipe, and of a Message at or after `before`, are not read.
- `story-scene-entries-delete`: a delete of the last Message removes its entries.
- `story-scene-entries-other-game [failure]`: a write that names another Game's Message is refused.
- `story-scene-entries-injection [failure]`: `Storage::with_failure` on `TxWrite::InsertSceneEntries` leaves no Swipe row and no entry.
- Migration: one case in `plumbing_tests.rs`.

**Plain domain cases (rule 2):** `scene_entry_tests.rs` (the cover mark and `uncovered`), and `history_cut_tests.rs` (`writer_calls`: one call for a small range, a split at the bound, one Message over the bound alone, and no call past the range end).

### Scenarios: `docs/specs/story_memory.md`, rewritten, then `tests/http/story_memory.rs`

- Rewrite 46.1 to 46.13 for the scene entries. 46.11 keeps the 700-token response length and drops its words clause.
- Add new scenarios after 46.13:
  - 46.14: one cut over 4,000 tokens of Messages makes several writer calls, one after the other, before the narrator call. Each later call holds the last entry of the call before it.
  - 46.15: a call in the middle of a split fails on both tries. The entries of the earlier calls are saved and shown, and the next cut starts at the Messages of the failed call.
- Retry (46.8), swipe switch (46.9), delete (46.10), Impersonate (46.7) and the event Narration (46.12) stay, rewritten.
- 46.5 and 46.6 become "both tries fail".
- The test helpers (`memory_block`, `previous_memory`, `shown_story_memory`, `story_memory.rs:113-210`) read `<Opening>`, `<SceneEntries>` and `<PreviousEntry>` instead.
- **Other tests that script the writer with plain text, which now fails to parse:** `tests/http/races.rs:811,893` (the stop during the writer call: also check its call counts with the second try) and `tests/http/support/app_wiring.rs:194`. Give each one a JSON answer.

---

## 9. The build phases (each under ~120k context, run one after the other)

The coupling chain is trait → `Job` → `NarrationValues` → assembler, so the prompt side must land before the writer side. Between phase 2 and phase 4, the scenarios in `tests/http/story_memory.rs` that read the narrator's block are red, because no code writes entries until phase 3. Each phase's done-check names its tests. Nobody commits between phases.

**Phase 1: domain, storage and trial pack (all green).**
- Files: `domain/model/scene_entry.rs` and its tests (new; `story_memory.rs` stays until phase 3), `domain/model/mod.rs`, `history_cut.rs` and `history_cut_tests.rs` (`writer_calls`), `storage/utils/plumbing.rs` and `plumbing_tests.rs` (step 35), `storage/transaction.rs`, `storage/changes.rs`, `storage/core.rs`, `storage/stories.rs`, `tests/storage/story_storage.rs`, `scripts/trial.py` and `scripts/tests/test_trial.py`.
- Done: `python build.py clippy`, `test-pattern "scene_entr"`, `test-pattern "history_cut"`, `test-pattern "plumbing"`, `test-pattern "story" --test-target storage`, and `python -m unittest scripts.tests.test_trial`.

**Phase 2: the prompt side.**
- Files: `prompting/assembler.rs` (the context type, the `cut_of` reserve, the block render and fit, `CutPrompt`), `prompting/builders/sections.rs` (`render_scene_entry`), `narration/narrator.rs` (`NarrationValues`), `job.rs` (only `narrate`: read `SceneEntries` from storage with `before`, pass it to the narrator, and compute `first_uncovered` from it), `assembler_tests.rs` and `narrator_tests.rs`.
- The base writer still runs in this phase, and its text still goes to `SwipeState`, but the prompt no longer shows it. Phase 3 removes it.
- Done: `clippy`, `test-pattern "assembler" --test-target lib`, `test-pattern "narrator" --test-target lib`. Known red: the narrator-block scenarios of `story_memory`.

**Phase 3: the writer and the Job write side, and the base goes.**
- Files: `agents/trait_def.rs`, `domain/model/agent.rs` and its tests (the hook input), `agents/memory/{agent.rs, prompt.rs, parser.rs (new), mod.rs, agent_tests.rs}`, `job.rs` (the split loop, `write_story_memory`, `story_memory_call`, `save_text`), `generation/gate.rs` (`JobWriter::save_text` and the two write helpers), `token_budget.rs` (the constants), `swipe_state.rs` and the four field literals, `story_memory.rs` and its tests deleted, and `headers.rs:34` copy.
- Done: `clippy`, `test-pattern "memory" --test-target lib`, `test-pattern "agent" --test-target lib`, and `python build.py guardrails` (test structure and seam tags).

**Phase 4: the spec and the scenarios.**
- Files: `docs/specs/story_memory.md` (46.1 to 46.15), `tests/http/story_memory.rs`, `tests/http/races.rs`, and `tests/http/support/app_wiring.rs`.
- Done: `clippy`, `test-pattern "story_memory"`, `test-pattern "races"`, `python build.py integration`, and `python scripts/validate_feature_spec.py` (each scenario has a test).

**After phase 4 (the coordinator, ticket steps 3 to 6):** `/verify-gate` (`gate.rs` and `job.rs` changed), `CONTEXT.md` "Scene entry", and the docs: `prompt_system.md`, `agent_system.md`, `game_flow.md` (the writer step's split), `storage.md` (the new table), and the `dashboard.md` 39.4 row if its words change. Then `/simple-after-plan`, the full `python build.py`, and the 100-Cycle trial.

---

## 10. Decisions that do not fit the code

None blocks the build. Three small mismatches:

1. **`GameWriter::save_text` is `JobWriter::save_text`** (`gate.rs:744`, `:793`). `GameWriter` (`:298`) has no `save_text`. This is a naming slip only, and the design is unchanged.
2. **The tool's parse tolerance.** The ticket says `parse_scene_answer` accepts "a Markdown code fence and text around the one JSON object". The tool accepts only a fence that wraps the whole body (`memory_test.py:336-339`, `re.match(r"^```(?:json)?\s*(.*?)\s*```$")`), and it refuses text around the object. The plan follows the ticket wording: first `{` to last `}`, which also covers the fence. The tool also requires `to` as an int (`:347`). The engine does not, because it ignores `to` (decision 2).
3. **"The history keeps its full size" with a large window (decision 1).** As written, the formula gives that only when the room is at least 1.5 × the history size. Between 1 × and 1.5 ×, the list keeps half the history size, and the history gets less than its full size and more than an even split. This is a consequence of the formula, not a conflict, and the trial setup is in the full-size zone (§5, inferred). The `PromptAssembler` cases pin both ends, and the plan does not add a case for the middle.

**Open choices left to the implementer (recommended answers above):** the `ChangeKind` of the new write (`STORY`), the `LlmFailure` for a refused answer (`ParseError`), where the constants and `render_scene_entry` live, the banner words, and an opening larger than the space (kept whole).
