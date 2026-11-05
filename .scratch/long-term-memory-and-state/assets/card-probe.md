# The card probe

Source: [Make the card probe](../issues/11-card-probe.md), 2026-11-04.

The card probe replays stored calls of story A (`tmp/lt15/new/trial-1`) with four card layouts. It counts card leaks, secrets that the narrator states, card contradictions (entry faults) and repeated first meetings. Only the card text of each request changes from arm to arm.

- The tool: [`card_probe.py`](card_probe.py).
- The scene set and the facts: [`card_probe_scenes.json`](card_probe_scenes.json), 18 scenes (8 Impersonate, 10 Narration and event).
- The card data of arms 2 to 4: [`card_probe_cards.json`](card_probe_cards.json). Every string is an exact cut of `data/characters/redmist_estate/*.json`, and the tool checks each cut.
- The raw results: `tmp/ltm11/` (`runs/` for the answers, the judge verdicts and the cost, and `report.md`). `tmp/` is not in git.

## Summary

- **known** The rule text has no measured effect. Arm 2 (no rule text) and arm 3 (rule text) have the same confirmed leaks on Impersonate calls (3 of 24 each) and nearly the same on Narration calls (2 and 3 of 30).
- **known** The full split (arm 4) is not better than arm 3. It has 6 confirmed Narration leaks against 3. By the decision rule of the ticket, the build does not use arm 4.
- **known** The label does not hold on this model. In scene I2, Julian says "you've been in this house near a decade" to Lisette in all 12 runs, in every arm. Arm 1 gives that fact in the short entry, and arms 2 to 4 give it in the card.
- **known** A secret in the prompt leaks when a Character is asked about an absent Character. In scene N5, Carla tells Gabriella's past ("A fishing town up the coast, family ran a tackle shop. She left at eighteen") in 8 of the 9 runs of arms 2 to 4. In arm 1 the past is not in the prompt. So the model makes up a past, which contradicts the card in 2 of 3 runs.
- **known** The full cards remove the entry faults. Arm 1 has 3 confirmed card contradictions in 30 answers, and arms 2 to 4 have 1, 0 and 0. The contradictions of arm 1 come from text that it does not send: Gabriella's past (N5) and Louise's relationship line to an absent Gabriella (N10).
- **known** The narrator stated no secret outright in any of the 216 answers. No Character repeated a first meeting, so `example_dialogue` caused no repeated meeting here.
- **inferred** The trade-off is plain. When every card is always on, the entry faults go away and secret leaks come in. Text on a card that the model must not use in a call has to stay out of that call.

## How to run it

Run from the repo root. The tool uses the model client of [`memory_test.py`](memory_test.py), which reads only `OPENCODE_GO_API_KEY` and never prints it. The answers use `deepseek-v4.1-flash` with `reasoning_effort: none` and temperature 0, as in the trials. The judge is `glm-5.3-flash` at temperature 0, a model of another family.

```bash
T=.scratch/long-term-memory-and-state/assets/card_probe.py
python3 $T check                 # each scene: the entry its call wrote, and facts already in the story
python3 $T show N5 minimal_rule  # the request of one scene and arm (add --judge for the judge prompt)
python3 $T run today --runs 3    # also minimal, minimal_rule, split; the four can run at the same time
python3 $T rescore               # score the stored verdicts again after a change of the score rules
python3 $T report --detail --findings
```

One run of 18 scenes takes about 2 minutes with 6 workers.

## How it works

1. **The scenes.** A scene is one stored call. Six scenes replay a call as stored. The other scenes change the Input of a Narration call (`<LastMessage>`) or add a Player direction to an Impersonate call. A change asks about a Character or her past, for example "Tell me about Gabriella. Where did she come from, before she married my grandfather?" (N5). `check` warns when the story before the call already holds a fact of the scene. The two warnings for `lisette_decade` are false: at #536 and #590, "a decade" is said of Jezebel.
2. **The arms.**
   - `today`: the stored request.
   - `minimal`: one `<CharacterCards>` block in place of `<KnownCharacters>`. It holds each of the five cards as written, with a `Secrets:` line cut out of `scenario`, no `summary`, all relationship lines, and `example_dialogue` under a header ("an example of her voice, not an event of the story"). Carla's line "Carla will never speak for {{user}} under any circumstances." is dropped. After the history, `<CharactersPresent>` gives only the names.
   - `minimal_rule`: `minimal` with the rule text of ticket 10 at the head of the block.
   - `split`: the full split of [the card structure mapping](card-structure-mapping.md), with the labels "Seen by anyone who meets her", "Known to the cast", "Known only to <name>" and "Writing guide", and the rule text. The rule text is in this arm too, so that the split is the only difference from arm 3.
   - The Persona card and the World rules are the same in every arm.
3. **The judge.** One judge call reads each answer. It gets the end of the scene, the answer, the World rules, the facts that nobody outside their owner has learned, the full cards of the focus Characters, and the Characters that Julian has met. It answers in JSON. Each finding must quote the answer, and the tool drops a finding whose quote is not in the answer. The score counts:
   - a leak: a fact that Julian, or a Character outside its owner and its "may know" list, states as known;
   - a stated secret: the narrating voice states a secret;
   - a contradiction: a portrayal or a mention against the card;
   - a repeated meeting: a Character that Julian has met introduces herself again.
   A fact that the World rules support is not a leak (for example, that Gabriella is manipulative).
