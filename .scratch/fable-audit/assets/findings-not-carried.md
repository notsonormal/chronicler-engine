# Findings that no decision and no ticket holds

Written on 2026-11-10, after the grilling session, at commit `e8d6512a`. The owner asked whether the five tickets hold everything useful from the five reports. They do not. This file lists what is left.

## State

- **N1 to N14: accepted** (owner, 2026-11-10, D19). [Ticket 06](../issues/06-task-engine-fixes-second-group.md) holds N1 to N12 and N14. [Ticket 05](../issues/05-task-text.md) holds N13.
- **Three parts of those rows are in no ticket.** Ticket 06 names them under "Not in scope", each with its reason: the provider `mock` (N11), a JSON mode for the Quantifier (N10), and two map checks that went to the invariant audit (N4).
- **The map edit that removes a Room (N4): decided** (D21, refuse the edit). Ticket 06, change 16.
- **Section 5, the conflict: decided** (D20, the banner shows). Ticket 06, change 15.
- **N15 to N18: not decided.**

## How the list was made

- I read every finding section of the five reports again and compared each point with `decisions.md` (D1 to D18) and with the five tickets.
- A point is in this file when no decision answered it and no ticket holds it. The owner never saw these points as a question.
- **Checked at `e8d6512a`:** the rows marked ✔. The other rows carry the proof of their report, which was written at `b0c8fe41`.
- The numbers N1 to N18 are only names for this file.

## 1. Defects with a plain fix

The agent chose not to ask about these. That choice was the agent's, not the owner's.

| # | Finding | Size | Report |
|---|---|---|---|
| N1 ✔ | **The prompt of an event Narration has three faults.** The last Narration is in it twice (`narrator.rs:191`). The Trigger direction is wrapped in `<PlayerInput>`, so the model reads it as the player's action. The Character of the Trigger is not in `<CharactersPresent>`, so the model gets no card for the Character that it introduces. The weekly retro routed the first fault (R7) to "a new task ticket", and no ticket exists | Medium | Engine 6, Story 5 |
| N2 ✔ | **The narrator call has one attempt.** One HTTP 429 or 502 ends a Job and an Autoplay run. The Quantifier and the Options Agent have two attempts. One limit of 180 s covers the whole call (`client.rs:26`), and each call builds a new HTTP client | Small | Engine 7 |
| N3 | **The Quantifier's movement type is read and then ignored.** The preset asks for "entering, in or leaving". An answer "leaving" with a destination moves the player into that Room. Either use the type or stop asking for it | Small | Story 3 |
| N4 ✔ | **The World form accepts a map that cannot work.** An exit to a Room that does not exist, and the same Room id twice. The form's own placeholder examples (`worlds.rs:185`) make a World with no Room. A map edit that removes the Room where a Game stands blanks the dashboard for that Game. The invariant audit holds the roomless World and the bad key, but not these four points | Medium | Story 1, 9 |
| N5 | **The text check runs on one route only.** `/action/check` runs it. `/action`, `/guide`, `/impersonate` and an edit do not. A fault of the checker is logged, and the Input is sent with no notice (`actions.rs:62-68`). The shipped setting is Disabled, so nobody meets this today | Small | Lenses 2 |
| N6 | **The boot drops an author's file with one log line.** A refused prompt preset file leaves no row, and a Game that names its id then fails each call. A preset file with no `id` gets the id `default`. A Character file that is not valid JSON is left out of the World. A fault in one map file stops the seed of the Worlds after it. `validate_data.py` checks the name of a Character file, and the engine uses the `id` inside it | Medium | Lenses 2, 4, 5 |
| N7 | **A placeholder in World text fills wrongly.** `{{…}}` in the opening and in a Trigger direction gets third person, past tense and empty Persona fields, whatever the Game's settings are (`opening.rs:18`, `game_rules.rs:99-100`) | Small | Lenses 5 |
| N8 | **A server that starts from another directory serves no assets.** `/` answers 200, and every script, style sheet and picture answers 404, with no message. The database path is relative to the binary, and the assets are relative to the working directory | Small | Engine 8 |
| N9 ✔ | **Small faults of the screen.** An added Room has no name on the screen, and the story-log header prints its raw id. Times have no date. An entry that is an event and a Room change shows one header. The Presets tab says "Active" for the default of new Games, and a running Game keeps its own preset | Small each | Story 4, 10 |
| N10 ✔ | **Small faults of the prompt.** An added Room has the fixed description "A place you have never seen before." in every later call (`game_rules.rs:127`). An empty Send keeps the sentence "The player's next action is provided above". `sanitize_llm_output` removes the indentation of every line (`sanitize.rs:25`). The Quantifier request has no JSON mode | Small each | Engine 4, 9. Story 10 |
| N11 ✔ | **Small faults of the process.** The server prints no address on stdout. A missing API key shows as "The engine failed with an internal error." One sentence, "The language model could not be reached.", covers HTTP 401, 402, 429 and 500. The Connection form accepts the provider `mock`. The log files are never deleted, and at level `info` they hold the text of each Narration | Small each | Engine 4, Product 6 |
| N12 ✔ | **Two dependency advisories, and no audit step in the gate.** `dotenv` is unmaintained. `libsqlite3-sys` has RUSTSEC-2026-0459, and its fix needs Rust 1.89. Neither is a risk for this engine today | Small | Engine 4 |
| N13 ✔ | **Two wrong texts in the product data and code.** `options/roadway_domains.json` gives fantasy examples for a modern estate. The comment at `headers.rs:37` says that a role failure lasts until the next success of the role, and the code clears it by age | Small | Product 2, Story 4 |
| N14 | **One fact in several places with no check.** The default preset ids are in four places (`settings_defaults.rs:8-10`, the seed files, `data/settings.json`, `plumbing.rs`). The default Agents are in two (scout only) | Small | Lenses 3 |

