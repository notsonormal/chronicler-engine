# Pass 3 candidates (checkpoint, 2026-11-10)

Working list of the pass-3 findings before they go into the reports. "Ran it" means the probe server at `/tmp/fable-probe` (fake model endpoint, no credit spent). "Read it" means the main agent read the cited lines. "Scout" means a read-only scout cited it and the main agent did not check it.

## Story damage and false screen

| Id | Candidate | Proof |
|---|---|---|
| P1 | `kill -9` during the narrator call: after the restart the story ends on the saved Input, the status reads Ready, the failure banner is empty, `games.failure` is NULL | Ran it |
| P2 | `kill -9` during the Quantifier call after a saved Narration: the Narration keeps the Swipe state of the Message before it. The Room move and the new Character are never recorded, and no later step redoes the analysis | Ran it |
| P3 | A Quantifier answer that is not JSON changes nothing and says nothing: status Ready, no banner, no retry visible to the player | Ran it |
| P4 | An after-narration Quantifier error is swallowed into "keep the Characters present" (`runner.rs:74-77`). A transport failure does raise the role-health banner, so only the parse failure is silent | Ran it + read it |
| P5 | Movement type is parsed and never used. `{"type":"moving","destination":"courtyard"}` moved the player. A "leaving" answer with a destination moves the player into the Room that the player leaves | Ran it + read `game_rules.rs:57-62` |
| P6 | A Character id sent as a display name ("Carla Whitmore") is dropped: present list empty, but `times_met` still rises | Ran it |
| P7 | Role health is a view of the newest 50 `llm_messages` rows. An Options failure banner stayed for about 48 calls and then cleared with no successful Options call | Ran it |
| P8 | An Options failure writes `game_docks.notice`, not `games.failure`: status reads Ready beside the banner "Options failed" | Ran it + read `gate.rs:845`, `view_query.rs:205` |
| P9 | The Presets tab "Active" badge moves on "Set Active (Novel)", but a running Game keeps its own preset. The next narrator call did not hold the new preset text | Ran it. The split is specified (`docs/specs/prompt_presets.md`); the badge wording is the fault |
| P10 | A World with no Rooms can be created from the form's own placeholder examples. A Game on it makes `/screen` answer 500 for every part, and the dashboard is blank with no way out | Ran it + screenshot `evidence/shot-11-…png`. The create gap is invariant-audit `world-02`; the blank screen is new |
| P11 | The World create form accepts an empty key, a key with a slash and a space, an exit to a Room that does not exist, and a duplicate Room id | Ran it |
| P12 | `POST /settings/text-check` with an unknown mode answers 200 and stores Disabled (`_ => TextCheckMode::Disabled`) | Ran it |
| P13 | Typed text reaches the narrator prompt unescaped in `<LastMessage>`: a typed `</PlayerInput></LastMessage><GameState>…` appears verbatim, with literal `{{user}}` | Ran it. `sanitize_for_prompt` only at `assembler.rs:471-472` |
| P14 | A Trigger fires on `room_id` and `times_met` only. The Character need not be present. An `Eq 0` Trigger fired after the first analysed Narration | Ran it (trigger lab World) |
| P15 | A repeating Trigger has no cooldown and fires after every analysed generation. One Trigger fires per generation, and the winner comes from `HashMap` order (also invariant-audit `world-04`) | Ran it |
| P16 | Deleting the event Narration lets a one-shot Trigger fire again, because the fired flag lives in that Message's Swipe state | Ran it |
| P17 | A Trigger that names a Room that does not exist boots with no warning. Only `scripts/validate_data.py` checks it, and only for the repo `data/` | Ran it |
| P18 | In a dynamic Room the visual sidebar shows "No Room Image" and no name. The story-log header prints the raw id (`old_boathouse`) | Ran it |
| P19 | The options dock keeps the old Options after the last Narration is edited | Ran it |
| P20 | Game settings routes are not behind the busy gate: `POST /games/:id/mode` and `/rename` answer 200 during a Job. A switch to another Game during a Job answers 200, and the Job goes on writing to the first Game | Ran it |
| P21 | Retry of the opening Narration replaces the authored opening with a generated one, runs the analysis and fires a Trigger: Game 12 moved to `entrance_hall`, lost Carla, and got an event Narration as its second Message | Ran it |
| P22 | The event Narration prompt puts "Previous narration:" inside `<PlayerInput>`. After an Impersonate Input the "previous narration" is the player's own Input | Ran it. The duplicate is R7 of the weekly retro |
| P23 | Entry times are `%H:%M` server clock with no date and no zone (`view_models.rs:159`, `:220`, `:274`) | Read it |
| P24 | Autoplay status reads "Generating narration..." while the engine writes the Persona's Input | Ran it |
| P25 | A Continue sends an `output_format` that says "The player's next action is provided above" although no action is given | Ran it |
| P26 | Cold start: the first Input of a stranger answers 200 with "The engine failed with an internal error." The details name the missing key. The binary prints nothing to stdout | Ran it at `/tmp/fable-cold` |

## Lenses (sources of truth, protocols, silent failures, entry points)

