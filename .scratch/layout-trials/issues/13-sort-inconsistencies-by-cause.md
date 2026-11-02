# Sort the inconsistencies by their cause

Type: task (AFK)
Status: open
Blocked by: —

## Question

For each inconsistency that the judge pilot found, is it inherent to the model, or does our structure (the prompt, the engine) or our data cause it?

## Context

- Origin (owner, 2026-11-02): `deepseek-v4.1-flash` is taken as good enough for roleplay. An inconsistency inherent to the model is accepted, not engineered around. But first separate those from the inconsistencies that our structure or our data cause, because only these are ours to fix.
- Material: [the inconsistencies of two trial stories](../assets/story-inconsistencies.md), the verdicts `tmp/lt05/judge-P.md` and `tmp/lt05/judge-Q.md`, and the packs with their stored calls: P is `tmp/lt01/redmist_julian/trial-1`, Q is `tmp/lt03/redmist_julian/trial-1`. Each entry of `pack.md` links the call that wrote it (`calls/NNNNN-<kind>.md`, with the full system and user prompt).
- Already sorted (no work here): the history cut (group 1), the missing story clock (group 5) and who knows what (group 6) are structure, for [Long-term memory and state](../../long-term-memory-and-state/idea.md). The father/grandfather conflict, the era and the short entries are data, for [Remove the data conflicts that distort a trial](12-remove-data-conflicts.md).
- The method for each case: open the stored prompt of the call that wrote the later entry. Check whether the fact was in the prompt and stated clearly, and whether any part of the prompt contradicted it or pushed the other way (a wrong Room or wrong Characters present in `<GameState>` or `<CharactersPresent>`, a short entry in `<KnownCharacters>`, the World text, an earlier breach in the history). A case with a clear, correct prompt is inherent. A case with a misleading part is structure or data, and the answer names the part.
- Cases to sort:
  - Group 2, the contradictions of a fact that was still in the history: P #376 → #385, #331 → #374, #353 → #359, #389 → #397; Q #567 → #581.
  - The role breaches (P #374, #410, #420, #438, #504, #506 and #375, #411, #439, #505; Q #594, #685). Does each breach follow a history that already holds a breach, or the label format alone? The labels come from `src/application/prompting/assembler.rs` (around line 354: `Narrator`, the Persona name, `System`).
  - The wrong Room and the wrong Characters present in the State lines. The map of `redmist_estate` has 10 Rooms, all on the estate. Is it data (no place off the estate), the Quantifier prompt, or inherent? Did a wrong value in the next Narration prompt cause a fault in the story (for example Jezebel present after she left, P #377)?
  - The drift from the cards (Louise, Jezebel, Gabriella, Julian's voice). Did the full card reach the prompt of those entries, or only the short entry?
  - The growth of the Impersonate Inputs (P: mean 115 words in the first 40 entries and 224 in the last 40; Q: 111 and 124). The "Response Length" text was removed from the Impersonate prompt in Prompt flexibility. Drift, or the missing length text? (Graduated from the map's "Not yet specified".)
- An agent may count with a script, but it must quote the prompt part that it names as a cause.

Done when the asset gives each case its cause (model, structure or data) with the evidence, each data cause is added to ticket 12, and each structure cause outside Long-term memory and state is a new ticket or a line in the map's "Not yet specified".