4. **The hand check.** The judge over-counts. I read each leak and each contradiction that the judge found against the answer and the card (below). The result table counts only the confirmed findings.

## The result

3 runs of each arm. The cells count the answers with at least one confirmed finding.

| Arm | Impersonate: leak (of 24) | Narration: leak (of 30) | Narration: contradiction (of 30) | Stated secret (of 54) | Repeated meeting (of 54) | Judge leak findings before the check (Impersonate + Narration) |
|---|---|---|---|---|---|---|
| 1 `today` | 3 | 0 | 3 | 0 | 0 | 3 + 3 |
| 2 `minimal` | 3 | 2 | 1 | 0 | 0 | 4 + 11 |
| 3 `minimal_rule` | 3 | 3 | 0 | 0 | 0 | 4 + 9 |
| 4 `split` | 3 | 6 | 0 | 0 | 0 | 4 + 8 |

The confirmed findings by scene (one mark for each run):

| Scene | What it asks | `today` | `minimal` | `minimal_rule` | `split` |
|---|---|---|---|---|---|
| I2 | Julian asks Lisette one personal question | leak ×3 | leak ×3 | leak ×3 | leak ×3 |
| N4 | Julian asks Carla how close Jezebel and Bernard were | — | contradiction ×1 | — | leak ×3 |
| N5 | Julian asks Carla where Gabriella came from | contradiction ×2 | leak ×2 | leak ×3 | leak ×3 |
| N10 | Julian asks Louise whether Gabriella comes to the kitchen | contradiction ×1 | — | — | — |
| the other 14 | | — | — | — | — |

What each confirmed finding is:

- **I2, all arms.** "You've been in this house near a decade" or "near ten years". No entry before #613 gives Lisette's years of service. The stored call I1 did not repeat the stored leak A-3.3 in any of its 12 runs, so the stored leak is rare without a direction (inferred).
- **N5, arms 2 to 4.** Carla gives Gabriella's secret past: "a fishing town up the coast, family ran a tackle shop. She left at eighteen" (arms 2 and 3), or "Turned up in the city with debts … within a year she was on Bernard's arm" (arm 4). In arm 3, run 1, she gives "a string of men before Bernard who paid them off".
- **N5, arm 1.** Gabriella "came from the theater" (run 1), or her father "was a shipping man who lost his fleet" (run 3). Both contradict the card. In both runs the narrator put Jezebel into the carriage scene.
- **N4, arm 4.** Carla: "Close enough that she knew his will before the lawyer read it." It gives the secret of Jezebel's talks about the will. Confidence: medium.
- **N4, arm 2, run 1.** Carla says that Bernard "stopped taking her counsel on the will". This contradicts Jezebel's card. The judge called it a leak.
- **N10, arm 1, run 2.** Louise says that Gabriella came down once and "was pleasant about it". Louise's card says that Gabriella berated her for an hour over a cold meal. Arm 1 sends only the relationship lines to the Characters present (`render_present_relationships`), so this line was not in the call.

The judge findings that the check dropped (15 of 46 leak findings, and 3 of 6 contradictions):

- General claims that the World rules support. For example, Lisette in N1: "She'll be pleasant to your face. That's the trouble." And Julian in I8: "You were a soldier, then you were Bernard's."
- Vague lines that do not give the specific content. For example, Carla in N4: "he told her things he wouldn't tell his lawyer".
- Wrong matches. In I1, the judge matched "a lawyer and a bodyguard" to Lisette's wish to be a lawyer.
- Made-up facts that the judge called leaks. These are contradictions or nothing.

## Cost

| Arm | Prompt tokens per run of 18 | Per answer call | Completion tokens per run | Judge prompt tokens per run |
|---|---|---|---|---|
| 1 `today` | 190,000 | 10,600 | 2,650 | 37,000 |
| 2 `minimal` | 237,000 | 13,200 | 2,850 | 37,500 |
| 3 `minimal_rule` | 238,000 | 13,200 | 2,850 | 37,500 |
| 4 `split` | 242,000 | 13,400 | 2,770 | 37,400 |

- The full card block adds about 2,600 prompt tokens to each call (+25 %), net of the cards that leave `<CharactersPresent>`. The rule text adds about 70.
- The whole probe used about 2.7 million answer prompt tokens and 0.45 million judge prompt tokens. The cached share (4 to 41 %) does not show the cache of a trial. Each run had its own session, with only 18 calls.

## Limits

- **Small counts.** One story, 18 scenes and 3 runs. A difference of 1 to 3 answers between two arms is inside the noise (inferred from the spread of the runs). The signals that hold are the ones that repeat in every run: I2 in every arm, and N5 in arms 2 to 4 against arm 1.
- **Directed scenes.** 12 of the 18 scenes ask about a Character on purpose. The rates are rates for those questions, not for play. The six stored calls gave no confirmed finding.
- **One judge and one checker.** The judge is one `glm-5.3-flash` call for each answer, and I checked its findings alone. A missed finding (a leak that the judge did not report) is not counted. I did not read every answer for misses.
- **Who may know is a choice of the scene set.** For example, the set says that only Jezebel may know that Gabriella married Bernard for money. A different choice changes the counts of N1, N5 and N10.
- **The Persona card did not change.** No arm moves Julian's past behind a label, and no scene asks another Character about it.
