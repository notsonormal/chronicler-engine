---
diataxis: reference
title: Options
---

# Options

## Overview

Options are pre-written player inputs the engine offers while a game is idle. The offered set is game state, persisted with the turn's snapshot, separate from the message history.

## Where Options Come From

The options agent generates the set. It is a named agent (`options`), and it runs in its own pipeline phase, `OptionsGeneration`. The pipeline dispatches it at exactly two gated sites:

1. **Turn end** — after a narration turn.
2. **On demand** — when the player enters the `/options` slash command.

The always-on toggle is a game setting, inherited from the World at game creation.

## Mode Neutrality

Options are mode-agnostic: the same agent and the same offered set serve both narrator modes.
