# Pass 3: does the owner's own records already hold each candidate?

The finding is the class only. No judgement on whether a candidate is right.
Searched: `.scratch/` (every effort, `_closed/`, every `assets/` folder), `docs/specs/`,
`docs/diataxis/`, `docs/plans/`, `PRODUCT.md`, `README.md`, `ENVIRONMENT.md`, `CONTEXT.md`,
`tests/STRATEGY.md`. Not searched: `tmp/` (except this audit), and `tmp/fable-audit` is not a record.
Verdicts: DECIDED = a record states the behaviour is intended or accepted. KNOWN = a record names
the same fault or gap. PARTLY = a record names the cause or a near neighbour, not this consequence.
NEW = three or more word choices, nothing found.

Counts: DECIDED 3, KNOWN 24, PARTLY 21, NEW 11.

## Story damage and false screen

| Id | Class | Record | Quote (<=30 words) / what is missing |
|---|---|---|---|
| P1 | KNOWN | `work/wayfinder-maps/state-machines-next/assets/03-dispositions.md:82` | "A restart during a Job leaves the Game idle with no failure line … The story can end on an Input with no answer." |
| P2 | KNOWN | `work/wayfinder-maps/state-machines-next/assets/scout-3-driven-and-bootstrap.md:91` | "a crash between `save_text` and `save_state_update` leaves a Game that reads `Idle` with a Swipe state that the game rules never updated" |
| P3 | KNOWN | `work/wayfinder-maps/state-machines-next/assets/scan2-application.md:116` | "(known) Role health … reflects only transport failures. A Quantifier that always answers unparseable text shows as healthy." |
| P4 | KNOWN | `work/wayfinder-maps/state-machines-next/assets/scan2-application.md:116` | same line as P3. The swallow itself is DECIDED at `docs/specs/actions.md:199`: "A failed or unparseable quantifier answer still completes the action". |
| P5 | KNOWN | `work/wayfinder-maps/state-machines-next/assets/scout-2-application.md:60` | "an unrecognised movement word still teleports the Persona, and only the log line ever showed the type" |
| P6 | KNOWN | `work/wayfinder-maps/invariant-audit/assets/REPORT.md:18` | "A model that answers names (`"Carla"`) instead of ids … empties the scene: every Character gets a `Left` transition." |
| P7 | KNOWN | `work/wayfinder-maps/state-machines-next/assets/scan2-driving.md:157` | "Health is lost by retention, not by a success … The banner then clears with no success." |
| P8 | KNOWN | `work/wayfinder-maps/state-machines-next/assets/scan2-driven.md:25` | "the Options failure is not part of this status at all. It lives in the dock … a Game can be *Idle* while the dock shows a failure line." |
| P9 | PARTLY | `docs/diataxis/reference/narrative/prompt_system.md:101` | "Each Game references one active preset per type … every call reads the referenced preset fresh from storage." Missing: that the badge wording reads as a Game change. |
| P10 | PARTLY | `work/wayfinder-maps/invariant-audit/assets/REPORT.md:23` | "A World with `scenarios_json: []` … passes the create form … The new Game is unplayable." Missing: the 500 for every `/screen` part and the blank dashboard. |
| P11 | PARTLY | `work/wayfinder-maps/invariant-audit/assets/http.md:169` | "A World keyed `a/b` … can never be edited or deleted from the dashboard again." Missing: the exit to a missing Room and the duplicate Room id. |
| P12 | KNOWN | `work/wayfinder-maps/code-concepts/assets/research-10-synthesis.md:262` | "an unknown `check_mode` turns text check off (`http/settings/handlers/settings.rs:132-137`)" |
| P13 | PARTLY | `docs/diataxis/reference/narrative/prompt_system.md:89` | "the filter does not touch that Input" (only `<PlayerInput>`, only `{{…}}`). Missing: the XML breakout in `<LastMessage>`. |
| P14 | KNOWN | `work/wayfinder-maps/state-machines-next/assets/03-dispositions.md:32` | "`check_condition` (`trigger.rs:94-101`) reads `times_met` only (known)" |
| P15 | KNOWN | `work/wayfinder-maps/invariant-audit/assets/REPORT.md:19` | "`evaluate_triggers` iterates `HashMap::values()` … The order is random per map instance." The no-cooldown half: `scan2-domain.md:16` "(nothing for a repeat Trigger)". |
| P16 | KNOWN | `work/wayfinder-maps/invariant-audit/assets/REPORT.md:22` | "Retrigger trusts the event record and skips the fired mark. Delete the event Narration, then Retrigger: the same event is written again." |
| P17 | KNOWN | `work/wayfinder-maps/invariant-audit/assets/REPORT.md:28` | "BUILD-TIME only. The Trigger never fires, and a DEBUG line is the only sign." |
| P18 | PARTLY | `work/wayfinder-maps/invariant-audit/assets/llm.md:30` | "the visual sidebar loses the real Room image (`image_path` is `None`)". Missing: the missing name and the raw id in the story-log header. |
| P19 | KNOWN | `work/wayfinder-maps/state-machines-next/assets/scout-2-application.md:24` | "An edited text leaves the old failure line and the old offered set." |
| P20 | KNOWN | `work/wayfinder-maps/invariant-audit/assets/REPORT.md:25` | "`save_config` refuses only an Autoplay holder … A mode, posture or preset change during a plain Job is accepted." |
| P21 | PARTLY | `work/_closed/turn-state-model/assets/clean-design-glm.md:362` | "the Opening Narration holds the World's opening text and no LLM call produces it … cannot be retried". Missing: the observed replacement and the Trigger fire. |
| P22 | KNOWN | `work/wayfinder-maps/weekly-retro-2026-11-04/assets/README.md:40` | "**An event Narration sends the Narration before it twice**: once as the last history entry, once as "Previous narration"" |
| P23 | PARTLY | `work/_closed/turn-state-model/assets/research-07-message-history-readers.md:82` | "`timestamp` as `%H:%M`". Missing: no date and no zone, and that this is a fault. |
| P24 | NEW | — | Four choices searched: "Generating narration", "step label", "data-status", "Autoplay: cycle". The label table at `docs/specs/dashboard.md:74` names only the narrator call. |
| P25 | PARTLY | `work/wayfinder-maps/prompt-flexibility/map.md:46` | "a blank layer renders nothing (the empty `<PlayerInput>` of Impersonate, Continue and Guide)". Missing: the output_format line that says an action is given. |
| P26 | PARTLY | `work/_closed/turn-state-model/assets/target-design.md:247` | "\| Internal error \| "The engine failed with an internal error." \|". Missing: the cold start, the missing key and the silent stdout. |

