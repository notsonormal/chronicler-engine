# Apply the rule to the server Game state

Type: research
Status: resolved
Blocked by: 01, 02

## Question

Apply the rule of ticket 01 to each machine of the server Game state. For each machine, what is its disposition and the evidence for it? Which product behaviour questions must the owner answer before the disposition holds?

## Machines

Catalogue rows. Ticket 02 can add rows.

- **Gate writes.** B1 Game status (only the parts outside `verify-gate`: the failure line after a restart, and the window where a Game is busy and has a failure line), B2 the offered set of the dock, B8 Swipe text against Swipe state.
- **Game rules.** B3 Trigger fire-once, B4 encounters, B9 movement and added Rooms.
- **Story records.** B5 story end (`Tail`), B6 Message type × Message kind × Swipe flags, B7 active Swipe index.
- **Model call health.** B10 Role health, E5 the LLM message retention window, C1 the Quantifier reading, C2 the Options attempt loop.
- B11 the change counters and the server id.
- **Game config (added by ticket 02).** B12 the Game config state: the narrator mode and the mode-bound preset slot ids.

## Behaviour questions already known (charting)

1. When the player edits the text of a Message, must the failure line and the Options failure go? They go when the player switches or deletes a Swipe. Today an edit keeps them.
2. After an edit, the Swipe state still holds the Room and the Characters of the old text. Must the edit run the Quantifier again, mark the state as old, or leave it?
3. When an event generation fails, its one-time Trigger stays unmarked. Must the next Action fire it again? Today it can.
4. A World edit that changes the order of the Triggers of a Character moves the "already fired" marks in saved Games to other Triggers. Must saved Games survive such an edit?
5. No code reads `currently_meeting`. Does a planned feature need it, or can it go?
6. A Role that did not run in the last 50 model calls shows "no calls", so its old failure leaves the banner. Must the banner keep the last result of each Role, however old? The comment at `view_query.rs:250` says that the window holds the newest call of each Role, which is not true.
7. A failed Quantifier looks like "nothing changed" and shows no line. Must it show a line, as a failed Options run does?

## Behaviour questions added by ticket 02 (scan 2)

8. B1 Game status. `ViewQuery::status` (`src/application/games/view_query.rs:198-211`) reads the busy registry, and then the `games.failure` column in a second read. When a Job is accepted between the two reads, a Game with a failure line reads as Idle, a state that never held. The next load corrects it, because the accept raises the busy-state number. Is one load with a wrong Idle acceptable, or must the status come from one consistent read?
9. B3 Trigger fire-once. When two Triggers match in one generation, `fired_trigger` takes the first of `evaluate_triggers` (`src/domain/model/utils/game_rules.rs:87-90`), which loops over a `HashMap` of Characters (`:166-173`). So the Trigger that fires can differ between two runs of the same story. Which Trigger must win?
10. B9 movement. The Quantifier prompt lists only the Rooms of the map (`src/application/agents/quantifier/utils/orchestration.rs:35-47`), never the added Rooms. `walk` looks a destination up by id (`game_rules.rs:136-145`), so a return to an added Room makes a new added Room each time (`attempt_movement`, `:124-134`). Must the Persona be able to return to an added Room?
11. B10 Role health. Health shows only whether the transport worked. An empty Narration is saved as a success row (`src/application/llm_recorder.rs:85`) before the Job fails with `EmptyText` (`src/application/narration/narrator.rs:151-156`). A Quantifier answer that does not parse also reads as a success. Must health count these as failures?
12. B10 Role health and E5 the log. A good answer fails the call when its log row does not save (`llm_recorder.rs:85`, and the prune at `src/adapters/driven/storage/llm_messages.rs:46` also returns `Err` after the insert). A failed call whose log row does not save only logs a warning (`llm_recorder.rs:72-76`), so health never sees that failure. Must a log write ever fail a generation?
13. A7 the Options failure (out of scope as a machine, but the behaviour is open). An accept clears the failure line but not the Options failure (`src/application/generation/gate.rs:700-704`). When the new Job fails before its Narration is saved, the old Options failure stays next to the new failure line. No scenario of `docs/specs/options.md` covers this. Must an accept also clear the Options failure? Work ticket: `state-machines-work` 09.

## To check (from scan 2)

Scan 2 labelled these inferred or unsure. Ticket 02 did not prove them.

