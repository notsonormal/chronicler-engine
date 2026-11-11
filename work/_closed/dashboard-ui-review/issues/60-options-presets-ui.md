# Expose Options presets in the UI

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Implements [Decide whether Options presets appear in Prompt Presets](18-decide-options-presets.md).

- Add an "Options Prompts" section to the Prompt Presets tab, beside System,
  Quantifier, and Impersonate: list, view, edit, activate. Activation sets the
  settings-level default (`active_options_prompt_preset_id`), because Options is
  mode-agnostic and has no bundle slot.
- Add a fourth select to the Games tab preset picker so a game chooses its own
  Options preset. The game already stores this id and the resolution already
  prefers it over the settings default.
- Extend the Prompt Presets handler and the Games picker form/handler to include
  Options. Today nothing calls `list_presets(PresetType::Options)`.
- The second seed (`data/prompt_presets/options/roadway_domains.json`) becomes
  reachable.

## Checks before starting

- `git status` for overlapping edits. This ticket shares the Prompt Presets and
  Games templates with
  [Make the dashboard panels consistent](58-panel-consistency.md).

## Done when

- `python build.py` is green.
- Spec scenarios cover the new section and the per-game selector.
- The user has reviewed the diff.

## Answer

Absorbed into [Panel consistency and Options presets](67-panel-consistency-and-options-presets.md).
The Options-preset section and the per-game selector moved there, in the same
sweep over the Prompt Presets and Games templates. Closed as absorbed, not fixed.
