# Find where evidence of a working feature lands today

Type: research (AFK)
Status: open
Blocked by: —

## Question

When someone sees a Chronicler feature work, where does the evidence end today? What place and form would keep it next to the behaviour?

## Context

- **Covers** C-12 (three proof states and a dated observed record), C-13 (a proof limit beside each check) and C-14 (known gaps beside the behaviour). See `assets/candidates.md`, theme C, and `assets/research-04-feature-map.md`.
- **Hex (known).** Hex keeps three states apart: the behaviour is built, a check exists, and someone saw it work. The third state is a dated paragraph that names the machine, the build and what was not exercised. `docs/features/` holds 17 "Proof limit" sentences and has "Gaps And Constraints" sections.
- **Chronicler (known).** No doc in `docs/` contains "observed", "manually verified" or "verified by hand" (K-26). A green `python build.py` is the only proof. Known gaps of a feature end in closed tickets in `.scratch/_closed/`.
- **The value is not shown.** The owner is the only person who uses Chronicler, and an AI writes and reads the docs. So the asset must say who can observe a feature: the owner, or an agent through `chronicler-ui-investigator`. It must also say who would read the record.
- **Do.**
  1. Find where evidence lands today. Look at `chronicler-ui-investigator` shots and DOM captures, `scripts/trial.py` reading packs, `tmp/`, ticket answers in `.scratch/` and `.scratch/_closed/`, and plan Status lines. Give counts and examples.
  2. For one capability (Autoplay is suggested), find the known gaps that closed tickets record. Say whether a reader of `docs/specs/autoplay.md` would find them.
  3. In the asset only, draft a proof record, proof-limit lines and a gaps list for that spec, in two forms: inside the spec, and in a separate file next to it. Check each form against `scripts/validate_feature_spec.py` and `scripts/validate_docs.py`.
- **What already exists.** `chronicler-ui-investigator` saves shots in `tmp/` and tells its user to write findings to the ticket. `scripts/trial.py` writes one reading pack for each repeat, under `tmp/<…>/trial-N`.
- **Failure modes.** `tmp/` is git-ignored and can be cleared, so a count there is a lower bound. State it.
- **Not in scope.** Do not edit `docs/specs/`. The doc layout waits for [Research how agents use the current docs and plans](./10-research-agent-use-of-docs-and-plans.md), so the draft must fit today's specs.
- **Output.** Write `assets/research-12-evidence.md`. The `## Answer` gives the gist, a keep or drop recommendation for each of C-12, C-13 and C-14, and at most 2 questions for ticket 16.
- **When done.** Add `## Answer` and set `Status: resolved`.
