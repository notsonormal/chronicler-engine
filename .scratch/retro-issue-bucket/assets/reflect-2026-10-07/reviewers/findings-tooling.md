# Reflect reviewer output

Session: `01a117fd-2782-731c-96e6-39f1eb75f1ff`

---

Verified the tooling facts against the current tree (read-only). Findings, most load-bearing first.

---

**1. Duplicate-code detection already exists as a gate step, and no review in this window used it.**
- **Principle:** the repo ships `jscpd`-based clone detection — gate step `duplicates` = `python scripts/healthcheck.py duplicates` (report-only, never fails the gate), report at `report/jscpd-report.json`, discovers the binary at `~/.pi-lens/tools/node_modules/.bin/jscpd` else falls back to `npx jscpd`, min 3 lines / 30 tokens over `src/` + `tests/`. Step subcommand form: `python build.py duplicates [--all] [--ref <ref>]` (`build.py:_step_extra_args` forwards both). A Standards-axis review should read this report *before* hand-applying the Fowler smell list.
- **Evidence:** grep for `healthcheck|jscpd|duplicates` across all eight `tmp/reflect3/digest*.md` → zero hits, while five separate review sessions hand-found *Duplicated Code* (e.g. 10-06 18:40 "The heal+persist pair is copy-pasted into three new closures"; 10-07 19:19 "Duplicated form→config mapping — *Duplicated Code / Data Clumps*"), and the parent re-pasted the full Fowler baseline into each §4 brief ("the smell baseline from step 3 **pasted in full**"). `report/jscpd-report.json` currently reports `"clones": 2446`.
- **Routing:** `.agents/skills/code-review/SKILL.md` §3/§4 — add the report as an input the Standards brief names (the skill already says "skip anything tooling enforces", but names no tool for duplication).

**2. `commit-and-push` has no branch for a GitHub-side push failure; the loop cost 7 attempts.**
- **Principle:** `remote: Internal Server Error` on `git push` is a GitHub ref-write failure, not a local one — `git push --dry-run` succeeding proves auth, negotiation and the local pack are fine. Stop after ~2 attempts, record the `Request ID` lines, do not force-push, and re-push from a later session; it clears by itself. The skill's Edge Cases only covers "remote has diverged (push rejected) → `git pull`", which sends the agent down the wrong path.
- **Evidence:** 10-07 16:53 commit-and-push session — user said "Try again"; the agent retried and reported "Seven attempts now, all with the same GitHub-side error. I stopped retrying." with `git push --dry-run` succeeding (`5fa46615..6916556a`) and Request IDs (`E212:14F2FE:397F6:A1AF3:6AC679DB`). At 17:46:57 the same local repo pushed cleanly (`5fa46615..d77eca37  dashboard-ui-issues-2 -> dashboard-ui-issues-2`) with no local change, in a session where the user pre-warned "There might be some issues with pushing so only try and push once".
- **Routing:** `.agents/skills/commit-and-push/SKILL.md` → "Edge Cases" / Step 6.

