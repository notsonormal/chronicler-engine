# Plan: Build the move to `work/`

Ticket: [Build the move to `work/`](../issues/10-build-the-move.md) (map: Work records).
Decisions it builds: tickets 06 to 09 as amended by ticket 11 ("Decide what a workstream is for, or drop it").

## Goal

All work records move from `.scratch/` and `docs/plans/` to `work/`. Every link that resolved before the move still resolves. The docs, skills and config name the new layout. A new gate check guards the links. The full `python build.py` passes.

## Target layout (ticket 11)

```
work/
├── wiki/index.md              the one wiki (empty index, chief-of-staff format)
├── plans/                     planFolder of plan mode (empty; plan mode creates it on first write)
├── wayfinder-maps/<map>/      the 17 live maps, unchanged inside
├── issues/{pending,ready,in-progress,closed}/NN-<slug>.md
└── _closed/                   the 19 closed maps, unchanged
```

## Facts checked on 2026-11-12

- `.scratch/` has 20 live folders and `_closed/` with 19 maps. 17 have a `map.md`. The other three are `autoplay-follow-ups` (has a `map.md`, but ticket 07 dissolves it), `state-machines-work` (a `README.md` and 14 issues) and `story-scroll-back` (`idea.md` only). This matches the table of ticket 07, so no unplaced folder exists today.
- No `map.md` has a `Status:` line today.
- Relative links in `.scratch/` and `docs/plans/`: 7,400 resolve, 234 do not. 37 resolved links point out of `.scratch/` (into `src/`, `docs/`, …). One reference-style link definition and 8 HTML `href`/`src` attributes exist.
- Maps go one level deeper (`.scratch/<map>/` to `work/wayfinder-maps/<map>/`). So every link that leaves a map folder changes, not only the links between maps.
- 1,508 backticked `.scratch/…` paths sit inside the records (text citations, not links).
- Ignored `assets/local-only/` folders exist in `fable-audit`, `weekly-retro-2026-11-04` and `statechart-models`. `.gitignore` line 215 names `.scratch/**/assets/local-only/`.
- Outside the records, these files name `.scratch` or `docs/plans`: `AGENTS.md:386`, `docs/AGENTS.md:170`, `docs/agents/issue-tracker.md`, `docs/agents/triage-labels.md:3`, `.pi/plan-mode.json`, `arch-lint.toml:18`, `.gitignore:215`, `guardrails.md:33` (generated from `arch-lint.toml`) and `:137` (hand-written, outside the generated blocks), `tests/infrastructure/guardrails/structure.rs:27,87` and `structure_tests.rs:28`, `scripts/validate_docs.py` (`plans` as TRANSIENT and route-exempt), `scripts/vale_lint.py:53`, `.vale.ini:29`, `scripts/tests/test_validate_docs.py:579`, `scripts/tests/test_prepare_review_bundle.py:40`, `.pi/extensions/smoke-break/index.ts:8`, the plan-mode README and test (generic fixtures), and the skills `simple-after-plan`, `chronicler-after-plan-workflow`, `chronicler-after-plan-workflow-plus-review`, `document-review`, `verify-gate` (`SKILL.md:172`, `mise.toml:2`), `chronicler-ui-investigator/ENVIRONMENT.md:206` and `code-review` (upstream, not edited).
- `build.py` runs each step command with `shell=True`. Each gate check is one script in one `StepSpec`, listed in `_GATE_CHECKS`. `scripts/tests/test_build_cli.py` pins the command of every step (`test_every_spec_command_pinned`).
- The 17 broken links of live maps and tickets are: 2 `(link)` placeholders in HTML comments (`unclebob-quality-tools`, `weekly-retro-2026-11-04`), 14 links into `tmp/` (all in `metacognitive-audit`), and `fable-audit/issues/02-task-engine-fixes.md` → `../invariant-audit/map.md`. Ticket 09 says that last one "points at the closed map", but `invariant-audit` is a **live** map. The fix is `../../invariant-audit/map.md` (and after the move, the same relative path still works).
- 3 links in `_closed/` point into live maps. They break unless rewritten. All other `_closed/` links stay valid, because `_closed/<map>/` keeps its depth.
- Broken links in live assets: 120 (ticket 09 said 120). They stay as they are.
- `writePlanFile` in the plan-mode extension creates `planFolder` if it is missing.
- The repo-root `plans` symlink points at `docs/plans`.