- B12: `save_config` refuses only an Autoplay holder (`src/application/generation/gate.rs`, `save_config`). A plain Job keeps running across a mode or preset change, and each Job step reads the Game row again. One Job can narrate with the preset of the old mode and run the Quantifier with the new one (`scan2-application.md`, finding 8).
- B12: `set_preset_selection` checks the preset against a Game row read before the hold (`src/application/games/catalogue.rs`, `set_preset_selection`). A `switch_mode` in between can store a preset that the new mode does not allow (`scan2-application.md`, finding 8; `scan2-domain.md`, *Game config write kinds*).
- B12: `rename_game` takes neither the pointer lock nor the hold. A rename that races a delete of the same Game updates a row that goes away (`scan2-application.md`, finding 8, unsure).
- B11: a late raise for a deleted Game re-creates its per-Game entry after `forget_game`, because `set_game_failure` succeeds on a missing Game (`scan2-application.md`, `scan2-driven.md`, *Change counters*).
- B6: a stored row can hold a Message type and a Message kind that do not agree, because the mapper rebuilds both from columns (`scan2-domain.md`, *Message kind and Swipe flags*).
- B5: when the Swipe state of the Message before the last is missing, `read_tail` gives `state_before_last = None` (`src/adapters/driven/storage/stories.rs:80-89`). `redo` then treats a plain Narration as the opening (`src/domain/model/generation.rs:330`) and `Tail::start_at` starts from Swipe 0 of the last Message (`src/domain/model/tail.rs:37-45`). Known code path; reachable only with a bad `active_swipe_index`.

## Output

- `assets/03-dispositions.md`: the disposition table in the form that ticket 01 decides (one row for each machine, with the columns of its Answer), with the evidence for each machine.
- A review sketch of each machine whose disposition is not "leave it": a Mermaid state diagram in `assets/03-sketches.md`, drawn from the code as it is today. Mark each arrow that the disposition adds or removes. Ticket 06 shows the owner these sketches, so the owner can check each machine against intent. A sketch is a review aid, not a doc: it never goes into `docs/`.
- The behaviour questions: the seven above, checked again, plus any new ones. Each one names its machine and its file:line.
- Each plain defect goes to `.scratch/state-machines-work/issues/` as a ticket.

## Answer

Resolved 2026-11-09. Details: [`assets/03-dispositions.md`](../assets/03-dispositions.md) and [`assets/03-sketches.md`](../assets/03-sketches.md).

Of 15 machines of the server Game state, 9 get "leave it" (B3, B5, B6, B7, B8, B9, E5, C1, C2). Six cost work, and some get more than one disposition:

- **B1 (the two-step status read) and B12 (the preset check outside the hold): TLA+ model, as an extension of `verify-gate`, and a diagram.** Each is a race that a client can see: `status` can answer Idle for a busy Game (`view_query.rs:198-211`), and a `switch_mode` between the check and the write of `set_preset_selection` stores a preset that the new mode refuses (`catalogue.rs:192-215`). Both share the busy registry or the config write with `VerifyGate.tla`. Draft work ticket W1.
- **B2 (the dock record) and B4 (the encounters, with the B3 fire marks): model-based test and diagram.** Nine places in `gate.rs` make the moves of the dock. W2 also deletes `Tx::clear_dock`, which no code calls. W3 also removes `currently_meeting`, which no production code reads.
- **B10 (Role health): explicit in the code.** Retention keeps the newest call of each Role, so a Role that made calls never shows "No calls yet". The comment at `view_query.rs:250` changes with it. Draft work ticket W4.
- **B11 (change counters): explicit in the code**, through bug 11.

Readings that ticket 06 confirms:
- The reference machine of a model-based test counts as a transition table, so B2 and B4 get a diagram. The parent session gave this reading to the owner, and the owner did not object.
- The rule has no cost test. B12 passes the TLA+ test, but its fix is one moved check, so the owner judges the cost.

Behaviour questions for ticket 06: 8 kept (1, 2, 3, 6, 10, 11, 12, 13) and 2 new (N1, the old offered set after an Input whose Narration fails; N2, a restart during a Job leaves the Game idle with no failure line). Five were dropped: 4 (no path reorders the Triggers of a stored Character), 5 (a code fact), 7 (scenario 2.5 of `docs/specs/actions.md` settles it), 8 (now the B1 TLA+ disposition), 9 (now bug 10). Bugs filed: 10 (the Trigger choice follows the hash order) and 11 (a refused accept on a deleted Game raises its counters again). W2, W3 and W4 wait for the answers of their questions.