## 2. Points of the test and environment reports

| # | Finding | Report |
|---|---|---|
| N15 | **The mutation run found code that does nothing and checks that no test fails.** `extract_json_from_code_fence` and the retry condition of the Quantifier can be replaced, and every unit test passes. No test makes one of the four `assert_*` state checks fail. Seven boundaries of the token budget are not pinned. Neither sanitize function has a direct test | Environment 5 |
| N16 | **Rows that the report called optional.** A "failed step" column in the build journal. An end date on a claimed ticket (two claims are older than 18 days). Ten prose rules that a small check can hold. Three document-review skills that claim the same task. `AGENTS.md` and `CODING_STANDARDS.md` do not name the generated `guardrails.md` | Environment 4, 6 |

## 3. Points that a ticket excludes with "no decision accepted that"

The tickets record these in their "Not in scope" lists. No question asked them.

| # | Finding | Excluded by |
|---|---|---|
| N17 | A stored API key is written into the HTML of the Connection edit form. `GET /data/settings.json` answers 200, because the whole `data/` folder is served (`router.rs:204` ✔). No `Content-Security-Policy` header | Ticket 01 |
| N17 | A `<think>` block in model text is stored as story text (inferred, not run). A boot check that reports a stored row with an unreadable value | Ticket 02 |
| N17 | A cut answer of the Quantifier or of the Options Agent is accepted as an answer | Ticket 03 |

## 4. Proposals that need a trial

D4 (no model credit) covers these, but `decisions.md` does not list them.

| # | Finding | Report |
|---|---|---|
| N18 | **The Impersonate preset.** Inputs over 100 words in 79 to 86 of 100, against "one beat only". The Persona repeats the last line of the Narration in 19 to 26 of 100 Inputs. The Persona writes a discovery itself in 1 to 5 of 100 | Product 3 |
| N18 | **The System preset.** "A pinch of dry humor" gives every speaker one voice. Narrations over 150 words in 29 to 83 of 100, against "under 150 words" | Product 1, 2 |
| N18 | **The Quantifier preset.** Ask for the Characters "in the scene of `<LatestMessage>`", not "in the current room" | Engine 11 |
| N18 | **Carla's card** ends with "Carla will never speak for Julian". The Impersonate call, whose task is to speak for Julian, gets this line too | Product 2 |

## 5. One conflict that the owner did not see

- D3 says "no change" to the meaning of "Ready".
- An earlier decision of the owner says, about a Quantifier answer that the parser cannot read: "The state then stays as it was, and the banner shows" (`.scratch/prompt-flexibility/issues/10-quantifier-options-layout.md:43`).
- **Ran it:** the banner does not show for that case. The status reads "Ready".
- The question for D3 gave the fact. It did not say that an earlier decision expects the banner. So D3 may not be the answer that the owner wants for this case.

## 6. Held by another effort

These are not lost. Each has an open ticket elsewhere.

| Finding | Where |
|---|---|
| `/mode` and `/rename` are accepted during a Job (gate-06) | `invariant-audit/issues/03` |
| `update_world` has no transaction (storage-04) | `invariant-audit/issues/07` |
| A Character named by display name is dropped (llm-01). A one-shot Trigger fires again after a delete (story-06) | `invariant-audit/issues/01` |
| A World with no Room, an empty key, a key with `/` (world-02, http-08) | `invariant-audit/issues/02` |
| The Action input is one line | The dashboard-design map |
| `sanitize_for_prompt` has no purpose (R16) | `weekly-retro-2026-11-04/issues/05` |