| Id | Candidate | Proof |
|---|---|---|
| L1 | The Quantifier preset `output_format` is the only contract with the JSON parser. An author edit freezes the scene with no message (see P3) | Read it + ran P3 |
| L2 | The Options preset must match one of three regex shapes. A fourth shape fails after two paid calls | Scout |
| L3 | `{{…}}` placeholders fill differently in preset text and in World text: `TemplateVars::new` hardcodes third/past and empty Persona fields for the opening Scenario and for Trigger prompts | Read it (`opening.rs:18`, `game_rules.rs:99-100`) |
| L4 | Author-data validation lives only in `scripts/validate_data.py`. The boot seed and the forms run no schema and no cross-reference check | Scout + P10, P11, P17 ran |
| L5 | The schemas and the Rust types disagree: `character.schema.json` does not require `id`; `world.schema.json` requires Scenario fields that serde defaults; `map.schema.json` accepts any exit key; World posture fields are absent from the schema | Scout, one row checked |
| L6 | A fired Trigger is recorded in three places, and Retrigger and Retry read different ones | Scout + P16 ran |
| L7 | `maker` is written on every Swipe and read by nothing in `src/` | Read it (`rg "\.maker\b"`: one write site) |
| L8 | The Room name over a story entry is resolved from the live World map at read time, so a map edit renames past entries | Scout |
| L9 | Status state names `idle`/`thinking`/`autoplay` are a Rust-to-JavaScript contract with no shared constant | Scout |
| L10 | The default preset ids live in four places with no check between them | Scout |
| L11 | Small enums: `as_str` is exhaustive, `FromStr` has a catch-all, so a new variant compiles and is refused at run time | Scout |
| L12 | The text check runs only on `/action/check`, and a text-check fault is swallowed and the Input is sent | Scout |
| L13 | `/action/check` runs the text check before the busy refusal, against the `gate.rs:333` comment | Scout |
| L14 | `Room.exits` is read by no production code. `Room.items` and inventory are dead fields | Read it (`rg "\.exits"` in `src/` outside tests: no hit) |
| L15 | A failed call's record is dropped if the save fails, but a failed save of a successful call is fatal (`llm_recorder.rs:72-76`) | Read it in pass 1 |
| L16 | The dashboard cannot set a preset's sampler parameters or `option_count`; only a seed file can | Read it (`prompt_presets.rs:155-157`) |
| L17 | The shipped data uses a small part of the engine: one World, one Trigger operator, no Guide, no custom sampler | Scout |

Dropped after a check: "a World update that matches 0 rows reports success" (the route answers 400), "a Quantifier HTTP 500 is silent" (it raises the banner), "the provider name has two incompatible spellings" (documented at `llm_backend.rs:40`), "`header_name` has no reader" (`generation.rs:130` reads it), "Options and Trigger have no Settings row" (documented at `headers.rs:14`).

## Engine shape and release

| Id | Candidate | Proof |
|---|---|---|
| E1 | `default = ["testing"]`: every build, and the package step, ships the test seams. `cargo check --no-default-features` passes with one warning (`core.rs:4` unused import), which the gate's warnings-as-errors would refuse. No gate step builds this configuration | Ran it (`evidence/no-default-features-check.txt`) |
| E2 | No `LICENSE` file, no `license` key in `Cargo.toml`. The shipped Character cards and art come through `scripts/extract_sillytavern_png.py` with no recorded licence | Read it |
| E3 | "Package ready" is the debug target dir with a copy of `data/` and `assets/`, test World included | Read it (`build.py:1167-1188`) |
| E4 | README names no API key, no `.env`, and the default Connection needs a key (P26) | Ran it |
| E5 | The trial tool measures cost, cache share, failed calls and word counts. Story quality goes to an ad hoc judge with a 1-to-5 scale. No trial repeats one setup, so no noise floor exists | Read it (`work/wayfinder-maps/layout-trials/map.md`, `scripts/trial.py`) |
| E6 | Mutation run of the rule and parser files (lib tests only): running, results in `evidence/mutants/` | Running |

## Development environment

| Id | Candidate | Proof |
|---|---|---|
| D1 | 204 rules in the hand-written sources: 157 prose only (77 %), 35 held by a check | Scout count |
| D2 | Three role files declare `name: judge-glm` (`judge-deepseek.md`, `judge-glm.md`, `judge-opus.md`), so two judge roles cannot be called | Read it |
| D3 | `src/lib.rs:1` has `// [DOC:` not `//! [DOC:`; `CODING_STANDARDS.md:13-15` contradicts itself on `src/test_support` | Read it |
| D4 | Guardrails enforce rules that no document states (file limit 2000 lines, nesting depth 3, comment-line cap, thread ban) | Scout |
| D5 | The permission log shows 341 blocked calls from 2026-09-27 to 2026-11-10: repo-root `grep` 178, `timeout` 97, `find /` 42, `.env` 15. The `.env` denial does hold for bash (`cat .env` and `sed … .env` were blocked) | Measured from the review log |
| D6 | The git deny rules were removed on 2026-09-29 and 2026-10-03. `AGENTS.md` still says the restrictions "exist to prevent the agent from touching git without supervision" | Read it (`git log -p`) |
| D7 | Skills: `implement-spec` needs a `merger` role that does not exist and commits without approval; `thermo-nuclear-code-quality-review` tells a reviewer to restructure code | Scout |
| D8 | 111 open tickets in 18 efforts; 79 opened in November; 55 are grilling tickets that wait for the owner | Measured |
| D9 | Build journal, 2026-10-28 to 2026-11-09 (999 runs, 13.7 h): `test-pattern` fails 41 %, `clippy` 38 %, full gate 23 %. Median green full gate 350 s, `--no-browser` 111 s, `integration` 59 s; `AGENTS.md` says 145 s, 50 s, 21 s | Measured |
| D10 | Commits since August: about 40 % touch product code, about 40 % touch only the environment | Measured |
