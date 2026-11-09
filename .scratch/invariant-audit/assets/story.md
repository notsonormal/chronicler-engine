# Story invariants: relied on, not enforced

Theme: Messages, Swipes, Swipe state, the end of the story.
Read-only investigation. Every claim cites the file it comes from. Terms follow `CONTEXT.md`.

Scope note: the write paths of the story are `Tx` (`adapters/driven/storage/transaction.rs`) reached
through `GameWriter`/`JobWriter` (`application/generation/gate.rs`). The busy state and the Job id
check are the gate theme; this report names them only where the story depends on them.

---

## story-01: The `swipe_index` values of a Message are `0..n` with no gap, and a Swipe's position in the loaded list equals its `swipe_index`

**Relied on at**

- `gate.rs:933` — `swipe: tail.last.swipes.len()`. The next Retry writes the list length as the new
  `swipe_index`. That is only the next free index if the stored indexes are contiguous.
- `gate.rs:542` — `if index >= last.last.swipes.len()` bounds the requested index by the loaded list
  length, and `gate.rs:545-553` writes that same number as the DB `swipe_index`
  (`tx.set_active_swipe(at)`). A gap makes the list position and the stored index two different numbers.
- `adapters/driven/storage/mappers/message.rs:69-72` — the mapper pushes the rows (ordered by
  `swipe_index ASC`, `stories.rs:311-322`) straight into `Message::swipes`, so position `i` is
  assumed to be `swipe_index i`.
- `stories.rs:19-24` — the story SQL joins `s.swipe_index = m.active_swipe_index`, and
  `stories.rs:264` reads `COUNT(*)` as `swipe_count`. `view_models.rs:150-181` turns
  `swipe_count` and `active_swipe_index` into the prev/next Swipe buttons and the `k / n` counter.

**Write paths that could violate it**

- `transaction.rs:183-204` `insert_swipe` takes `at.swipe` from its caller and writes it as is. It
  checks only that the Message belongs to the Game (`utils/records.rs:15-26`).
- Today's callers pass `0` (`gate.rs:636-641`, `826-831`, `911-916`) or `swipes.len()`
  (`gate.rs:933`), so they preserve contiguity. Nothing stops a third one.
- DB: `utils/plumbing.rs:685-698` declares `swipe_index INTEGER NOT NULL` with
  `UNIQUE(message_id, swipe_index)` only. No `CHECK`, no `DEFAULT`, and no delete path for a single
  Swipe exists, so a gap can only come from a code bug, a hand edit, or an older version.

**Enforcement: NONE.** The only constraint is the uniqueness of the pair. Nothing ties the stored
index to the position in the list, and nothing forces the set to start at 0 with no hole.

**Consequence if violated** (with `swipe_index` 0 and 2, `active_swipe_index` 2): the story window
read fails hard at `stories.rs:229-238` (`"message {} has no swipe"`) once the active index names no
row, so the dashboard screen load answers 500. Before that, `mappers/message.rs:72`
(`ensure_valid_swipe_index`) silently rewrites the in-memory index to 0, so a swipe switch to the
repaired index writes 0 over the stored 2 (`transaction.rs:158-167`) and the story quietly moves to
the wrong text. The next Retry collides with `UNIQUE(message_id, swipe_index)` and ends with the
save-failure line.

**Likelihood**: only a code bug, a hand edit, or data from an older version. Both current insert
sites use `0` or `len()`.

**Epistemic status: VERIFIED** (every write site of `swipe_index` read).

**Suggested enforcement point**: give `Tx::insert_swipe` the Message and no index (it computes
`MAX(swipe_index) + 1`, or refuses when `at.swipe` is not that value), and make the mapper reject
`db_swipe.swipe_index != position`.

---

## story-02: `messages.active_swipe_index` names a Swipe row that exists

**Relied on at**

- `stories.rs:19-24` — every story entry's text and header come from a `LEFT JOIN` on the active
  index. A missing row gives `COALESCE(s.text, '')`: an entry that renders as an empty Message.
- `stories.rs:150-168` `has_text` joins on the active index too, so a broken entry counts as textless.
- `stories.rs:264-265` and `stories.rs:129-140` cast the column with `as usize` (`i64` → `usize`), so
  a negative stored value becomes a huge index that matches nothing and raises nothing.
- `view_models.rs:150-181` renders the active index and the count to the player as `k / n`.

**Write paths that could violate it**

- `transaction.rs:158-167` `set_active_swipe` checks only `require_message_of_game`. It never checks
  that `(message_id, swipe_index)` names a row.
