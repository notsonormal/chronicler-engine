# The memory faults of the newest trial, and the facts it lost

Source: [Count the memory faults of the newest trial](../issues/03-baseline-faults.md), 2026-11-03.

- **Story A** is `tmp/lt15/new/trial-1`: the newest story, on the build of [Try the history in the ME format](../../layout-trials/issues/15-history-in-me-format.md) (`1989d3fe`). It has 100 Cycles and 201 entries (#524 to #724).
- **Story B** is `tmp/lt15/reference/trial-1`: the reference story of the same ticket, on `8a015b71`, before the ME format. It has 202 entries (#524 to #725).
- The work files are in `tmp/ltm03/`: `judge-A1.md`, `judge-A2.md`, `judge-B1.md`, `checked.md` and `facts-A.md`. `tmp/` is not in git.

**Method.** Three `reviewer` judges used the prompt below. Two ran on A in separate sessions, and one ran on B. Each judge read the whole story with a neutral name. A fourth `reviewer` checked every finding against the story and the data, merged the two runs on A, and dropped the false findings. The coordinator spot-checked two confirmed findings (A-2.1 and A-3.5) in the story text and the World file. An `implementer` with no view of the verdicts made the fact list from the dropped entries of A.

## Summary

- **known** Neither story has a confirmed fault of kind 1, a fact that had left the history. The judges found 1 and 2 kind-1 findings in A and 2 in B, all at low confidence. The check dropped all of them: in each case the fact had already changed inside the history, or no fact was contradicted.
- **known** The checked counts are small. Story A has 0, 4 and 5 of the three memory faults, and story B has 0, 1 and 3.
- **known** The raw judge counts are not a measure. A1 found 19 instances and A2 found 16, but only 9 were confirmed. Eight more were date conflicts inside the history, which is fault kind 2 of [the inconsistencies](../../layout-trials/assets/story-inconsistencies.md) and out of scope in this map. With the wording of the judge prompt, the clock count is 12 for A and 2 for B.
- **known** The two runs on A agree on the confirmed instances: both found all 4 clock faults and 3 of the 5 who-knows-what faults.
- **inferred** The story loses its old facts by dropping threads, not by contradicting them. The login, the master-key handoff, Gabriella and Jezebel are not used again after the first cuts (fact list, column `later use`). A count of contradictions does not see this loss. The fact list and the memory test measure it.
- **inferred** A memory of the dropped entries fixes none of the confirmed faults. Several of them come before the first cut. These fixes can help (guessed):
  - a story clock, for A-2.3, A-2.4 and B-2.1;
  - a record of who was present at each talk, for A-3.1, A-3.2, A-3.5, B-3.1 and B-3.2;
  - a rule that keeps card facts away from a Character who has not learned them, for A-3.3, A-3.4 and B-3.3. In these, Julian knows Louise's surname or Lisette's years of service, which only the cards state.
- **inferred** Each checked count is 5 or less, and kind 1 is 0 in both stories. So a 100-Cycle check trial with one judge run for each side cannot show a change of any one kind on its own.

## The judge prompt

`{L}` is the neutral name of the story, and `{N}` is the run number.

```text
You are a judge of one interactive-fiction story. You look for three kinds of memory fault only.

Rules: Do NOT read tmp/ltm03/key.txt. Do not read any other story-*.md file or any judge-*.md file. Do not read anything else under tmp/. Do not change any file except your own output file.

The story: tmp/ltm03/story-{L}.md (about 200 entries, 32,000 to 38,000 words). Read ALL of it: page through with offset/limit. Entries alternate: "Narration" (written by the narrator model) and "Impersonate Input" (written by a model that plays the player character, Julian). Each entry has a "> State:" line with the Room and the Characters present that the engine tracked. Each entry also has a line that says how many story entries the history of the call that wrote it held. A "history cut" means that the oldest entries left the model's context. Use the "history holds N entries" lines to find, for each entry, the oldest entry that its call could still see: for entry #k whose call held N entries, the history ended at the entry before #k and held the N entries before #k.

Reference material (read before the story):
- The World: data/worlds/redmist_estate/world.json (description, global_rules, scenarios) and data/worlds/redmist_estate/map.json.
- The Characters: data/characters/redmist_estate/*.json.
- The Persona (the player character): data/personas/julian.json.
- Earlier examples of the three fault kinds, from other stories of the same setup: .scratch/layout-trials/assets/story-inconsistencies.md, sections 1, 5 and 6. Ignore its entry numbers: they belong to other stories.

The three fault kinds:

1. A fact that had left the history. A later entry contradicts, forgets or repeats as new an earlier fact (a name, a number, an object and where it is, a past event, a promise, a meeting that already happened), AND the earlier entry was no longer in the history of the call that wrote the later entry. If the earlier entry WAS still in the history, it is not this kind: list it under "Not counted" at the end with one line.
2. No story clock. The story states or implies a duration, a date or a time of day that does not match the time that has passed in the story itself (for example "four days in the house" when the story covers the second day), or that contradicts a time fact of the World or an earlier entry.
3. Who knows what. A Character (or Julian) knows, says or acts on something that the Character could not know: the Character was not present when it was said or done, nobody told the Character, and it is not common knowledge in the World. Also count a Character who forgets something that the Character itself saw or did (for example asks about an object that the Character found).

For each kind, list every instance that you find, not a sample. For each instance give:
- the later entry (#NNN) and the earlier entry or entries it conflicts with (#NNN), or the World / card file for a time fact;
- a short quote from each side;
- for kind 1: the history count of the later entry, and the oldest entry that its call could see;
- for kind 3: the Character, and why the Character could not know it (where was the Character at the earlier entry: check the State lines and the text, because the State lines can be wrong);
- one sentence on the fault;
- your confidence: high, medium or low.

Then give:
- a count table: kind, count, count at high confidence;
- the entry where the first history cut takes effect, and the oldest entry in the history at the last entry;
- a short "Not counted" list: faults that look like these kinds but that you excluded, with the reason (for example, the earlier fact was still in the history);
- how sure you are that you found every instance of each kind (high / medium / low), and what made it hard.

Write your full verdict to tmp/ltm03/judge-{L}{N}.md, in Markdown, with the count table at the top. Your final message: the count table and the path.
```

## Count tables

### 1. Count by kind

| Kind | A1 raw | A2 raw | A merged and checked | B1 raw | B checked |
|---|---|---|---|---|---|
| 1. A fact that had left the history | 1 | 2 | 0 | 2 | 0 |
| 2. No story clock | 12 | 10 | 4 | 3 | 1 |
| 3. Who knows what | 6 | 4 | 5 | 4 | 3 |
| Moved to another kind: a date conflict inside the history (asset kind 2) | — | — | 8 | — | 1 |

### 2. Agreement on story A (confirmed instances only)

| Kind | Both | A1 only | A2 only |
|---|---|---|---|
| 1 | 0 | 0 | 0 |
| 2 | 4 | 0 | 0 |
| 3 | 3 | 1 | 1 |
| Moved to asset kind 2 (for reference) | 6 | 2 | 0 |

## The rule for kind 2

The judge prompt counts a time that "contradicts a time fact of the World or an earlier entry". The inconsistencies asset (`work/wayfinder-maps/layout-trials/assets/story-inconsistencies.md`) puts a date conflict between two entries of the history in its kind 2 ("The dust is 30 years old, the crate is dated 18 years ago"). It keeps its kind 5 (no story clock) for the time now, the time that passed in the story, the time of day, and a time fact of the World. I use the asset's line:

- **Kind 2, confirmed:** the time of day, the time that passed in the story, or a conflict with a time fact of the World.
- **Moved to asset kind 2:** a date or duration of a past event that conflicts only with another date of the same event, when that other date was still in the history. These are out of scope here, but they are listed below because a list of fixed facts could prevent them.

## History windows (checked by arithmetic)

Both judges have the correct windows. For entry #k with N entries in the history, the call saw #k−N to #k−1.

| Story | Entries | Oldest entry seen |
|---|---|---|
| A | #525–#610 | #524 (no cut) |
| A | #611–#633 | #549 |
| A | #634–#656 | #569 |
| A | #657–#679 | #591 |
| A | #680–#704 | #615 |
| A | #705–#724 | #638 |
| B | #525–#599 | #524 (no cut) |
| B | #600–#618 | #548 |
| B | #619–#637 | #568 |
| B | #638–#658 | #588 |
| B | #659–#677 | #604 |
| B | #678–#697 | #625 |
| B | #698–#719 | #645 |
| B | #720–#725 | #665 |

## Confirmed instances: story A

### Kind 2: no story clock

| # | Later entry | Earlier entry or file | Quote, later | Quote, earlier | Runs |
|---|---|---|---|---|---|
| A-2.1 | #592 (echo in #593) | World `global_rules` | "The last payment cleared three weeks ago. After the funeral" | "Bernard Redmist died two weeks ago." | A1, A2 |
| A-2.2 | #602 (echo in #603) | World `description`, `global_rules` | "The Draught ... Nineteen-oh-five, by the calendar." With #600: "The trust established in the winter of the year of the Draught" | "The story takes place in the present day." "Bernard Redmist died two weeks ago." | A1, A2 |
| A-2.3 | #652 | #638 | "the grandfather clock chimed the quarter hour ... 'Half past six.'" | Marguerite on arrival: "Half six in the morning." Then 13 entries of talk, Tomas is woken and the chest is opened. | A1, A2 (both low) |
| A-2.4 | #710 | #685, #698, #706 | Halvorsen: "The Trust's doorkeeper came by at noon and told me a box came open" | #685: "A quarter past twelve. The steps were empty." The box opens at #698. #706: "Half past one". | A1, A2 |

Notes:
- A-2.2: #603 notices the problem ("Nearly a century before Tomas Hollis was born") but the story then treats the Draught as an event in living memory.
- A-2.3 is small. It is a confirmed slip, but the times are approximate.
- A-2.4 is also a knowledge fault (Halvorsen knows an event before it happened). It is counted once, here.

### Kind 3: who knows what

| # | Later entry | Character | Quote, later | Earlier entry and presence evidence | Runs |
|---|---|---|---|---|---|
| A-3.1 | #588 | Carla | "Marguerite won't make a fuss about the key either — she's had twelve years of keeping quiet and letting the money do the talking for her." "One o'clock at the Trust, then." | Marguerite says "I let the silence do the talking" (#582) and names the Islaport Trust (#584). Tomas sets "the Trust at one" (#586). All of this is inside the parlor. Carla "stopped at the gate herself" (#576), "Carla at the gate, who would hear none of this" (#579). On the step Julian says only "One o'clock, then" (#587), and he "sat down before he answered" her question. The State lines at #582–#584 and #586 list Carla, but the text puts her at the gate. | A1, A2 |
| A-3.2 | #592 (echo in #593) | Jezebel (her note) | Carla on the note: Halvorsen "he's read the Trust's rulebook more than once". The note (#593): "Halvorsen will take the Trust" | Marguerite names the Islaport Trust first at #584, in her parlor. Jezebel stays at the estate (#562: "I'll ring the bank from my office now"). The gatekeeper gives the note "the moment you were back" (#590), so Jezebel wrote it before Julian came back. The bank call is a possible source for the Trust, but not for Julian's plan to go to the Trust with a lawyer (raised in the carriage, #585, #590). Confidence: medium. | A2 |
| A-3.3 | #613 | Julian | "Lisette — you've served this house ten years." | No entry gives Lisette's years of service. Julian meets her at #594. The World names only "Lisette (personal maid)". The fact is in `lisette.json` ("For almost a decade"). Cause (inferred): card text in the prompt. Confidence: low, because an off-page briefing is possible. | A1, A2 |
| A-3.4 | #617 | Julian | "Louise Petit, I'll tell you where I'm going" | Julian meets Louise at #614–#615. Nobody says her surname. The World says only "Louise (personal chef)". The surname is in `louise.json`. Same cause and confidence as A-3.3. | A1 (A2 saw it and excluded it: "an employer can know a surname") |
| A-3.5 | #700 | Tomas | "Not Ardo Voss. The manager said the clerk's name was Ardo." | Fennick names "Ardo Voss" in the chandlery at #676. Tomas was not there: he left with Carla at #653, and Carla fetches him from his mother's house at #684. In #686–#699 nobody says "Ardo" or "Voss". Julian says only "Fennick's manager" (#699). #676 was in the history (oldest #615), so this is not kind 1. | A1, A2 |

Notes:
- A-3.3 and A-3.4 have the same cause. I treat them the same.

## Confirmed instances: story B

### Kind 2: no story clock

| # | Later entry | Earlier entry | Quote, later | Quote, earlier | Runs |
|---|---|---|---|---|---|
| B-2.1 | #613 (kept in #615, #625, #627) | #602, #607, #610, #611 | Rook: "Out among the loads, since the noon bell went." | #602: "it's not nine yet". #610: "West gate in twenty minutes." #611: "The drive down to the Halloran yard took less time than he'd allowed". #612: Julian greets Rook with "Morning". | B1 |

About three hours disappear. #602 and #607 were in the history (oldest #548), so this is a clock fault, not kind 1.

### Kind 3: who knows what

| # | Later entry | Character | Quote, later | Earlier entry and presence evidence | Runs |
|---|---|---|---|---|---|
| B-3.1 | #655 | Carla | "Not bonded goods going round a loop ... the cart's for show — a load on the road looks like a load in transit." | Cade tells only Julian at #629: "It's gone round the loop". Julian says "bonded goods going round a loop" to Jezebel in the study at #634. Carla was at the dock: sent at #600–#601, still away at #636 ("Carla's due back from the dock at ten — she's late"). Julian meets her again at #644–#645. In #644–#654 they talk only about the watchers. | B1 |
| B-3.2 | #671 | Carla | "Boy go? ... Nellis'll want a witness who'll swear the goods weren't moved" | At #657 Carla names Nellis but says "he's the last mouth on the quay I'd pick", and she proposes the boy as the courier "to the revenue office". Julian chooses Nellis at the fish sheds (#666), with Fen only. Carla stays at the cooperage (#659). She names Nellis before Julian speaks. Confidence: medium, because she can guess. | B1 |
| B-3.3 | #724 | Julian | "Louise Petit, the personal chef — or Lisette, the maid" | No entry of story B names Louise before #724. "Louise (personal chef)" is in the World, but "Petit" is only in `louise.json`. The same fault as A-3.4. The State lines of #714–#720 list `louise` (and others) as present, but she is not in the text. The judge's link between these lines and the card in the prompt is inferred, not checked. Confidence: low. | B1 |

## Moved to another kind: a date conflict inside the history (asset kind 2)

Story A, the date of the Draught. Every earlier date was still in the history of the later entry.

| Later entry | Conflict | Earlier entry (in the history) | Runs |
|---|---|---|---|
| #626 | Edmund "kept the accounts" of the Draught winter. Edmund died twelve years ago. | #602 "Nineteen-oh-five". #626 agrees with the World and with the later "thirteen years". The wrong side is #602. | A1, A2 |
| #644 | "That's what I told myself for thirty years." | #602 (1905), #626 | A1, A2 |
| #656 (echo in #658) | The deposit was "sent to Bernard thirteen years ago", in "the last week of the Draught". | #602 (1905), #644 (thirty years) | A1, A2 |
| #662 | The Exchange "Wound up in ninety-eight", but its stamp is on the thirteen-year-old letter. | #656, #658. With the World's "present day", thirteen years ago is after '98. | A1 (A2 put the '98 conflict inside its #665 finding) |
| #665 (echo in #666) | "an account from ninety-five". No entry gives '95. | #656, #662 | A1, A2 |
| #676 | The clerk "Dead this twenty years". | #658: "The clerk who took it in is dead." The deposit is thirteen years old. | A1 |
| #680 | "Twenty years I've wondered what became of the envelope". | #656, #658 (the envelope is thirteen years old) | A1, A2 |
| #694 | "Fifteen years of money moving quietly through one household". | #656 (thirteen years), #624 and #694 itself (the box was shut twelve years; Edmund died twelve years ago) | A1, A2 |

Story B:

| Later entry | Conflict | Earlier entry (in the history) | Runs |
|---|---|---|---|
| #552 | "Fifteen minutes until closing, she said." | #545: Jezebel said "a half-hour walk ... near closing". The figure fits the walk time. The fault is only the false "she said". | B1 (as kind 2) |

## Dropped findings

| Finding | Runs | Reason |
|---|---|---|
| A #690: "The codicil came by messenger" against #603, #604 (kind 1) | A1, A2 | False. The fact had already changed inside the history. At #607 Julian orders "You take the codicil to Tomas tonight", and #622 and #624 (in the window of #690, oldest #615) say Carla has the codicil. The promise of #604 was broken before the cut. A messenger contradicts no fact that the window holds. |
| A #690–#694: the Trust opens the box with no court order (kind 1) | A2 | False. No fact is contradicted. #584 says "a court order ... Unless the paper names you". The deputy accepts the ledger as the paper: "a purpose written plain enough that a court could read it without a lawyer" (#694), and Tomas holds the key of his father's box (#695). |
| A #722: Carla forgets the Halvorsen meeting (kind 3) | A1 | False. The text has a coherent reading: Carla proposes a second visit to put the new fund to Halvorsen ("Then Halvorsen's Friday just got bigger than a name. A fund wants trustees and a paper ... If we're to catch Halvorsen before his door shuts for good"). Julian answers "not today" (#723). |
| A #657: Julian says "your codicil" to Halvorsen (kind 3) | A1 | False. Loose wording. Julian knows the codicil, and he planned to tell Halvorsen (#603, in the history). Julian acts on nothing that he could not know. |
| B #715: the name "Pell" is used for a second man (kind 1) | B1 | False. No fact is contradicted. Two people can have one surname, and nothing in the story mixes up the board member Pell (#543) and the quay man. Keep it as a candidate for the fact list (Pell is a board member). |
| B #724: the note goes to the kitchen staff, not to Jezebel (kind 1) | B1 | False. An omission, not a contradiction. Julian gives a reason: not "the lady of the house". Nellis asked for a note "to the trust ... so nobody at the house hears a version of it from Vasch's side first", and the note goes to the house. |
| B #576: "an hour's sleep in me at most before six o'clock" (kind 2) | B1 | False. The story never states the hour of the ride home. #575 ("He'll have been and gone and come back twice by now") and #582 (Julian sleeps briefly, then wakes for half five) agree with a late hour. |
| B #552: "Fifteen minutes until closing, she said" (kind 2) | B1 | Another kind. See the asset kind 2 table above. |
| B #595: Pieterse says "Mr. Redmist" (kind 3) | B1 | False. The will of the Redmist heir is public news in the World ("the shocking will that left his fortune to an unknown grandson"). Pieterse also says "Word travels among clerks". Judge A2 excluded the same case in story A (#676, Fennick: "You'll be the grandson"). |

The "Not counted" lists of the three judges are correct, with one exception: A2 excluded A #617 (Louise Petit), but it has the same cause as A #613, which A2 counted. I count both.

## Observations

Neither story has a confirmed kind 1 fault. Every contradiction that the judges found had its earlier entry still in the history, or broke a fact that the history had already changed. Many confirmed faults sit before the first cut or just after it: A-3.1, A-2.1 and A-2.2 come before the first cut (#611), and B-2.1 comes 13 entries after the first cut (#600) with its anchors still in the history. So a memory of the dropped entries would fix none of the confirmed faults (inferred). A story clock that the engine keeps (the day, the time of day, and "now" relative to the World facts) could fix A-2.3, A-2.4 and B-2.1 (guessed). A record of who was present at each talk could fix A-3.1, A-3.2, A-3.5, B-3.1 and B-3.2 (guessed). The card leaks (A-3.3, A-3.4, B-3.3) need a rule that keeps card facts away from a Character who has not learned them. The Draught dates in the moved table need a list of fixed facts, because all of their anchors were still in the history. Each merged count is 5 or less, and kind 1 is 0 in both stories, so the check trial cannot show a change of any one of these kinds on its own (inferred).

## The fact list of story A

Made from the dropped entries of story A (step 4 of the ticket). These are 40 facts: 21 of kind fact, 11 of time and 8 of who knows what (4 "yes" and 4 "no"). Nobody checked each row against the story. The no-memory control of [Make the memory test from the stored stories](../issues/04-memory-test.md) drops each question that the model answers with no memory.

**Rows to treat with care (coordinator):**

- **F10:** #624 and #628 use "twelve years" inside the history window of the ask entry (#634). The fact can still be in the history there.
- **F21:** the story contradicts the World. "Three weeks ago, after the funeral" conflicts with "died two weeks ago". The answer is what #592 says, and the row is also the confirmed fault A-2.1.
- **F29:** the story contradicts itself on the year of the Draught (1905, thirteen, fifteen and thirty years). The question asks what Campion said, so it has one answer. A question on the true year has none.
- **F39 and F40:** the day count is inferred from "Friday gives us two days" (#604). The story names no other weekday.
- **The confirmed who-knows-what faults** (A-3.1 to A-3.5) are not yet rows. The memory test ticket can add them as questions. For example: "At #700, can Tomas know the clerk's name that Fennick gave?" The answer is no.

First cut takes effect at #611 (the call at #611 holds 62 entries and drops 25).
Oldest entry in the history at #724: #638 (the call at #724 holds 86 entries, and 724 - 86 = 638).
Derivation: for every entry #k, the oldest entry that the call could see is #(k - N), with N from the "history holds N entries" line. The number N falls at #611 (86 to 62), #634 (84 to 65), #657 (87 to 66), #680 (88 to 65) and #705 (89 to 67). So the oldest visible entry is #524 up to #610, #549 for #611-#633, #569 for #634-#656, #591 for #657-#679, #615 for #680-#704, and #638 for #705-#724. The five cuts drop 25, 20, 22, 24 and 23 entries, 114 in all (#524 to #637). Rule for the column "left the history at": the first entry #k whose oldest visible entry is newer than the last stating entry.

Notes on the story clock (worked out from the text; the story never names a weekday except Friday):
- Day 1 is the arrival day. It runs from #524 (morning, front gates) to #634 (night; #622 "the evening had gone fully dark", Tomas comes to the house at night).
- Day 2 starts at #635 ("the dark suit from the night before", "Half past five"). #724 is the evening of day 2 ("Dinner at seven").
- #604 (day 1, failing light): Campion says "Friday", Carla says "Friday gives us two days". So day 1 is a Wednesday, day 2 is a Thursday, and the handoff is the day after #724 (inferred).
- Kind "who-knows": the State lines are wrong in places. For example, the State lines of #624-#630 list Gabriella as present, but the text of these entries never names her.

| id | kind | fact | stated at | left the history at | ask at | question | answer | who-knows detail | later use |
|---|---|---|---|---|---|---|---|---|---|
| F01 | fact | Julian came to Islaport by train. He says the odd sign-in came "after I booked the train". | #529, #530 | #611 | #611 | How did Julian travel to Islaport? | By train. | | none |
| F02 | time | Someone signed in to Julian's account three days before he reached the gates. | #530, #531, #547, #549, #557 | #634 | #634 | How many days before Julian reached the front gates did the unknown sign-in to his account happen? | Three days. | | none. The whole login thread is never used again after #557. |
| F03 | fact | The vault room of the master-key handoff is off the old dispensary. | #544, #545 | #611 | #611 | Where is the vault room in which the master key is handed over, according to Jezebel? | Off the old dispensary. | | #694-#696 (a different vault room, at the Islaport Trust, can confuse a model) |
| F04 | fact | A retired locksmith had the sealed slip in his hand four years ago, when he serviced the vault door. | #546, #547, #549 | #634 | #640 | What was the trade of the outsider who once held the sealed slip of the master key? | A locksmith (retired, four years ago). | | none |
| F05 | fact | Jezebel first names Marguerite as Bernard's nurse who sat with him for his last two years. | #548 | #611 | #611 | For how long did Marguerite sit with Bernard as his nurse before the end, according to Jezebel? | His last two years. | | #556 (Marguerite is Edmund's widow and was a hospital nurse), #562, #580. The roles do not match. |
| F06 | fact | The bottom drawer of Bernard's private desk in the study was empty. The log was gone. | #550, #551, #552, #555, #556, #557, #560 | #634 | #634 | What did Jezebel find in the bottom drawer of Bernard's desk? | Nothing. The drawer was empty (the log was gone). | | none. The missing log is never looked for again. |
| F07 | fact | Gabriella wore a lavender gown with pearls when she met Julian in the hall. | #538, #540 | #611 | #611 | What colour was Gabriella's gown when she met Julian in the entrance hall? | Lavender. | | none |
| F08 | fact | Gabriella said she would be in the west parlor that afternoon. | #540 | #611 | #611 | Where did Gabriella say she would be that afternoon? | In the west parlor. | | none. The afternoon passes without her. |
| F09 | fact | Gabriella warned of a draught in the east wing. Carla said there is none and that it is a warning-off. Julian said he wanted to know whether there is a draught. | #540, #541 | #611 | #611 | What warning did Gabriella give about the east wing? | A draught that nobody has fixed (Carla said there is none). | | none. Julian never checks. |
| F10 | time | Edmund, Bernard's brother, died twelve years ago. | #548, #556 | #634 | #634 | How many years ago did Bernard's brother Edmund die? | Twelve years. | | #624, #628, #694, #698 use "twelve years" for the strongbox and the ledger. #644 says "thirty years". |
| F11 | who-knows | Gabriella does not know about the odd sign-in on Julian's phone. | #534, #541, #547, #549, #557 | #634 | #634 | At #634, can Gabriella know about the odd sign-in on Julian's phone? | No. | Gabriella. She is in the text only at #538-#540. Carla says the sign-in stays "ours" (#534), and Julian says "Nothing about the login" (#541). Nobody tells her afterwards. | none |
| F12 | who-knows | Jezebel knows about the odd sign-in. Julian told her himself. | #534, #547, #549 | #634 | #634 | At #634, does Jezebel know about the odd sign-in on Julian's phone? | Yes. | Jezebel. At #534 Carla says "Jezebel doesn't know yet". At #547 Julian tells her in her office: "Someone signed in to my account three days ago." | none |
| F13 | fact | Jezebel found the brass key with the tag "Hollis" under the flat green cushion of the reading chair in the library. | #552, #554, #555, #581 | #657 | #657 | Where did Jezebel find the brass key with the name tag? | Under the cushion of Bernard's reading chair in the library. | | none |
| F14 | who-knows | Marguerite Hollis knows that the key lay under Bernard's cushion. Julian told her. | #552, #581 | #657 | #680 | At #680, does Marguerite Hollis know that the key with her name was found under a cushion? | Yes. | Marguerite. At #581 Julian puts the key on her table and says: "And this, under his cushion, with your name on it." She is in the room (#580-#584). | none |
| F15 | who-knows | Marguerite Hollis does not know that Bernard's drawer was found empty. | #556, #557, #560, #581 | #634 | #657 | At #657, can Marguerite Hollis know that Bernard's private drawer was found empty? | No. | Marguerite. Jezebel advises not to mention the drawer (#556). Julian agrees (#557). Carla repeats it (#560). In #581 Julian names the key, the cushion and the ledger note, but not the drawer. | none |
| F16 | fact | Bernard wrote a note beside the last quarterly payment to the Hollis house: "For the boy's schooling." | #558, #581 | #657 | #657 | What note did Bernard write in the ledger beside the last payment to the Hollis house? | "For the boy's schooling." | | none |
| F17 | fact | Jezebel said the quarterly draughts to Marguerite went through Marchand Mutual. | #560, #561, #573, #583, #584, #601, #602 | #680 | #680 | Which bank did Jezebel name as the one that paid the quarterly draughts to Marguerite? | Marchand Mutual. | | none |
| F18 | fact | The gatekeeper has a cousin on the east side. Carla and Julian fear that word will travel. | #572, #573, #574, #587, #607 | #680 | #680 | Which member of the staff did Carla say has a cousin on the east side? | The gatekeeper. | | none |
| F19 | fact | Tomas set the Trust meeting for the afternoon because the clerk's window opens at nine and a queue of shopkeepers fills the morning. | #586 | #657 | #680 | Why did Tomas refuse a morning meeting at the Islaport Trust? | The clerk's window opens at nine and a queue of shopkeepers fills the morning. | | none |
| F20 | fact | Jezebel (by note) recommended the lawyer Halvorsen. She said she would ring him that night. | #590, #592, #593 | #680 | #680 | Who recommended the lawyer Halvorsen to Julian? | Jezebel (in her note). | | none |
| F21 | time | The last quarterly payment cleared three weeks ago, after the funeral and before the will was read. | #592, #593 | #680 | #680 | How long before the day of the story did the last quarterly payment to the Hollis house clear? | Three weeks ago. | | none. The World file says Bernard died two weeks ago, so "three weeks, after the funeral" is itself a clash. |
| F22 | fact | Lisette took the message that the solicitor Campion telephoned and asked for a call back before evening. | #594, #595, #596 | #680 | #680 | Who took the telephone message from the solicitor Campion and gave it to Julian? | Lisette. | | none |
| F23 | who-knows | Lisette knows that Campion telephoned. She took the message herself. | #594, #595, #596 | #680 | #690 | At #690, does Lisette know that the solicitor Campion telephoned and wants a call back? | Yes. | Lisette. At #594 she meets Julian at the door and hands him the card: "Mr. Campion rang for you, sir." At #596 she adds that he takes the receiver off after six. | none |
| F24 | fact | Carla had the sitting room with the fire sorted that morning. Nobody had used the room since the spring. | #596 | #680 | #680 | Who had the sitting room prepared for Julian with a fire lit? | Carla. | | none. The State line gives the Room as `library`. |
| F25 | fact | Campion's codicil is six lines long. | #598, #599, #600, #605 | #680 | #680 | How many lines long is Bernard's codicil? | Six. | | #690 (the deputy has read the codicil) |
| F26 | time | Campion had held the codicil since three o'clock that afternoon. | #600, #604 | #680 | #680 | Since what hour had Campion held the codicil when he spoke to Julian? | Since three o'clock. | | none |
| F27 | fact | Campion said the codicil has not left his desk. | #598, #604 | #680 | #690 | Where did Campion say the codicil is kept? | On his desk. It has not left it. | | #606-#608 (Carla takes a "note" to Tomas), #622 and #624 (Carla "still had" the codicil), #690 (the codicil "came by messenger"). |
| F28 | who-knows | Campion cannot name the beneficiaries of Bernard's trust. | #600, #601, #602 | #680 | #680 | At #680, can Campion name the beneficiaries of the trust that the codicil mentions? | No. | Campion. The codicil says the beneficiaries are "known only to me and to my banker" (#600). Campion says he knew nothing of the trust until he read the codicil that afternoon. | none |
| F29 | time | Campion said the winter of the Draught was in 1905 ("Nineteen-oh-five, by the calendar"). | #602, #603 | #680 | #680 | In which year was the winter of the Draught, according to Campion? | 1905. | | #656, #658, #663, #713 and #714 say "thirteen years" and #694 says "fifteen years" for the same events. #644 says "thirty years". |
| F30 | time | The master-key handoff is at ten in the morning, on a Friday, before the offices open. | #604 | #680 | #690 | At what hour of the morning is the master-key handoff? | At ten o'clock. | | none. The handoff is never mentioned again. "Friday" returns at #662-#664 and #714 only for Halvorsen's work. |
| F31 | time | Carla said Julian must leave the house by half past seven for his eight o'clock meeting with Halvorsen. | #608, #609, #611 | #680 | #680 | By what hour did Carla say Julian must leave the house for Halvorsen's office? | By half past seven. | | #633-#634 (the plan changes to a Marchand Street visit first, at half past six) |
| F32 | fact | Lisette said the knocking sound in the study on foggy nights is the wind, because the west window does not shut true. | #614 | #680 | #680 | What did Lisette say causes the knocking sound in the study on foggy nights? | The wind (the study's west window does not shut true). | | none |
| F33 | time | Lisette said the east corridor clock runs nine minutes fast. | #614, #635 | #705 | #705 | By how many minutes does the east corridor clock run fast? | Nine minutes. | | none |
| F34 | fact | Louise said she would cook a fish stew with mussels and a little cream (her mother's dish) if nobody were watching. | #616 | #705 | #705 | What dish did Louise say she would make "if nobody were watching"? | A fish stew with mussels and a little cream. | | #716-#722 (a mutton stew at the Anchor and Bell can confuse a model). #724 (dinner at seven, no dish named). |
| F35 | time | Louise keeps a pot going in the kitchen from six in the morning until ten at night. | #616, #618, #619 | #705 | #705 | Between which hours does Louise keep a pot going in the kitchen? | From six in the morning until ten at night. | | #724 (dinner at seven) |
| F36 | who-knows | Tomas knows what the codicil says. Carla told him on the evening of day 1. | #605, #606, #607, #608, #622, #624 | #705 | #705 | At #705, does Tomas know what Bernard's codicil says? | Yes. | Tomas. Julian says Tomas must hear the six lines (#605). Carla takes the codicil to him (#606-#608). Tomas says "She told me what the codicil says" (#624). Note: #690 has the deputy say the codicil "came by messenger". | none |
| F37 | fact | Tomas's father told him that Bernard came to the house one night in the Draught winter with a young doctor from the hospital and a list. | #626 | #705 | #705 | Who came with Bernard to the Hollis house one night in the Draught winter, according to Tomas's father? | A young doctor from the hospital. | | none |
| F38 | who-knows | Gabriella cannot know what Tomas said about his father's account of the Draught. She is not in the library. | #623-#630 | #705 | #705 | At #705, can Gabriella know what Tomas told Julian in the library about his father's account of the Draught? | No. | Gabriella. The State lines of #624-#630 list her as present, but the text of #623-#630 names only Julian, Carla and Tomas. | none |
| F39 | time | Julian arrived on day 1. At #724 it is the evening of day 2. One night has passed. | #524-#634 (day 1), #635 (start of day 2) | #705 | #724 | How many nights has Julian slept at the estate by the evening of #724? | One night. | | none |
| F40 | time | The master-key handoff is the day after #724. | #604, #635 | #705 | #724 | At #724, how many days remain until the master-key handoff? | One day (it is Friday morning, and #724 is the Thursday evening). | | none. Worked out as in the notes: #604 on day 1 says Friday is "two days" away, so day 1 is a Wednesday. This answer is inferred. |


### Facts left out because the World, map, card or Persona files state them

- Bernard died two weeks ago; Gabriella lives in the guest wing; Carla is an ex-soldier and Julian's bodyguard; Jezebel manages the finances; Lisette and Louise serve Julian (World global rules).
- Carla wears sunglasses and suits; Jezebel has orange eyes and curly hair; Gabriella has purple eyes and a dyed blonde look (cards). Used in #528, #542, #544, #716.
- Louise's mother ran a restaurant (card). Louise's mention of it at #616 is left out for that reason. Only the dish is used (the fish stew row).
- Lisette served the house for about ten years (card: "almost a decade"; #613).
- Julian was raised by a single mother (Persona). His line "told anyone but my mother" (#531) is left out.
- Room layout: Jezebel's office in the east, the kitchen west of the hall, the library north of the hall (map). The sitting room at #596 is not on the map, so the sitting-room row stays in.
