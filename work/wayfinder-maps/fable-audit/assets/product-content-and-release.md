# The product: the shipped content, the measuring tool, the genre and the release

Audit of 2026-11-10 at commit `b0c8fe41`. This report covers group C of the catalogue (`audit-catalogue.md`): C2 "what can be deleted", C3 "the measuring tool", C4 "comparison with the genre", C5 "the content as writing", C6 "readiness for the release" and C7 "a cold start". C1 "does each mechanism earn its cost" is not done, because it spends model credit. Section 8 says what it needs.

## How to read this report

- **This report asks a different question from the other three.** They ask "does the engine do what it means to do?". This one asks "is that the right thing, and can a stranger use it?".
- **The evidence for the story is the seven stored 100-Cycle trials** under `tmp/` (`pf20`, `lt01`, `lt03`, `lt10`, `lt15/reference`, `lt15/new`, `ltm09`). All use one setup: the World `redmist_estate`, the Persona `julian`, the default presets and `deepseek-v4.1-flash`. A finding may not hold for another model.
- **Labels.** "Measured" means a script counted it over the stored stories, and the script is in `local-only/evidence/` or the count is one that I ran again. "Read it" means I read the cited file. "Reviewer" means a second model (Opus, with no knowledge of my findings) judged it, and I checked the counts that this report uses. "Researcher" means a model read primary sources of other products.
- **Your records already hold much of this ground.** `work/wayfinder-maps/layout-trials/assets/story-inconsistencies.md` has the era drift, the wrong Room and the drift of single Characters. `work/wayfinder-maps/prompt-flexibility/assets/` has the card format and the memory research. I report what those records do not have.

## 1. Summary

| # | Finding | Kind | Size |
|---|---|---|---|
| 1 | The Characters lose their voices, because the field that holds each voice never reaches a prompt | Content and engine | High |
| 2 | The cast shrinks to two Characters, and the model invents its own | Content | High |
| 3 | The Persona's Inputs are as long as the Narrations, and one in five repeats the last line it heard | Content (one preset) | Medium |
| 4 | No count in the trial pack moves when the story gets worse, and two runs of one setup differ more than two layouts | Measuring tool | High |
| 5 | The shipped content uses a small part of the engine | Scope | Medium |
| 6 | A stranger cannot start: the README names no key, and the first Input fails with "internal error" | Release | High for a release |
| 7 | Five features that players of this genre expect are in none of your records | Scope | Your decision |
| 8 | Nothing records a licence, for the code or for the Character art | Release | High for a release |

## 2. The content as writing

A reviewer read the World, the five Character cards, the Persona and the presets as direction for a model, with three of the stored stories as evidence (L = `ltm09`, N = `lt15/new`, R = `lt15/reference`). Its full table has 15 rows with a rewrite for each: `leads/c5-content-as-writing.md`. I ran the counts of rows 1 to 4 again.

### Finding 1. The voices are lost

- **Measured.** Each card's voice is in its `example_dialogue` field: Louise's "Bonjour, monsieur!" and her accent, Lisette's "Master", Julian's stammer. In the Narrations of all three stories, "Bonjour", "monsieur", "accent", "Paris" and "Master" each occur 0 times.
- **Read it.** No prompt code reads `example_dialogue`. The field is stored and loaded (`storage/characters.rs`), and nothing under `src/application/prompting/` names it. Your record `card-and-world-format.md` notes that the field is not sent. The link to the lost voices is new.
- **Reviewer (inferred).** With no voice in the prompt, every speaker takes the one voice that the System preset asks for: "a pinch of dry humor". The word "dry" describes different speakers in 5, 11 and 15 Narrations of the three stories.
- **Options.** Send two lines of `example_dialogue` for each present Character (about 40 words). Or add one `Speech:` line to `personality`. And move "dry humor" to the narrator's own voice.

### Finding 2. The cast shrinks

**Measured.** Narrations that name each Character, of about 100 in each story:

| Character | L | N | R |
|---|---|---|---|
| Carla | 13 | 51 | 45 |
| Jezebel | 11 | 18 | 23 |
| Gabriella | 21 | 2 | 5 |
| Lisette | 14 | 5 | 1 |
| Louise | 11 | 6 | 0 |
| Invented: Ashby, Aldridge, Calloway | 14, 14, 17 | 0 | 0 |

- In two of three stories, Louise and Lisette almost vanish. In the third, three invented people each appear more often than four of the five authored Characters.
- **Reviewer (inferred).** Louise's and Lisette's goals are pure service. Nothing ties them to the will or to a conflict, so the plot that the model invents has no use for them. The opening gives one promise (the phone), which every story delivers within six entries. After that the World gives nothing to pursue, and each story invents a money mystery in the harbour city.
- **The one Trigger works.** Gabriella's introduction fires and is followed closely in all three stories. After it she is named in 2 and 5 Narrations (N, R). Her event call says "No Characters are present in this Room", so her full card is absent from her own scene (read it: `tmp/ltm09/redmist_julian/trial-1/calls/01450-event-narration.md:214`).
- **Options.** Give each staff Character one stake in the will. Author one threat and one deadline in `global_rules`. Add one Trigger for each Character in its own Room.

### Finding 3. The Persona's Inputs

- **Measured.** Inputs over 100 words: 80, 79 and 86 of 100. The Impersonate preset says "One beat only" and gives no length. The median Input has 124 to 162 words across the seven trials, against 138 to 193 for a Narration.
- **Reviewer, measured by its script.** The first quoted line of an Input repeats four words in a row from the Narration before it in 22, 19 and 26 of 100 Inputs. Narrations do this in 4 of 100. The rule against an echo is in the System preset only. Your review `impersonate-output-review.md` read six Inputs. It names the length and a repeated rhythm, and it does not name this echo.
- **Reviewer.** In about 1 to 5 Inputs of 100 the Persona writes a discovery itself ("the login list scrolled up onto the screen…", N 5). That Input starts the whole plot of story N. The preset's word "perceives" allows it.
- **Why it matters.** Each Input is in the history of every later call. A long Input costs tokens 100 times, and the model copies the length that it sees.
- **Options.** A number in the preset's `output_format` ("At most 60 words. One action and at most one line of speech."). The echo rule in the Impersonate preset. "What the Persona finds is the narrator's to tell."

### Smaller content points (read it, with no fault traced to them)

- The Quantifier preset's rules name only "entering". Its answer format offers "entering|in|leaving". The parser knows "entering" and "leaving". The Game rules ignore the type (finding 3 of `story-damage-and-false-screen.md`).
- Narrations over 150 words: 39, 29 and 83 of about 100. The preset asks for under 150 words in a conversation.
- `options/roadway_domains.json` gives fantasy examples ("forest creatures", "corrupt city guard") for a modern estate.
- Carla's card ends with an instruction to the model ("Carla will never speak for Julian"). The Impersonate call, whose task is to speak for Julian, gets this line too.

### What the reviewer found sound

The opening lands in every story. The Trigger direction is followed. The relationships show when both Characters are present. The narrator obeys its rule against an echo. The examples in the presets are not copied into the stories. Phrase loops are weak: no five-word phrase occurs in five or more Narrations of any trial.

## 3. The measuring tool

**The question:** when a change makes the story worse, does a number move?

- **Read it.** `scripts/trial.py` records cost, speed, cache share, failed calls and word counts. It records no fact about the story. Story quality goes to an ad hoc judge, by your decision in `work/wayfinder-maps/layout-trials/issues/05-read-an-output-trial.md`.
- **Measured.** I wrote `local-only/evidence/trial_truth_counts.py`. It counts facts that a script can decide from `story.json` alone: the Rooms used and added, a Narration that names a place off the estate while a map Room is recorded, a Narration with no authored Character, invented titled names, modern and period words, repeated phrases, and Inputs that write another Character's reaction. The table for the seven trials is `local-only/evidence/trial-truth-counts.txt`.