- Callers: `gate.rs:545-553` (`switch_swipe`, checked by `last.last.swipes.len()`), and
  `gate.rs:936-937` (insert the Swipe, then activate it, always valid).
- DB: `utils/plumbing.rs:674-683` — `active_swipe_index INTEGER NOT NULL DEFAULT 0` with no FK and no
  cross-table constraint. A hand edit or an older version can set any value.

**Enforcement: READ-SIDE-ONLY, and the three readers disagree.** `mappers/message.rs:72` repairs the
index to 0 in memory (`message.rs:148-155`). `stories.rs:19` hides it behind an empty string.
`stories.rs:229-238` turns it into an `Internal` error. `stories.rs:338` returns `Ok(None)`.
`Message::set_active_swipe` (`message.rs:117-122`) also ignores an out-of-range index without a
report. No write refuses the violation.

**Consequence if violated**: three different failures for one bad row. The dashboard screen load
answers 500 (story window). The prompt history (`stories.rs:183-207`, used by `Job::window_before`)
silently holds an empty entry, so the model is prompted with a blank turn and the story drifts. A
swipe switch or a later save can silently persist the repaired `0` over the stored index, so the
player loses which Swipe was active.

**Likelihood**: a hand edit or a code bug. A player cannot produce it through the app today.

**Epistemic status: VERIFIED** for the readers and the writers; INFERRED for "no app path produces
it" (I traced `set_active_swipe` and both callers, not every future one).

**Suggested enforcement point**: `Tx::set_active_swipe` refuses when
`SELECT 1 FROM message_swipes WHERE message_id = ? AND swipe_index = ?` finds no row, and the mapper
rejects instead of repairing.

---

## story-03: When the story holds more than one Message, `Tail::state_before_last` is `Some`

The start state of the last Message (the active Swipe state of the Message before it) must be
readable, so a Retry starts where that Message started.

**Relied on at**

- `generation.rs:329-334` `redo` — `MessageKind::Narration if tail.state_before_last.is_none()` means
  "this is the opening", which drops the guide (`guide: None`) and re-applies nothing.
- `tail.rs:37-43` `start_at` — with `state_before_last` absent it answers
  `SwipeAt { message: self.last.id, swipe: 0 }`, the last Message's *own* first Swipe.
- `docs/specs/swipe_new.md:401` (I.7) promises the opposite outcome: a missing start state is a 500
  with the missing-data line.

**Write paths that could violate it**

- No write creates the violation directly. `stories.rs:80-90` builds the value:
  `Self::load_swipe_state(conn, at)?.map(|state| (at, state))`. `stories.rs:328-344` answers
  `Ok(None)` when no row matches `(message_id, swipe_index)`. So the read turns "the previous
  Message's active Swipe has no state row" into "there is no state before the last Message".
