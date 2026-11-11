# The dashboard after the Rust rebuild: UI review findings

Ticket: [Review the dashboard after the Rust rebuild](../../issues/18-review-the-ui-after-the-rebuild.md). Run on 2026-10-19, 23:33 to 23:58.

- **Probe:** `wf/rebuild-core` at `d7cf3eb0`. `src/`, `tests/`, `data/` and `docs/` are the same as in `2885df2c` (`git diff --stat 2885df2c HEAD` on those paths is empty). Port 3001, target dir `target/ui-investigator`, world `redmist_estate`.
- **Baseline:** `bc7d6cd4`, extracted with `git archive` into `tmp/ui-review/base/`. Port 3002, target dir `target/ui-baseline`. The baseline shows if a fault is new in the rebuild or older.
- **Connections:** the real connections of `data/settings.json`. Narrator `deepseek-v4-flash` (OpenRouter). The Quantifier started on `ollama-gemma-4-e2b`, which fails as expected. Then it ran on `openrouter-gpt-4o-mini`, with one turn on `mistral-nemo`. On some turns gpt-4o-mini returned HTTP 400 from the Azure content filter. The dashboard showed that failure correctly (banner and Settings role row). It is a provider failure, not a finding.
- **Story played:** 1 Game of 21 short turns plus 10 manual turns in Redmist Estate. The Trigger "Gabriella Introduction" fired three times: two on an Action into the Entrance Hall, and one on the analysis of an `/impersonate`. A Retrigger also ran twice (see suspect 2). A second Game ran in Test Realm.
- **Shots:** `tmp/ui-review/NN-*.png`, dated 2026-10-19 23:34 to 23:56. The folder also holds shots of older reviews (2026-09-29 to 2026-10-07), so use the file names in this document. I compared each finding below with its shot.
- **Severity:** P1 a player loses work or cannot go on. P2 a feature is wrong or missing. P3 a visual or wording fault.
- **Labels:** *known* = seen in the running UI or read in the code. *inferred* = concluded from the code without a run.

## Count

| Severity | Count |
|---|---|
| P1 | 1 |
| P2 | 1 |
| P3 | 3 |

## Findings

### F1 (P1): Past 50 Messages, the story log hides the newest Messages, and its last-entry controls act on a hidden Message

- **Surface:** story log (`#story-log`), all controls of the last entry.
- **Steps:** Play a Game until its story has more than 50 Messages. Here: `tmp/ui-review/fill.sh 21` posted 21 Actions, which took the story from 11 to 53 Messages. Open the Game tab. Then click Delete on the last visible entry.
- **Expected:** The story log shows the newest Messages, and the last visible entry is the last Message of the story. Spec `story_log.md` 8.4 ("Delete-last removes the rendered log entry"): the Delete control removes the entry that the player sees.
- **Actual (known):** The DOM holds 50 entries, ids 1 to 64. The server holds 53 Messages, up to id 67. The player sees "I wait a moment and listen. (turn 20)" as the last entry. The Narration of turn 20 and all of turn 21 are not visible. Delete, Retry and Retrigger render on the last *visible* entry (`loop.last` in `NarrativeLogTemplate`), but `POST /history/delete` and `POST /swipe/new` act on the last *stored* Message. A click on Delete of entry 64 deleted the hidden Narration 67. Entry 64 stayed on screen, and the server then had 52 Messages, up to id 66. Every later turn is also invisible, so the player cannot go on.
- **Cause (known):** `AppState::render_story_log` (`src/adapters/driving/http/app_state.rs`) keeps `entries.into_iter().take(MAX_LOG_DISPLAY)` with `MAX_LOG_DISPLAY = 50`. `take` keeps the 50 oldest Messages.
- **Since `bc7d6cd4`:** not new. The baseline has the same `take(MAX_LOG_DISPLAY)` at `app_state.rs:109` (known from the code; not run past 50 on the baseline).
- **Shots:** `tmp/ui-review/22-story-over-50.png` (turn 20's Input is the last visible entry, with Delete and Retry on it), `tmp/ui-review/27-debug-state-step.png` (after the Delete: the server tail ends at id 66, turn 21's Input).

### F2 (P2): A bare `/impersonate` fails with 500, but it worked before the rebuild

