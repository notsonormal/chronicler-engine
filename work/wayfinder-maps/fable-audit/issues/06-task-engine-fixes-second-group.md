# Engine fixes, second group: the findings that no question asked

Type: task
Status: open
Blocked by: —

## Question

Put the changes below in place (owner, 2026-11-10, D19, D20 and D21 of [the decisions](../decisions.md)). They are the rows N1 to N12 and N14 of [the list of findings that were not carried](../assets/findings-not-carried.md). The owner accepted them as a group and did not read each one. So a change that needs a choice of product behaviour stops, and the choice goes to the owner.

The changes are in five groups, and one agent can take one group. The owner decided the two changes of group E one by one.

### Group A: the prompt and the model call

1. **The prompt of an event Narration (N1).** Read it in call 1450 of the trial `ltm09`, and in the code.
   - **The last Narration is in the prompt twice.** It is the `<LastMessage>` entry of the history, and `continuation_user_message` (`narration/narrator.rs:185`) puts it in again after "Previous narration:". Remove the copy. This is R7 of the weekly retro of 2026-11-04.
   - **The Trigger direction is labelled as the player's Input.** The message goes through the user layer, which wraps it in `<PlayerInput>` (`prompting/assembler.rs:472`). Give the event its own closing layer with the direction and the instruction.
   - **The Character of the Trigger has no card in the call.** In the trial the block read "No Characters are present in this Room", and the Trigger was "Gabriella greets the player". Add the card of that Character to the present block for this one call.
   - Check the cached share: the blocks before the history must keep their bytes.
2. **The narrator call gets a second attempt (N2).** Read it.
   - The Quantifier and the Options Agent have two attempts (`quantifier/utils/orchestration.rs:94`, `options/utils/orchestration.rs:18`). The narrator has one, so one HTTP 429 or 502 ends a Job and an Autoplay run.
   - The change: one more attempt for the narrator on a network fault, a 429 or a 5xx. Not on a 4xx of another kind.
   - Each call builds a new HTTP client (`transport/utils/client.rs:25`). Share one client in the provider.
   - One limit of 180 s covers the whole call (`client.rs:26`). Add a connect limit. Do not change the total limit without a measured reason.
3. **The Quantifier is not asked for a movement type (N3).** Ran it and read it.
   - The preset asks for `"type": "entering|in|leaving"` (`data/prompt_presets/quantifier/default.json:6`). The parser reads "entering" and "leaving" (`quantifier/utils/parser.rs:172`). No Game rule reads the type: `movement` gets the destination only (`game_rules.rs:57`). So an answer "leaving" with a destination moves the player into that Room.
   - The change: remove the type from the preset text and from the parser. An answer that still has the key is accepted, because a database of today keeps its preset text.
   - `parser_tests.rs:142` pins the parsed type. Change it with the code.
4. **A placeholder in World text gets the values of the Game (N7).** Read it.
   - The opening and a Trigger direction render with `TemplateVars::new(persona_name)` (`utils/opening.rs:18`, `utils/game_rules.rs:99`). That gives third person, past tense and empty Persona fields, whatever the Game has (`template.rs:29`).
   - The change: both sites get the same values as a preset text of that Game.
5. **Three small faults of the prompt (N10).**
   - An added Room has the description "A place you have never seen before." (`game_rules.rs:127`), and the narrator prompt shows it on every later call in that Room. Leave the description empty, and render no description line for it.
   - An empty Send keeps the sentence "The player's next action is provided above" of the System preset (`data/prompt_presets/system/default.json:7`). No action is above. This is a change to preset text: keep it to that one sentence.
   - `sanitize_llm_output` removes the indentation of every line (`prompting/sanitize.rs:25`), so a nested list or a code block in a Narration loses its shape. Find the reason for the trim in `git log` first. If a model needs it, stop and say so.

### Group B: the author's records and the boot

6. **The boot does what `startup.md` says, and it says what it skipped (N6).** Read it.
   - `docs/diataxis/reference/startup.md:26` says: "A malformed world, persona, or preset file is skipped without halting bootstrap." A malformed map file and a malformed Persona file end the whole seed of the game data instead (`bootstrap/load.rs:122`, `:178`, both with `?`). The Worlds and Personas after that file are not seeded. Skip the one file.
   - A preset file with no `id` gets the id `default`, and with no name the name "Default" (`bootstrap/run.rs:237-240`). Refuse the file.
   - A skipped file is reported only in the log file. Print one line for each skipped file on stderr at boot, with the path and the reason.
   - `scripts/validate_data.py` checks that a Scenario's Character has a file `<id>.json`. The engine uses the `id` inside the file (`load.rs:147`). Make the script compare the `id` inside the file.