- The row can be absent through story-02 (the previous Message's active index names no Swipe), a
  hand edit, or a bug. No writer checks the pair.

**Enforcement: READ-SIDE-ONLY.** The `map` at `stories.rs:86-90` silently converts a broken record into
the opening case; `Job::start_state` (`job.rs:456-466`) then loads `order.start`, which now names the
last Message itself, and succeeds. The designed 500 never happens.

**Consequence if violated**: a Retry of a Narration starts from the Narration's **own post-analysis**
state instead of its start state. The Game rules apply movement again from the room the Narration
ended in (double movement), the Characters present are re-derived from the wrong scene, and the
guide is dropped. The player gets a 200 and a contradicting story instead of the 500 that I.7
promises.

**Likelihood**: a hand edit or a code bug; it also needs story-02 to have happened, or the row to be
absent. A player cannot do it.

**Epistemic status: VERIFIED** (the reader, `redo`, `start_at` and every writer of `message_swipes.state`
read).

**Suggested enforcement point**: `Tail` carries `Option<SwipeAt>` plus an explicit start-state result
that separates "the opening" from "the record is missing"; `read_tail` returns an error when the
Message before the last has no active Swipe state.

---

## story-04: A Message's `kind` agrees with its `message_type`

`TypedInput`/`ImpersonateInput` ⇔ `Input`; `Narration`/`EventNarration` ⇔ `Narration`.

**Relied on at**

- `tail.rs:65-67` `awaits_narration` decides the next request from `message_type`
  (`AutoplayRun` reads it through `JobRequest::next_in_autoplay`, `generation.rs:168-176`).
- `generation.rs:329-341` `redo` decides the generation from `kind`.
- `tail.rs:50-58` `retrigger_record` mixes both: `kind` allows it, the state supplies the record.
- `assembler.rs:449-457` maps `message_type` to the prompt role, so a `TypedInput` on a
  Narration-typed Message sends the Persona's words to the model as the narrator's.

**Write paths that could violate it**

- `generation.rs:95-131` and `utils/opening.rs:21` are consistent pairs today.
- `Message::new(message_type, kind, swipe)` (`message.rs:66-76`) and the struct itself
  (`message.rs:55-61`, public fields) accept any pair.
- `mappers/message.rs:49-50` parses the two columns independently, so a hand edit or an older row can
  hold any pair.

**Enforcement: DOC-ONLY.** `docs/diataxis/reference/game_flow.md` states that
`GenerationOrder::write` "is the one place that decides the Message kind, the message type and the
maker". Nothing in the type system or in a check holds that sentence, and `MessageKind` and
`MessageType` are two independent fields.

**Consequence if violated**: the two readers disagree. An Input-typed Message whose kind is
`Narration` makes `awaits_narration` true, so an Autoplay run sends a Continue, while a Retry redoes
a Narration on an Input Message and appends a Narration-kind Swipe under it. The prompt role flips,
so the model reads the Persona's text as its own narration. No panic, no error: a wrong story.

**Likelihood**: a code bug or a hand edit only.

**Epistemic status: VERIFIED** for the reliance and the two write sites; INFERRED that no other code
constructs a `Message` (I grepped `Message::new`, `Message::from_db` and struct literals).

**Suggested enforcement point**: derive `message_type` from `kind` (a method on `MessageKind`), so
only `kind` is stored and the pair cannot disagree.

---

## story-05: A new Swipe of a Retry is appended to the Message that is still the last one

**Relied on at**

- `gate.rs:924-941` `write_new_swipe` — `message: tail.last.id`, written inside the transaction with
  only `require_message_of_game` (`transaction.rs:185`). Nothing inside the write asks whether that
  Message is still the last one.
- `job.rs:288-293` — `save_text` reads the `Tail` *before* it calls `self.writer.save_text(...)`, so
  the id it writes comes from a read taken outside the registry hold.
- `stories.rs:80-90` — `read_tail` derives the current state from the second-newest Message's active
  Swipe. A Swipe made active on an older Message therefore changes the start state of the last
  Message, retroactively.

**Write paths that could violate it**

- `gate.rs:936-937` (the write above).
- The invariant is protected today only by the busy rule: `gate.rs:353-358` `require_idle` refuses a
  second Job, and `gate.rs:535` (`switch_swipe`), `gate.rs:576` (`delete_last`) and the edit path
  refuse every other story write while the Game is busy. `accept` (`gate.rs:360-372`) reads the
  story end and installs the holder in one registry hold, so the plan's `last.id` is right at accept.

**Enforcement: PARTIAL.** The accept checks the id (`generation.rs:263-269` `require_last`), the gate
refuses competing story writes, but the write itself (`gate.rs:924-941`) skips the check, and the
tail it uses (`job.rs:290`) was read outside the hold that the write takes.

**Consequence if violated**: the Retry appends a Swipe to a Message that is no longer last and makes
it active. The current state of the Game then silently becomes that older Swipe's state
(`tail.rs:20-28`), so the next generation starts from a state the story has already moved past: wrong
room, wrong Characters present, wrong Triggers. The wrong data is stored, and nothing reports it.

**Likelihood**: only if the busy rule is relaxed or a gate bug lets a second writer in. Not reachable
by a player today.

**Epistemic status: VERIFIED** for the code; INFERRED for "no concurrent writer can reach the
window" (that rests on the gate, which I read but did not model).

**Suggested enforcement point**: `Tx` gets a "must still be the last Message" write
(`tx.append_swipe_if_last(tail.last.id, swipe)`), so the write refuses like
`require_message_of_game` does.

---

## story-06: A one-shot Trigger produces at most one event Narration in the story

**Relied on at**

- `utils/game_rules.rs:200-210` — a Trigger with `repeat == false` that is already fired is refused,
  so the normal path cannot fire it twice.
- `tail.rs:50-58` `retrigger_record` — a Retrigger is offered purely from
  `current_state().event_record` plus the kind of the last Message. It never asks
  `is_trigger_fired`.
- `utils/game_rules.rs:78` / `generation.rs:117-122` — the "fired" mark lives in the Swipe state of
  the *event Narration* itself, not in the state of the Message that fired the Trigger.

**Write paths that could violate it**

- `gate.rs:574-596` `delete_last` removes the event Narration and, through the cascade
  (`plumbing.rs:687`, the `ON DELETE CASCADE` FK), the Swipe that holds the fired mark. The firing
  Message's state keeps its `event_record`, so `retrigger_allowed` is true again
  (`tail.rs:60-62`).
- `generation.rs:251-262` builds the event order straight from that record, and
  `generation.rs:126-131` marks the Trigger fired again with no check.

**Enforcement: PARTIAL.** The Game rules check the fired mark; the Retrigger path skips the Game
rules entirely and trusts the event record. The record and the fired mark disagree as soon as the
event Narration is deleted.

**Consequence if violated**: delete the event Narration, then press Retrigger, and the same
irreversible event is written into the story a second time (and again after another delete). The
story contradicts itself; no record is corrupted and no error is raised.

**Likelihood**: a player, with two clicks (delete, then Retrigger). The flag reaches the last entry
at `templates.rs:36-39`, and the button renders at `templates.rs:25`.

**Epistemic status: VERIFIED** for the code paths; INFERRED for whether this rewind is intended
deletion semantics. It is a conflict between two stated rules, not a crash.

**Suggested enforcement point**: `Tail::retrigger_record` also refuses when the record names a
one-shot Trigger that the current state already marks fired.

---

## Checked and enforced

One line per invariant I checked and found held.

- **Every Message has at least one Swipe.** The Message and its first Swipe are inserted in one
  transaction (`gate.rs:826-832`, `gate.rs:911-917`, `gate.rs:635-641`), and a Message row with no
  Swipe row fails loudly on read (`mappers/message.rs:63-67`, a load error as `storage.md` promises).
- **Exactly one Swipe of a Message is active.** One column holds it (`plumbing.rs:674-683`); no second
  active mark exists anywhere.
- **A Message belongs to the Game a request names.** `require_message_of_game` guards every
  Message-naming write (`utils/records.rs:15-26`; called at `transaction.rs:160, 173, 185, 208, 220`),
  `insert_message` writes the `Tx`'s own Game (`transaction.rs:139-155`), the reader filters by Game
  (`stories.rs:129-140`), and `messages.game_id` is a real FK (`plumbing.rs:676`, pragma on at
  `db.rs:20`).
- **Message order is by id.** `insert_message` never names an id (`transaction.rs:139-155`) on an
  `AUTOINCREMENT` key, and every story read orders by `m.id` (`stories.rs:191`, `stories.rs:216`,
  `stories.rs:280-297`).
- **A Game keeps at least one Message.** The only `delete_message` call is `gate.rs:590`, behind
  `Tail::can_delete` (`tail.rs:70-72`, `gate.rs:583`), and creation always writes the opening
  (`gate.rs:627-645`).
- **A Retry, a Retrigger, a swipe switch and a delete name the last Message.** `generation.rs:263-269`
  (`require_last`) and `gate.rs:538-541`, `gate.rs:580-582`; the accept reads the story end and
  installs the holder in one registry hold (`gate.rs:360-372`).
- **The last entry of the story window is the story's last Message.** Entries and the end-of-story
  facts are read under one backend-mutex hold (`stories.rs:55-70`), newest first by id
  (`stories.rs:216`), then reversed.
- **Every Swipe has a Swipe state, and it parses.** `state TEXT NOT NULL` (`plumbing.rs:695`),
  written in the same INSERT (`transaction.rs:187-203`), and a parse failure is an error
  (`mappers/message.rs:36`).
- **The first Message of a Game is the opening Narration.** The create transaction writes the Game
  row and then the opening in one unit (`transaction.rs:378-392`, `gate.rs:627-645`), and `replace`
  creates a new Game rather than reusing one (`gate.rs:606-620`).
- **A Swipe's state is written at most twice per generation.** `SavedText` is taken by value
  (`gate.rs:865-871`), so the second write cannot be repeated.
- **The history cut drops a whole prefix of Messages and never splits one.** The kept range and the
  dropped blocks share one index space (`history_cut.rs:23-62`), the index is bounded by the same
  `len` (`history_cut.rs:54`), and the assembler slices `entries[kept_from..]`
  (`assembler.rs:197`, `assembler.rs:251-253`) from the same list it measured.
- **An event Narration's record and its header come from one source.** `generation.rs:126-133` writes
  the `MessageKind::EventNarration` record and the Swipe `event_header` from the same
  `EventRecord`; no separate writer exists.
- **A Game with zero Messages is answered, not panicked.** `Tail` is `None` (`stories.rs:69-96`), the
  plan refuses with `PlanError::Missing(Record::Story)` (`generation.rs:307-315`), and the view
  answers an `Internal` error (`view_query.rs:77-86`); the state cannot be reached through the app.