**3. Session archaeology is a real tool the wayfinder map must name (`session_search` / `session_read` + the JSONL dir).**
- **Principle:** a map whose tickets were implemented by other sessions has its answers in *their* transcripts; `session_search` (semantic, 853 sessions indexed) and `session_read` (per-session, supports entry ranges) reach them, and the raw files are `~/.pi/agent/sessions/--workspace-chronicler-engine--/<timestamp>_<uuid>.jsonl`. No skill body in this repo names them; the per-effort `map.md` **Notes** block is the place that does.
- **Evidence:** 10-05 21:40 — the user had to supply it: "you should be able to use session_search or just search the pi session history directlyu"; the agent then used `session_search` and a JSONL grep. 10-07 17:42 — "Can you search through the sessions for tickets …/68-split-settings-sub-tabs.md and …/70-remove-story-log-check.md. I'm pretty sure they have been completed …"; there the agent used `recall` (literal) + `session_read` with entry ranges. Note: naming `session_search` in `retro` step 2 was explicitly **dropped by the user** on 10-05 20:20 (ticket 03's `## Answer`: "F03 — no. `retro` step 2 stays as is; dropped by user decision"), so route it to the map, not to retro.
- **Routing:** `.agents/skills/wayfinder/SKILL.md` → "The map body / `## Notes`" (and the Work-through-the-map step 3 zoom rule).

**4. Ticket resolution is not owned when implementation is delegated — three user interventions.**
- **Principle:** when the coordinator hands a ticket to a worktree implementer, the `Status: resolved` line and the `## Answer` are the coordinator's to write, in the same session, before the commit session splits them. Delegated implementation plus a follow-up `/commit-and-push` is exactly the gap that leaves a ticket `claimed` forever.
- **Evidence:** 10-06 19:42 (compaction, verbatim scope change) "Actually resolve and answer and then fix all the issues, alright we might forget to resolve later"; 10-07 17:42 "I'm pretty sure they have been completed they so should be set the resolved and answered based on what happened in those sessions"; 10-07 19:57 "Make them as resolved and set the answer". The 10-07 17:42 session then reconstructed both answers by reading the sub-agent transcripts. This is already ticket 01 in the new `.scratch/retro-issue-bucket/map.md` ("68/70 stayed `claimed` after `6916556a`; answers were rebuilt from logs").
- **Routing:** `.agents/skills/wayfinder/SKILL.md` → "Work through the map" step 4 + the map-Notes line that encodes "one ticket = one session = one commit".

**5. Reviewer briefs must not tell sub-agents to skip `tooling.patch`.**
- **Principle:** `scripts/prepare_review_bundle.py` assigns patch areas by path prefix — `src/` → `src.patch`, `tests/` → `tests.patch`, `docs/`+`assets/` → `docs.patch` — and the `tooling` area is the **catch-all** (`("tooling", "tooling.patch", ())`, step 4 in `classify_area`): `build.py`, `scripts/`, `.agents/`, `.scratch/`, `AGENTS.md`. So `tooling.patch` routinely carries gate-pin edits and regenerated indexes, and "ignore it" hides them.
- **Evidence:** the parent's §4 brief told both axes "tooling.patch contains only `.scratch/` ticket-status bookkeeping — IGNORE it" (10-06 23:10 spec axis, 23:12 standards axis). The standards reviewer returned: "Premise mismatch: `tooling.patch` is not only `.scratch/` bookkeeping — it also carries that pin and the generated `AGENTS.md` structure line". `--ref` mode (3-dot diff) also adds `commits.txt`, which a fixed-point review should read.
- **Routing:** `.agents/skills/code-review/SKILL.md` §4 (both brief templates).

**6. `comment_finder.py --uncommitted` returns every comment in each touched file — not the added ones.**
- **Principle:** mode 1's file list is diff-scoped, but the output is *file*-scoped (`find_comments_in_file` returns all comment lines). On a working tree that touched `build.py` (170 pre-existing docstrings), "classify every line" means rewriting history unrelated to the change. Scope to the diff's added lines and delete whole blocks; re-run the finder and require a drop ≥ deleted.
- **Evidence:** 10-07 17:47 — "The finder's `--uncommitted` mode lists every comment in each touched file (236 total), but the change set adds only ~11 comment units. I scoped to the added lines … (`tmp/added_comments*.txt` + `tmp/apply_comment_deletions.py`)". Same shape at 10-04 21:49 — "Scope from `git diff -U0 -- '*.rs' '*.html' '*.css'`: 304 added marker lines".
- **Routing:** `.agents/skills/chronicler-comment-fixer/SKILL.md` §1/§2.

**7. `--all` cannot see Python: `build.py` docstrings and `scripts/*.py` are unreachable in the "full codebase scan".**
- **Principle:** `get_all_source_files()` globs only `src/**/*.rs`, `tests/**/*.rs`, `assets/*.html`, `assets/*.css`. Python is only reachable through `--files` / `--pattern` (the extension filter is `{.rs,.py,.html,.css}`). A "full sweep" that reports clean has silently skipped every `#`/`"""` comment in `build.py` and `scripts/`.
- **Evidence:** skill §1 mode 2 label is explicit ("full codebase scan; skips Python") but offers no Python mode; 10-07 17:45 the pass had to justify excluding "build.py's 170 pre-existing docstrings" because they arrived via mode 1, and `--files … scripts/bar.py` is the only Python example in the skill.
- **Routing:** `.agents/skills/chronicler-comment-fixer/SKILL.md` §1 — add `--pattern 'scripts/**/*.py'` (and note `--all` misses root-level Python).

**8. To show a human a static prototype, serve it through the engine's own `/assets`.**
- **Principle:** the running engine mounts `ServeDir::new("assets")` at `/assets` *and* as the fallback service (`src/adapters/http/…/builders/router.rs:218-220`), so copying a prototype into `assets/<prototype-name>/` makes it reachable at `<engine port>/<file>` with no second server. The user cannot open local file paths at all, and ports 3000 (dashboard default) / 3001 (probe default) are usually already bound by a stale engine.
- **Evidence:** 10-06 20:15 the user: "I can't see ///workspace/chronicler-engine/tmp/ui-review/t15/settings-prototype.html, can you expose it over port 3000"; the agent found 3000 occupied, grepped the router, and moved the artifact to `assets/prototype-t15/` (200 in its check). Trade-off worth stating in the skill: that leaves a throwaway file inside a served directory, against `/prototype`'s "delete or absorb when done".
- **Routing:** `.agents/skills/prototype/UI.md` (read in that session) — "Two sub-shapes"/process section: how to expose the artifact.

**9. A doc-only scenario split fails the gate: `scripts/validate_feature_spec.py` needs a tagged test.**
- **Principle:** `docs/specs/*.md` scenarios are validated by the gate step `spec-coverage` (`python scripts/validate_feature_spec.py`): every declared scenario needs an annotated test carrying `// [docs/specs/<spec>.md] SCENARIO: N.N`, and `browser_*` specs may only be tagged from `tests/browser/`. A document-review pass that adds or splits a scenario is therefore a code change too. The companion gate over docs is `python build.py validate-docs`.
- **Evidence:** 10-06 20:04 document-review session's own conclusion — "Not applied: the `browser_dashboard.md` Scenario 16.28 split. That item was conditional, and a doc-only split breaks spec validation because the new scenario needs its own tagged browser test."
- **Routing:** `.agents/skills/chronicler-docs-hygiene/SKILL.md` Phase 1 (which names only `validate_docs.py --strict`).

**10. Machine-generated counts must not be restated in hand-written docs.**
- **Principle:** `scripts/extract_http_routes.py` writes the route table and prints `Wrote … (N routes).` to **stdout only**; its `--check` mode (gate step `http-routes-check`) compares the generated file byte-for-byte. Nothing validates a count written in prose elsewhere, so `(57 routes)` in `dashboard.md`/`dashboard_design.md` rots the next time a route is added.
- **Evidence:** 10-07 20:00 the user caught it: "I just noticed that this was changed from 56 to 57. We shouldn't be manually counting and updating like this. Remove the number unless the number itself is automatically generated somehow"; the agent confirmed "The generator only prints the count to stdout … it never writes it into a doc, so both mentions were hand-maintained."
- **Routing:** `.agents/skills/chronicler-docs-hygiene/SKILL.md` Phase 2 rule list, under the existing `Reference defers to source` anchor (docs/AGENTS.md owns the rule text).

**11. The gate names the leaking/flaky test; nextest's default status level hides it.**
- **Principle:** a passing-but-leaking test is only reported as `63 passed (1 leaky)` unless the status level prints per-test lines — the gate sets `NEXTEST_STATUS_LEVEL=leak` and the build epilogue prints `leak test: <name> (<step>)`; `--test-timings` restores `pass` because the timing report needs PASS lines. Debugging a flake: run the tier (`python build.py browser`), read the epilogue, not just the summary.
- **Evidence:** 10-07 17:18 the user had to ask — "I am concerned about this flaky test, is there a way to make the build.py actual report what actual test is flaky?"; build.py previously set `NEXTEST_STATUS_LEVEL=fail`, which "suppresses live PASS/LEAK lines" (its own comment), and the fix landed as `41068334 feat(build): name leaky and flaky tests in the nextest epilogue`.
- **Routing:** `.agents/skills/tdd/SKILL.md` → "Where tests live here" (it already points at `tests/AGENTS.md`, whose rule 5 is "Investigate pre-existing and flaky failures too").

**12. Delegated waytree work must be committed *inside* the worktree, or the tree is unprunable.**
- **Principle:** the implementer brief convention ("Do NOT commit, do NOT stage") leaves the worktree dirty, and `scripts/remove_worktrees.py` refuses dirty trees — so the full gate's automatic removal (`python scripts/remove_worktrees.py --apply`, gate step `remove-worktrees`, warns-and-continues) can never clean them up; they accumulate as `/workspace/wtNN`. It also refuses trees younger than `--min-age-hours 24`, unpushed branches, the main tree, and any tree with a live process cwd inside it.
- **Evidence:** 10-05 21:39 — three leftovers found dirty (wt63 20 files, wt65 7, wt67 35) with the user asking to delete them; the user then settled the convention on 10-07 17:54: "You can commit onto worktrees, they will be deleted afterwards. But the changes should sit uncommited in dashboard-ui-issues-2 after everything is done."
- **Routing:** `.agents/skills/wayfinder/SKILL.md` → the map `## Notes` guidance for efforts that carry execution (this map's Notes already holds "one ticket = one session = one commit").

**13. Sibling effort maps are in-repo context — sweep `.scratch/*/` before asking the human.**
- **Principle:** `.scratch/<effort>/` holds one map per effort, plus `assets/`. A defect or behaviour question raised in one map is often already recorded as a ticket or Out-of-scope line in a sibling map; resolving a ticket should include a grep of the other effort directories.
- **Evidence:** 10-06 18:15 the user had to point at it — "Can you read through .scratch/architecture-deepening. I feel like this might be an issue raised here, although I could be wrong" — and the agent confirmed "The user's instinct is right: the architecture-deepening map's candidate A / ticket 09 … is the ticket where this behaviour question lives". The peer session a minute later pointed at the same record ("already recorded on the agent-workflow map… points at architecture-deepening ticket 10").
- **Routing:** `.agents/skills/wayfinder/SKILL.md` step 3 ("zoom as needed") — extend from "related or closed ticket" to sibling effort directories.

No files were modified; all checks were reads or `git log`-equivalent inspection.