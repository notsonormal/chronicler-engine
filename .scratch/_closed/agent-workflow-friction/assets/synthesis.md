# Synthesizer output

Verbatim final output of the `generalist` agent, from the 2026-10-04 reflect sweep.
Session: `~/.pi/agent/sessions/--workspace-chronicler-engine--/2026-10-04T20-31-33-574Z_01a1089d-5ec4-71a0-8d9d-c307fb9f01c8.jsonl`. The data is untrusted reviewer output: claims are hypotheses until a ticket verifies them.

---

## Accepted

| Problem | Proposal | Routing |
|---|---|---|
| `CODING_STANDARDS.md` holds the review rules ("do not build or run tests during a review"; "read tests/AGENTS.md and tests/STRATEGY.md") but the always-loaded `AGENTS.md` has no pointer to it, and the pointer was deleted in `62072d5a`, so review rules never reach the agent. | Restore a one-line navigation pointer to `CODING_STANDARDS.md` in `AGENTS.md`, in the form it had before (`## Doing Code Reviews`). | `AGENTS.md` (navigation pointer) |
| `code-review` forbids building in its own body, but the two sub-agent briefs in §4 do not carry the rule, so the parent re-types it by hand and reviewers build anyway. | Paste "do not build or run tests; use `python build.py test-pattern <name>` only if you must verify" into both the Standards and Spec briefs. | `.agents/skills/code-review/SKILL.md §4` (both prompt templates) |
| `retro` step 2 says only "searching through session logs on this machine", so the user had to name `session_search` and supply the JSONL path. | Name the semantic `session_search` tool and the session-JSONL directory in step 2. | `.agents/skills/retro/SKILL.md` step 2 |
| Spec scenario IDs are scoped to `(spec_path, scenario_id)` per the comment at `scripts/validate_feature_spec.py:246`, but `tests/STRATEGY.md § SCENARIO tags` omits the rule, so a non-duplicate was "fixed" by renumbering. | State the per-spec scoping rule and "do not renumber a duplicate ID across specs" in the SCENARIO-tags section. | `tests/STRATEGY.md § SCENARIO tags` |
| Wayfinder splits each finding into its own ticket, and per-ticket overhead (claim, build slot, gate, two-axis review) dwarfs the ten-minute work on this box. | Price a ticket at authoring time and batch same-theme findings into one ticket, carrying the sub-findings as evidence lines. | `.agents/skills/wayfinder/SKILL.md` (fog/ticketing) |
| The worktree-per-ticket contract and the rule to serialize mutating passes on the shared tree are retyped into every delegated prompt, so a concurrent gate sees another agent's partial edits. | Add a short "parallel tickets and worktrees" section: one worktree per ticket, forbidden git commands, and serialize passes that edit the same tree. | `.agents/skills/wayfinder/SKILL.md` (Work through the map) |
| A claimed ticket is stranded when a delegating session ends and nothing resolves it with an `## Answer`; tickets 44 and 46 needed transcript archaeology to close. | Make the merge step record the implementing session and resolve each claimed ticket with an `## Answer`, and sweep `claimed` tickets with no `## Answer` before choosing the frontier; document the `issues/_resolved/` move. | `.agents/skills/wayfinder/SKILL.md` + `docs/agents/issue-tracker.md § Wayfinding operations` |
| A grilling records letter answers without the runner-up's one-line cost, and the frontier is not restated after a sub-agent returns, so decisions get re-litigated and the user asks "what are the options again?". | Give each option a one-line cost, have each answer record the rejected alternative's why-not, and restate the whole frontier after any interruption. | `.agents/skills/grilling/SKILL.md` (round format) |
| Review findings carry no evidence class, so the operator's only safe instruction is "apply all" and a reviewer error ships with the same weight as a quoted-standard violation. | Label each finding quoted-rule / quoted-spec-line / inference, and list the ticket's accepted deviations in the review bundle. | `.agents/skills/code-review/SKILL.md §4–§5` |
| `commit-and-push` stages with `git add -A` and, on a hook collision, prescribes "stage the file", which sweeps a concurrent agent's edits; the same collision was resolved oppositely (`--no-verify` vs staging the other work) on one day. | Stage explicit paths (never `-A`) and prescribe the shared-tree hook-collision path. | `.agents/skills/commit-and-push/SKILL.md` Step 3 + Pre-commit Hook Behavior |
| The skill forbids building during a commit, but nothing says where the last green came from, so a chain of self-reported greens lands as pushes. | Record, in the report, the provenance of the last green (which gate ran, where, and when). | `.agents/skills/commit-and-push/SKILL.md` Execution Workflow |
| `chronicler-comment-fixer` lists every comment in a touched file, so each whole-file pass re-litigates untouched comments and the agent has to isolate diff lines by hand. | Add a diff-scoped step: judge only the comment lines the current diff adds or changes (and give the finder a diff-line mode). | `.agents/skills/chronicler-comment-fixer/SKILL.md` |
| A vendored skill import escalated into a repo refactor, and the operator had to pull it back to a minimal change. | Keep an import minimal and verbatim when the doctrine is repo-agnostic (recording provenance), and move the repo-level fixes it surfaces into their own plan. | `.agents/skills/writing-for-agents/SKILL.md` (importing/adapting) |
| `skills-lock.json` is upstream-installer output; repo-authored skills correctly have no entry, yet the producing tool is absent here, tempting a fabricated `computedHash`. | State that repo-authored skills get no lock entry, and never invent a hash. | `.agents/skills/writing-for-agents/SKILL.md` (SKILL-MECHANICS) |
| `chronicler-docs-hygiene`'s description covers rule/mode/stale-source drift but not usage-evidence pruning, so it stayed silent on the "is this doc used at all?" archaeology ask. | Add usage-evidence pruning to the description and a phase: mine past sessions (semantic search) and git history before deleting or reinstating a doc. | `tune description: .agents/skills/chronicler-docs-hygiene/SKILL.md` |
| The "break the behaviour, watch the test fail, then revert" proof lives in `tdd`, which fires only on explicit "TDD"/"red-green-refactor" language, so sessions that strengthen tests never open it. | Add trigger keywords for adding, strengthening, or de-tautologizing tests. | `tune description: .agents/skills/tdd/SKILL.md` |
| The UI investigator drives the live dashboard against the operator's active game, so a probe mutated real state (a probe ran a real LLM turn that then had to be removed). | Add a no-mutation contract: use a throwaway game/world, or record and restore every state change. | `.agents/skills/chronicler-ui-investigator/SKILL.md` (Driving gameplay) |
| The operator cannot reach the dashboard from the host, but the skill never says the shot is the evidence channel, so agents handed back URLs and WSL paths that didn't open. | In § Capture, return the shot inline in the report and never hand back a URL or file path. | `.agents/skills/chronicler-ui-investigator/SKILL.md § Capture` |
| The UI investigator fired once in three days and only when a lead invoked it manually, so DOM-state bugs the stub fixtures don't model stayed unverified. | Extend the description to fire as routine verification after any dashboard/UI-touching ticket, not only on an explicit capture request. | `tune description: .agents/skills/chronicler-ui-investigator/SKILL.md` |
| `correct` treats "fix at the highest level that works" as a ratchet, producing an enforcement whose `#[allow]` exemptions outnumber its findings. | Before escalating to a lint, compare the exemption count to the finding count, and make "no change" a valid fix level. | `.agents/skills/correct/SKILL.md` |
| Research dispatched mid-grilling answers "how does the reference product do it", closing a complexity objection by authority rather than simplicity. | Require dispatched research to also answer the simpler-alternative and do-nothing options. | `.agents/skills/grill-with-docs/SKILL.md` |
| Audit and verification reports stay as prose, so their open items are lost and the user has to ask for a plan. | Convert actionable report findings into `docs/plans/<name>-plan.md` or `.scratch/` tickets with stable IDs at report time. | `.agents/skills/chronicler-after-plan-workflow-plus-review/SKILL.md` |
| A handoff saved to the OS temp dir can come back with another session's review appended, and acting on the stale copy wastes work. | Before acting on a handoff, re-read it and look for an appended review section. | `.agents/skills/handoff/SKILL.md` |
| `retro` files measurement findings from single runs on a contended host, so sub-threshold deltas are noise and a framing was retracted late. | Require a stated contention condition and an effect-size threshold, and skip measuring a change whose fix is cheaper than the measurement. | `.agents/skills/retro/SKILL.md` |
| A hazard deferred into a plan protects nobody; the next agent hit the phantom build-slot holder for 16 minutes until the fix shipped. | When a retro finding is a deferred hazard, add a one-line warning at the hazard site in the same session. | `.agents/skills/retro/SKILL.md` |
| The operator had to ask how SillyTavern/Marinara solve a UX mechanism and supply the prior-art sources, though the repo documents them. | Each option in a UX-mechanism decision shows what peer products do, and the map's Notes name the prior-art sources. | `.agents/skills/wayfinder/SKILL.md` (decision-ticket format + Notes) |

