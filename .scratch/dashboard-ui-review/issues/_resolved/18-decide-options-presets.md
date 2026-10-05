# Decide whether Options presets appear in Prompt Presets

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

`data/prompt_presets/options/` exists, but the Prompt Presets tab shows only System, Quantifier, and Impersonate. Should Options presets be editable in the tab?

## Context

- The review report lists this as an open question.
- Check first: how does the Options agent pick its preset today, and is there a per-game selector for it (the Games tab shows System, Quantifier, Impersonate)?

## Done when

- The decision is in the ticket answer. Implementation tickets are created, or the ticket is ruled out of scope with a reason.

## Answer

**Edit and select them — option C.** Options presets already drive the Options
agent, so making them invisible in the UI is the defect.

- **Edit in the Prompt Presets tab.** Add an "Options Prompts" section beside
  System, Quantifier, and Impersonate: list, view, edit, activate. Activation
  sets the settings-level default (`active_options_prompt_preset_id`), because
  Options is mode-agnostic and has no `ModePresetBundle` slot.
- **Pick one per game.** Add a fourth select to the Games tab preset picker. The
  game already stores an active Options preset id and the resolution already
  prefers it over the settings default, so this is a small addition.
- The second seed (`data/prompt_presets/options/roadway_domains.json`) becomes
  reachable; today nothing calls `list_presets(PresetType::Options)`.

Marinara-Engine was checked and does not inform this: it has no preset-type
categories at all, and its choices are "Preset Variables" inside a preset. So it
neither argues for nor against exposing Options, and it does not change this
decision.

### Graduated

[Expose Options presets in the UI](60-options-presets-ui.md).
