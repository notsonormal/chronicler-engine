# Score the five other tabs

Type: task (AFK)
Status: open
Blocked by: none

## Question

What does `/impeccable critique` score for each of the five other tabs: Settings, Prompt Presets, Worlds, Games and LLM Messages?

## Context

- This is the start of the outside check (map Notes). The Game tab scored 23/40: [the Game tab critique](../assets/game-tab-critique/report.md).
- Run each critique as the Game tab critique ran: two isolated agents, A (design review, `reviewer`) and B (detector, `scout`), each in its own Chrome tab on the probe server, then one combined report. Read `.pi/skills/impeccable/reference/critique.md`. Give each tab its own URL query (for example `?critique=settings-a`) so that `detect --cdp 9222 --target` picks the right tab, and switch the tab inside the page (`.tab[data-tab="<name>"]`).
- An earlier detector-only scan found: Settings 0, Worlds 0, Games 1 advisory (a false positive on the em-dash in Game names), Prompt Presets 5 `side-tab` (`assets/styles.css`, the coloured left border on the preset cards), LLM Messages 50 `tiny-text` (`.llm-message-model` at 11px). Screenshots: `../assets/baseline-shots/`.
- Read-only: never send an Action, never Save, never POST.
- Run the Game tab critique once more with fresh agents and the same method (map Notes, "A fixed method for the check"). The difference from the first score (23/40) shows how much a score moves by chance. Record it in the map Notes.
- Mode: the five tabs are Operate surfaces for the tuner role of `PRODUCT.md`.

Done when each tab has a saved report in `.impeccable/critique/` (tracked in git, so the map links to it and keeps no copy), and the answer lists for each tab its score, its P0 and P1 issues in one line each, and the issues that it shares with the Game tab, and the second Game tab score with its difference from 23/40.
