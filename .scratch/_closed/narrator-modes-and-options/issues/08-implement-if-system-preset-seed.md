# Task: IF system preset seed (author `data/prompt_presets/system/if_default.json` from the approved draft)

Type: task
Status: resolved
Blocked by: 13

## Question

Create the IF system preset seed file from the approved draft produced by ticket 02. This is implementation per the map's Notes override (decide, then implement, then build-green).

### Scope

1. **Author the seed file.** Copy the approved draft at `.scratch/narrator-modes-and-options/research/02-if-preset-draft.json` to `data/prompt_presets/system/if_default.json`. The seeding loader (`src/bootstrap/run.rs` `process_preset_file`) keys off the JSON `id` field (`system_if_default`), not the filename; the filename `if_default.json` mirrors the novel `default.json` for readability.

2. **Verify the seed loads.** The loader seeds every `.json` under `data/prompt_presets/system/` with `is_default: true` (the JSON `is_default` field is ignored on seed — `src/bootstrap/run.rs:284`). Confirm the preset appears in the Prompt Presets panel as a System preset with id `system_if_default`, protected from edit/delete (the `is_default` semantics). It is not yet selectable as any mode's default — that wiring lands in ticket 05's per-mode registry.

3. **Build-green.** `python build.py` (fast suite). Novel mode must stay green. The IF preset is inert until ticket 05 (registry) and ticket 06 (pipeline reads game preset) land; this ticket only ships the seed file.

### Notes for the session

- The preset *content* is decided (ticket 02). This ticket is mechanical: write the file, confirm it loads, stay green. Do not re-edit the content; if the draft needs changes, reopen ticket 02.
- The role-opener phrase "parser-style interactive fiction narrator" is a working placeholder accepted by the author pending a final genre label; leave it as-is unless the author directs otherwise.
- Read before implementing: `src/bootstrap/run.rs` (`process_preset_file`), `data/prompt_presets/system/default.json` (the sibling to mirror in shape).
- No blockers: the file is self-contained. Tickets 05 (registry) and 06 (pipeline) reference the id `system_if_default` as a string contract; they do not require this file to exist to compile.
- Skills: `/domain-modeling` (only if the id or field names need sharpening — they should not).

## Answer

Superseded — closed without being worked.

- Ticket 13's grilling redesigned the preset-selection mechanic around this seed: `if_default.json` now ships with `"allowed_modes": ["interactive_fiction"]` as part of **ticket 14** (implementation), which absorbed this ticket's file-creation half.
- The seed content itself remains ticket 02's approved draft (`.scratch/narrator-modes-and-options/research/02-if-preset-draft.json`), unchanged.
- The loader change and seeding verification from this ticket's scope are ticket 14 scope items 3–4.
