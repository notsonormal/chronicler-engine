# Two checks: story counts in the trial comparison, and the schemas against the Rust types

Type: task
Status: open
Blocked by: —

## Question

Add two checks that compute a fact (owner, 2026-11-10, D14 b and c of [the decisions](../decisions.md)). Neither needs a model call.

### 1. Story counts in `scripts/trial.py compare` (D14 b)

- **The gap (read it).** A trial pack records cost, speed and failures. It records no fact about the story. So the comparison table cannot show that a story got worse.
- **What exists.** [trial_truth_counts.py](../assets/trial_truth_counts.py) counts facts from a stored pack with fixed text rules, and [its output](../assets/trial-truth-counts.txt) has one row for each of seven stored trials. The counts are: Rooms used, Rooms added, Room changes, state changes on an Input, Narrations that name a place off the estate, Narrations with no authored Character, uses of invented titled names, modern and period words, repeated 5-grams, and the median words of a Narration and of an Input.
- **The change.** Move the counts that do not depend on one World into `scripts/trial.py`, as columns of `compare_packs` (`scripts/trial.py:1376`). The script was written for the audit: it has word lists for `redmist_estate`. A count that needs such a list reads the list from the test setup, or it is left out. Name each count that you leave out in `## Answer`.
- **A warning that the table must carry.** Two trials with the same prompts (`lt01` and `lt10`) gave 26 against 135 uses of invented names. So a difference between two single runs is not a result. Print that sentence with the columns. The repeat of one setup needs model credit and waits (D4).
- **The failing test first.** A case in `scripts/tests/test_trial.py` with a small stored story. Also add the case that the file lacks today: a read of a Swipe state (`scene.npcs_in_area`, `scripts/trial.py:971`).

### 2. Each JSON schema agrees with its Rust type (D14 c)

- **The gap (read it).** `scripts/validate_data.py` checks a data file against its JSON schema. The boot reads the same file into a Rust type. The two disagree in three known places:
  - `data/schemas/character.schema.json` does not require `id`. `CharacterCard.id` is required, so the boot drops such a file with one warning.
  - `data/schemas/map.schema.json` accepts any key for an exit. The Rust type allows six directions, so a bad key passes the gate and stops the load of the Worlds.
  - A Persona file is checked against the Character schema.
- **A limit of the decision as the owner read it.** The question said "one test that loads each shipped data file through its JSON schema and through its Rust type". That test passes today, because every shipped file is valid for both. So it does not find the three disagreements. The check needs files that one side refuses.
- **The proposed design.** One Rust test. For each shipped data file and each field that its schema does not list as required, remove the field and read the result into the Rust type. When the Rust type refuses it, the test fails and names the schema and the field. This needs no schema library in Rust: the test reads the `required` list of the schema as plain JSON. A second case does the same for the keys of an exit.
- **The schemas change with the test.** The test fails on the three places above. Correct the schemas so that it passes.
- If the design does not hold, stop and say so in `## Answer`.

## Rules for every change

- Write the failing test first. Then make the change.
- Line numbers are from commit `e8d6512a`. Find the site again by its name before you edit.
- One change that turns out not to be local stops. Say so in `## Answer`, and do the other changes.
- Use `python build.py clippy` and `python build.py test-pattern` during the work. Run `python build.py` once at the end.
- Do not commit.

## Not in scope

- A generator that writes the schemas from the Rust types.
- An LLM judge, and any run of a trial.
- The Room name stored with a story entry (D14 a: no).

## What already exists

- Finding 4 of [the product report](../assets/product-content-and-release.md) and the protocol rows of [the lenses report](../assets/code-lenses.md).
- `.scratch/layout-trials/map.md`: the trial method of the owner.
