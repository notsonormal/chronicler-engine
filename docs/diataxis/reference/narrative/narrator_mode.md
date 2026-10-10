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

The mode is chosen on the World and inherited by every Game created from it. A Game can switch mode later.

## The Agency Rule

The two modes differ in one core rule, the Agency Rule, carried inside the mode's system preset.

## Posture

Posture is the per-game narration stance: narrator mode, narrative perspective, and narrative tense. The triple lives on the World template and is copied into the Game at creation. The player can edit it per game.

## Preset Selection

Two mechanisms decide which prompt presets a mode uses:

| Mechanism | Question it answers | Where it lives |
|---|---|---|
| Mode preset registry | Which preset is the *default* for a mode — new games, resets, mode switches, and the no-game fallback read it | Settings (one JSON list of per-mode bundles) |
| Allowed modes | Which modes may *select* a preset at all | Each preset carries its own `allowed_modes` list |

Allowed modes gate selection surfaces only — the preset picker and activation. The narrate path uses the preset ids a Game already stores as they are.

## Steering Neutrality

Steering is mode-agnostic.
