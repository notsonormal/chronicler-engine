# Charting facts of the Work records map

Facts that the charting session found on 2026-11-04, at `e6715c60`. Each one is known (checked with a command) unless it says otherwise. The tree moves, so check a fact again before a decision rests on it.

## What `.scratch/` holds

- 15 live folders and 19 closed ones in `_closed/`. Each live folder has a `map.md`, except `story-scroll-back` (one `idea.md`).
- Two maps are buckets of one-off items. `autoplay-follow-ups/map.md` says: "Bucket choice. No existing `.scratch/` effort fits". `retro-issue-bucket` holds retro findings.
- `docs/plans/` holds 4 plans. `.pi/plan-mode.json` writes plans there (`"planFolder": "docs/plans"`). Step 2 of `chronicler-after-plan-workflow` archives a plan to `old-docs/archived-plans/`.
- `docs/agents/triage-labels.md` gives five `Status:` values for `.scratch/` files (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). Wayfinder tickets use `open`, `claimed` and `resolved` (`docs/agents/issue-tracker.md`).

## Rules copied between maps

These rules are in the Notes of `prompt-flexibility`, `layout-trials` and `long-term-memory-and-state`: "Standing rules (from the earlier maps)", plans "not through Plannotator", "Any new API takes a `GameId`", and the migration rule (`ALTER TABLE … ADD COLUMN`). "How a trial runs" is in `layout-trials` and `long-term-memory-and-state`. The Notes section of `long-term-memory-and-state/map.md` is 11,353 bytes.

## Links

- The three long-play maps hold 600 relative links. If the three move together, 588 stay valid. 8 links go to other efforts, 4 go to repo files outside `.scratch/`, and 12 links in other efforts point into the three maps.
- The maps have 513 `../assets/` links and 221 `../issues/` links, so a map, its tickets and its assets must stay together.
- Nothing checks the links in `.scratch/`. A rough scan found more than 100 relative links in live maps that do not resolve. Some are placeholders (`link`), some point into the git-ignored `tmp/`, and some use `%20` (approximate).

## The name `.scratch`

- "Scratch" means two other things in this repo. `AGENTS.md` says "`tmp/` is the git-ignored scratch folder". In `.pi/plan-mode.json`, `scratchFolders` lists the folders that plan mode may write to (`tmp`, `docs/plans`, `.scratch`).
- In Pocock's skills (`mattpocock/skills@4ae2317`), `.scratch/` is the local-markdown tracker. `setup-matt-pocock-skills` offers it as one option, "good for solo projects or repos without a remote". The upstream `code-review` and `to-tickets` skills name `.scratch/` directly.
- 11 live files name `.scratch`: `AGENTS.md`, `docs/AGENTS.md`, `docs/agents/issue-tracker.md`, `docs/agents/triage-labels.md`, `docs/diataxis/reference/coding_standards/guardrails.md` (generated, so change its source), six skills, and `.pi/plan-mode.json` (plus a test value in `.pi/extensions/pi-plan-mode/test/`). `.gitignore` line 215 names `.scratch/**/assets/local-only/`. Archived plans in `old-docs/`, the mrn-wiki pages and old sessions also name it. They are history, so they keep the old path.
- `rg` skips hidden folders by default. A search from the repo root for "Decisions so far" finds 0 map files, and 41 with `--hidden`. A visible folder puts the maps into every default code search. For example, `max_history_tokens` appears 102 times in `.scratch/` and 36 times in `src/`.

## The words

Counts in `CONTEXT.md` (the game glossary) and in `.scratch/` prose:

| Word | Finding |
|---|---|
| project | 540 uses in `.scratch/` prose |
| goal | 3,062 uses in `.scratch/` prose; the word of the chief-of-staff skill |
| campaign | `CONTEXT.md` lists it under _Avoid_ for **Scenario** |
| mission | Used by Factory and OpenRig for their units of agent work (mrn-wiki) |
| atlas | A sprite atlas is a graphics term; `character-art-from-code` uses it |
| expedition | No use in the repo |
| errand | Only in the story text of trial evidence |
| task, job, brief | A wayfinder ticket type; a `CONTEXT.md` term (`Job`); a `loop-me` term |

## Concurrency

Other sessions write to `.scratch/` at the same time. The map `weekly-retro-2026-11-04` appeared while the charting session ran, and `HEAD` moved from `20d6d23c` to `e6715c60`.