**The result that matters.** `lt01` and `lt10` sent the same prompts. Ticket 10 of layout-trials changed only a session header (inferred from the ticket; I did not compare the request bodies). Their story counts still differ widely:

| Count | `lt01` | `lt10` |
|---|---|---|
| Narrations with no authored Character | 27 | 47 |
| Uses of invented titled names | 26 | 135 |
| Period words ("carriage", "shilling" and others) | 74 | 17 |
| Rooms that the Quantifier added | 3 | 0 |
| Narrations where a present Character goes unnamed for five entries | 1 | 30 |

- **Reading.** Two runs of one setup differ more than any two layouts that the trials compared. So one run for each side cannot show a story effect of this size. Your ticket 05 names this ("several runs per side are needed") and defers it. These numbers say how large the noise is.
- **The judge.** The pilot gave `lt01` and `lt03` the same score of 6 of 10. The counts for those two differ in many rows. A 1-to-10 score is too coarse for what these trials compare.
- **What I recommend.** Add the deterministic counts as columns to `trial.py compare`. They cost no tokens. Run one setup three times before the next story comparison, to get the noise for each count. Keep the judge for what a script cannot count, and give it checkable claims in place of a scale.
- **Limit.** My counts are simple text rules. A row such as "period words" has false hits. They show a difference between runs. They do not measure quality.

## 4. What the shipped content exercises

**Scout, with my checks on the rows marked "read it".** The full table has 27 concepts: `leads/c2-concept-inventory.md`.

| Group | Concepts |
|---|---|
| Fields that no production code reads | `Room.exits` (read it: the Game rules check only that the destination exists, so the player can go from any Room to any Room). `Room.items`, `inventory` of a Character and a Persona. `maker` of a Swipe (read it). `currently_meeting` (read it, and the mutation run agrees). `example_dialogue` for prompts (read it) |
| Code with no shipped content | The Trigger operators `Gte` and `Lt` (the one shipped Trigger uses `Eq`). A second Trigger. `single_user_message`. `provider_order`. Custom sampler parameters and `option_count` of a preset. The text check (shipped as Disabled) |
| Code that only tests reach | The Mock provider. The four `assert_*` checks behind the `diagnostics` feature (the mutation run: no test makes one fail) |

- **Reading.** The engine is wider than the one World that tests it in play. Each row of the second group is a path that real play has never run. The Trigger faults in `story-damage-and-false-screen.md` come from this gap.
- **Two ways to close it, and they differ in cost.** Delete what has no user: your decision for each row. Or ship a second small World whose purpose is to use the rest: several Triggers, each operator, a Room chain that needs exits. The lab World of my probe (`trigger_lab`, three Rooms, four Characters) is a start.
- **I recommend the second World first.** It turns "no user" into a fact for each concept, and it is cheap.

## 5. Comparison with the genre

**Researcher.** A model compared Chronicler with SillyTavern, Marinara Engine, RisuAI, KoboldAI Lite and AI Dungeon from their own documents: `leads/c4-genre-comparison.md` (26 rows, sources listed). It could not fetch NovelAI. I checked two of its open points in the code.

**Where Chronicler is ahead:** game state that the engine keeps (the Room, the Characters present, Triggers), Autoplay with a budget, and the stored record of every model call.

**Gaps that none of your records names**, in the researcher's order of weight (the order is its judgement):

| Gap | Who has it | Note |
|---|---|---|
| Streaming of a Narration | All five | The page polls, by design (`dashboard_design.md`). No record weighs that choice against a turn of several seconds. Finding 7 of `engine-defects-and-risks.md` has the engine side |
| A Character editor or a card import in the dashboard | All five | The Worlds tab edits the World and the map only. A World made in the dashboard cannot get a Character |
| Export, import and backup of a Game | Kobold Lite, Marinara, RisuAI | One SQLite file beside the binary. No record tells a player this |
| A branch, or a repair of an older entry | Kobold Lite, SillyTavern, Marinara | Retry and delete act on the last entry. Edit acts on any entry |
| Text to speech, scene pictures, interface language | Several | Low weight for a first release |