## Steps

### 1. Shared link code: `scripts/work_records.py`

One module that the gate check, the issue mover and the move script share:

- `iter_links(text)`: relative markdown links, reference-style link definitions (1 today) and image links, outside fenced code and HTML comments, with their spans. Reuses `MARKDOWN_LINK` and `FENCE_DELIMITER` from `scripts/validate_docs.py` where they fit (`MARKDOWN_LINK` skips image links, so the module adds them).
- `rewrite_links(files, moves, root)`: `moves` maps old absolute paths (files and folders) to new ones. For each link in each file: resolve against the file's **old** location, map the target through `moves` (longest prefix wins), and write the path relative to the file's **new** location. Keeps `#anchor` and `?query` parts. Leaves URLs, anchors-only and unresolvable links alone.
- `rewrite_backticked(text, moves)`: rewrites backticked repo paths (`` `.scratch/…` ``, `` `docs/plans/…` ``) through the same table.

### 2. Gate check: `scripts/validate_work_records.py` (ticket 09)

- (a) Every relative link in the live records resolves: `work/wayfinder-maps/*/map.md`, `work/wayfinder-maps/*/issues/*.md`, other `.md` beside `map.md`, `work/issues/*/*.md`, `work/wiki/**`, `work/plans/**`. Skips `assets/` and `_closed/`.
- (b) Every backticked `work/…` path in `AGENTS.md`, `docs/**/*.md` and `.agents/skills/**` exists (a trailing `/`, `<placeholder>` or glob part is cut back to the existing prefix).
- Wire: a new `StepSpec` `validate-work-records` in `build.py`, listed in `_GATE_CHECKS` right after `validate-docs`, and pinned in `test_build_cli.py`. This follows the one-script-per-step pattern. It departs from the wording of ticket 09 ("in `build.py validate-docs`"): the check runs as `python build.py validate-work-records` instead. A compound `&&` command would be the only one of its kind in the registry.
- Tests: `scripts/tests/test_validate_work_records.py`, on a temp tree: a good link, a broken link, a link in fenced code and in an HTML comment, a skipped asset, a stale backticked path, a placeholder path.

### 3. Issue mover: `scripts/move_issue.py` (ticket 11, point 6)

`python scripts/move_issue.py <NN> <pending|ready|in-progress|closed>`. Finds `work/issues/*/NN-*.md`, moves it, and rewrites the incoming links in every `.md` under `work/`, `docs/`, `AGENTS.md` and `.agents/skills/` through `rewrite_links`. The status folders sit at one depth, so the moved file's own links need no rewrite. Fails when the number matches zero or two files. Tests in `scripts/tests/test_move_issue.py`.

### 4. Move script: `move_to_work.py` (one-off, kept as an asset)

Lives at `.scratch/work-records/assets/move_to_work.py` and moves with its map. Arguments: `--root <dir>`, `--dry-run`, `--reverse <path-moves.md>`.