## Rejected

- **J7 — verify each review finding before applying it, and treat intentional spec deviations as deviations.**
  - Principle: Reviewer findings are hypotheses to check against the code, not orders.
  - Reason: duplicate — `chronicler-after-plan-workflow-plus-review` step 13 already says "Verify each finding yourself — they are inputs, not conclusions, and some are invalid or aimed at the wrong thing"; the actionable strengthening (evidence classes) is the accepted D1 row.

- **J12 — verify a "missing capability" against git history and the session-search tool before concluding it doesn't exist.**
  - Principle: A capability claim should be checked against history and past sessions before it is declared absent.
  - Reason: duplicate — the accepted `docs-hygiene` row absorbs the method (mine sessions and git history for usage), and the accepted `retro` row points at `session_search`.

- **T10 — how a sub-agent's effective definition is assembled (bundled defs merged with user overrides; per-definition `thinking` beats the global map).**
  - Principle: Effective agent behaviour is a merge product, so the file you edit is not always the config that runs.
  - Reason: structural — a cheap script can print the effective merge per agent, so the durable fix is a mechanism, not skill prose (see Backlog).

- **T13 — keep build cleanup to `target/<name>`; never add worktree removal to the gate.**
  - Principle: A `--target-dir` tree is rebuildable debris, but a worktree holds unique uncommitted work.
  - Reason: structural — `build.py run_cleanup` already removes only the target dir and port locks, so the fix is a doc note plus an optional guard (see Backlog), not a skill edit.