- **Surface:** Send and the slash menu.
- **Steps:** Type `/`, pick `/impersonate` (the input then holds `/impersonate `), and click Send with no direction.
- **Expected:** The baseline writes an Input as the persona with no direction: `POST /action/check` with `command=/impersonate ` answered 200 "Thinking...", and the story gained an Impersonate Input. No spec covers a bare `/impersonate`. `actions.md` 1.9 covers only `/impersonate hello`, and `browser_slash_menu.md` 31.4/31.6 leave `/impersonate ` in the input, ready to send.
- **Actual (known):** `POST /action/check` answers 500 `The data for this request is missing: the impersonate direction.` The status display shows the clamped line "The data for this request is missing: the impersonate…" with a Details disclosure, and "That action failed." shows under the input. The story does not change. The wording also tells the player that data is missing from the system, not that a direction is needed.
- **Since `bc7d6cd4`:** new. The behaviour changed without a spec.
- **Shot:** `tmp/ui-review/08-bare-impersonate.png`.

### F3 (P3): `POST /retrigger` runs with an Impersonate Input as the last Message, but the spec and the button say it should not

- **Surface:** Retrigger.
- **Steps:** Fire a Trigger. Delete its event Narration. Send `/impersonate <direction>`. If the analysis fires the Trigger again, delete that event Narration too, so the Impersonate Input is the last Message. Then `POST /retrigger`.
- **Expected:** `retrigger.md` 14.3: "Given a Game whose last Message is an Input message (not a Narration) … Then the response is 400". The dashboard agrees: it shows no Retrigger button on the Input (`show_retrigger` needs `log_type == "narration"`).
- **Actual (known):** `POST /retrigger` answers 200 `<span class="status ready">Retriggering...</span>`. The status poll then answers `generating-event`, and the story gains a "Gabriella Introduction" event Narration after the Impersonate Input. The cause is `Tail::retrigger_allowed` (`src/domain/model/tail.rs`), which accepts `MessageKind::ImpersonateInput`. This is suspect 2. A player cannot reach this from the dashboard, so it is P3: the endpoint, the spec and the button disagree.
- **Since `bc7d6cd4`:** new (inferred). The baseline refuses: `retrigger.rs` answers "Last message must be a narration to retrigger". I did not run this on the baseline.
- **Shots:** `tmp/ui-review/17-impersonate-last-no-retrigger.png` (no Retrigger button on the Impersonate Input), `tmp/ui-review/18b-retrigger-after-impersonate.png` (the event Narration that the POST added).

### F4 (P3): `/debug/state` reports `step: "Narrating"` for an idle Game

- **Surface:** `GET /debug/state` (a debug surface, not a player surface).
- **Steps:** With no Job running, open `/debug/state`.
- **Expected:** The step is empty, or names no running step, when `generation_status` is `Idle`.
- **Actual (known):** `"generation_status":"Idle","step":"Narrating"`. I saw this on every idle read: after a successful turn, after a failed turn, and on a fresh `--world nosuch` server on port 3003. This is suspect 5.
- **Since `bc7d6cd4`:** not new. The baseline answers `"generation_status":"Idle"` with `"generation_phase":"Narrating"` (known, run on port 3002). The rebuild renamed the field to `step` and kept the fault.
- **Shot:** `tmp/ui-review/27-debug-state-step.png`.

### F5 (P3): The refusal of a World delete reads "Cannot delete world with 1 games"

