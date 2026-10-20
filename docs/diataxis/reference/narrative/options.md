---
diataxis: reference
title: Options
---

# Options

## Overview

Options are pre-written player inputs the engine offers while a game is idle. The offered set is held in the Game's dock record, apart from the story.

## Where Options Come From

The options agent generates the set. It is a named agent (`options`), and it runs at the Options hook. A Job runs the hook at exactly two gated sites:

1. **After a Narration** — at the end of a Job that saved a Narration.
2. **On demand** — when the player enters the `/options` slash command.

The always-on toggle is a game setting, inherited from the World at game creation.

## Mode Neutrality

Options are mode-agnostic: the same agent and the same offered set serve both narrator modes.
