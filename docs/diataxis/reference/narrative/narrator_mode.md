---
diataxis: reference
title: Narrator Mode
---

# Narrator Mode

## Overview

A Game plays in one of two narrator modes:

| Mode | Who writes the player character | Player input shape |
|---|---|---|
| Novel | The player writes the protagonist in prose; the narrator world-builds around it | Rich prose |
| Interactive Fiction | The player directs with terse commands; the narrator elaborates them | Terse commands ("take vase", "go north") |

The mode is chosen on the World and inherited by every Game created from it. A Game can switch mode later; the switch retargets the system preset and re-renders the posture.

## The Agency Rule

The two modes differ in one core rule, the Agency Rule, carried inside the mode's system preset:

- **Novel**: the narrator never writes, assumes, or infers the player's actions, thoughts, or feelings. The player's speech lines are reported in indirect speech. The player character stays the player's voice.
- **Interactive Fiction**: the rule inverts. The narrator voices the protagonist's commanded actions and dialogue directly and adjudicates the world's response. The player character is not a cipher.

One hard line holds in both modes: the narrator elaborates only what the command directed. It does not invent uncommanded actions. Both mode presets keep the shared safeguards — input validation, state tracking, world dynamics, accuracy over creativity — and only the posture blocks differ.

## Posture

Posture is the per-game narration stance: narrator mode, narrative perspective, and narrative tense. The triple lives on the World template and is copied into the Game at creation. The player can edit it per game through the posture controls, which auto-save.

The perspective and tense values are rendered into the user prompt's post-history section on every generation — they are prompt macros, not system-prompt text. A mode switch does not move them by force: it re-renders the posture controls with the mode's suggested perspective, but a perspective the player set deliberately survives the switch.

## Preset Selection

Two mechanisms decide which prompt presets a mode uses:

| Mechanism | Question it answers | Where it lives |
|---|---|---|
| Mode preset registry | Which preset is the *default* for a mode — new games, resets, mode switches, and the no-game fallback read it | Settings (one JSON list of per-mode bundles) |
| Allowed modes | Which modes may *select* a preset at all | Each preset carries its own `allowed_modes` list |

The registry ships with one bundle per mode. The Novel bundle points at the seeded `system_default` preset; the Interactive Fiction bundle points at `system_if_default`. Quantifier and impersonate presets are shared by both bundles. Activation is mode-targeted: a preset is set as the default for one chosen mode, refused when that mode is not in the preset's allowed modes.

Allowed modes gate selection surfaces only — the preset picker, the mode-switch retarget, and activation. The narrate path never re-validates the ids a Game already stores; curation is not a hard gate. A preset without an allowed-modes list is selectable in every mode.

## Steering Neutrality

Steering is mode-agnostic. `/guide` steers content and `/impersonate` voices the player in either mode; both stay transient and never become history entries. Impersonation stays available in Interactive Fiction games — the narrator hands the pen back for one line, then the posture resumes.

## Document References

- [`./narration_system.md`](./narration_system.md) — the generation pipeline the mode feeds: model configuration, Game Master role, per-action flow.
- [`./prompt_system.md`](./prompt_system.md) — prompt assembly layers; where the posture macros and preset text land.
- [`./ai_steering.md`](./ai_steering.md) — the transient steering commands and how swipes re-apply them.
- [`../game_flow.md`](../game_flow.md) — the action pipeline phases, retry flow, and generation gate.
- [`../../../specs/narrator_mode.md`](../../../specs/narrator_mode.md) — the behaviour contract (scenarios 23.1–23.5): posture inheritance, mode switching, steering availability.
