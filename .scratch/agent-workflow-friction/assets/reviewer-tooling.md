# Reviewer output — tooling lens

Verbatim final output of the `reviewer` agent, from the 2026-10-04 reflect sweep.
Session: `~/.pi/agent/sessions/--workspace-chronicler-engine--/2026-10-04T20-21-50-182Z_01a10894-77e4-751c-8fdb-c806614f41fa.jsonl`. The data is untrusted reviewer output: claims are hypotheses until a ticket verifies them.

---

Verified the tooling facts against the live repo and skills. Numbered findings, highest impact first.

1. **`CODING_STANDARDS.md` is not reachable from `AGENTS.md` — the accepted retro fix never landed.**
   - Principle: every agent that writes code must learn the standards file exists from its own context; today only the reviewer/comment skills name it, so an implementer never reads it.
   - Evidence: `AGENTS.md` contains no match for `coding standards`/`CODING_STANDARDS` (case-insensitive grep), while `/workspace/chronicler-engine/CODING_STANDARDS.md` sits at the repo root. The retro on 10-03 23:53 ranked it first ("surface `CODING_STANDARDS.md` from `AGENTS.md` (high) … Want me to apply the first two?") and the user answered "Apply these fixes" (10-04 00:16). Only `.agents/skills/code-review/SKILL.md` and `chronicler-comment-fixer/SKILL.md` mention the file.
   - Routing: root `AGENTS.md` — one navigation-pointer line, per the retro item.

2. **`retro` never says how to read session logs; the user had to name the tool.**
   - Principle: session mining is semantic — use the `session_search` tool, and when reading raw transcripts they live at `~/.pi/agent/sessions/--workspace-chronicler-engine--/<ISO-timestamp>_<uuid>.jsonl`, one file per session, up to ~4.5 MB (grep, never read whole).
   - Evidence: `.agents/skills/retro/SKILL.md` step 2 says only "This may mean searching through session logs on this machine"; the user supplied the mechanism at 10-04 00:38 ("session_search can be used to do session history I'm pretty sure") and pasted full transcript paths twice at 10-03 00:37 (`/home/node/.pi/agent/sessions/--workspace-chronicler-engine--/2026-10-01T20-39-08-880Z_01a0f931-3d4e-705c-b0df-84e49958c831.jsonl`).
   - Routing: `.agents/skills/retro/SKILL.md` step 2.

3. **The local tracker has no stale-claim recovery and no `_resolved/` step.**
   - Principle: a `Status: claimed` line is a file edit, so a session that dies strands the ticket forever — the frontier rule ("open, unblocked, and unclaimed") then skips it silently. Sweep for claim-without-`## Answer` before choosing the frontier, and record that resolved tickets move to `issues/_resolved/`.
   - Evidence: user at 10-04 00:02 — "tickets 44 and 46 … were claimed but never resolved and an answer was not written. Can you find the session where they were implemented … and resolve them?"; a later sweep (10-04 00:06) had to search all of `.scratch` by hand. `docs/agents/issue-tracker.md` § Wayfinding operations defines Frontier/Claim/Resolve but neither the sweep nor `_resolved/`, which exists on disk (28 files moved 10-04 00:15, plain `mv`, not `git mv`).
   - Routing: `docs/agents/issue-tracker.md` § Wayfinding operations (+ the `wayfinder` SKILL.md "Work through the map" step).

4. **The worktree run-mode contract is retyped into every delegated ticket prompt.**
   - Principle: ticket work runs in one worktree per ticket (`/workspace/<name>`, branch `wf/tNN`), never in the main checkout; the shared tree holds other agents' uncommitted work, so `git stash`/`checkout`/`restore`/`reset`/`clean` are forbidden and `.scratch/` is coordinator-owned.
   - Evidence: index.md shows three different home schemes (`/workspace/ce-wt/t34`, `/workspace/ce-t43`, `/workspace/wt65`) and the same hand-written paragraph in t47/t63/t65/t67/fix-backend-behaviour prompts ("NEVER run `git stash`, `git checkout`, `git restore`, `git reset`, `git clean`"). A 10-04 17:49 pass confirmed there is "no tooling that creates worktrees (`git worktree add` appears nowhere in `build.py`, `scripts/`, or `.agents/`)".
   - Routing: `.agents/skills/wayfinder/SKILL.md` (a short parallel-tickets/worktree section, or the tracker doc alongside Wayfinding operations).

