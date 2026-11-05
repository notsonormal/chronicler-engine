# Make the card probe

Type: task (AFK)
Status: resolved
Blocked by: 10

## Question

Do the card structure and the rule text of [Decide the structure of a Character card and who knows each part](10-card-structure.md) reduce card leaks and entry faults on our model, before any build? Make a probe on stored requests, in the shape of the memory test, and run it.

## Context

- **Why a probe.** The `redmist_julian` trial seldom mentions an absent Character (judge X of lt15, `tmp/lt15/judge/judge-X.md`, section 3). So a 100-Cycle trial cannot measure entry faults. The label also needs a measure: in the Curse of Knowledge preprint, a self-filter prompt cut leaks by only 64.7 % on average ([LLM memory research](../assets/llm-memory-research.md), section 4).
- **The model to copy (known, [Make the memory test from the stored stories](04-memory-test.md)).** `assets/memory_test.py` replays a stored Narration request and changes one block only. One run of 42 questions costs about 640,000 to 1,200,000 prompt tokens for each arm. At temperature 0 the answers change from run to run, so each arm needs 3 runs, and a gap under about 10 points is noise.
- **The arms (decided in [10](10-card-structure.md), Answer).** Every arm except arm 1 puts every card always on, in one block before the history, in place of `<KnownCharacters>`. Each arm runs 3 times, on Narration calls and on Impersonate calls. The Impersonate call matters most: the leak A-3.3 came from it (`tmp/lt15/new/trial-1/calls/01210-impersonate.md`).
  1. Today's prompt.
  2. The minimal form without the rule text: the cards as written, with `secrets` moved out of `scenario`, `summary` removed, and `example_dialogue` included.
  3. The minimal form with the rule text at the head of the card block. The text is in the Answer of [10](10-card-structure.md).
  4. The full split, with a label on each part ([the card structure mapping](../assets/card-structure-mapping.md)).
- **The decision rule.** The build uses arm 3. It uses arm 4 only if arm 4 has clearly fewer leaks than arm 3, by more than the noise of about 10 points.
- **What to count.**
  - A card leak: a Character says or uses another Character's secret, or a card fact that she has not learned. The Impersonate call counts too: a Julian Input that uses a card fact or a secret of another Character.
  - A secret that a Narration states outright. It is a count, not a fault: no rule forbids it ([10](10-card-structure.md), Answer). It also shows whether such a Narration goes with leaks in Julian's Input.
  - A repeated first meeting: a scene that copies the first meeting of an `example_dialogue`. Arms 2 to 4 add the examples.
  - An entry fault: a first portrayal or a mention that contradicts the card. Sources: stored calls just before a Character joins the State line (the Δ lines of `tmp/lt15/judge/story-*.md`), stored calls where a present Character mentions an absent one, and a changed Input that names an absent Character ("Julian asks Carla about the cook").
- **The scorer.** Use a rule where a fact is a fixed string (a surname, a number of years). Use a judge that quotes the evidence for the rest, with the method of [Decide how to read an output trial](../../layout-trials/issues/05-read-an-output-trial.md).
- **The source data.** The stored calls of `tmp/lt15/new/trial-1/calls/` and the stored stories of ticket 04. The probe makes the cards of arms 2 to 4 from the five Redmist cards and `julian.json`, as test data under `assets/`: arms 2 and 3 from the `secrets` table of the Answer of [10](10-card-structure.md), arm 4 from [the card structure mapping](../assets/card-structure-mapping.md). It does not change `data/`.
- An agent may not read `.env` files, and never reads or prints a key.

## Output

- The probe tool and its scene set under `assets/`.
- A results asset: the leak and entry-fault rates of each arm, 3 runs each, and the token cost.
- A recommendation: arm 3 or arm 4 to build, or that the label does not hold on this model. In that case the build or [Decide how a Character or a lore entry enters the prompt](12-prompt-entry-modes.md) must hide more.

## Answer

Resolved 2026-11-04 (AFK). The probe ran 18 scenes of story A in 4 arms, 3 runs each: 216 answers of `deepseek-v4.1-flash`. A `glm-5.3-flash` judge quoted its evidence, and I checked every finding by hand. Details, the per-scene table and the limits: [the card probe](../assets/card-probe.md). Tool and data: [`card_probe.py`](../assets/card_probe.py), [`card_probe_scenes.json`](../assets/card_probe_scenes.json), [`card_probe_cards.json`](../assets/card_probe_cards.json). Raw runs: `tmp/ltm11/`.

**The confirmed counts.** The cells count answers with at least one confirmed finding.

| Arm | Impersonate leak (of 24) | Narration leak (of 30) | Contradiction of a card (of 30) | Stated secret | Repeated meeting |
|---|---|---|---|---|---|
| 1 today | 3 | 0 | 3 | 0 | 0 |
| 2 minimal | 3 | 2 | 1 | 0 | 0 |
| 3 minimal with the rule text | 3 | 3 | 0 | 0 | 0 |
| 4 full split | 3 | 6 | 0 | 0 | 0 |

**The recommendation: the label does not hold on this model.** The build or [Decide how a Character or a lore entry enters the prompt](12-prompt-entry-modes.md) must hide more.

- **The rule text did nothing that the probe can see (known).** Arms 2 and 3 are the same within one answer. In scene I2, Julian tells Lisette "you've been in this house near a decade" in all 12 runs, in every arm, with or without the rule text.
- **Arm 4 is not built (known).** It has more confirmed leaks than arm 3 (6 against 3), not fewer. The decision rule of this ticket keeps arm 3 over arm 4.
- **Full cards remove entry faults and bring in leaks (known).** With only short entries (arm 1), the model invents a past for Gabriella and a meeting between Louise and Gabriella, and these contradict the cards (3 answers). With every card always on, Carla gives Gabriella's secret past when Julian asks about her (8 of 9 runs of arms 2 to 4, against 0 of 3).
- **What stays from ticket 10 (inferred).** The `secrets` field, the removal of `summary` and `example_dialogue` in the card are not refuted. `example_dialogue` caused no repeated meeting in 54 answers for each arm. The narrator stated no secret, so no Narration rule is needed (the condition of [10](10-card-structure.md), Answer). The rule text costs about 70 tokens and showed no effect. [12](12-prompt-entry-modes.md) decides whether to keep it.
- **Where hiding must act (inferred from the scenes that leaked).** (1) The `secrets` of a Character who is not present: N5 leaked while Gabriella was absent. (2) The other Characters' cards in the Impersonate call: I2 leaked a plain `scenario` fact, not a secret, so hiding `secrets` alone does not fix I2. The probe can test a candidate layout as a new arm with no build.

**Cost.** The always-on card block adds about 2,600 prompt tokens to each call (+25 %), net of the cards that leave `<CharactersPresent>`. The probe used about 2.7 million answer prompt tokens and 0.45 million judge prompt tokens.

**Limits.** One story, 18 scenes, 3 runs: only the effects that repeat in every run hold (I2, and N5 in arms 2 to 4 against arm 1). 12 scenes are directed questions. The six stored calls gave no confirmed finding, and the stored leak A-3.3 did not come back in any of 12 runs of its call. Misses of the judge are not counted.
