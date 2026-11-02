# The judge prompt of the pilot

Used in [Decide how to read an output trial](../issues/05-read-an-output-trial.md) on 2026-11-02, with the `reviewer` role (`claude-opus-5-5`), one judge for each story. An example, not a standard: each check writes its own prompt for its own question (owner). `{L}` is the neutral name of the story.

Lesson from the pilot: the story copy must keep the "History cut" line of the pack, or the judge cannot place the cut.

```text
You are a judge of one interactive-fiction story. Do NOT read tmp/lt05/key.txt and do not read any other story-*.md file. Do not change any file except your own output file.

The story: tmp/lt05/story-{L}.md (about 200 entries, about 55,000 tokens; read ALL of it, page through with offset/limit). Entries alternate: "Narration" (written by the narrator model) and "Impersonate Input" (written by a model that plays the player character, Julian). Each entry has a "> State:" line with the Room and the Characters present that the engine tracked. A "history cut" means the oldest entries left the model's context; entries after it may forget early facts.

Reference material (read before the story):
- The World: data/worlds/redmist_estate/world.json (description, global_rules, scenarios).
- The Characters: data/characters/redmist_estate/*.json (carla, gabriella, jezebel, lisette, louise).
- The Persona (the player character): data/personas/julian.json.
- Mode: novel, third person, past tense. The Impersonate instructions ask for one beat only, in Julian's voice, with no narration of other characters' actions or outcomes.

Judge the story on these five dimensions. For each, give a score from 1 to 5 (5 = no real fault; 3 = noticeable faults that a reader would mind; 1 = the fault breaks the story) and 3 to 6 findings. Each finding has the entry number (#NNN), a short quote, and one sentence on why it matters. Also give positive findings, not only faults.

1. Consistency: facts, objects, places, names, time of day and relationships stay true to earlier entries and to the World. Contradictions, forgotten facts (especially after the history cut), and characters present who should not be.
2. Follows the story: each entry answers the entry before it; threads that the story opens are followed up or closed; the story moves forward and does not stall or loop.
3. Understands the characters: each Character acts and speaks like her card (personality, role, speech); the Characters sound different from each other; nobody knows what she could not know.
4. The Persona Inputs: Julian stays in character as his card describes; one beat per Input; no scripting of other Characters' actions or outcomes; he acts like a player who pushes the story.
5. Repetition and prose: repeated phrases, tics, beats or scene shapes across the story; whether the prose gets flatter or more formulaic in the late Cycles.

Then give:
- An overall score from 1 to 10 for "how good is this as a story that a player would want to keep playing", with two sentences of reasons.
- The five worst faults of the whole story, ranked.
- Whether the quality changes over the story: compare the first 40 entries, the middle, and the last 40 entries in two or three sentences.
- How confident you are in each score (high / medium / low), and what in the material made a dimension hard to judge.

Write your full verdict to tmp/lt05/judge-{L}.md, in Markdown, with a score table at the top. Your final message: the score table and the path.
```