## Lenses

| Id | Class | Record | Quote (<=30 words) / what is missing |
|---|---|---|---|
| L1 | DECIDED | `work/wayfinder-maps/prompt-flexibility/issues/10-quantifier-options-layout.md:43` | "**The answer format stays preset text (owner, option A).** … A changed format makes every parse fail. The state then stays as it was" |
| L2 | DECIDED | `docs/diataxis/reference/narrative/agent_system.md:60` | "**Options.** One of three shapes: a `<suggestion>` tag for each option, a numbered list, or a `Suggestion N:` line" |
| L3 | KNOWN | `work/wayfinder-maps/prompt-flexibility/map.md:46` | "The agent calls still fix the posture placeholders to third and past" |
| L4 | KNOWN | `work/wayfinder-maps/invariant-audit/issues/02-grilling-runtime-world-validation.md:13` | "**Scenario Characters that do not exist (world-03).** Only `validate_data.py` checks them, at build time." |
| L5 | PARTLY | `work/wayfinder-maps/code-concepts/assets/research-09-docs.md:175` | "Which one wins when schema, type and database row differ". Missing: the named defects (character `id`, exit keys, posture fields). |
| L6 | KNOWN | `work/wayfinder-maps/invariant-audit/assets/REPORT.md:22` | story-06. The three homes: `work/wayfinder-maps/state-machines-next/assets/scan2-domain.md:11` names `event_record`, `trigger_fired` and `MessageKind::EventNarration` as state variables. |
| L7 | NEW | — | Three choices: "maker", "Maker::", "the maker". The doc line is `docs/diataxis/reference/game_flow.md:28`; no record says it has no reader. |
| L8 | PARTLY | `work/wayfinder-maps/code-concepts/assets/research-09-docs.md:148` | "The room name of a Message is derived from the Swipe state when it is read … Which derived values does the system store?" Missing: the rename of past entries. |
| L9 | NEW | — | Three choices: "data-status", "shared constant", "statusShows". `scout-4-driving.md:3` lists the values; no record names the missing shared constant. |
| L10 | PARTLY | `work/wayfinder-maps/state-machines-next/assets/04-dispositions.md:91` | "`seed_settings` checks Connection slots only (`load.rs:69`), not preset ids". Missing: the four places that hold a default id. |
| L11 | PARTLY | `work/wayfinder-maps/code-concepts/assets/research-02-domain.md:57` | "`NarratorMode` … give a strict `FromStr` … and also a `parse_or_default` that logs and falls back". Missing: a new variant that compiles and is refused at run time. |
| L12 | PARTLY | `work/wayfinder-maps/code-concepts/assets/research-05-driving-adapters.md:46` | "`http/action/handlers/actions.rs:87-93,104-108` (text-check fault → send the command)". Missing: that only one route runs the check. |
| L13 | NEW | — | Three choices: "busy refusal", "action/check", "before every other check". `game_flow.md:52` lists only edit, delete and swipe switch. |
| L14 | KNOWN | `work/wayfinder-maps/invariant-audit/assets/llm.md:40` | "`Room::exits` … is written by World authors and **never read** anywhere in `src/`". `items`/inventory: only column lists (`storage.md:513`). |
| L15 | KNOWN | `work/wayfinder-maps/code-concepts/assets/research-06-composition-and-cross-cutting.md:195` | "a save error of the failure record is logged and the model error returns … a save error fails the call, though the text exists." |
| L16 | DECIDED | `work/wayfinder-maps/prompt-flexibility/issues/22-call-parameters-on-presets.md:200` | "The new preset fields and the two optional slots have no UI." Accepted at `:135` "Accepted (O3). Use a new preset id or a new database". |
| L17 | NEW | — | Three choices: "one World", "shipped data", "one Trigger operator". Nothing found. |

