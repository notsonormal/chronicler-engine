# The data conflicts of the test setup

Source: [Remove the data conflicts that distort a trial](../issues/12-remove-data-conflicts.md), 2026-11-02. The scan compared every fact across `data/worlds/redmist_estate/world.json`, `map.json`, the five cards in `data/characters/redmist_estate/` and `data/personas/julian.json`. It also compared each short entry (`summary`) with its full card. "Known" means read in the files. Each row ends with a proposal for the owner.

## How the data reaches a prompt (known)

- **The short entry is the `summary` field.** `render_known_character_entry` (`src/application/prompting/builders/sections.rs`) sends `summary` to `<KnownCharacters>`. Only a card with no `summary` falls back to the first three lines of `description`.
- **The probe DB keeps its own copy of the World, the map and the cards.** `process_world_dir` (`src/bootstrap/load.rs`) seeds a World only when its key is missing, and a card only when its id is missing in that World. `seed_personas` does the same for a Persona. So a data change must also update the rows `worlds`, `maps`, `characters` and `personas` of `target/ui-investigator/debug/chronicler_3001.db`, or the next trial plays the old text.
- On 2026-11-02 the probe DB rows matched the JSON files exactly (the World, the map, the five cards and `julian`). The DB holds 8 Games.

## Findings

| # | Fact | Where | Conflict or gap | Proposal |
|---|---|---|---|---|
| 1 | The era | `world.json` ("Built in the colonial era", "colonial architecture", "distinct social classes"); Lisette ("Master {{user}}", "very traditional black maid dress"); against Carla (phone, sunglasses), Gabriella ("your ride", clubbing, celebrities, brands), Jezebel (college, analyst), Louise (franchise), the map ("stainless steel kitchen") | No sentence states the present day. The period words win late in both stories (shillings in P: 12 times) | Add a global rule that states the present day. Rewrite "Built in the colonial era" as the age of the house, not of the story |
| 2 | Louise's surname | Card: "Louise Petit". Summary: none. World: "Louise" | The model invented "Louise Marchand" (Q #533) | Add "Louise Petit" to the summary |
| 3 | Lisette's surname | Card: "Lisette Franklin". Summary: none. World: "Lisette" | Same gap as #2. Not seen in a story yet | Add "Lisette Franklin" to the summary |
| 4 | Jezebel's background | Card: "born into wealth", a family in finance. Summary: none | Carla says "no family" (Q #555) | Add "Born into a wealthy family of financiers" to the summary |
| 5 | Louise: naive or worldly | Personality: "naive". Scenario: "ventured around the world working in various kitchens" | The card contradicts itself. Both judges saw "dry and worldly" Louise | Owner: keep "naive" and soften the travel, or replace "naive" (for example "eager", "trusting") |
| 6 | Who is Julian's parent | Julian: "Julian Redmist", "raised by a single mother". World: "a grandson he never knew". Jezebel: "couldn't trust his wealth with his immediate family" | No file says which of Bernard's children is Julian's parent, or whether Bernard has other living family. "Immediate family" hints at relatives that no file names | Owner: name the parent (for example "Bernard's estranged son, who left Julian's mother before Julian was born") and say whether other relatives exist. Or remove "immediate family" from Jezebel's example dialogue |
| 7 | Where Julian grew up | Julian: "raised by a single mother in the city", "travel gear". World: Islaport, "a large coastal city" | "The city" can read as Islaport. Then the stranger who never heard of the most famous family in town is odd | Owner: name the place, or write "in a city far from Islaport" |
| 8 | Bernard's trade | World: the family were "surgeons and physicians" for generations. No file says what Bernard did | P invents physicians and surgeons 7 times. Not a conflict yet | Leave it. Fill only if ticket 13 shows a fault from it |
| 9 | Second person in a Room | `map.json`, `master_quarters`: "Your luxurious bedroom" | The test setup is third person, past tense. `<GameState>` sends the Room description as written | Rewrite as "The master's luxurious bedroom" |
| 10 | Places off the estate | `map.json`: 10 Rooms, all on the estate | The bank, the harbour and the almshouse get the nearest wrong Room | Wait for [Sort the inconsistencies by their cause](../issues/13-sort-inconsistencies-by-cause.md), which decides data, Quantifier prompt or model |

## Checked with no conflict (known)

- Ages: Carla 30 (army at 20, out after 4 years, then gate guard and promotions), Gabriella 31 (World and card), Jezebel 40 (advisor "over a decade"), Lisette 28 (out of the orphanage at 18, "almost a decade" at the estate), Louise 25, Julian mid-20s.
- Roles and who works for whom: the World and every card agree (Carla from Bernard's protection detail, Jezebel on the finances and the foundations, Lisette and Louise reassigned to Julian, Gabriella in the guest wing).
- Relations: each pair agrees in kind. Two pairs differ in warmth (Louise "warm friendship" and Lisette "friendly acquaintance"; Louise "friendly respect" and Carla "distant tolerance"). This reads as an intended one-sided view, not a conflict.
- The father/grandfather conflict: fixed before this scan.

## Outside this ticket (known)

- The summaries and the global rules of the World write "Julian" where the cards write `{{user}}`. The name is right for this test setup. For another Persona it is wrong, which is authoring work (map, Out of scope).
