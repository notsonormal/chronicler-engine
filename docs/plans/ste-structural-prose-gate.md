# Plan: STE structural prose rules in the gate, and a passive-voice review check

**Date:** 2026-10-31
**Status:** Draft — not started
**Goal:** The gate fails on three STE faults in the docs in scope: a sentence of more than 25 words, a paragraph of more than six sentences, and a semicolon. `document-review` asks the reviewer to name the actor of each passive verb.

## Origin

A trial in this session compared two ASD-STE100 tools and Vale rules against the ASD-STE100 Issue 9 standard. The standard is at `tmp/ASD-STE100_ISSUE9.pdf`, and its text extraction is at `tmp/ste_trial/ste9.txt`. ASD gave free access on request. Its copyright does not allow us to reproduce it, so neither file goes into the repo.

The trial files are in `tmp/ste_trial/`:

- `run_trial.py`, `results.json`: the `ste100-checker` run.
- `styles/STE/`, `.vale.ini`: the Vale rule drafts.
- `vale_D.json`, `vale_S.json`: the Vale runs on the docs and on the skills.
- `bin/vale`: Vale 3.25.1.

## Decisions made in the conversation

- **Structural rules only.** The gate checks sentence length (STE 6.3), paragraph length (STE 6.6) and semicolons (STE 8.1). These checks need no word list and no judgement, and the rule files do not change over time.
- **No vocabulary check.** STE's dictionary is a whitelist of general words. Its own rules 1.5–1.11 allow the project's technical nouns, so `CONTEXT.md` is the technical-noun list. A substitution list needs ongoing upkeep and gives little to an LLM reader.
- **No `ste100-checker`.** Its install is broken with the newest `preshed`. It installs a top-level module named `src`. Its 313-word dictionary flagged 43% of all words, including approved STE words such as `that`, `its`, `you` and `which`.
- **No regex check for passive voice or complex tenses.** Vale finds the passive form, but it cannot decide STE's condition: is the actor known? Part-of-speech `sequence` rules gave the same precision as regex, and more false hits on tenses. A reviewer judges the passive voice instead (Phase 5).
- **The rules run at `error` level, and Vale becomes a gate dependency.**
- **Scope.** `docs/` (without `plans/` and `old-docs/`), the specs included, and the root docs: `AGENTS.md`, `CODING_STANDARDS.md`, `CONTEXT.md`, `ENVIRONMENT.md`, `README.md`, `tests/AGENTS.md` and `tests/STRATEGY.md`.
- **Install.** Vale goes into `~/.local/bin`. The install note is in the module docstring of `build.py`.

## Current state (known, 2026-10-31)

- `.vale.ini` loads the `Chronicler` style from `styles/Chronicler/` (`ForumLinks.yml`, `FutureTense.yml`, `PastDeletion.yml`, all at `warning` level).
- `scripts/vale_lint.py` wraps Vale. Its default scope is `docs/` without `plans/`, `old-docs/`, and files named `AGENTS.md` or `README.md`. A file outside `docs/` is in scope when you name it. It exits 0 when clean, 1 when Vale reports an `error`, and 2 when Vale is missing or fails. Its missing-Vale message says `brew install vale`.
- Vale exits 1 only for an `error`-level alert. Warnings and suggestions exit 0 (checked in the trial).
- `build.py` does not call Vale. `_GATE_CHECKS` ends with `validate-docs`. `scripts/tests/test_build_cli.py` asserts the step list and the step commands.
- Vale is not installed on this machine. `~/.local/bin` is on `PATH`.
- The `Chronicler` rules have 0 hits on `docs/diataxis/`, `CONTEXT.md`, `AGENTS.md` and `CODING_STANDARDS.md`. A gate step fails only on the new rules.

### Hits of the three rules (trial run, 2026-10-31)