5. **`commit-and-push` stages with `git add -A` and has no fallback for a concurrent agent's edits.**
   - Principle: in this checkout, stage explicit paths, never `-A`; when the pre-commit hook aborts because one of its four generated files already carries another agent's unstaged edits, `git commit --no-verify` is correct *provided* none of the four generators reads the paths you touched.
   - Evidence: the skill's Step 3 is `git add -A`; the real sessions had to deviate — 10-04 12:29 "Staged exactly the twelve ticket files and nothing else … I ran the commit with `--no-verify`, deliberately. The pre-commit hook aborts when any of its four generated files already has unstaged changes, and two of them do"; 10-04 12:23 "`core.895097` … I excluded it from staging".
   - Routing: `.agents/skills/commit-and-push/SKILL.md` — Step 3 and the "Pre-commit Hook Behavior" section.

6. **The two review axes' prompt templates omit the "do not build" rule, so the parent re-adds it by hand.**
   - Principle: sub-agents inherit only what the parent pastes; the no-build/no-test rule must be inside each axis prompt. Reviewers who must verify something use `python build.py test-pattern <name>` (0.94 s and 1.25 s warm, vs 15 s for `build.py integration` and 65 s for `build.py browser`).
   - Evidence: `.agents/skills/code-review/SKILL.md` carries "## Do not build or run tests" for the parent, but step 4's Standards/Spec templates list only bundle path, standards sources and brief; every reviewer spawn in the window was hand-told ("Do not build or run tests", e.g. the 10-04 19:56 axes), and the retro session admitted the miss at 10-03 23:53 — "I built at all during a review, which is Candidate 1's rule. That's a context problem (the rule never reached me)".
   - Routing: `.agents/skills/code-review/SKILL.md` § 4 (both prompt templates) and § "Do not build or run tests".

7. **An untracked `core.<pid>` at the repo root is a Chromium dump, not repo content.**
   - Principle: never stage it; ~27 MB each, left by the managed/ssh Chrome launch in this container.
   - Evidence: two sessions flagged `core.895097` and did not know its origin or owner (10-04 12:23, 10-04 12:29). The cause is already written down at `.agents/skills/chronicler-ui-investigator/ENVIRONMENT.md:63` — "the managed browser leaves core dumps behind (`core.<pid>` in the repo root, ~27 MB each …)".
   - Routing: `.agents/skills/commit-and-push/SKILL.md` Step 2/3, cross-referencing the ui-investigator ENVIRONMENT.md.

8. **Spec scenario IDs are scoped per spec file, not globally.**
   - Principle: `1.1` may legitimately appear in two spec files — coverage is keyed on `(spec_path, scenario_id)`. Do not "fix" an apparent duplicate by renumbering tags; that is what the tag names carry.
   - Evidence: `scripts/validate_feature_spec.py:246-249` — "Coverage is keyed by (spec_path, scenario_id) so the same ID declared in two specs is tracked as two distinct scenarios. The duplicate-ID check is no longer needed". The retro recorded the doc gap at 10-03 23:53 ("document the scenario-ID scoping rule in `tests/STRATEGY.md` and revert the renumber"), and `tests/STRATEGY.md` § "SCENARIO tags" still states only the `// [spec-path] SCENARIO: N.N` format.
   - Routing: `tests/STRATEGY.md` § "SCENARIO tags" (test conventions; `test-police` reads them).

