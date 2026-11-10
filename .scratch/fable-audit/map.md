# Map: the audit of 2026-11-10

Labels: wayfinder:map

## Destination

Each change that the owner accepted in the audit of 2026-11-10 is in place and `python build.py` passes. The decisions that the owner did not accept stay recorded, so that a later audit does not report them as new.

## Notes

- **Origin.** A whole-product audit at commit `b0c8fe41`: five reports, a list of 32 questions, and a grilling session of five rounds on those questions (owner, 2026-11-10).
- **The decisions** are in [decisions.md](decisions.md), D1 to D21. A ticket names the decisions that it puts in place.
- **The reports** are in `assets/`. Start with [the questions](assets/questions-for-the-owner.md). The raw evidence is in `assets/local-only/evidence/`, which git ignores.
- **How a finding was proved.** A report marks each finding "ran it" (on a probe server with a scripted model), "read it" (in the source), "measured" (in stored trials) or "scout only". Trust them in that order.
- **Autoplay only.** Every story finding comes from Autoplay, where the model writes the Inputs of the Persona. No person played a session for the audit (D16).
- **Shared ground with the invariant audit.** [That map](../invariant-audit/map.md) owns its 53 findings. The owner let four of them go ahead here (D7 e). Ticket 02 names them and their tickets there.
- **Terms.** `CONTEXT.md` holds the terms. The session changed no term.

## Decisions so far

- **Stays as it is.** A place off the map (D1). The meaning of "Ready" (D3). The meaning of a repeating Trigger (D10). A changed data file does not reach an existing database, and a Character form in the dashboard comes later (D11). The Room name is not stored with a story entry (D14 a). No message after a hard stop, and no mark in the Games tab for a Game whose Job still runs (D8 b, c). The database stays in the folder of the binary (D15). The LLM messages stay a debug record of 50 rows.
- **Waits for a release.** The licence, streaming, the listen address (D2).
- **Waits for model credit.** `example_dialogue` in the prompt, the analysis of an Impersonate Input, the present day, three runs of one setup, the value of each mechanism (D4).
- **Not decided.** Whether the Redmist World gives the story a plot (D12).
- **Accepted work.** Six tickets:
  1. [Close the key leak and the links in the story log](issues/01-task-security.md) (D6, D17)
  2. [Engine fixes](issues/02-task-engine-fixes.md) (D7, D13, D17)
  3. [The notice for a cut Narration and the Retry of the opening](issues/03-task-story-entries.md) (D8 a, D9)
  4. [Two checks](issues/04-task-checks.md) (D14 b, c)
  5. [Correct the wrong text](issues/05-task-text.md) (D17, D19)
  6. [Engine fixes, second group](issues/06-task-engine-fixes-second-group.md) (D19, D20, D21)

## Not yet specified

- A Character form in the dashboard (D11). One note for that effort is in `decisions.md`: a Swipe state stores whole Character cards, so a Retry would write with the old card.

## Out of scope

- Every row under "Stays as it is", "Waits" and "Not decided" above.
- The findings that no question asked: rows N15 to N18 of [the list](assets/findings-not-carried.md), and the three parts that ticket 06 names as not in scope. The owner has not decided on them.
