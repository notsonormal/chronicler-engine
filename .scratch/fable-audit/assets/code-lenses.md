# Code lenses: one fact in several places, agreements that no code checks, and silent defaults

Audit of 2026-11-10 at commit `b0c8fe41`. This report covers audits B2 to B6 of the catalogue (`audit-catalogue.md`): "more than one source of truth", "implicit protocols", "silent failures", "one rule, several entry points" and "the pattern behind the findings".

## How to read this report

- **This report is about causes.** The other reports list what goes wrong. This one asks which design choices make those faults easy, so that one change can remove several of them.
- **Method.** Four cheap models each read the source through one lens and wrote a table (`leads/b2-…` to `leads/b5-…`, 77 rows). I did not trust those rows. A second model tried to refute 13 of them. I ran or read the rows that this report uses.
- **Labels.** "Ran it" means I saw it on the probe server. "Read it" means I read the cited lines. "Second model" means that the refuting model confirmed it from the code. A row with only a scout behind it is marked "scout only".
- **Known or new.** A scout searched your records for each row. Most causes are known in parts. `leads/pass3-known-or-new.md` has the quotes. What is new here is the grouping: your records hold these as separate facts in several efforts.

## 1. Summary: four design choices behind the findings

| # | The choice | Faults that follow from it | One change that removes them |
|---|---|---|---|
| 1 | **A check sits on one path, not on the record.** The rule is written where the first request came in | The World checks, the escape of text for a prompt, the text check, the busy gate on settings, the analysis of an opening | One function for each record, called by every path |
| 2 | **One fact has several homes, and each home has its own lifetime** | Three health reports that disagree. The Trigger that fires again after a delete. The "Active" badge. Old story entries that change their Room name | For each fact: one home, and a derived view where a second reader needs it |
| 3 | **A value that cannot be read becomes a default, and only the log says so** | A text-check mode that turns the check off. A Quantifier answer that freezes the scene. A posture select that snaps back. A movement type that is ignored | A wrong value is refused at the edge where it enters, with a sentence for the person who sent it |
| 4 | **Two parts agree on a text shape, and no code holds the agreement** | The Quantifier preset against its parser. The schemas against the Rust types. The validation script against the seed. The trial script against the stored JSON | Derive one side from the other, or test the two together |

A fifth cause is in `product-content-and-release.md`: the shipped content uses a small part of the engine, so real play never reaches many paths.

## 2. A check sits on one path, not on the record