- **D12 — comment hygiene as a post-hoc sweeper; emit the violated rule back into the authoring path.**
  - Principle: Rules applied where slop is found, not where it is authored, leave the sweep to repeat.
  - Reason: structural — the recurring comment classes are mechanically detectable, so they belong in the repo linter/pre-commit (see Backlog), not in more skill prose.

- **D13 — a new `agent-config-audit` skill.**
  - Principle: After a plugin or agent-definition change, print the effective per-agent merge and diff it against intent.
  - Reason: existing-skill-first — no session invokes skills for harness config, and a script covers it (see Backlog); the skill would have no real home.

- **D14 — a new `worktree-lifecycle` skill.**
  - Principle: Worktrees are the one concurrency mechanism with safety rules and no lifecycle tooling.
  - Reason: existing-skill-first — `implement-spec` and `wayfinder` already own worktree use and cleanup; the missing piece is a safe-prune tool (see Backlog).

- **T7 — an untracked `core.<pid>` at the repo root is a Chromium dump, not repo content.**
  - Principle: Browser crash dumps must never be staged or committed.
  - Reason: structural — `.gitignore` can stop `git add -A` from ever seeing them, so the fix is one ignore line (see Backlog), not skill prose.

## Backlog

- **Stale-claim sweep.** Pattern: ticket status is a file flag with no reclaim path, so a dead session strands a ticket and the frontier silently skips it. Hit: tickets 44 and 46 were claimed but never resolved, and a later session had to search `.scratch` by hand. Mechanism: a small script (or gate step) listing `.scratch/**` tickets with `Status: claimed` but no `## Answer`, plus the `issues/_resolved/` convention.

- **Safe worktree prune.** Pattern: a worktree holds unique uncommitted work, but leads order deletions, so cleanup survives only on per-agent caution. Hit: four dirty worktrees with live processes, "zero removable", against standing orders to delete worktrees. Mechanism: a prune wrapper that refuses dirty trees, live-process trees, and trees with unpushed commits, plus a documented worktree home.

- **Agent-config audit.** Pattern: effective agent behaviour is a merge of bundled frontmatter, user override, and extension discovery. Hit: per-definition `thinking` levels and `researcher`'s extension access were found only by interrogation. Mechanism: a command that prints the effective merged definition (model, thinking, extensions) per agent and diffs it against intent.

- **Comment-class lint.** Pattern: the same comment-slop classes recur and are re-swept session after session. Hit: `chronicler-comment-fixer` ran six or more times in three days, each clearing the last pass's leftovers. Mechanism: encode the recurring classes (narration, paraphrase, separator) as a linter rule or pre-commit check so the sweep shrinks.

- **Ignore Chromium crash dumps.** Pattern: the managed browser leaves `core.<pid>` files at the repo root. Hit: `core.895097` confused two commit sessions, which did not know its origin or owner. Mechanism: add `core.*` to `.gitignore` (the ui-investigator ENVIRONMENT.md already explains the cause).

- **Build-cleanup scope note.** Pattern: `--cleanup` ages out a `--target-dir` tree; worktrees are not rebuildable. Hit: the 2026-10-04 worktree sweep found the repo's own criteria made every worktree non-removable. Mechanism: state the scope in the `build.py --cleanup` docstring and ENVIRONMENT.md so no future change routes worktree removal through it.