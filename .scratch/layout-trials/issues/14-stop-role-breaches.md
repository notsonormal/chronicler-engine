# Stop the role breaches with stop sequences

Type: task (AFK)
Status: open
Blocked by: 12, 13

## Question

Does a `stop` list of the history labels remove the role breaches from the Narrations and the Inputs, with no harm to the text?

## Context

- Replaces [Try the history as chat messages](07-history-as-chat-messages.md) (owner, 2026-11-02). That candidate needs a change of the model port; this test needs none.
- The fault: an Input writes `Narrator:` and `Julian:` turns for the other side, and a Narration writes Julian. P has 10 such entries (one of 847 words), Q has 2 (one of 1344 words). See [the inconsistencies of two trial stories](../assets/story-inconsistencies.md).
- Known: the history is one text block. Each entry has a label from `src/application/prompting/assembler.rs` (around line 354): `Narrator` for a Narration, the Persona name for an Input, `System` for a System entry.
- Known: a preset can add any request key that is not an engine key (`GenerationParameters::custom`, `src/domain/model/generation_parameters.rs`). `stop` is not in `ENGINE_KEYS` or `TYPED_KEYS`, so a preset can send it today.
- Not known: whether OpenCode Go and `deepseek-v4.1-flash` honour `stop`. Check this first with one call on the probe server, before a trial.
- Not known: whether a custom key renders `{{user}}`. The trial can write `Julian` as text, because the test setup fixes the Persona. A build cannot, because the Persona name changes. So a kept result makes the engine add the stops itself, since the engine owns the labels.
- Trial choices, each stated in the answer: the stop strings (for example `"\nNarrator:"`, `"\nJulian:"`, `"\nSystem:"`), the presets that carry them (the narrator preset for Narration, the impersonate preset), and copies of the default presets in the trial setup.
- Limits: a stop does not catch a breach with no label, for example a Narration that writes Julian's words in prose. A label at the very start of an output does not match a stop that begins with a newline. Count both kinds.
- Measure: one 100-Cycle trial (about 3 M tokens) on the fixed data of [Remove the data conflicts that distort a trial](12-remove-data-conflicts.md). Count the outputs that hold a label line, the outputs that a stop cut (`finish_reason`), and the breaches with no label. An ad hoc judge reads the cut outputs: does the text end in a usable place? Because it plays the fixed data, this trial is also the new reference for later checks (graduated from the map's "Not yet specified").
- Keep when the labelled breaches go to about zero and the cut outputs read well. A kept result gets a build ticket (map Notes, "Decide, then build").
- How: `scripts/trial.py run` with a copy of `scripts/trial_setups/redmist_julian.json` under `tmp/lt14/`. Check after the trial that the pack still shows the history and the call kinds (map Notes, "Failure modes").

Done when the owner keeps or drops the result, and the answer gives the counts and the stop strings.