## Engine shape and release

| Id | Class | Record | Quote (<=30 words) / what is missing |
|---|---|---|---|
| E1 | KNOWN | `work/wayfinder-maps/code-concepts/assets/research-07-test-infrastructure.md:170` | "Does the `testing` feature ship in a release build? … probably compile into the shipped binary (inferred, not tested)." Also `research-10-synthesis.md:319`: "no `--no-default-features`". |
| E2 | PARTLY | `work/wayfinder-maps/character-art-from-code/issues/04-prototype-the-art-style.md:57` | "The reference pictures and the parts sample have no known source or licence." Missing: the repo `LICENSE` and the shipped cards and art. |
| E3 | KNOWN | `work/wayfinder-maps/weekly-retro-2026-11-04/assets/dispositions.md:78` | "the dead DB dump and the stale-data trap … Copy `data/` only with `--release`." |
| E4 | NEW | — | Three choices: "api key", ".env", "onboarding". `README.md` holds no key or `.env` line; no record names the gap. |
| E5 | PARTLY | `work/wayfinder-maps/layout-trials/map.md:45` | "An ad hoc LLM judge reads one whole story … The output candidates are set aside". Missing: the 1-to-5 scale as the only measure and the absent noise floor. |
| E6 | PARTLY | `work/wayfinder-maps/unclebob-quality-tools/map.md:28` | "both cargo-mutants and a driver over `build.py test-pattern` run (in a worktree); 10 survivors on `parser.rs` and `gate.rs`". Missing: this run and its results. |

## Development environment

| Id | Class | Record | Quote (<=30 words) / what is missing |
|---|---|---|---|
| D1 | NEW | — | Three choices: "204 rules", "prose only", "held by a check". Nothing found. |
| D2 | NEW | — | Three choices: "judge-glm", "judge-deepseek", "role file". `conceptual-integrity/assets/research-04-ways-to-align.md:53` names the role files but not the duplicate `name`. |
| D3 | NEW | — | Three choices: "DOC:", "lib.rs:1", "test_support". Nothing found for either half. |
| D4 | PARTLY | `work/_closed/platform-choice/issues/13-decide-module-shape-docs-and-guardrails.md:121` | "**Keep (20)** … file length … nesting depth". Missing: that no prose states them. |
| D5 | PARTLY | `work/wayfinder-maps/weekly-retro-2026-11-04/assets/findings/sessions-delegates.md:42` | "**108 denials, 90 sessions, one turn each; 4 lost a build.**" Missing: the 341 count since 2026-09-27 and the `.env` denials. |
| D6 | PARTLY | `work/_closed/hex-comparison/assets/research-01-agent-instructions.md:30` | "Owner approval, git rules and permissions are inline in `AGENTS.md:410-414`". Missing: that the config's git deny rules went away. |
| D7 | NEW | — | Three choices: "merger", "implement-spec", "commit without approval". Nothing found for either skill. |
| D8 | NEW | — | Three choices: "open tickets", "111", "grilling tickets". Counts exist per effort; no record holds the total. |
| D9 | KNOWN | `work/wayfinder-maps/weekly-retro-2026-11-04/assets/findings/tooling-env.md:5` | "every cost number in `AGENTS.md` and `ENVIRONMENT.md` still shows the old state." The data: `assets/README.md:15` "999 runs, 13.4 h … `test-pattern` 378 runs, 44 % red". |
| D10 | PARTLY | `work/wayfinder-maps/weekly-retro-2026-11-04/assets/README.md:9` | "Commits since `3525193c` \| 79 (21 `feat`, 11 `fix`, 12 `docs`, 5 `refactor`)". Missing: the product-against-environment split since August. |
