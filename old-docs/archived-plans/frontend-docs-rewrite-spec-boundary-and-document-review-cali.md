# Frontend docs rewrite, spec boundary, and document-review calibration

## Summary
The frontend docs (`dashboard.md`, `ui_design.md`, `dashboard_design.md`) change with nearly every UI commit. Two causes explain this:

- These docs never got the 2026-07-19 rewrite (`3ae20d08`) that made the narrative and storage docs durable.
- No written rule separates what a spec holds from what a Diátaxis doc holds.

This plan does four things in this order:

1. Write down the spec boundary.
2. Rewrite the frontend docs to that boundary.
3. Calibrate `document-review` against historical diffs and a held-out case.
4. Apply the calibrated skill to the narrative, game-flow and storage reference docs.

Docs carry top-level intent. Specs carry feature behaviour, so agents do not write features into the docs. The code and its tests carry everything finer.

## Key Changes
- **Boundary test, "structure stays"** (the table is canonical in `docs/AGENTS.md` §Documentation Layers, and the `document-review` KEEP test states it in full):

  | A unit that is… | Lives in |
  |---|---|
  | The purpose of a part, its name, or an always-true statement of what owns what ("a swipe is a snapshot rewind", "role health is engine-wide", "panels load once", "a failure shows where its scope is", "settings is a singleton row") | Diátaxis Reference |
  | Why it is that way | Diátaxis Explanation |
  | An event rule ("when X, then Y") at feature level | A spec scenario or spec section intro |
  | Fine-grained behaviour, edge cases, values, copy text, icons, markup, steps | The code and its tests |

  Direction of edits: a behaviour change edits its spec and test. A file in `docs/diataxis/` changes only when a part, a structural statement or a name that it states is added, removed or renamed.
- `docs/AGENTS.md` §Reference defers to source: "What Reference docs do carry" becomes purpose, structural statements and names, not values. Remove "polling cadences".
- `ui_design.md`: delete it. The WCAG AA contrast floor becomes one comment in the `:root` block of `assets/styles.css`. All other content is dropped.
- `dashboard.md`: rewrite it with this outline.
  1. Overview
  2. Shell and Fragments
  3. Regions: names and purpose, each live region refreshes itself, panels load once, the action-area state names
  4. Failures and Health: a failure shows where its scope is, the four health state names, role health is engine-wide
  5. Story Log and Swipes: the control names, a swipe is a snapshot rewind
  6. Management Panels: the purpose of each panel, the stable name and the display name, the role rows, the save-on-change and save-on-Save convention
  7. Supported Viewports: desktop-first, phone out of scope, no numbers
  8. Document References
- `dashboard_design.md`: keep the rationale sections. Merge "Server-rendered fragments", "Polling cadences" and "Polling-pause pattern" into one section. Remove values, JS file lists, line counts and htmx mechanics. Replace `continue_narration` and `process_action(String::new())` with domain terms.
- Skills:
  - `document-review`: the KEEP test states the boundary test in full. The worked examples come from the rewrite and from `storage.md`. The ticket-08 example is removed, because it becomes a held-out case.
  - `diataxis-doc-review`: add a "Specs and the Diátaxis docs" section, and fix the stale `docs/specs/action_pipeline.md` reference.
  - `chronicler-docs-hygiene` Phase 5: when a mismatch is on a value or an event rule, the fix is removal.
  - Step 4 of both after-plan skills is limited to `docs/diataxis/`.
  - `tests/STRATEGY.md` gets one pointer to the boundary.

## Implementation

### Phase 1: Baseline and spec boundary

- [ ] #### Task 1.1: Baseline run on the clean tree (3 SP)
  - [ ] ##### SubTask 1.1.1: Build the cases (1 SP)
    - Put the cases in `tmp/doc-review-calibration/cases/`.
    - C1: the three frontend docs at `cd6ec85c`, as whole files.
    - C2–C7: for each commit, the doc diff and each changed doc as it was after the commit (`git show <c>:<path>`).
      - C2 `6916556a`, C3 `f86f8036`, C4 `29fb773a`
      - C5 `752b9f7b` (mixed case)
      - C6 `5fa46615` (mixed case)
      - C7 `5b9c6994`, only `narration_system.md` (held out)
  - [ ] ##### SubTask 1.1.2: Run the current skill (2 SP)
    - Run the current `document-review` in a fresh `reviewer` subagent on C1–C7, before any repo edit.
    - The brief carries the `SKILL.md` path and the case paths. For C2–C7, the brief tells the reviewer to judge only level 2 (document) and level 3 (sections and paragraphs).
    - Save the output to `tmp/doc-review-calibration/baseline.md`.