- **Surface:** Worlds tab, Delete.
- **Steps:** Delete a World that has one Game.
- **Expected:** `browser_worlds.md`: the row holds the refusal message in place of the World. It does. The wording is the fault: "1 games".
- **Actual (known):** "Error: Cannot delete world with 1 games" (`src/error.rs:168`, `{game_count} games`).
- **Since `bc7d6cd4`:** not new (same line in the baseline's `src/error.rs`).
- **Shot:** `tmp/ui-review/24-world-delete-refused.png`.

## Suspects

1. **Bare `/impersonate` answers 500: confirmed.** See F2. The baseline did not fail.
2. **Retrigger button on a Narration only, while `Tail::retrigger_allowed` also allows an Impersonate Input: confirmed.** See F3. The button follows spec 14.3, and the endpoint does not.
3. **More than 50 Messages hides the newest: confirmed, and worse than suspected.** See F1. The last-entry controls act on a hidden Message.
4. **The stop and the status poll with no active Game: skipped for "no active Game". That state is not reachable.** Boot always points the app at a Game. An unknown `--world` falls back to the first World: `--world nosuch` started on Test Realm (known, port 3003). `GameCatalogue::delete_game` refuses "Cannot delete the active game", and a World delete is refused while it has Games (F5). With an active Game, the stop works (known): a `POST /status/reset-generating` during `narrating` answered `200 reset`, the poll answered `idle`, Send came back, the Input stayed, and the late model answer added nothing (spec `dashboard.md` 39.4/39.5). No shot shows the stop. The evidence is the DOM and endpoint read in the run log.
5. **`/debug/state` shows `step: Narrating` for an idle or failed Game: confirmed.** See F4. The baseline had the same fault.
6. **A Retrigger of a Trigger whose direction holds `{{user}}`: skipped.** Triggers live in `data/characters/<world>/*.json`. No Redmist or Test Realm Trigger direction holds `{{user}}`, and `rg "{{user}}" data/` finds it only in openings, presets and character text. The Worlds form (`key`, `name`, `description`, `global_rules`, `default_room_image`, posture, `options_always_on`, `map_json`, `scenarios_json`) cannot edit Characters or Triggers. So a test needs a change to `data/`, which the ticket forbids.

## Surfaces that work (known, with shot)

- Opening Narration, Send, the Narration with a room header after a move, the sidebar location image and headshots: `01-initial.png`, `05-turn2-courtyard.png`, `06-trigger-fired.png` (the event header "Gabriella Introduction", and Lisette and Gabriella in the sidebar).
- Status steps "Thinking..." → "Generating narration..." → "Quantifying scene..." → "Generating event..." → "Ready". The poll answers `narrating`, `quantifying`, `generating-event`, `options` and `idle`: `12-retry-in-flight.png`.
- Failure display: the Quantifier on Ollama raises the banner "Quantifier failed — using fallback NPC IDs (engine-wide role health)", with Details per role (Narrator, Quantifier, Options, Trigger). The Narrator on Ollama shows "The language model could not be reached." with Details, the banner "Narrator failed — the turn was left unnarrated", and keeps the Input: `04-turn1-done.png`, `19-narrator-failure.png`. An empty Send then narrates the kept Input (spec 1.5).
- Slash menu (three suggestions), `/impersonate <direction>`, `/guide`, `/options` and the options dock. Using an option submits it, and a Narration with the toggle off clears the set: `07-slash-menu.png`, `09-impersonate.png`, `10-guide.png`, `11-options-dock.png`.
- Retry (new Swipe) on a Narration, Swipe switch ("Restored swipe 1 / 2"), Retry on a typed Input (user-regen, spec 22.2), edit of an Input (kept after a reload), and Delete-last below 50 Messages (the state goes back to the earlier Swipe): `13-swipe-switch.png`.
- Retrigger from the button on a main Narration after its event Narration was deleted: `15-retrigger-button.png`, `16-retriggered.png`.
- Settings: role selects, role health cells, connection Test OK and failed: `02-settings.png`, `26-settings-tests.png`. Prompt Presets: duplicate, edit (rename), Set Active, the refusal "Preset is a mode default; change the default before deleting" (spec 21.37), and delete: `20-prompt-presets.png`. Games: New Game (Test Realm), Switch: `21-games.png`, `23-new-game.png`. Worlds: list, Create form, delete refusal: `24-world-delete-refused.png`, `25-worlds-create-form.png`. LLM Messages: list and expand: `14-llm-messages.png`.

## Notes that are not findings

- **The dashboard has no stop control** in either version. Only `POST /status/reset-generating` stops a Job, and no file under `assets/` names it (known, `rg`). No spec promises a button.
- **The status display can read "Ready" up to about 2 s before the story log shows the new text.** The status polls every 5 s and the story log every 2 s, independently (`assets/index.html`). The baseline has the same design. In one run, a user-regen Swipe appeared after "Ready".
- **During a user-regen Retry, the status reads "Generating narration..."**, but the Job writes a new Input. The step name `narrating` covers both.
- **The narrator mixes the second and third person.** The prompt carries "third-person limited perspective" and "past tense" (known, from `llm_messages.raw_request_json`). The model drifts, and the engine does not cause it.
- **Headless Chrome returned a stale frame** on three shots (`03-turn1.png`, `18-…png`, and the first `27-…` capture). I took each shot again after a repaint. These shots are not evidence of a fault.
- **Not driven:** the always-on options toggle (spec 24.6), the Text Check tab, Games rename, mode and posture, persona edits, and Add Connection.
