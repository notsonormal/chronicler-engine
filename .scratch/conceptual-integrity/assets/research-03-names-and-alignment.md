# Research 03: do writers stop using an avoided word after the entry?

Ticket: `issues/03-research-names-and-alignment.md`. Evidence: local git history only. Head used: `c103092f` (2026-10-22).
Epistemic labels: **known** = read straight from git; **inferred** = my reading of known facts; **guessed** = no direct evidence.

## Summary

- Known: 15 groups of terms (18 words) have a usable count: Snapshot, Action Pipeline, Job, Swipe state, Save unit and Transaction, Replay blob, Narrator Action, Steering, NPC, Session and Playthrough, Phase, Turn, Engine commit, Error status, Retry and regen. 8 more groups have zero or near-zero counts and show no effect.
- Known: an avoided word fell to about zero in code for 5 terms: Action Pipeline, Engine commit, Narrator Action, Replay blob (as a type), Snapshot (as a game concept). In each case the fall came in a code commit, not in the commit that changed `CONTEXT.md`. For 4 terms it came 1 to 7 days after the entry. For Snapshot it came on the same day, 2 commits before the entry.
- Known: the entry commit itself changed code counts in 3 events only: Hook/Phase (`ac938c25`, -55 lines), Steering entry (`915b748c`, +7) and the Steering carve-out (`0da64383`, +122).
- Known: for 6 terms the avoided word stayed or grew after the entry: NPC (1222 -> 1019 code lines, preferred word 226 -> 340), Transaction (module `transaction.rs`), Turn (`ScriptedModel::turn`, `struct Turn`), Steering (`steering_instruction`), Session (2 comments, unchanged), Retry.
- Known: the order is not one-way. The entry came before the code for Job (3 days). It came after the code for Swipe state, Save unit and Narrator Action (2 to 3 days). Two entries changed to match usage (Retry reversed; Steering carve-out).
- Inferred: the counts show that names and later code changes go together in a planned rebuild. They do not show that an entry caused a writer to align.
- Known: one git identity wrote 782 of 787 commits. Inferred: most code commits after July came from agent sessions, committed by the owner. Git cannot separate the two.
- Main limit: a count of a word cannot show whether the writer read the entry. A plan, a rebuild commit or a review can explain each fall as well as the entry can.

## Method

Commits and commands. All read-only. No checkout. No change to the tree, index or branches. A scratch script in `tmp/r03/` ran the commands and was deleted.