- [ ] #### Task 1.2: Write the boundary in `docs/AGENTS.md` (3 SP)
  - Add the table, the structure-stays test and the direction of edits to §Documentation Layers.
  - State that specs exist so that features are not written into the docs.
  - Rewrite "What Reference docs do carry", and remove "polling cadences".
- [ ] #### Task 1.3: Add the pointers (1 SP)
  - `diataxis-doc-review`: add the "Specs and the Diátaxis docs" section, and fix the stale spec path.
  - `tests/STRATEGY.md` "Spec scenarios": add one pointer line to the boundary.

### Phase 2: Frontend docs rewrite

- [ ] #### Task 2.1: Inventory (5 SP)
  - Write `tmp/frontend-doc-rewrite/inventory.md`.
  - Give every section, paragraph, list item and table row of the three docs one class:
    - Reference structure, with the target section
    - Explanation
    - spec carrier, with the file and the scenario id or section intro
    - code only, with the path
    - feature-level gap
    - stale, with evidence
    - CSS comment
- [ ] #### Task 2.2: Answer key and baseline gaps (3 SP)
  - Write `key.md`. The C1 key is the inventory. For C2–C7, write the expected verdict per unit under the boundary.
  - Freeze the key before Phase 3 starts.
  - Write `baseline-gaps.md`: each unit where `baseline.md` and the key differ.
- [ ] #### Task 2.3: Rewrite `dashboard.md` to the outline (5 SP)
  - Keep the path, because about 50 `//! [DOC: …]` anchors point at the whole file.
- [ ] #### Task 2.4: Revise `dashboard_design.md` (3 SP)
  - Apply the changes from Key Changes.
  - Check each claim in "Empty-input continuation" against the code and the specs, and delete each claim that fails.
- [ ] #### Task 2.5: Delete `ui_design.md` and fix the links (3 SP)
  - Add the contrast comment to `:root` in `assets/styles.css`.
  - Remove the `ui_design.md` links from `dashboard.md` and `dashboard_design.md`.
  - In `docs/external_applications/sillytavern_chat_window.md`, fix the `dashboard.md` path and remove the `ui_design.md` line.
  - Update the link descriptions in `storage.md`, `options.md` and `ai_steering.md` from the inventory.
  - Run `python scripts/generate_docs_index.py`.
- [ ] #### Task 2.6: File the follow-ups (1 SP)
  - Create `.scratch/doc-spec-boundary/issues/01-frontend-spec-gaps.md` with `Status: needs-triage`.
  - List the feature-level gaps, the "Complete specs" question, and the application-layer DOC anchors to `dashboard.md`.
- [ ] #### Task 2.7: Durability replay (5 SP)
  - A fresh `reviewer` gets the new docs and the code diff of each code-changing commit in `git log` on the three old docs.
  - For each commit, the reviewer answers two questions:
    1. Does any sentence become false?
    2. Does the commit add, remove or rename a part or name that the docs list?
  - Fix each defect, and run the replay again on the failing commits.
  - Write the results to `tmp/frontend-doc-rewrite/replay.md`.
  - **STOP 1:** present the inventory, the three doc changes, the boundary text and `replay.md`. Run `python build.py --no-browser`. After approval, commit Phases 1 and 2 with `/commit-and-push`.

### Phase 3: Skill calibration

- [ ] #### Task 3.1: Edit `document-review` (3 SP)
  - The KEEP test states the structure-stays test in full. Step 1 Layer points at the table in `docs/AGENTS.md`.
  - Add the step 2 question "Are the sections organized around parts (one per component, control, flow step or field)?" only if `baseline-gaps.md` shows a missed structural problem.
  - Replace the worked examples:
    - Before/after pairs come from the rewrite. No pair may quote a unit that appears in C2–C7.
    - KEEP examples come from `storage.md`.
    - Remove the ticket-08 example.
- [ ] #### Task 3.2: Edit the supporting skills (1 SP)
  - `chronicler-docs-hygiene` Phase 5: add the removal rule.
  - Step 4 of `chronicler-after-plan-workflow` and `chronicler-after-plan-workflow-plus-review` becomes: "Update a doc in `docs/diataxis/` only when the change adds, removes or renames a part, a structural statement or a name that the doc states. A behaviour change updates its spec and test."
