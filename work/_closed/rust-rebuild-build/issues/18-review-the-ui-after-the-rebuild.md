# Review the dashboard after the Rust rebuild

Type: research
Status: resolved
Blocked by: —

## Question

What is the state of the dashboard after the Rust rebuild? Which bugs does a player hit, which features that a spec in `docs/specs/` promises are missing or broken, and which behaviour changed since `bc7d6cd4` (the last `main` commit before P0) without a spec saying so?

- **Method:** `/chronicler-ui-investigator` (`.agents/skills/chronicler-ui-investigator/SKILL.md`). A probe server of the code at `2885df2c` (`wf/rebuild-core`; the commits after it touch only `.scratch/`), with its own target dir and port: `python build.py run --target-dir target/ui-investigator -- --world redmist_estate --port 3001`. Use the real connections the settings hold. A role on an Ollama connection fails on every turn; that failure is expected.
- **Coverage:** every dashboard surface the specs name: the story log (edit, delete), Send and the slash menu (`/impersonate`, `/guide`, `/options`), new Swipe and Swipe switch, Retrigger, the options dock, the status display and the stop, the failure display, the visual sidebar and headshots, and the Settings, Prompt presets, Games, Worlds and LLM messages tabs. Play a story of several Narrations, with at least one fired Trigger.
- **Suspects from reading the code** (confirm or reject each in the running UI; do not stop at these):
  1. A bare `/impersonate` with no direction answers `500` "The data for this request is missing: the impersonate direction."
  2. The Retrigger button shows on a Narration only, but `Tail::retrigger_allowed` also allows an Impersonate Input.
  3. A story of more than 50 Messages: the story log shows the 50 oldest, so the newest Messages are hidden.
  4. The stop (`POST /status/reset-generating`) and the status poll when no Game is active.
  5. `/debug/state` shows `step: Narrating` for a Game that is idle or failed.
  6. A Retrigger of a Trigger whose direction holds `{{user}}` sends the placeholder raw (only if a world can be given such a Trigger without a code change; else skip with the reason).
- **Severity:** P1 a player loses work or cannot go on; P2 a feature is wrong or missing; P3 a visual or wording fault.
- **Read-only on the code.** No change to `src/`, `tests/`, `docs/` or `data/`. Do not touch the live dashboard. Stop the probe server and Chrome at the end (skill step 4).

## Done when

- Each finding is in [../assets/ui-review/findings.md](../assets/ui-review/findings.md): severity, surface, steps to reproduce, expected (with the spec scenario, if one exists), actual, and the shot path. Each suspect above has a confirmed, rejected or skipped line with its evidence.
- Shots are under `tmp/ui-review/`. Each finding was compared against a shot by the agent that reports it.
- The Answer gives the count by severity, the three worst findings in one line each, and the asset link.

## Answer

The dashboard works on every surface I drove, except at the edges below. The count is 1 P1, 1 P2 and 3 P3. Of the five findings, two are new in the rebuild (F2, F3) and three are older than `bc7d6cd4` (F1, F4, F5). I checked them against a baseline server of `bc7d6cd4`.

The three worst findings:

1. **P1, older than the rebuild:** past 50 Messages, the story log shows the 50 oldest Messages (`take(MAX_LOG_DISPLAY)`). New turns are invisible, and Delete/Retry on the last visible entry act on a hidden Message. A click deleted hidden Narration 67.
2. **P2, new:** a bare `/impersonate` answers 500 "The data for this request is missing: the impersonate direction." The baseline wrote an Input with no direction.
3. **P3, new:** `POST /retrigger` runs with an Impersonate Input as the last Message. Spec 14.3 and the hidden button say it must be refused (`Tail::retrigger_allowed`).

Suspects: 1, 2, 3 and 5 are confirmed. For 4, the "no active Game" state cannot be reached: boot always picks a Game, and the active Game cannot be deleted. The stop with an active Game works. Suspect 6 is skipped: no Trigger holds `{{user}}`, and the Worlds form cannot edit Triggers without a change to `data/`.

Asset: [../assets/ui-review/findings.md](../assets/ui-review/findings.md). The probe servers (3001, 3002, 3003) and Chrome are stopped. `target/ui-baseline/` (the baseline build) remains on disk.