7. **The World form's examples make a World that works (N4, first part).** Ran it: the placeholder examples of the form (`worlds/templates/worlds.rs:185`) are a map with no Room and an empty Scenario list. A World made from them blanks the dashboard. Change the examples to the smallest World that plays: one Room and one Scenario that starts in it.
8. **The server says where it is, and it does not start without its assets (N8, N11).**
   - Ran it: the server prints nothing on stdout. Print one line with the address when the port is open.
   - Ran it: a server that starts from another directory answers 200 for `/` and 404 for every script, style sheet and picture. The assets are relative to the working directory (`builders/router.rs:203-205`). At boot, check that `assets/index.html` exists from the working directory. If not, end with one sentence that names the folder.
9. **The log (N11).** Read it: `logging.rs` makes one file for each day and deletes none. At level `info` the Quantifier writes the text of each Narration to the log (`quantifier/utils/orchestration.rs:101`). Keep a fixed number of files, and move the story text to level `debug`.
10. **One test for each fact that has several homes (N14).**
    - The default preset ids are in the Rust constants (`utils/settings_defaults.rs:8-10`), in the seed files and in `data/settings.json`. One test compares them. The literals in the migrations are history and stay.
    - The default Agents are in `AgentConfig::defaults()` and in the `agents` list of `data/settings.json`. Only a scout read this. Check it first, then add the same test.

### Group C: the screen and the failure sentences

11. **Small faults of the screen (N9).** Ran each one.
    - In an added Room the visual sidebar shows "No Room Image" and no name. Show the name of the Room as text.
    - Times are `%H:%M` of the server clock (`view_models.rs:159`, `:220`, `:274`). An entry of yesterday and an entry of today look the same. Show the date for an entry that is not of the current day.
    - An entry that is an event and also a Room change shows one of the two headers. Show both.
    - The Presets tab says "Active" and "Set Active" for the preset that a new Game gets (`builders/presets.rs:196-219`). A running Game keeps its own preset, as `docs/specs/prompt_presets.md` specifies. Change the words so that they say "default for new Games". The spec and its tests change with them.
12. **A failed model call names its cause (N11).** Ran it for HTTP 500 and for a missing key.
    - A missing API key shows "The engine failed with an internal error." The Details name the key. Give this case its own failure sentence, which names the Connection and says where to set the key.
    - Every HTTP status maps to "The language model could not be reached." (`state/generation_status.rs:207`). Give a 401 or 403 (the key is refused), a 402 (no credit) and a 429 (too many requests) each their own sentence.
    - `docs/specs/failure_display.md` and the answer-words table hold these sentences. Change them with the code.

### Group D: the text check and the dependencies

13. **The text check (N5).** Read it. The shipped setting is Disabled, so nobody meets this today.
    - `/action/check` runs the text check before the busy refusal. The comment at `generation/gate.rs:333` says that the busy refusal comes before any other check. Make the route do what the comment says.
    - A fault of the checker is logged, and the Input is sent with no notice (`action/handlers/actions.rs:79-82`). **Open design point:** the Input is sent and the answer says that the check did not run, or the Input is not sent. Choose, and write the reason in `## Answer`.
14. **The dependencies (N12).** Checked against the RustSec pages on 2026-11-09.
    - `dotenv` 0.15.0 is unmaintained (RUSTSEC-2021-0141). Replace it with `dotenvy`.
    - `pulldown-cmark` 0.13.3 has two advisories for an option that the engine does not set. Move to 0.13.6.
    - `libsqlite3-sys` 0.32.0 has RUSTSEC-2026-0459. The fix is `rusqlite` 0.41, which needs Rust 1.89, and `rust-toolchain.toml` pins 1.88.0. Do not change the toolchain here. Write the advisory into `ENVIRONMENT.md`.
    - An audit step in the gate needs `cargo-audit` on the machine and a network call. Check both. If one is missing, say so in `## Answer` and add no step.

### Group E: two decisions of the owner

