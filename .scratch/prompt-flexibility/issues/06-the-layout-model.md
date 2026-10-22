# Decide which parts of a Narrator-role call an author can change

Type: grilling
Status: open
Blocked by: 01, 02, 04

## Question

For each fixed part of a Narrator-role call (Narration, Event Narration, the regenerate of a user Input, Impersonate) that [List what reaches the model in each call](01-what-reaches-the-model.md) found: does it become changeable by an author, or does it stay an engine rule with a written reason? Which layout model carries the changeable parts?

## Context

- Candidate models, to test against the findings: (A) an ordered template of blocks, where each block is author text or a marker for an engine layer (as ST's Prompt Manager and ME's sections); (B) a fixed layout with a switch for each layer and author text for tags and labels; (C) a fixed layout where only named texts move into preset fields (for example the length rule, the history labels, the empty direction block).
- The engine rules in `docs/diataxis/explanation/prompt_system_design.md` already have written reasons (system and user halves, the style and format sections after the history, XML tags as content labels, two tiers of NPC cards). Each one either stays a rule with that reason or becomes changeable.
- The Quantifier and Options are a separate ticket: [Decide which parts of a Quantifier or Options call an author can change](10-quantifier-options-layout.md).
- The card fields that [Compare our card and World format with ST and ME](04-card-and-world-format.md) finds missing may need a place in the layout (for example a card's own post-history text).
