# Try `example_dialogue` in the prompt

Type: task (HITL)
Status: open
Blocked by: 05

## Question

Does the Narration get better when the prompt sends the `example_dialogue` of each Character?

## Context

- Target part: the Narration, and in it the voice of the Characters.
- Known: `CharacterSheet::example_dialogue` (`src/domain/model/character.rs`, flattened into `CharacterCard`) is stored, but no prompt reads it. In this World it holds the SillyTavern `first_mes` of each card: the greeting, not the ST example dialogue (`mes_example`, which our conversion dropped). See [Compare our card and World format with ST and ME](../../prompt-flexibility/issues/04-card-and-world-format.md).
- Known (2026-11-01): each of the five cards in `data/characters/redmist_estate/` holds a greeting scene of 149 to 221 words, written as narration with `{{user}}` placeholders. So the placeholders must render, as in the rest of the card text. Five greetings add about 860 words to a call that sends all of them.
- Known: the opening of the World (`data/worlds/redmist_estate/world.json`) is a short form of Carla's greeting. Both start "Standing tall and commanding, Carla …". Inferred: send the greeting as an example, and the model may repeat the opening. The reading looks for this.
- So the trial tests "send the ST greeting as an example of the voice". The place in the prompt is a trial choice (inside the full card of `<CharactersPresent>`, or as a block of its own). The answer states the choice and its reason.
- Source: candidate 3 of [Decide which parts of a Narrator-role call an author can change](../../prompt-flexibility/issues/06-the-layout-model.md).
- How: a linked worktree from `layout-trials` with a detached HEAD, as in [Try the changing blocks after the history](03-changing-blocks-after-history.md). Run the number of trials and Cycles that [Decide how to read an output trial](05-read-an-output-trial.md) sets. Then read the comparison with the owner, with that method.

Done when the owner keeps or drops the candidate, the reading notes extend the criteria asset, and the worktree is removed. A kept candidate gets a new task ticket (map Notes).