**Checked, not gaps:** the Connection form has a base-URL field (`settings.rs:78`), so another OpenAI-compatible host can be entered (I did not run one). An empty Send is Continue.

## 6. Readiness for a release

`PRODUCT.md` names a public release as a goal and leaves two decisions open (accounts, and how much of the tuning surface a player sees).

### A cold start by a stranger (ran it)

I copied the package to `/tmp/fable-cold` and followed `README.md` only.

- The server started. It printed nothing to stdout, so a person cannot see the address or that it runs.
- The first Input answered 200, and the status read "The engine failed with an internal error." The Details name the missing API key.
- `README.md` says that Connections "live in `data/settings.json`". It names no key, no `.env` file and no environment variable. The default Connection needs `OPENCODE_GO_API_KEY`.

**Fix.** Three lines in `README.md`. A line on stdout with the address. A failure sentence for a missing key that says "no API key" and where to set it.

### What blocks a public release (read it)

| Item | Fact |
|---|---|
| Licence of the code | No `LICENSE` file. `Cargo.toml` has no `license` key |
| Licence of the content | The five Character cards and their art came from SillyTavern cards (`scripts/extract_sillytavern_png.py`). No file records their source or licence. `work/wayfinder-maps/character-art-from-code/issues/04` notes the same for the reference pictures |
| Licences of the dependencies | Of the 347 packages on disk, all are permissive except `option-ext` (MPL-2.0, through `dirs`). No tool makes a notice file |
| The package | `build.py` copies `data/` and `assets/` into the target folder. The package includes the test World. The three `/debug` routes are always on |
| The test seams | `default = ["testing"]`, so the shipped binary carries them. None is reachable at run time. A build without the feature compiles with one warning, which the gate would refuse, and no gate step builds it |
| The network | The server binds `0.0.0.0` by default and has no login (finding 1 of `engine-defects-and-risks.md`) |
| The story on disk | The log files hold story text beside the database. No key was in the logs that I searched |
| A guide | No text for a player or an author. `docs/diataxis/` has explanation and reference only |
| Author data | Only `scripts/validate_data.py` checks a World, and only in the repository. See finding 9 of `story-damage-and-false-screen.md` |

Your records hold three of these: the package content (weekly retro), the `testing` feature (code-concepts research 07) and accounts (`PRODUCT.md`).

## 7. Questions for you

| # | Question | My recommendation |
|---|---|---|
| 1 | Should a Character's `example_dialogue` reach the prompt? | Yes, cut to two lines for each present Character. It is the cheapest test of finding 1, and one trial shows the effect |
| 2 | Is the Redmist World meant to carry a plot, or to be an open stage? | Your choice. If a plot: one threat, one deadline, and a stake for each Character. If a stage: the invented cast is the expected result |
| 3 | Should the trial pack count story facts? | Yes. The counts are free. They also need three runs of one setup first |
| 4 | Delete the concepts with no user, or ship a World that uses them? | The World first |
| 5 | Which licence, and what is the source of the cards and the art? | Only you know. It blocks a release and nothing else |
| 6 | Is the release a local install for one person? | If yes, the list of section 6 is short: the README, the bind address, a licence, a player guide |

## 8. What I did not do

- **C1, the value of each mechanism.** The test is the same trial with one mechanism off: no Quantifier, or no Room in `<GameState>`, or no analysis after an Impersonate Input. It needs three or four trials of about 3M tokens each, and three runs of each because of the noise in section 3. I did not spend your credit. This audit could change the product direction more than any other in the catalogue.
- **Real play.** Every story in this report comes from Autoplay. A person types shorter Inputs and pushes differently. Finding 3 is about the Impersonate preset, so it applies to Autoplay and to `/impersonate` only.
- **Another model.** All trials use one model.
- **The Options presets and the interactive-fiction preset.** No stored trial uses them.
- **A phone layout and accounts.** `PRODUCT.md` names both as open. I did not test the page at phone width.