| Rule | Where it is checked | Where it is not | Proof |
|---|---|---|---|
| A World is well formed: its Rooms exist, an exit leads to a Room, a Scenario's Characters and starting Room exist, a Trigger's Room exists | `scripts/validate_data.py`, for the repository's `data/` at build time | The boot seed. The World create form. The World update form | Ran it: findings 1 and 9 of `story-damage-and-false-screen.md`. Known: ticket 02 of invariant-audit |
| Player text cannot hold a `{{…}}` placeholder | The typed Input of the current request | The same text when a later prompt carries it in the history. Model text. An edited entry | Ran it: finding 6 of the same report |
| The text check | `/action/check` | `/action` (the page's "Send Original"), `/guide`, `/impersonate`, an edit of an entry | Second model. A fault of the checker is logged, and the Input is sent with no notice (`actions.rs:62-68`) |
| A busy Game refuses a write | Every story route. A config write under Autoplay | A config write under a plain Job (`save_config`) | Ran it: finding 7. Known: `gate-06` |
| "The busy refusal comes before any other check of the request" (`gate.rs:333`) | `/action` | `/action/check` runs the text check first | Second model: holds in part. No spec states the order. New |
| A Narration is analysed (Quantifier, Game rules, Triggers) | Every model Narration | The opening Narration is not analysed. Its Retry is | Ran it: finding 8 |
| A change to the story clears the failure line and the offered Options | Swipe switch, delete of the last entry | Edit of an entry | Ran it for the Options: the dock kept its three Options after an edit of the last Narration. Known: `scout-2-application.md:24` |
| A preset's fields are valid | The preset form (answers 400) | The preset seed: a refused file gives a warning, and no row exists. A Game that names the id then fails each call | Read it (`run.rs:252-255`) |

**What to do.** For a World, the first report already recommends one validation function for the seed and both forms. The same shape fits the others: the rule belongs to the record (a World, a prompt part, a config write), and each path calls it. A guardrail can then ask a simple question: does any route write this record without the function?

## 3. One fact has several homes

| Fact | Its homes | How they came apart | Proof |
|---|---|---|---|
| Is this Game well? | `games.failure` (the status line). `game_docks.notice` (an Options failure). The newest 50 rows of `llm_messages` (role health) | An Options failure showed "Ready" beside a failure banner. The banner cleared itself after 26 Actions with no successful Options call | Ran it: finding 4. Known as code readings |
| Did this Trigger fire? | The kind of the event Message. The `event_record` in the Swipe state before it. The `trigger_fired` mark in the state of the event | Retrigger reads one copy and Retry reads another. A delete of the event Narration removes the mark, and the Trigger fires again | Ran it: finding 5. Known: `story-06` |
| Who wrote this Swipe? | `message_type`, `kind`, `impersonated`, `maker` | Nothing in `src/` reads `maker`. For an Impersonate Input it says `Player`, though the model wrote the text. The first reader of this field will be wrong | Read it. New |
| Which preset is active? | The mode bundle in the settings. Each Game's own preset ids | "Set Active" moves the badge. A running Game keeps its preset. This split is intended (`prompt_presets.md`). The badge does not say so | Ran it. The wording is new |
| What is this Room called? | The live World map, read each time the story log renders | A rename of a Room changed the header above old entries of the story. Two Rooms with one name: a move between them showed no header | Ran it on the probe, World `maptest`. The cause is in `.scratch/code-concepts/assets/research-09-docs.md:148` as a question |
| Is this Character in the scene? | `scene.characters_present`. `encounters.currently_meeting` | No production code reads `currently_meeting`: three writes, and the one reader is called by tests only. The mutation run inverted the condition of one write (`game_rules.rs:116`), and no test failed | Read it |
| The default preset ids | Rust constants (`settings_defaults.rs:8-10`). The seed files. `data/settings.json`. Column defaults and migration literals (`plumbing.rs:239-240`, `:352-358`, `:392`) | Nothing compares them. A rename in one place gives a Game that names a missing preset | Second model. Known in part |
| The default Agents | `AgentConfig::defaults()`. The `agents` list in `data/settings.json` | A database with no settings row runs the Rust list. A seeded one runs the file's list | Scout only |

**What to do.** Decide the home of each fact, and make the other readers a view of it. Two of these are cheap and stop a later fault: delete `maker` or make it right, and delete `currently_meeting`. The health fact is the one that the player sees. It needs your decision (question 1 of section 7).

## 4. A value that cannot be read becomes a default

| Where | The value | What happens | Who is told | Proof |
|---|---|---|---|---|
| `POST /settings/text-check` | A `check_mode` that the handler does not know | 200, and the mode is stored as Disabled | Nobody | Ran it. The template writes `spell_grammar`; I sent `spell-grammar` |
| The World forms, and each read of a World or a Game row | A narrator mode, perspective or tense that does not parse | The default (`parse_or_default`, 13 call sites) | The log | Read it. A new enum variant compiles, and each place that forgot it falls back |
| The Quantifier answer | Text that is not JSON | The Room and the Characters stay. The Job ends "Ready" | Nobody | Ran it: finding 3. Scenario 2.5 specifies the kept state. Your ticket 10 expects a banner |
| The Quantifier answer | A movement type that is not "entering" or "leaving" | The type becomes none, and the player still moves to the destination | Nobody | Ran it. Known: `scout-2-application.md:60` |
| A preset seed file | A missing `id` | The id becomes the literal `default`, and a missing name becomes "Default" | Nobody | Read it (`run.rs:237-240`) |
| A Character file | Invalid JSON, or no `id` | The Character is left out of the World | The log, one line | Second model |
| A map file | A Room with no `exits` key | The map read fails. The Worlds after it are not seeded. The boot goes on | The log, one line | Second model: holds in part |
| `current_exe()` fails | — | The server opens `chronicler_<port>.db` in the data folder, so it starts with a new empty database | Nobody | Read it (`run.rs:46-50`). I did not make it fail |

**What to do.** The log is not a channel to an author or a player. Each of these edges has a person on the other side: a form answer, a boot line on stdout, or a failure banner. The rule "refuse at the edge, with a sentence" would also have caught the blank dashboard.

**Not silent, against a scout's claim:** an HTTP error of the narrator, the Quantifier or the Options call raises a banner with the provider's message. A World update for an unknown key answers 400.

## 5. Two parts agree on a text shape

| Side A | Side B | What holds the agreement | Proof |
|---|---|---|---|
| The Quantifier preset's `output_format` (author text) | `parser.rs`: the JSON keys | Nothing. **Your decision**, ticket 10 of prompt-flexibility: the format stays preset text | Decided |
| The Options preset (author text) | Three regex shapes in the Options parser | `agent_system.md:60` lists the shapes. A fourth shape fails after two paid calls | Decided |
| `{{…}}` in preset text | `{{…}}` in World text (the opening, a Trigger's direction) | Nothing. `TemplateVars::new` fills third person, past tense and empty Persona fields for World text, whatever the Game's settings are | Read it (`opening.rs:18`, `game_rules.rs:99-100`). Known: prompt-flexibility map |
| `data/schemas/character.schema.json` | `CharacterCard` in Rust | Nothing. The schema does not require `id`. The type does. A file passes the gate and is dropped at boot | Second model |
| `data/schemas/world.schema.json` | `World` in Rust | Nothing. The schema names none of the four posture fields and allows any extra field. A typo in a posture value passes the gate | Read it |
| `validate_data.py`: a Scenario's Character has a file `<id>.json` | `load.rs`: the seed keys a Character on the `id` inside the file | Nothing. A file whose name and `id` differ passes the gate, and the Scenario's Character never appears | Read it (`validate_data.py:46`, `load.rs:147-151`) |
| `scripts/trial.py:971` reads `scene.npcs_in_area` from the stored Swipe state | The serde names of `SceneState` | Nothing. `scripts/tests/test_trial.py` has no case that reads a Swipe state. A rename gives trial packs with empty scenes | Read it |
| The page's `X-Chronicler-Game` header | The Rust constant in `request_layers.rs` | A browser test (`tests/browser/stub/game_header.rs:29`). **Sound** | Read it: the test exists (scenario 16.53) |
| The status names `idle`, `thinking`, `autoplay` | `assets/dashboard-action-area.js` | Tests in `tests/http/dashboard.rs`. **Sound.** A scout claimed that nothing held it | Second model |

**What to do.** The two decided rows stay. For the schemas, the cheap fix is one test that loads each shipped data file through both the schema and the Rust type, and a generated schema later if you want one. For `trial.py`, let its test read a database that the engine wrote.

## 6. How far to trust the scouts

A cheap model reads fast and states each row with confidence. I checked what this report uses, and the numbers are worth a record:

- A second model tried to refute 13 scout rows. 9 held, 2 held in part, and 2 were wrong.
- I found six more wrong rows myself. Five are listed in `leads/pass3-candidates.md`. The sixth reported two tools as absent that this harness has.
- So 8 of the rows that I or the second model checked were wrong, and 2 were overstated. I did not check every row, so I give no rate. Most of the wrong rows were stated as "known".
- **The cost of the check was small.** A row that I can run on the probe takes about two minutes.

This supports a rule that your retro audit already has: spend compute on verification. A scout table is a list of places to look. It is not a finding.

## 7. Questions for you

| # | Question | My recommendation |
|---|---|---|
| 1 | Where does the health of a Game live? | One record for each Game that every Job failure writes, the Options failure included. Role health stays a view for the tuner, and it must not clear by retention |
| 2 | May a form or a boot step fall back to a default when a value does not parse? | No for a form: answer 400. For a stored row, keep the fallback and add a boot check that reports it once |
| 3 | Should `maker` and `currently_meeting` go? | Yes. Both are written and never read, and both hold values that would mislead a reader |
| 4 | Should the schemas be generated from the Rust types? | Later. First one test that loads the shipped data through both |
| 5 | Should the Room name be stored with the entry? | Store the name at write time. An old entry then keeps the name that the player read |

## 8. What I did not check

- **The row marked "scout only".** One row in section 3.
- **The scout rows that this report does not use.** The four lens files in `leads/` hold 77 rows. This report uses about 30 of them.
- **The page scripts.** The lenses read the Rust source. `.scratch/state-machines-next/assets/05-dispositions.md` covers the page as state machines.
- **Storage under two writers.** The first audit covers transactions.
