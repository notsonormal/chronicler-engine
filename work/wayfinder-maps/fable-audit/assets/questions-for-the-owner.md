# Questions for the owner: one list for all audit reports

Date: 2026-11-10. Commit: `b0c8fe41`.

The five documents in this folder hold 36 questions. Several ask the same thing from two sides. This list has each decision once, with my recommendation and the report that holds the evidence. I stopped for none of them. Each report says what it assumes.

## How to read this list

- **Report codes.** E = `engine-defects-and-risks.md`. S = `story-damage-and-false-screen.md`. P = `product-content-and-release.md`. L = `code-lenses.md`. C = `audit-catalogue.md`. "E 4" is question 4 of report E.
- **Order.** Each group starts with the decision that I judge to matter most. The order is my judgement.
- **A short answer is enough.** "Yes", "no" or "later" for each row lets an agent start. Group 1 has the rows where a "yes" needs no design work.

## 1. Yes-or-no decisions with a small fix behind them

| # | Decision | My recommendation | Source |
|---|---|---|---|
| 1 | When one part of the screen fails, do the other parts still render? | Yes. Today a World with no Rooms blanks the whole dashboard, and the Games and Worlds tabs are the way out | S 1 |
| 2 | Does a form refuse a value that it cannot read, with 400? | Yes. Today the text-check mode and the World posture fall back to a default with no message | L 2 |
| 3 | Is story text escaped where it enters a prompt? | Yes. It is one place | S 7 |
| 4 | Does the binary listen on `127.0.0.1` by default? | Yes. Keep `0.0.0.0` only in `build.py run` | E 3 |
| 5 | May a player Retry the opening Narration? | No. Hide Retry on the opening. Today the Retry replaces the authored opening and moves the scene | S 4, E 12 |
| 6 | Does a Narration that the model cut get a notice? | Yes, a notice on the Swipe, not a failure line | E 2 |
| 7 | After a hard stop, does the Game say that work was lost? | Yes: a failure line at boot for a story that ends on an Input | S 2 |
| 8 | Do `maker` and `currently_meeting` go? | Yes. Both are written and never read | L 3 |
| 9 | May the four one-line fixes of the invariant audit go ahead outside the ticket order? | Yes, one commit each with its breaking test | E 10 |
| 10 | Does the Games tab mark a Game whose Job still runs? | Yes | S 6 |
| 11 | Does `example_dialogue` reach the prompt? | Yes, two lines for each present Character. One trial shows the effect | P 1 |

## 2. Decisions about what the engine is

These change the direction. Each needs your judgement more than my evidence.

| # | Decision | My recommendation | Source |
|---|---|---|---|
| 12 | What is a place off the map? In the trials about 40 % of the Narrations play where the engine has no Room | An added Room with a name, found again by name. No picture, no description | E 5 |
| 13 | May the model make a Character that the World does not have? | Decide it with the card-change ticket of the long-term-memory map. Until then, count them in the trial pack | E 6 |
| 14 | Is the Redmist World meant to carry a plot, or to be an open stage? | Your choice. With no plot given, every trial invents the same one, and three of five Characters fade | P 2 |
| 15 | Why does an Impersonate Input get an analysis? | Remove it. It moves the Room before the narrator answers, and it is one of four calls in each Cycle | E 4 |
| 16 | Is the present day a rule of this World? | Yes, stated in the block after the history | E 11 |
| 17 | How often may a repeating Trigger fire, and may it fire while its Character is absent? | At most once for each entry of the Character into the scene | S 3 |
| 18 | Where does the health of a Game live? | One record for each Game that every Job failure writes, the Options failure and the kept-present fallback included | L 1, S 5 |
| 19 | Does a data file or the database win for a seeded World? | The file wins, and the dashboard shows that World as read-only | E 1 |
| 20 | Is the Room name stored with the story entry? | Yes, at write time | L 5 |
| 21 | Does `llm_messages` stay a debug record of 50 rows? | Change it when the link from a story entry to its call is built: a Game and a Swipe on each row | E 9 |

## 3. Decisions about the release

| # | Decision | My recommendation | Source |
|---|---|---|---|
| 22 | Is the first release a local install for one person? | If yes, the blocking list is short: the README, the bind address, a licence, a player guide | P 6 |
| 23 | Which licence, and what is the source of the Character cards and the art? | Only you know. It blocks a release and nothing else | P 5 |
| 24 | Is streaming wanted before the release? | Yes, after the security and Ctrl-C findings. Start with its own map | E 7 |
| 25 | Where does the database live after a release? | A `--data-dir` flag with a default outside the build output | E 8 |
| 26 | Delete the concepts with no user, or ship a World that uses them? | The World first. It turns "no user" into a fact for each concept | P 4 |

## 4. Decisions about the method

| # | Decision | My recommendation | Source |
|---|---|---|---|
| 27 | May an audit spend model credit, and how much? | Yes, with a limit that you name. The audit of each mechanism's value (C1) needs 9 to 12 trials: three or four variants, three runs each | C 1, P section 8 |
| 28 | Does the trial pack count story facts, and does one setup run three times first? | Yes to both. Two runs of one setup differ more than two layouts did | P 3 |
| 29 | Are the schemas generated from the Rust types? | Later. First one test that loads the shipped data through both | L 4 |
| 30 | Where do audit reports live? `build.py` deletes entries under `tmp/` after 30 days | A folder under `.scratch/` for a report that a ticket cites | C 5 |
| 31 | Do the findings become tickets? | Your decision for each report. 111 tickets are open and 55 wait for you, so I filed none | C 6 |
| 32 | For an audit of real play: do you play the session? | You play 30 Actions. I read the result | C 2 |

## 5. If you answer only three

1. **Row 12**, the place off the map. It decides the largest story finding, and rows 13 and 14 depend on it.
2. **Row 27**, the credit. It opens the one audit that can say whether the Quantifier, the Rooms and the Characters present make the story better.
3. **Group 1 as a block.** Eleven small fixes that need only "yes".
