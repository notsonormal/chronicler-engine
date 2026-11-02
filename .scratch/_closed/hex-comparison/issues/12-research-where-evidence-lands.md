# Find where evidence of a working feature lands today

Type: research (AFK)
Status: resolved
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

## Answer

Asset: [research-12-evidence.md](../assets/research-12-evidence.md).

**Gist (all known unless the asset says inferred).**

- Evidence that someone saw a feature work lands in ticket answers (60 ticket files mention a shot), in untracked `tmp/` (754 PNG files, a lower bound) and in 6 trial reading packs. `docs/` has 0 files with "observed", "manually verified", "verified by hand" or "saw it".
- The link rots. Tracked notes cite 207 concrete `tmp/` paths, and 73 (35%) do not exist now. A trial pack names no engine commit.
- Autoplay has 3 open gaps in `.scratch/autoplay-follow-ups/` and 2 more only in a plan that commit `3e42a26f` deleted. `docs/specs/autoplay.md` shows none. All 32 tags of its 25 scenarios run on `ScriptedModel`. No real-provider run is cited from the spec.
- An agent can observe a feature (`chronicler-ui-investigator`, `scripts/trial.py`). The reader that matters is a later agent, which reads the spec first.
- Draft of a proof limit, an observed record and a gaps list: 21 lines, in two forms (inside the spec, and a file next to it). Both pass `validate_feature_spec.py` (561 declared, 561 covered), `validate_docs.py` (specs are EXCLUDED, routes checked) and Vale. One trap: a heading "Scenario N.M" in a separate file in `docs/specs/` counts as a scenario with no test. No check keeps a record true.

**Recommendation.**

| Practice | Recommendation | Why |
|---|---|---|
| C-14 known gaps beside the behavior | Keep | Cheapest of the three. One sentence and a ticket link per gap. Autoplay shows the gap: the spec lists none. |
| C-13 proof limit beside each check | Keep, narrow | One list for each spec, one line for each stand-in that hides something (`ScriptedModel`, the stop moments, the browser tier). No line per scenario. |
| C-12 three states and a dated observed record | Keep, narrow | Only for what the gate cannot show: a trial on a real provider, or a look at a shot. One paragraph for each spec, with date, commit and what was not exercised. A newer record replaces it. Side task of about one session: write the commit into `run.json` of `scripts/trial.py`. |

**Questions for ticket 16.**

1. Where does the record sit: inside the spec, or in a file next to it? Recommendation: inside, because agents read the spec first. It needs one added row in `docs/AGENTS.md` "Where a fact lives".
2. Does the owner accept an "Observed" paragraph that an agent writes from a trial or a shot, with no gate check that it stays true? If no, drop C-12 and keep C-13 and C-14.
