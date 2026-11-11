# The inconsistencies of two trial stories, and their causes

Source: the pilot of an LLM judge in [Decide how to read an output trial](../issues/05-read-an-output-trial.md). Two `reviewer` judges each read one whole story: P is `tmp/lt01/redmist_julian/trial-1` (the baseline) and Q is `tmp/lt03/redmist_julian/trial-1` (the changing blocks after the history). Verdicts: `tmp/lt05/judge-P.md` and `tmp/lt05/judge-Q.md`. The two trials differ only in a cache change, and both judges gave the same scores (consistency 2/5, overall 6/10).

Method of the cause check: each call of a pack records how many entries its history holds. A contradiction is "out of the history" when the earlier entry was no longer in the history of the call that wrote the later entry. "Known" means checked in the data or in the stored calls. "Inferred" and "guessed" are not checked.

## 1. The fact had left the history (known)

The first history cut is at #400 in P and at #595 in Q. After it the history holds about 36 to 75 entries.

| Story | Earlier → later | Fault | History at the later entry |
|---|---|---|---|
| P | #338 → #476, #508 | Julian and Gabriella forget that they already met | 44 entries |
| P | #369 → #435, #445 | Bernard stopped paying "eight weeks before he died", then "March", then "two years" ago | 52, 43 |
| P | #363 → #433 | The debt is 740,000, then "eleven" thousand | 50 |
| P | #367 → #431 | Pell has served 30 years, then 40 | 48 |
| Q | #541 → #687 | The desk was opened on day one; late, it has a locked drawer again, and oak becomes mahogany | 67 |
| Q | #547 → #721 | The revolver disappears | 62 |
| Q | #543 → #725 | A second "If you are reading this" letter repeats the first | 66 |

Owner of the fix: [Long-term memory and state](../../long-term-memory-and-state/idea.md).

## 2. The model contradicted a fact that was in the prompt (known)

| Story | Earlier → later | Fault | History at the later entry |
|---|---|---|---|
| P | #376 → #385 | Julian pockets the key, then it is "still on its nail" | 62 |
| P | #331 → #374 | Carla questioned "the housekeeper", who "died two winters back" | 51 |
| P | #353 → #359 | The attic stair was boarded "before I came here", but Lisette heard the argument "the same year" | 36 |
| P | #389 → #397 | The dust is 30 years old, the crate is dated 18 years ago | 74 |
| Q | #567 → #581 | The instruction "came by post", then "on the telephone" | 57 |

Cause: the attention of the model (inferred). No layout candidate targets it. A stronger model, or a list of fixed facts, may help (guessed).

## 3. Our data (known)

- **Father or grandfather.** `data/personas/julian.json` said "father", and the World and the cards of Jezebel and Gabriella say "grandfather". The stories drift between both (P #416, #467; Q #664). Fixed on 2026-11-02 in the JSON and in the `julian` row of the probe DB `target/ui-investigator/debug/chronicler_3001.db`.
- **The era.** `world.json` does not state the present day. It says "Built in the colonial era", "colonial architecture" and "distinct social classes". Only Carla's card and her opening ask for a phone. Both stories start modern (smartphone, car) and drift to a period world (carriages, shillings, porter's lads, lamp wicks), P from about #333 and Q from about #594.

## 4. The short entry of an absent Character (known)

A Character who is not present gets only a short entry in `<KnownCharacters>` (the two tiers of NPC cards).

- Louise's short entry has no surname, so the model invented "Louise Marchand" (Q #533). Her card says Petit.
- Jezebel's short entry has no "born into wealth", so Carla says she "came up through the numbers herself, no family" (Q #555).

## 5. No story clock (inferred)

The World says "Bernard Redmist died two weeks ago". The engine does not track the time that passes in the story, so the model invents durations: "four days" in the house (P #418, #472, #502) when the story covers the second day; "two weeks" in the house (Q #598, #682) after one day; Mira quit "a month after the funeral" (Q #589).

Owner of the fix: Long-term memory and state.

## 6. Who knows what (inferred)

The narrator sees the whole history, and the engine does not record which Character saw or heard what. Carla knows about Louise's wax after she heard one word of that talk (Q #605). Gabriella knows that Mira is in an almshouse (Q #683). Carla asks whether the desk is still in the house after she uncovered it herself (Q #685).

Owner of the fix: Long-term memory and state.

## Two engine faults that can feed the inconsistencies

- **Role breaches.** Inputs write `Narrator:` and `Julian:` turns for the other side: P #374, #410, #420 (847 words), #438, #504, #506; Q #594 (1344 words). Narrations write Julian: P #375, #411, #439, #505; Q #685. Known: the history is one text block with `Narrator: ` and `Julian: ` labels (a stored Narration call of Q holds 6 such lines). Inferred: the model continues that format. Guessed: a breach in the history is copied, since five more follow the first one in P. See [Try the history as chat messages](../issues/07-history-as-chat-messages.md).
- **The wrong Room in the State lines.** Known: the map of `redmist_estate` has 10 Rooms, all on the estate. The bank, the harbour, the almshouse and the registrar get the nearest wrong Room (P #413, #415, #421–#500; Q #571–#591, #573, #639). The Characters present also lose Carla while she is on stage (P #516–#523; Q #535, #581), and keep Jezebel after she left (P #377). Guessed: the next Narration reads these wrong values in `<GameState>` and `<CharactersPresent>`.

## Not inconsistencies, but drift from the card

Louise is dry and worldly, not naive (P #377–#413, Q #693). Jezebel is cold, not warm (P #343–#377, Q #559–#567). Gabriella does not try to seduce Julian (P #511–#523, Q #594–#701). Julian becomes a confident detective in a period dialect ("I'll not": 0 in the first 40 entries of P, 3.0 per 1,000 words in the last 40). Cause guessed: the model's own style wins over the card.
