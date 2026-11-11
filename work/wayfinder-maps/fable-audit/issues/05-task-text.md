# Correct the wrong text: the README, eight sentences of the development environment, and two texts of the product

Type: task
Status: open
Blocked by: —

## Question

Correct the text below (owner, 2026-11-10, D17 and D19 of [the decisions](../decisions.md)). Each row is a sentence or a file that says something false today. No row adds a rule.

### 1. `README.md` names the API key

- **The fault (ran it as a stranger).** The README names no API key and no `.env` file. The first Input of a new user fails with "The engine failed with an internal error."
- **The change.** Say which key the shipped `data/settings.json` needs, where it goes, and how to choose another Connection in the Settings tab. Do not read or print a `.env` file: the permission rules deny it.

### 2. Eight rows of the environment report

From section 3 of [the environment report](../assets/development-environment.md). Row 1 of that table (three role files with one name) was corrected in commit `068b682f`.

| Row | What is wrong | The correction |
|---|---|---|
| 2 | `AGENTS.md` says that the permission restrictions "exist to prevent the agent from touching git without supervision". No permission rule names git since `5142f1da` and `19712ddd` | Say what the rules deny today. The rule "no commit without approval" stays as a sentence |
| 3 | The warm costs in the `AGENTS.md` table are two to three times too low. Medians of the green runs in `logs/build_history.txt`: full gate 350 s, `--no-browser` 111 s, `integration` 59 s, `guardrails` 27 s, `clippy` 18 s, `browser` 234 s | Measure the medians again on the day of the change and write them. Or delete the column and name the journal |
| 4 | `CODING_STANDARDS.md` requires `//! [DOC: …]` on line 1 of every source file. `src/lib.rs` has `// [DOC: …]`, and `src/main.rs` has none. The guardrail exempts both | Name the two exemptions in the rule |
| 5 | The generated `guardrails.md` says "Warns" for three rules (long comment runs, separator comments, single-letter variables). Each one fails the build | Change the doc comment of the three checks in `tests/infrastructure/guardrails/style.rs`. The generator copies it |
| 6 | The `implement-spec` skill tells the lead to merge with a "merger subagent" and to commit on an integration branch. No such role exists, and `AGENTS.md` forbids a commit without approval | Correct the skill to the roles that the delegate tool has |
| 7 | The `thermo-nuclear-code-quality-review` skill says "go for it" about a restructure. `AGENTS.md` says that a review does not change code | Make the skill agree with `AGENTS.md` |
| 8 | `.pi/plan-mode.json` lists tools that this harness does not register | Check plan mode once. Then correct the list. The audit did not test what plan mode does with an unknown tool |
| 9 | The `wait-what` skill restates the prose rules that `.pi/APPEND_SYSTEM.md` appends to every turn. That file changed in `e8d6512a`, so the two now differ | Replace the copy with a pointer |

Row 9 of the report also names `chronicler-after-plan-workflow-plus-review`. That skill says that it is a copy on purpose, so it stays.

### 3. Two texts of the product (D19, row N13)

- **The comment on a role failure.** The comment at `src/adapters/driving/http/builders/headers.rs:37` says that a role failure lasts until the next call of that role succeeds. Ran it: the banner cleared 26 Actions after a failed Options call, with no Options call between. Role health reads the newest 50 LLM messages, so a failure leaves when its row leaves. D3 keeps this behaviour. Make the comment say what the code does.
- **The examples of an Options preset.** `data/prompt_presets/options/roadway_domains.json:5` gives fantasy examples ("forest creatures", "corrupt city guard"). The one shipped World is a modern estate. Replace the examples with ones that fit no single era. This is preset text, and the audit did not judge the Options presets in a trial: change the examples only. A database of today keeps its stored preset, so the change reaches only a new database.

## Rules for every change

- Read the `writing-for-agents` skill before you change `AGENTS.md` or a skill.
- Check each row against the file first. If the file no longer says it, skip the row and say so in `## Answer`.
- Run `python build.py validate-docs` and `python build.py vale`. Row 5 changes Rust, so it also needs `python build.py guardrails`, and the full gate regenerates `guardrails.md`.
- Do not commit.

## Not in scope

- A new rule in `AGENTS.md` or `CODING_STANDARDS.md`.
- A player guide (D2: waits for a release).
- The six optional rows of section 6 of the environment report.

## What already exists

- Finding 6 of [the product report](../assets/product-content-and-release.md): the cold start.
- `work/wayfinder-maps/retro-issue-bucket/audit.md`: the owner's record that a correction of wrong text is a fix that lasts.