| File | Sentence > 25 words | Paragraph > 6 sentences | Semicolon |
|---|---|---|---|
| `AGENTS.md` | 4 | 0 | 9 |
| `CODING_STANDARDS.md` | 1 | 0 | 3 |
| `CONTEXT.md` | 3 | 0 | 3 |
| `docs/diataxis/explanation/agent_system_design.md` | 1 | 0 | 1 |
| `docs/diataxis/explanation/architecture.md` | 6 | 0 | 11 |
| `docs/diataxis/explanation/dashboard_design.md` | 2 | 0 | 0 |
| `docs/diataxis/explanation/prompt_system_design.md` | 5 | 0 | 4 |
| `docs/diataxis/explanation/storage_design.md` | 1 | 0 | 0 |
| `docs/diataxis/reference/architecture_system.md` | 3 | 0 | 4 |
| `docs/diataxis/reference/coding_standards/guardrails.md` (generated) | 7 | 0 | 12 |
| `docs/diataxis/reference/coding_standards/integration_test_standards.md` | 1 | 0 | 0 |
| `docs/diataxis/reference/frontend/dashboard.md` | 2 | 0 | 4 |
| `docs/diataxis/reference/frontend/http_routes.md` (generated) | 0 | 0 | 1 |
| `docs/diataxis/reference/game_flow.md` | 5 | 1 | 0 |
| `docs/diataxis/reference/narrative/narration_system.md` | 2 | 0 | 1 |
| `docs/diataxis/reference/narrative/narrator_mode.md` | 0 | 0 | 2 |
| `docs/diataxis/reference/narrative/prompt_system.md` | 1 | 0 | 3 |
| `docs/diataxis/reference/startup.md` | 0 | 0 | 1 |
| `docs/diataxis/reference/storage.md` | 4 | 0 | 1 |
| `ENVIRONMENT.md` | 3 | 0 | 5 |
| `README.md` | 0 | 0 | 1 |
| `tests/AGENTS.md` (35 of 41 hits generated) | 27 | 0 | 14 |
| `tests/STRATEGY.md` | 6 | 0 | 11 |
| `docs/specs/` (13 of 31 specs) | 22 | 3 | 9 |
| **Total** | **106** | **4** | **100** |

A sample of 14 sentence-length hits showed no false hits from lists, tables or code. The skills (`.agents/skills/*/SKILL.md`) have about 95 sentence-length hits and 87 semicolon hits. They are out of scope (see "Out of scope").

### Generated text

Four files in scope hold generated text. A fix goes into the source, not into the generated file:

| Generated text | Generator | Source of the prose |
|---|---|---|
| `AGENTS.md`, the `AUTO-STRUCTURE` block | `scripts/generate_structure_index.py` | The `//!` summary line of each Rust module and the module docstring of each script |
| `docs/diataxis/reference/coding_standards/guardrails.md` | `scripts/generate_guardrails_doc.py` | `src/lib.rs` comments, `arch-lint.toml`, the `///` comments of `tests/infrastructure/guardrails/*.rs`, the row reasons in `walker_rows.rs` |
| `docs/diataxis/reference/frontend/http_routes.md` | `scripts/extract_http_routes.py` | The comments in `router.rs` |
| `tests/AGENTS.md`, the `AUTO-STRUCTURE-TESTS` block | `scripts/generate_tests_structure_index.py` | The module summary of each integration test file |

## The rules

Three files go into `styles/Chronicler/`. `.vale.ini` already loads that style, so it needs no new `BasedOnStyles` entry.

`styles/Chronicler/SentenceLength.yml`:

```yaml
# ASD-STE100 rule 6.3: a descriptive sentence has a maximum of 25 words.
# Word count per rules 8.5-8.7: text in parentheses, quoted text and a hyphenated word count as one word.
extends: occurrence
message: "STE 6.3: this sentence has more than 25 words. Split it."
level: error
scope: sentence
max: 25
token: '\([^)]*\)|"[^"]*"|“[^”]*”|[\w''’]+(?:-[\w''’]+)*'
```

`styles/Chronicler/ParagraphLength.yml`:

```yaml
# ASD-STE100 rule 6.6: no paragraph has more than six sentences.
extends: occurrence
message: "STE 6.6: this paragraph has more than six sentences. Split it."
level: error
scope: paragraph
max: 6
token: '[^.!?]+[.!?]+(?:\s|$)'
```

`styles/Chronicler/Semicolon.yml`:

```yaml
# ASD-STE100 rule 8.1: no semicolons. Write two sentences.
extends: existence
message: "STE 8.1: do not use a semicolon. Write two sentences."
level: error
nonword: true
tokens:
  - ';'
```

The `token` in `SentenceLength.yml` follows STE's word count. The simpler token of `Microsoft.SentenceLength`, `\b(\w+)\b`, counts each part of a hyphenated word and each word in parentheses. It would flag more sentences than STE does.

## Phases

### Phase 1 — Install Vale

1. Install Vale 3.25.1 into `~/.local/bin`:

   ```bash
   curl -sSL https://github.com/vale-cli/vale/releases/download/v3.25.1/vale_3.25.1_Linux_64-bit.tar.gz | tar -xz -C ~/.local/bin vale
   ```

2. Add a short install note to the module docstring of `build.py`. It says that the `vale` step needs Vale on `PATH`, and it gives the command above.
3. In `scripts/vale_lint.py`, replace `brew install vale` in the missing-Vale message with a pointer to the `build.py` docstring.

Check: `vale --version` prints `3.25.1`. `python scripts/vale_lint.py` exits 0.

### Phase 2 — Add the rules