1. Entry history of the file. The file moved on 2026-08-11 (`d8644394`, rename from `chronicler_engine/CONTEXT.md`).
   - `git log --format='%h %ad %an | %s' --date=short -- CONTEXT.md` gives 16 commits (the ticket's figure).
   - `git log ... -- chronicler_engine/CONTEXT.md` gives 7 more. **Known:** the file began on 2026-06-29 (`ec22cc44`), so history is 22 commits, not 16.
   - `git show <commit> --format= -U0 -- CONTEXT.md` (and the old path) for each commit, to read each added or removed term and `_Avoid_` line.
2. Revisions. For each of the 22 commits `C`, I counted at `C^` and at `C`. I added commits in the rebuild window (`622d5a5e`, `fcbe318f`, `959ca7d6`, `51efaafc`, `8a8ba6bb`, `36a1e1c7`, `04c9180d`), `d03c337f`, `7e2eb13a`, and `c103092f` (head). 54 revisions in all.
3. Count. For each term and revision:
   `git grep -n -I -i -E '<pattern>' <rev> -- src tests docs` (paths prefixed with `chronicler_engine/` before `d8644394`).
   The script put each hit line in one bucket:
   - **code**: a line of a `.rs` file under `src/` or `tests/` that does not start with `//`. A hit only in a trailing ` // ` comment goes to **comment**. Strings and identifiers both count as code.
   - **comment**: a line of a `.rs` file that starts with `//`, `///` or `//!`.
   - **docs**: lines under `docs/` outside `docs/plans/`, `docs/external_applications/`, `docs/AGENTS.md` and `docs/README.md` (the two indexes are generated).
   - **plans**: lines under `docs/plans/`. These are dated working plans. I count them apart because nobody revises them to follow the glossary.
   - Non-`.rs` files in `src/` and `tests/` (html, md, json): called "other" and shown only where they matter.
   - Not counted: `.scratch/`, `old-docs/`, `data/`, `AGENTS.md` files, `CONTEXT.md` itself. `.scratch/` held the planning notes. One spot check is in "Job".
4. Patterns (case-insensitive ERE): `snapshot`; `action[ _-]?pipeline`; `\bjobs?\b|job_|_job|Job[A-Z]`; `swipe[ _-]?state`; `save[ _-]?unit`; `replay[ _-]?blob`; `narrator[ _-]?action`; `steering`; `transaction`; `\bnpcs?\b|npc_|_npc|Npc[A-Z]`; `session`; `playthrough`; `phase`; `(^|[^a-z])turns?([^a-z]|$)`; `engine[ _-]?commit`; `error[ _-]?status|failure[ _-]?notice`; `reroll|resend|\bregen\b|regen_|_regen`; `\bretr(y|ies)\b|retry_|_retry|Retry[A-Z]`; zero-count set: `scorer|evaluator`, `trigger[ _-]?log|event[ _-]?log`, `dock[ _-]?state|options[ _-]?record([^a-z]|$)`, `draft[ _-]?state|provisional`, `live[ _-]?state`, `previous[ _-]?state|prev[_ ]state|base[ _-]?state`, `avatar`, `checkpoint`, `playthrough`.
5. Scale check: `git grep -c '' <rev> -- src tests`, `.rs` files only: 44,779 lines (`ec22cc44`), 70,759 (`1d8f3422`), 85,221 (`51efaafc`), 71,183 (`36a1e1c7`), 74,823 (`c103092f`).
6. Spot checks of remaining hits: `git grep -n -I -i -E '<pattern>' c103092f -- src tests docs/diataxis docs/specs`.
7. Writer evidence: `git log --format='%an <%ae>|%cn <%ce>'`, `git log --grep='Co-authored-by'`, `git log -1 --format=%B <commit>`.

## Why these terms

- **Chosen** when the avoided word is a distinctive string with one main sense in this code, and the count is not zero on both sides. Examples: `snapshot`, `action pipeline`, `replay blob`, `narrator action`, `steering`, `npc`, `engine commit`.
- **Chosen with a caveat** when the word has a second sense that I could split by reading the hits: `transaction` (SQLite's own word), `phase` (a UI status word), `turn` (verb and model tokens), `error status`, `retry`.
- **Added as positive counts** for entries that name a new thing: Job, Swipe state, Save unit. They show when the new word entered code and docs.
- **Excluded as common English or as another sense:** run, match, input, command, verb, text, output, story, setting, environment, scenario, title, label, nickname, line, entry, chat, variant, version, alternate, page, history, hint, bot, assistant, operator, style, tone, event, hook, callback, warning, log line, batch, `system message` (an LLM API role name), `suggestion` (Harper's own word and the `<suggestion>` tag of the Options prompt), `nudge` (a perspective "nudge" in World settings), `draft`/`provisional` in other senses. A count of these cannot separate a writer who ignored the entry from a writer who meant the plain word.
- **Counted, zero before and after** (no information about effect): scorer/evaluator, trigger log/event log, dock state/options record, draft/provisional state, live state, previous/base state, avatar, checkpoint (2 code lines, a different sense), playthrough in code. See the last group in Per term.

## Per term

Notation: counts are `code / comment / docs / plans`. "At" is the entry commit. "Pre" is its parent. All dates are 2026.

### 1. Snapshot (to Swipe state)
- Entry: `ec22cc44` 06-29 (Snapshot as a term; `_Avoid_`: Save, checkpoint, dump). `4eeadba2` 10-19 removed the entry and put Snapshot in the `_Avoid_` list of Swipe state (`_Avoid_`: Snapshot, save, checkpoint).
- Pre `04c9180d`: 37 / 6 / 27 / 46. At `4eeadba2`: 37 / 5 / 0 / 46. Head: 37 / 5 / 0 / 46.
- Before the rename: `ac938c25` 789 / 34 / 41 / 46, `51efaafc` 760 / 33 / 27 / 46. The code fall (760 -> 37) is in `36a1e1c7` (core C6, "the deletions", 10-19). The docs fall (27 -> 0) is in the entry commit.
- Remaining 42 code and comment lines (known, read at head): `tree_snapshot` helpers in `tests/infrastructure/guardrails/mod.rs` (15), `htmx_settle.rs` (8) and the browser stub (7), meaning a DOM or source-tree snapshot; and the migration code that names the old table `game_state_snapshots` (12, `plumbing.rs` and its tests). 46 lines remain in `docs/plans/`.
- Rename: later than the first entry, before the new entry. The entry that moved the word to `_Avoid_` came 3 commits after the code deletion.
- Earlier: the first entry `ec22cc44` did not touch code (653 -> 653).

### 2. Action Pipeline (to Job)
- Entry: `ec22cc44` 06-29. `4ad85da8` 10-14 made Job the entry; "Action Pipeline: Don't use. Use Job." went to Deprecated Terms. `4eeadba2` 10-19 removed that line.
- Pre `9cbf3112` -> at `4ad85da8`: 41 / 6 / 12 / 13 -> 41 / 6 / 12 / 13 (no change).
- `51efaafc`: 45 / 5 / 10 / 15. `36a1e1c7`: 0 / 0 / 10 / 15. `4eeadba2`: 0 / 0 / 0 / 15. Head: 0 / 0 / 0 / 15 (all 15 in `docs/plans/`).
- Rename: later. Code changed 5 days after the glossary named Job (core C6). Docs changed in the commit that removed the Deprecated line.

### 3. Job (positive count)
- Entry: `4ad85da8` 10-14. Pre `9cbf3112` -> at: 0 / 2 / 2 / 0 -> 0 / 2 / 2 / 0. The remaining hits are the plain English "job" ("their job is to arrange").
- First use in tests and plans: `90128752` 10-17: 22 / 7 / 6 / 21 (known: `tests/http/races.rs` and `tests/AGENTS.md` name "a Job holds the slot"). `622d5a5e` 10-18: 107 / 37 / 43 / 21. `51efaafc`: 211 / 72 / 53 / 21. Head: 484 / 126 / 224 / 60.
- Spot check outside the counted paths (known): `.scratch/` had 0 lines with the word "Job" at `9cbf3112` and 93 lines in 10 files at `4ad85da8`. The commit `4ad85da8` has the message "Updating after various planning", 52 files and 7,580 added lines. So the glossary entry and the planning notes entered together.
- Order: the entry came before the code, by 3 days. It came with the plans, not after them.

### 4. Swipe state (positive count)
- Entry: `4eeadba2` 10-19. Pre `04c9180d` -> at: 135 / 15 / 2 / 11 -> 135 / 16 / 20 / 11. Head: 143 / 18 / 26 / 25.
- First use: `90128752` 10-17 (31 code lines in `src/`). The word was in code 2 days before the entry. The entry commit added docs (2 -> 20).
- Order: the entry came after the code.

### 5. Save unit (positive) and Transaction (avoided)
- Entry: `4eeadba2` 10-19 (`_Avoid_`: Transaction, batch).
- Save unit, pre -> at: 7 / 6 / 1 / 5 -> 7 / 6 / 20 / 5. Head: 7 / 7 / 21 / 7. First seen in tests at `ac938c25` 10-17 (5 code lines).
- Transaction, pre -> at: 63 / 3 / 3 / 8 -> 63 / 3 / 4 / 8. Head: 64 / 3 / 4 / 11. Before the core step: `ac938c25` 3 code lines; `51efaafc` 10-18: 52. So the word entered code in bulk one day before the entry that avoids it.
- Remaining uses at head (known): `src/adapters/driven/storage/transaction.rs` (module doc: "The transaction: one unit of record writes that lands whole"), `Storage::transaction`, `transaction_new_game`, `StorageCall::Transaction`, `tests/storage/transaction_storage.rs`; and the library type `rusqlite::Transaction`, which the count cannot separate. `docs/diataxis/reference/storage.md` has a heading "Transactions" and then says "`Storage::transaction` runs one **save unit**".
- Rename: no code rename. Docs adopted "save unit" in the entry commit.

### 6. Replay blob
- Entry: `915b748c` 08-23 (`_Avoid_`: replay data, steering cache). `5523248f` 08-23 removed the entry. `ae7bc6ea` 08-30 added "Replay blob: Don't use."
- The word was in code before the entry: `d4427bba` 08-17 ("Narrator message type + replay blob on Swipe").
- Pre -> at `915b748c`: 1 / 9 / 0 / 2 -> 1 / 10 / 10 / 2 (the entry commit added `ai_steering.md`, 10 docs lines). `ae7bc6ea`: 1 / 6 / 10 / 3, no change at the Deprecated entry.
- `45033a6e` 09-06 ("flatten swipe generation inputs"; the message says it removes `GenerationReplay`, the replay field and column and the `ai_steering.md` Replay Blob section): docs 10 -> 0. Head: 1 / 1 / 0 / 2. The one code line is a test name (`..._from_replay_blob_and_drops_column`); the comment is `plumbing.rs:484`, about old stored data.
- Rename: later. 7 days after the Deprecated entry, in a refactor whose message does not cite the entry but does list a `CONTEXT.md` change.
- Limit: the pattern does not match `GenerationReplay` or a field named `replay`.

### 7. Narrator Action
- Entry: `915b748c` 08-23. Deprecated `ae7bc6ea` 08-30 ("Player direction is transient...").
- The word was in code before the entry: `b721622b` 08-20: 2; `2c9e78f6` (the parent of the entry): 17 / 1 / 1 / 0.
- At `915b748c`: 17 / 1 / 5 / 0. At `ae7bc6ea`: 17 / 0 / 5 / 0 (Pre `e7edcd14`: 17 / 0 / 5 / 0, no change).
- `d03c337f` 08-31 ("Extract narration_generation module and fix test-police findings 1-4", 52 files): 0 / 0 / 0 / 14. Head: 0 / 0 / 0 / 0 outside plans. The message does not mention the word.
- Rename: later, 1 day after the Deprecated entry. Inferred only that the two are linked; the message says no.

### 8. Steering
- Entry: `915b748c` 08-23 (Steering as an umbrella term). Same day it entered `_Avoid_` for Guided Generation. `ae7bc6ea` 08-30: Deprecated ("no umbrella concept"). `0da64383` 09-06: the Deprecated line gains "The word survives only in `steering_instruction`". `a2656820` 09-06: the Deprecated line and the `_Avoid_` word are removed.
- At `915b748c` (pre `2c9e78f6`): 8 / 12 / 18 / 7 (pre: 1 / 9 / 2 / 7). `ae7bc6ea`: 21 / 7 / 20 / 13. `0da64383` pre `f5a6a2fc`: 13 / 8 / 17 / 31 -> at: 135 / 15 / 21 / 31. `a2656820`: 138 / 16 / 21 / 31. `1d8f3422`: 136 / 16 / 30 / 1. `9cbf3112`: 29 / 8 / 9 / 1. Head: 28 / 7 / 9 / 1.
- Known: the commit that wrote the carve-out is the commit that added about 120 lines of `steering_instruction` (its message: "rename the merged steering text to steering_instruction ... the Steering deprecated-term entry gains a carve-out for the field name"). The writer changed the entry to fit the code, in one commit.
- Head spelling (known): `steering_instruction` 15, `steering` 14, `is_steering` 8, `Steering` 5, `ai_steering` (doc file) 1.
- Rename: none. The entry moved.

### 9. NPC (avoided for Character)
- Entry: `bc774107` 07-04 (`_Avoid_`: Person, actor, avatar, NPC (use Character)).
- Pre `5673d3e4` -> at: 1222 / 45 / 444 / 71 -> 1222 / 45 / 251 / 29 (docs fall because the commit deleted docs; code unchanged). `1d8f3422`: 1329 / 40 / 90 / 8. `ac938c25`: 1253 / 39 / 49 / 9. Head: 1019 / 29 / 48 / 9.
- Preferred word `character`, code lines: 226 (`bc774107`), 270 (`1d8f3422`), 340 (head). `.rs` lines: 45,437 -> 74,823.
- Remaining uses (known): `NpcCard`, `npcs` fields, `npc_encounter_log` and similar. 336 lines with `NpcCard` or `npc_` in `src/` at head. The entry text itself defines Character as "An NPC in a World".
- Rename: none, at any point.

### 10. Session and Playthrough (avoided for Game)
- Entry: `ec22cc44` 06-29 (`_Avoid_`: Session, run, match). `bc774107` added "playthrough".
- Session pre -> at `ec22cc44`: 0 / 2 / 24 / 2 -> 0 / 2 / 24 / 2. Head: 0 / 2 / 4 / 1. Code has never used it. The 2 comments (`src/domain/model/game.rs:2` "Game state and session management", `src/error.rs:146` "a live game session") are the same since 06-29. Docs fall (24 -> 4) came with docs deleted and rewritten.
- Playthrough: docs 7 (`ec22cc44`), 6 (`bc774107`), 0 from `d8644394` 08-11; code 0 throughout.
- Known: the `CONTEXT.md` Game entry itself says "A concrete playthrough session bound to one World and one Persona", and its `_Avoid_` list names Session and playthrough. This has been so since `ec22cc44` ("A single playthrough session").
- Rename: not applicable for code (no use). Docs fell by edits whose messages do not cite the entry.

### 11. Phase (avoided for Hook and for Step)
- Entry: Hook `ac938c25` 10-17 (`_Avoid_`: Phase, stage, step). Step `4eeadba2` 10-19 (`_Avoid_`: Phase, stage, hook). "Phase" was also the core word of the old Action Pipeline entry (06-29 to 10-14).
- Pre `90128752` -> at `ac938c25`: 298 / 20 / 36 / 78 -> 243 / 17 / 29 / 78 (-55 code in the same commit; the message says the Hook enum and `AgentRunner` replace the registry). `51efaafc`: 147 / 13 / 22 / 78. `36a1e1c7`: 12 / 2 / 22 / 78. At `4eeadba2`: 12 / 2 / 5 / 78. Head: 9 / 2 / 5 / 78.
- Remaining (known): the browser tests' "phase" announcer and poll text, the `PhaseError` enum in a guardrails test fixture, a comment "Phase 2.1" in `openrouter_tests.rs`. These are other senses or old names in test fixtures.
- Rename: partly in the entry commit (the Hook commit replaced the Agent registry), the rest in C6.

### 12. Turn (Deprecated: "Use Message + Swipe")
- Entry: `bc774107` 07-04. Pre `5673d3e4` -> at: 14 / 1 / 29 / 44 -> 14 / 1 / 9 / 44.
- Later: `ae7bc6ea` 15 / 4 / 21 / 9. `1d8f3422` 58 / 34 / 39 / 4. `4ad85da8` 71 / 38 / 8 / 4. `ac938c25` 182 / 48 / 14 / 10. `4eeadba2` 171 / 23 / 6 / 10. Head: 222 / 18 / 3 / 12.
- Known cleanup commits: `321f78e2` 10-11 (message: "Say 'message', not the deprecated 'turn', in options.md and the CONTEXT.md Options entry"): docs 18 -> 15, code 62 -> 65. `2885df2c` 10-19 (message: "'turn' and plan words out of comments and error text"): comments 23 -> 18, code 173 -> 175.
- Remaining at head (known): 119 call or definition lines of `ScriptedModel::turn` (added in `90128752` 10-17, 15 weeks after the Deprecated entry), 14 lines with `struct Turn` or `Turn::new` (`agent_tests.rs`), model chat-template tokens `<|turn>` (`ollama.rs`, `sanitize.rs`), plain verb uses, the error text "the turn was left unnarrated" (`headers.rs:31`, and its test), and a path `.scratch/turn-state-model/` in a doc. The 2885df2c message says the error text was cleaned; this one line remains.
- Rename: partial, by edits of single commits that cite the entry. A new use of the word began after the entry (test fake).

### 13. Engine commit (avoided for Game rules)
- Entry: `4ad85da8` 10-14 (`_Avoid_`: World rules, engine commit). The word was a phase name in the old Action Pipeline entry.
- Pre `9cbf3112` -> at: 3 / 2 / 3 / 0 -> 3 / 2 / 3 / 0. `90128752`: 3 / 0 / 0 / 0 (docs 3 -> 0). `36a1e1c7`: 0 / 0 / 0 / 0. Head: 0 / 0 / 0 / 0.
- Rename: later (docs by 10-17, code by C6 on 10-19).

### 14. Error status (avoided for Failure line)
- Entry: `4eeadba2` 10-19 (`_Avoid_`: Error status, error message, failure notice).
- Pre `04c9180d` -> at: 2 / 0 / 0 / 0 -> 2 / 0 / 0 / 0. Before: `51efaafc` 26 / 0 / 0 / 0; `36a1e1c7` 2 / 0 / 0 / 0 (the fall is in C6, before the entry). Head: 2 code (test names `..._sets_error_status_http`), 1 in `tests/STRATEGY.md` ("each error status", a generic phrase).
- Rename: before the entry. The entry followed the code.

### 15. Retry (entry reversed) and the words regen, reroll, resend
- History of the entry: `ae7bc6ea` 08-30 added "Retry: Don't use for the operation. Redoing the last generation is a new Swipe." `1d8f3422` 10-08 replaced it with an entry "Retry: Redoing the last generation as a new Swipe on the last Message" (`_Avoid_`: Regenerate, reroll, resend).
- Count of the word `retry` (code / comment / docs / plans): `915b748c` 217 / 41 / 130 / 5; `ae7bc6ea` 312 / 38 / 139 / 46; `1d8f3422` (pre = at) 308 / 39 / 98 / 2; `4eeadba2` 147 / 32 / 85 / 3; head 180 / 33 / 91 / 10.
- Known: writers kept the word `retry` in 300+ code lines while the entry said not to use it for the operation. Then the entry changed. The commit message of `1d8f3422` says "`Display name` and `Retry` become entries".
- Avoided words: `reroll|resend|\bregen\b`: `915b748c` 0 / 0 / 0 / 0; `ae7bc6ea` 23 / 2 / 5 / 23; `1d8f3422` (pre = at) 22 / 1 / 5 / 0; head 8 / 8 / 6 / 6. Known: the word "regen" is in `CONTEXT.md` itself ("A user regen keeps its start state", `4eeadba2`) and in `game_flow.md`.
- Limit: the avoided word "Regenerate" is not counted alone: its hits are mostly other senses (aria label "Regenerate options", "Regenerate docs index").

### 16. Avoided words with zero or near-zero counts (no effect can show)
| Avoided word (term) | Entry commit | Counts |
|---|---|---|
| scorer, evaluator (Quantifier) | `ec22cc44` | 0 / 0 / 0 / 0 at entry; 0 at head (2 docs lines on 07-10 to 08-10 only) |
| trigger log, event log (Event record) | `4eeadba2` | 0 at entry and head; 3 docs lines in 06-29 |
| dock state, options record (Dock record) | `4eeadba2` | 0 at every revision I checked |
| draft state, provisional state (Pre-analysis state) | `4eeadba2` | 0 at every revision |
| live state (Current state) | `4eeadba2` | 1 comment before the entry (to `ac938c25`); 2 docs lines after (I did not read them) |
| previous state, base state (Start state) | `4eeadba2` | 0 / 0 / 0 at entry and head |
| avatar (Persona, Character) | `bc774107` | 0 outside the saved external pages |
| checkpoint (Snapshot, Swipe state) | `ec22cc44` | 2 code lines to `51efaafc`, 0 after; a different sense |
Known: I first got 7 false hits for "dock state" because `options_recorder` matched `options[ _-]?record`. I fixed the pattern.

## What the counts can show, and what they cannot

Can show (known):
- When an avoided word left code and docs, and in which commit.
- Whether the entry commit changed code. It did in 3 of the events I counted.
- Which words stayed after the entry, and where (identifiers, tests, migrations, other senses).
- The order of a new word in plans, tests, code and the glossary (Job, Swipe state, Save unit).

Cannot show:
- **Cause.** The same commit window holds a plan, a rebuild and a review. The rebuild plan (`docs/plans/rust-rebuild-build-plan.md`) names Snapshot and Action Pipeline for deletion. A fall in C6 has that cause without any entry. The entry cannot be separated from the plan.
- **Direction.** The entry can follow divergence: Retry (the word stayed in 300+ lines and the entry reversed), Steering (the carve-out), Swipe state, Save unit and error status (the code came first). The entry can also lead: Job (3 days before the code). Both occur.
- **Reading.** Nothing in git shows a writer read an entry before writing. The project's `AGENTS.md` points to `CONTEXT.md` (via `docs/agents/domain.md`), and a skills library was added the same day as the file (`f15323be`, 06-29). That shows access, not use.
- **Rate.** Counts depend on repo size (44,779 -> 74,823 `.rs` lines) and on files deleted or rewritten. A fall in docs often means a deleted doc.
- **Sense.** A regex cannot split every sense (Transaction vs `rusqlite::Transaction`; phase as a UI word; turn as a verb).
- **Misses.** Identifiers that do not contain the phrase (`GenerationReplay`, a field `replay`) are not counted. `.scratch/` is not counted, and it holds the planning words first.
- **Control.** I have no count of a term that had no entry. A term with no entry might fall just as often. I did not make that comparison.

## Who wrote the later changes (inferred, from git)

- Known: 782 of 787 commits have author and committer `notsonormal`; 4 were committed by GitHub (PR merges); 1 has author `pi-agent` (`50c46a31`, 2026-08-01, "resolve tickets 02-08 from post-review hygiene map"). 2 commits (04-18, before the glossary) carry a `Co-authored-by: Sisyphus <clio-agent@sisyphuslabs.ai>` trailer.
- Known: the 22 `CONTEXT.md` commits come in two styles. 7 have no body and a short subject ("Adding CONTEXT.md and cleaning up some docs", "Removed ADRs...", "Updating after various planning", "Apply changes to after plan skill and context files"). The other 15 have a conventional prefix and a body of 4 to 56 lines, often with ticket numbers ("Ticket: narrator-modes-and-options #17", "core C7", "R4", "tickets 20 to 23").
- Known: the repo holds a delegate extension and subagent role files (`f3af8992`, `c05e2395`, 10-18), branches named `wf/rebuild-*` and a PR with "CodeRabbit review follow-up fixes" (`f354d126`).
- Inferred: the long-body commits from 07-04 on were mostly written by agent sessions that follow plans and tickets, then committed under the owner's identity. The short-body commits look hand-made or batch commits by the owner (the Job entry in `4ad85da8` is one of these: inferred that the owner or a planning session wrote it).
- Inferred: the three events where the entry and code moved in one commit (`ac938c25`, `0da64383`, `915b748c`) all have long bodies. Inferred that one agent session wrote both parts. For `0da64383` the body says the field name "follows the source vocabulary (Marinara's ...)", so the writer chose a name from an outside source and then edited the glossary to allow it.
- Guessed: whether each code-writing session read `CONTEXT.md`. Git does not record it.
- Writer assumption this implies for the map: the record has writers that do not remember each other (agent sessions) and one writer who does (the owner). It has no case of two writers talking about a name.

## Questions raised

1. Do the falls in Action Pipeline, Snapshot and Engine commit come from the glossary or from the plan that named the deletions? The history has no case of a deletion without a plan.
2. Job and Swipe state show a new word used in tests and docs 2 to 3 days before or after its entry. Does the entry follow the plan, or does the plan carry the word to writers? Is the entry then a record and not a rule?
3. Why did the avoided word grow after the entry in NPC, Transaction, Turn and Steering? Did the writers not read the entry, or did they find the entry wrong for the code? In two cases the entry changed (Retry, Steering).
4. Is a name in an identifier (`NpcCard`, `Storage::transaction`, `ScriptedModel::turn`) a gap in the entry, or a different layer from the entry? The entry says "use Character" and the code says `NpcCard` in 336 lines.
5. A glossary that defines a term with its own avoided words (Game: "playthrough session"; Character: "An NPC in a World"): does a writer read the definition or the `_Avoid_` line?
6. Cleanup commits cite the entry ("the deprecated 'turn'", "slop pass"). Is cleanup a review step that follows the entry, and is that the step that makes writers align?
7. Which of the other ways to align (plan, review, test, guardrail) would produce the same falls without the entry? Ticket 04 can test this from the repo.