1. **Precheck.** List the folders of `.scratch/`. Stop on a folder that its table does not name. Stop when `work/` exists.
2. **Before-scan.** Record every relative link in every `.md` under `.scratch/`, `docs/plans/` and the repo-wide citing files, and whether it resolves.
3. **Build the path table.** Maps: `.scratch/<map>/` → `work/wayfinder-maps/<map>/`. `_closed/` → `work/_closed/`. The 23 one-off issues get numbers 01 to 23 in this order: `autoplay-follow-ups` 01–04, `state-machines-work` in number order, `story-scroll-back/idea.md`, then the four plans alphabetically. Each goes to `pending/`, except `autoplay-follow-ups/04-…` (`Status: closed`), which goes to `closed/`. Slugs keep their old text; `idea.md` becomes `NN-story-scroll-back.md`.
4. **Rewrite, then move.** Rewrite links and backticked paths in every moved file and in each citing file outside (step 5 list), through the table. In `_closed/`, rewrite only the links that would break (3 today), and no backticked paths (decision 3). Then move the files (`os.replace`, so untracked and ignored files move too, `local-only/` included). Remove the old `autoplay-follow-ups/map.md`, `state-machines-work/README.md`, the empty folders, `docs/plans/` and the `plans` symlink.
5. **Edit records.** Add `Status: open` or `Status: closed` under `Labels:` in each `map.md` (closed: `prompt-flexibility`, `layout-trials`, `statechart-models`, `metacognitive-audit`). Remove the `Status:` line of each one-off issue. Add the cross-link between `19-story-scroll-back.md` and the story-log plan where it is missing. Fix 15 of the 17 broken links of live maps and tickets: the 14 `tmp/` links become backticked text, and the `fable-audit` ticket 02 link points at the live `invariant-audit` map. The 2 placeholders in HTML comments stay; the gate check skips comments. Create `work/wiki/index.md`.
6. **After-scan.** Every link that resolved before resolves after. Run `validate_work_records.py`. Write `work/wayfinder-maps/work-records/assets/path-moves.md` (old → new, one row per moved file or folder).
7. `--reverse` reads `path-moves.md` and moves back through the same code.

Dry run first on a copy (`cp -a` of the repo minus `target/` into `tmp/`), then on the real tree.

### 5. Hand edits (in the same session, right after the real run)

- `docs/agents/issue-tracker.md`: a "Names" section and the layout tree; the status table (map, ticket, one-off issue by folder); the `Status:` line of `map.md`; the old-path line linking `path-moves.md`; every `.scratch/` path becomes `work/…`; where a one-off issue goes and how it moves (`scripts/move_issue.py`).
- `docs/agents/triage-labels.md`: the table maps each role to a folder.
- `AGENTS.md:386`: "Local markdown under `work/` (per `docs/agents/issue-tracker.md`)".
- `docs/AGENTS.md:170`: "Plans (`work/`) are exempt."
- Skills: `chronicler-after-plan-workflow` and `-plus-review` steps 2–3 and `document-review` line 18 point at the tracker doc. `simple-after-plan`, `verify-gate`, `chronicler-ui-investigator/ENVIRONMENT.md` keep their citations, rewritten by the script.
- Config: `.pi/plan-mode.json` (`planFolder: "work/plans"`, `scratchFolders: ["tmp", "work"]`), `arch-lint.toml` (`**/work/**`), `.gitignore` (`work/**/assets/local-only/`), `guardrails.md:137` (the new `_closed` path).
- `structure.rs`: a DOC anchor may not point into `work/` (was `docs/plans`); update `structure_tests.rs`.
- `smoke-break/index.ts:8`: the cited plan is gone; drop the reference.

### 6. Verify

- `python build.py validate-docs`, then the full `python build.py`.
- A spot check by hand: open five moved links in different kinds of files.
- Record the answer on ticket 10, close it, and add the line to Decisions so far. Then `/simple-after-plan`.

## Decisions (owner, 2026-11-12, round 1 of the plan review)

1. The other pi sessions are idle. The real move runs after a clean dry run on a copy.
2. The status text of the five new one-off issues (four plans with `**Status:**`, the idea with `Status: seed …`) stays, with the label renamed to `**Before the move:**`. The 18 issues from `autoplay-follow-ups` and `state-machines-work` lose their `Status:` line.
3. Backticked `.scratch/…` paths are rewritten in live maps, issues, plans, the wiki and assets. `_closed/` is not touched, except the 3 links into live maps.
4. The `plans` exemptions in `scripts/validate_docs.py` (`ROUTE_EXEMPT_DIR_NAMES`, the TRANSIENT branch, `STANDARD_PLAN_LINK` if it only serves `docs/plans`), `scripts/vale_lint.py` (`TRANSIENT_DIR_NAMES`), the `.vale.ini` comment and their test fixtures are removed. `old-docs` exemptions stay.
5. The tracker doc names `work/issues/assets/` as the asset folder of one-off issues. The move does not create it.