1. Add the three rule files of "The rules" to `styles/Chronicler/`.
2. Update the header comment of `.vale.ini`: the style now holds STE structural rules, and these run at `error` level.
3. In `scripts/vale_lint.py`, add the seven root docs of "Scope" to the default targets, next to `docs/`. Update the docstring and the `.vale.ini` header comment that describe the scope.
4. Run `python scripts/vale_lint.py`. Compare the counts with the table in "Current state". `AGENTS.md` and `tests/AGENTS.md` change while other work goes on, so their counts can differ.

### Phase 3 — Fix the hits

1. Fix one file per change. Start with the hand-written docs, then do the generated text at its source.
2. A semicolon becomes two sentences. When the second sentence gives a reason or a result, start it with a key word ("So", "Thus", "Because of this"), as STE rule 6.2 recommends.
3. A long sentence becomes two or more sentences with one statement each. Do not delete facts to get under the limit.
4. A rewrite follows `docs/AGENTS.md` and the per-edit rules of the doc. It does not change what the doc states.
5. For generated text, edit the source and run the generator. `python build.py validate-docs`, `python scripts/generate_guardrails_doc.py --check` and `python scripts/extract_http_routes.py --check` must pass. A change to a Rust `//!` or `///` comment needs `python build.py clippy`.

The specs come last. Their scenarios state contracts, so a rewrite must keep the meaning of each scenario. `python scripts/validate_feature_spec.py` must still pass.

Check: `python scripts/vale_lint.py` exits 0.

### Phase 4 — Add the gate step

1. In `build.py`, add a `StepSpec` named `vale`. Its command is `python scripts/vale_lint.py --quiet`. Add `vale` to `_GATE_CHECKS` after `validate-docs`.
2. A missing Vale binary fails the step (exit 2). The step does not skip, because Vale is a gate dependency.
3. Update `scripts/tests/test_build_cli.py` for the new step and its command.
4. Add the `vale` row to the command table in `AGENTS.md`, "Development Loop", if the step can run on its own.

Check: `python build.py vale` passes. `python -m unittest discover scripts/tests` passes. `python build.py --no-browser` passes, then the full gate passes once.

### Phase 5 — Passive voice in `document-review`

Edit `.agents/skills/document-review/SKILL.md`, section 5 "Wording". Follow `.agents/skills/writing-for-agents/SKILL.md`.

1. Add this line to the checklist:
   > - Name the actor of each passive verb, unless the actor is unknown. In a skill, the reader must know whether it does the action itself.
2. Remove "Split a sentence longer than about 25 words." The gate enforces it.
3. Change "Periods, not semicolons or em dashes." to "Periods, not em dashes." The gate enforces the semicolon part.

Today the passive rule exists only as rule 29 of `.agents/skills/unslop/SKILL.md` and in `.agents/skills/technical-writing/SKILL.md`. The checklist in section 5 does not repeat it, so a reviewer can skip it. Leave both of those skills as they are.

## Failure modes

| Failure | Cause | Response |
|---|---|---|
| A paragraph-length false hit | The token counts sentence ends by punctuation, so "e.g." or "i.e." counts as an extra sentence. | Write "for example" or "that is". This also follows STE. |
| A sentence-length false hit on a list item | Vale joins a list item without end punctuation to the next text. | Give the item end punctuation, or split the item. |
| A hit in a table cell | A table cell is a sentence to Vale. | Shorten the cell. A table cell with more than 25 words is hard to read too. |
| A rebuilt container drops Vale | Vale installs outside the repo. | The `vale` step fails with exit 2. Reinstall with the command in the `build.py` docstring. |
| A hit in a quoted example of bad prose | A doc quotes a non-STE sentence to show a rule. | Put the quote in a code span or code block. Vale skips code. |

## Out of scope

- Skills (`.agents/skills/`). Some skills may come from outside sources, and a rewrite makes later updates from the source harder. A later plan can add the skills that the project owns, one at a time.
- `docs/plans/`, `docs/old-docs/` and `docs/AGENTS.md`. `vale_lint.py` already excludes them.
- A vocabulary or substitution check, and the `Microsoft` or `Google` Vale packages.
- Regex checks for passive voice and complex tenses.

## Done when

- `styles/Chronicler/` holds `SentenceLength.yml`, `ParagraphLength.yml` and `Semicolon.yml` at `error` level.
- `python build.py vale` passes, and the full gate runs the `vale` step.
- The `build.py` docstring gives the Vale version and install command.
- `python scripts/vale_lint.py` with no arguments covers `docs/` and the seven root docs.
- The section 5 checklist in `document-review` has the passive-voice line and no length or semicolon line.

## Open questions

None. The root docs, the specs and the install location are decided (see "Decisions made in the conversation").