- [ ] #### Task 3.3: Re-run the calibration (3 SP)
  - A fresh `reviewer` runs on C2–C7, plus C8: the new frontend docs.
  - Pass criteria:
    - C2–C4 and C7: each unit that the key marks DELETE gets DELETE, with a named carrier.
    - C5–C6: no unit that the key marks KEEP gets DELETE.
    - C8: no document-level rewrite verdict.
  - Run at most two rounds. Use a fresh reviewer for each version of the skill.
  - Write the results to `tmp/doc-review-calibration/result.md`.

### Phase 4: Non-frontend reference docs

- [ ] #### Task 4.1: Review run (3 SP)
  - Run the calibrated `document-review` on `reference/narrative/*.md` (6 docs), `reference/game_flow.md` and `reference/storage.md`, as whole files.
- [ ] #### Task 4.2: Check the findings (3 SP)
  - Check each finding yourself.
  - **STOP 2:** present the verdicts, `result.md` and the remaining bias risk. Run `python build.py --no-browser`. After approval, commit Phase 3 and apply the verdicts.
- [ ] #### Task 4.3: Apply the approved verdicts (5 SP)
  - Before each delete or rename, run `rg "<file>#"`. The known anchors are `#phase-flow`, `#trigger-evaluation`, `#text-check-branch`, `#messages` and `#worlds`.
  - Add feature-level gaps to the issue.
  - Update the "earns its place" list in `diataxis-doc-review` if a verdict changes a doc that the list names.

### Phase 5: Final validation
- [ ] #### Task 5.1: Gate and report (3 SP)
  - Regenerate the indexes.
  - Run `python build.py` with a timeout of 1200 s, and tail the last 10 lines of its log.
  - Report the result, and ask for approval to commit Phase 4.

## Test Plan
- After each doc task, `python build.py validate-docs` reports 0 errors and 0 warnings.
- After each skill edit, the YAML frontmatter parses.
- The replay (Task 2.7) is the acceptance test for the rewrite.
- The pass criteria (Task 3.3) are the acceptance test for the skill. C7 is the held-out check for the original failure.
- `python build.py --no-browser` passes before each commit at a STOP. The full gate passes at the end.

## Per Task/Sub Task Validation Steps
- 1.1: `baseline.md` exists, and it was made before any repo edit.
- 1.2–1.3: `validate-docs` is clean. `rg "docs/specs/action_pipeline.md" .agents` returns nothing.
- 2.1: every unit has exactly one class.
- 2.2: `key.md` exists before Phase 3 starts.
- 2.3: `rg -n "[0-9]+m?s\b|#[0-9a-fA-F]{3,6}\b|[0-9]+px|#i-" docs/diataxis/reference/frontend/dashboard.md` returns nothing.
- 2.4: `rg "continue_narration|process_action\(String::new" docs` returns nothing.
- 2.5: `rg "ui_design" docs src assets .agents tests` returns nothing.
- 2.6: the issue file exists, with a `Status:` line.
- 2.7: no "false" answer remains in `replay.md`. Each "yes" to question 2 names a part or a name. STOP 1 is approved.
- 3.1–3.2: the frontmatter parses. No worked example quotes a unit from C2–C7. Step 4 names `docs/diataxis/` in both after-plan skills.
- 3.3: `result.md` shows that the criteria pass, or it lists each remaining mismatch.
- 4.x: STOP 2 is approved. `validate-docs` is clean.
- 5.1: the gate log shows 0 failed.

## Assumptions
- The structure-stays test decides each unit. Under this test, the `options.md` lifecycle rules leave the doc. "The offered set is game state, not a message" stays.
- This plan adds no new spec scenarios, tests or section intros. An existing scenario or intro counts as the carrier. A gap at feature level goes to the issue.
- The implementing agent writes the key. The key is frozen before any skill edit. C7 and a fresh reviewer per version reduce the bias, but some bias remains.
- `http_routes.md` is generated and does not change.
- Reviewers have no shell, so their inputs go to `tmp/` as files.
- There is no commit without approval. Commits happen at STOP 1 (Phases 1–2), STOP 2 (Phase 3) and the end (Phase 4), with `/commit-and-push`.