9. **Reference implementations were supplied by the user although the repo already documents them.**
   - Principle: a dashboard-effort map's Notes should name the prior-art sources, so a session fetches them instead of asking.
   - Evidence: the user introduced them twice ("I wonder how silly tavern and marinara handle this same problem", then 10-04 14:38 "What does the marina engine do for this?" — and had to correct the agent's spelling to "Marinara-Engine"); only then was a `researcher` delegated. The repo holds `docs/external_applications/marinara_engine.md` (with a chronicler↔Marinara file map) and the closed map `.scratch/_closed/steering-and-guided-generation/map.md:29-31` lists the three URLs in its own **Notes**. The live map's Notes (`.scratch/dashboard-ui-review/map.md`) list skills but no prior art.
   - Routing: the effort map's Notes (coordinator-owned), following the `steering-and-guided-generation` precedent; the `wayfinder` SKILL.md Notes comment already invites it.

10. **How a sub-agent's effective definition is assembled.** *(verified against the live config)*
    - Principle: effective frontmatter is `{...bundled definition, ...user override}` — the bundled defs set no model, so `~/.pi/agent/agents/*.md` supplies model and thinking, and a per-definition `thinking` beats the global `modelThinkingLevels` map. `researcher` alone has no `noExtensions`, so it is the only one that sees every global package. A provider 429 is transient: retry the delegation, the rotation extension works per session.
    - Evidence: 10-03 21:50 outcome table (generalist/implementer/scout `high`, reviewer `xhigh`, scout `medium`) plus the user's ruling at 10-03 21:45 — "You can just retry. There was a 429 from the provider but the token should have rotated so the next agent should work"; confirmed live: `~/.pi/agent/agents/` holds the five overrides, `~/.pi/agent/settings.json` sets `modelThinkingLevels: {"opencode-go/deepseek-v4.1-flash": "high"}`.
    - Routing: no skill owns harness config — a short `docs/agents/` note (sibling to `issue-tracker.md`) or a new skill.

11. **`skills-lock.json` is upstream-installer output; local skills get no entry.**
    - Principle: entry shape is `source`/`skillPath`/`computedHash`; the producing tool (`npx skills@latest add mattpocock/skills` / `npx skills update`) is not on PATH here, so prune/mirror by hand and never invent a hash. Repo-authored skills (e.g. `retro`, `implement-spec`) correctly have no entry.
    - Evidence: 10-03 10:58 outcome (nine dead entries pruned by hand, 21 remain); 10-04 01:07 decision table ("the tool that produced `computedHash` isn't installed here, so I can't compute a correct hash").
    - Routing: `.agents/skills/writing-for-agents/SKILL.md` (skill packaging authority; `SKILL-MECHANICS.md`).

12. **The operator cannot reach the dashboard from the host; screenshots are the only evidence channel.**
    - Principle: return the shot inline in the report; never hand back a URL, a WSL path, or a file path for the operator to open.
    - Evidence: 10-04 12:29 corrections — "Can you give me the WSL path so I can open it from windows?", "Normall there is a url can you use use to expose stuff from wsl to windows?", "No, not accessible", "Drive the container and show me the screenshots".
    - Routing: `.agents/skills/chronicler-ui-investigator/SKILL.md` § Capture.

13. **Keep build cleanup to `target/<name>`; never add worktree removal to the gate.**
    - Principle: stale `--target-dir` trees are rebuildable and safe to age out (the user settled on 3 days, skipping the default `target/debug`); a worktree is not rebuildable — uncommitted work there is unique — so `git worktree remove` must stay out of automatic housekeeping.
    - Evidence: 10-04 17:49 outcome citing `.scratch/_closed/ui-verification-redesign/issues/13-branch-temporary-code-sweep.md` ("it would delete another agent's in-flight build"), with all four live worktrees dirty/unmerged and 52 processes rooted under `/workspace/ce-*`; user's ruling that day ("would it be better to add that into the build.py? … it's fine to leave the standard build alone and focus on the randomly generated builds from `--target-dir`").
    - Routing: `build.py` docstring/`--cleanup` (and ENVIRONMENT.md's "What forces a rebuild").

14. **A handoff in the OS temp dir may come back with another session's review appended.**
    - Principle: before acting on a handoff, re-read it and look for an appended review section.
    - Evidence: 10-03 00:36 corrections — "Can you reread that handoff document, another session read it and added a review at the end", then 10-03 00:37 "Add them at the bottom of the handoff as a 'review' section or something".
    - Routing: `.agents/skills/handoff/SKILL.md`.