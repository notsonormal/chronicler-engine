# Shape the test setup and the trial

Type: prototype
Status: open
Blocked by: 01

## Question

What does a test setup fix, how does a trial start, how many times does it repeat, what does it save, and what does the reading pack look like?

## Context

- Autoplay plays N cycles on one Game (`POST /autoplay` with a budget). The `llm_messages` table already stores every prompt, request and answer.
- A first rough reading pack exists: [story transcript](../assets/autoplay-story-transcript.md) and [the stored Impersonate prompt](../assets/impersonate-prompt-157.txt), made by hand from the probe DB.
- Points to settle: what is fixed (World, Persona, opening, presets, connections, cycles); how a trial starts (a script against a server, a CLI subcommand, or a test harness); repeats, because the same setup gives different output each time; how failed calls show (the Quantifier failed every cycle with `content_filter`, which left the game state stale with no visible sign); how two trials sit side by side. Do not define criteria for good output: that is not yet specified (see the map).
- A trial calls a paid model many times. Settle a cost limit for one trial.
- Reuse what exists: the LLM messages panel (`/fragment/llm-messages`) and the probe server of `/chronicler-ui-investigator` (`target/ui-investigator/`, port 3001), which the first Autoplay run used.
- Use `/prototype`: a script that exports a trial from the database into a reading pack is a cheap first artifact.