15. **A Quantifier answer that the engine cannot use raises the Quantifier banner (D20).**
    - Ran it: the Quantifier answered "I cannot help with that request." on both attempts. The Room and the Characters present stayed, the status read "Ready", and no banner showed. An HTTP 500 of the same call does raise the banner "Quantifier failed — keeping the Room and the Characters present".
    - The owner's earlier decision expects the banner: "The state then stays as it was, and the banner shows" (`work/wayfinder-maps/prompt-flexibility/issues/10-quantifier-options-layout.md:43`). An author who edits the `output_format` of the Quantifier preset reaches this case on every call.
    - The change: when both attempts end with a reading of Low confidence, the result counts as a failure of the Quantifier role, with the same banner. The state is kept, as scenario 2.5 of `docs/specs/actions.md` specifies. The status line still reads "Ready" (D3).
    - **Open design point.** Role health is the `error_message` of the newest LLM message of the role (`games/view_query.rs:249`). An answer that cannot be read is a call that succeeded, so its record has no error. Choose the home of the fact and write the reason in `## Answer`. The banner must go away when a later Quantifier call gives a reading that the engine uses.
    - An answer that is valid JSON with an empty Character list is a valid reading. It raises no banner.
    - The failing test first: an HTTP test with a scripted Quantifier answer that is plain text on both attempts. The banner names the Quantifier, and the Swipe state is the state before.
16. **A map edit that removes the Room where a Game stands is refused (D21).**
    - Ran it: a World with the Rooms `hall` and `yard`, a Game that stands in `yard`, then a save of the World with a map that holds only `hall`. The save answered 200. The dashboard of that Game was blank, and each Action was refused with "The room yard is not on the map."
    - The change: the World update refuses the edit when a Game of that World stands in a Room that the new map does not have. The answer is a 400 that names the Room and the Games. A Room id that the edit renames counts as removed.
    - Where a Game stands: the current Room in the Swipe state at the end of its story. A Room that the Game added is in the Swipe state and not in the map, so the check does not apply to it.
    - The site: `WorldCatalogue::update_world` (`application/world_catalogue.rs:52`). [Ticket 07 of the invariant audit](../../invariant-audit/issues/07-task-all-or-nothing-world-edit-and-reset.md) puts the World edit in one transaction (storage-04). The check and the write belong in the same unit. If that ticket is not done, say in `## Answer` what gap stays.
    - A Job that runs during the edit can move its Game after the check. Say in `## Answer` how the change handles that case, or that it does not.
    - The failing test first: the run above as an HTTP test. The answer is a 400, and the stored map still has `yard`.

## Rules for every change

- Write the failing test first. Then make the change.
- Line numbers are from commit `e8d6512a`. Find the site again by its name before you edit.
- One change that turns out not to be local stops. Say so in `## Answer`, and do the other changes.
- A change to a spec scenario changes the scenario and its test together.
- Use `python build.py clippy` and `python build.py test-pattern` during the work. Run `python build.py` once at the end.
- Do not commit.

## Not in scope

- **The form accepts the provider `mock` (part of N11).** Five tests in `tests/http/settings_connection_form.rs` send `conn_provider=mock` through the form. So the refusal is not a local change: the tests need another way to make a Connection first.
- **A JSON mode for the Quantifier call (part of N10).** A provider that does not know `response_format` can refuse the request. No provider was tested, and all 602 Quantifier answers of the three trials were valid JSON.
- **An exit to a Room that does not exist, and the same Room id twice (part of N4).** They belong to the one World validation of [ticket 02 of the invariant audit](../../invariant-audit/issues/02-grilling-runtime-world-validation.md). A note there names them.
- A change to the status line for change 15 (D3: "Ready" stays).
- N13 (two wrong texts): [ticket 05](05-task-text.md) holds them.
- N15 to N18 of the list. The owner has not decided on them.

## What already exists

- [The list](../assets/findings-not-carried.md): each row names its report section.
- [The engine report](../assets/engine-defects-and-risks.md): findings 6, 7, 8 and 9, and section 4.
- [The story report](../assets/story-damage-and-false-screen.md): findings 1, 3, 4 and 10.
- [The lenses report](../assets/code-lenses.md): sections 2 to 5.
- `leads/pass3-tests-vs-behaviours.md` in `assets/`: which of these behaviours a test pins today.
