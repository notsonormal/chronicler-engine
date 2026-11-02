# Remove the data conflicts that distort a trial

Type: task (HITL)
Status: claimed
Blocked by: 13

## Question

Which contradictions and missing facts in the data of the test setup make the stories inconsistent, and what is the smallest data change that removes each one before the output trials run?

## Context

- Origin (owner, 2026-11-02): the pilot of an LLM judge in [Decide how to read an output trial](05-read-an-output-trial.md) found faults that come from our own data, not from the model or the layout. A trial that carries them measures the data as well as the candidate. The owner: "at least we should avoid inconsistencies with data distorting things". Evidence and causes: [the inconsistencies of two trial stories](../assets/story-inconsistencies.md).
- Scope: the data that `scripts/trial_setups/redmist_julian.json` uses: `data/worlds/redmist_estate/` (`world.json`, `map.json`), the five cards in `data/characters/redmist_estate/`, and `data/personas/julian.json`. Only contradictions between these files, and missing facts that the stories contradict. Better writing of the World, the cards or the presets stays out of scope (map, Out of scope).
- Done already (2026-11-02): the father/grandfather conflict. `julian.json` now says "grandfather", in the JSON and in the `julian` row of the probe DB.
- Known faults to fix:
  - **The era.** `world.json` does not state the present day ("Built in the colonial era", "colonial architecture", "distinct social classes"), while Carla's card and her opening ask for a phone. Both stories drift from a smartphone to carriages and shillings. The owner chooses the era. The World then states it, and the cards that disagree change.
  - **The short entry of an absent Character.** `<KnownCharacters>` sends a short entry, which lacks facts that the stories then invent: Louise's surname Petit ("Louise Marchand", Q #533), and Jezebel's family background ("no family", Q #555). Find which card field fills the short entry, and add the fixed facts (full name, the relations to Bernard and Julian) to it.
- A scan for other conflicts: the two stories show only the conflicts that they happened to touch. An agent compares every fact across the World, the map, the five cards and the Persona (names, ages, relations, dates, places, who works for whom), and compares each short entry with its full card. The owner decides each conflict that the scan finds.
- A probe DB keeps a copy of a Persona once seeded (`seed_personas` in `src/bootstrap/load.rs` inserts only a missing key). Find whether the cards and the World are copied in the same way. If they are, a data change must also reach the probe DB, or the next trial still plays the old text.
- Not in this ticket: the faults that come from the history cut, the story clock and who knows what (Long-term memory and state), the attention of the model, and the role breaches ([Try the history as chat messages](07-history-as-chat-messages.md)).
- Open, for the answer: does the wrong Room for places off the estate (the map has 10 Rooms, all on the estate) count as a data conflict to fix here, or is it an engine question? See the asset.
- [Sort the inconsistencies by their cause](13-sort-inconsistencies-by-cause.md) runs first and adds each data cause it finds to this ticket. [Stop the role breaches with stop sequences](14-stop-role-breaches.md) waits for this ticket, so its trial plays the fixed data.

- Scan done (2026-11-02): [the data conflicts of the test setup](../assets/data-conflict-scan.md). The short entry is the `summary` field. The probe DB keeps its own copy of the World, the map and the cards, so each change must also update those rows. The owner's decisions on the findings are pending.

## Progress (2026-11-02)

The owner decided the rows of the scan. The changes are in the JSON files and in the probe DB rows (checked equal to the JSON after the update). `validate-data` passes.

- Row 1, the era: `world.json` `description` now says "The story takes place in the present day." The owner put it in the description, not in a global rule. No currency is named.
- Rows 2 and 3: the summaries of Louise and Lisette start with "Louise Petit" and "Lisette Franklin".
- Row 4: Jezebel's summary adds "Born into a wealthy family of financiers."
- Row 6: Jezebel's example dialogue no longer says "immediate family" ("he didn't trust anyone else with his wealth").
- Row 7: `julian.json` `scenario` says "in a city far from Islaport".
- Row 9: `master_quarters` says "The master's luxurious bedroom."
- Row 5 (Louise naive or worldly): left for now by the owner. Row 8 (Bernard's trade): left as it is. Row 10 (places off the estate): waits for ticket 13.
- Still open: row 5, row 10, and the data causes that ticket 13 adds. Restart the probe server before the next trial, in case it holds the old World in memory (guessed).

Done when each known fault and each conflict from the scan is fixed or ruled out by the owner, the probe DB holds the fixed data, `scripts/validate_data.py` passes, and the answer lists each change.
