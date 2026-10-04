# Mechanism notes from the first filing

The 8 files first filed under `.scratch/reflect-backlog/issues/` as `Type: task (AFK)` / `Status: needs-triage`. That filing was wrong: no map, wrong status vocabulary, and decisions pre-made. They are kept here as evidence only. Their proposals are **not** decisions. Each ticket's grilling decides.

---

<!-- was: issues/01-stale-claim-sweep.md -->

## Sweep `.scratch` for tickets claimed but never resolved

Type: task (AFK)
Status: needs-triage
Blocked by: —

### Question

Can a script list every tracker ticket that carries `Status: claimed` without an
`## Answer`, so a dead session's claim cannot silently hide a ticket from the frontier?

### Context

`docs/agents/issue-tracker.md` defines the frontier as files that are "open, unblocked, and
unclaimed". A claim is a file edit, so a session that dies mid-ticket leaves `Status: claimed`
forever, and the frontier rule then skips that ticket without a word.

Hit on 2026-10-04: tickets 44 and 46 in `.scratch/dashboard-ui-review/` were claimed but never
resolved and never written up. Recovering them required searching session transcripts to
reconstruct what had been implemented. A follow-up sweep searched all of `.scratch` by hand.

Also missing: the tracker doc does not record that resolved tickets move to
`issues/_resolved/`, although that directory exists on disk (28 files moved there on
2026-10-04 with a plain `mv`, not `git mv`).

Changes:

- Add a script (or a `scripts/healthcheck.py` sub-dispatch) that prints every ticket under
  `.scratch/**` with `Status: claimed` and no `## Answer` heading, with its path.
- Add the `issues/_resolved/` convention to the Resolve step in
  `docs/agents/issue-tracker.md`, including that the move uses `git mv`.
- Decide whether the sweep runs on demand or as part of an existing gate step. It must not
  fail the build — a claim without an answer is legitimate while an agent is working.

---

<!-- was: issues/02-safe-worktree-prune.md -->

## Add a safe `git worktree` prune with dirty/process/unpushed checks

Type: task (AFK)
Status: needs-triage
Blocked by: —

### Question

Can worktree cleanup become a tool with safety checks, instead of a standing order to
"delete the worktree" that individual agents must refuse on their own judgement?

### Context

A worktree holds unique uncommitted work; a `--target-dir` tree does not. Cleanup today
survives only on per-agent caution.

Measured on 2026-10-04: four live worktrees, all dirty or unmerged, with 52 processes rooted
under `/workspace/ce-*`. Under the repo's own criteria, zero were removable. Meanwhile the
operator's notes across the window include "Remvoe the worktree", "delete the worktree and then
push the commits", and "d) delete the worktrees".

There is no tooling that creates worktrees either — `git worktree add` appears nowhere in
`build.py`, `scripts/`, or `.agents/` — and no documented home for them. Live worktrees were
found under three different naming schemes (`/workspace/ce-wt/t34`, `/workspace/ce-t43`,
`/workspace/wt65`), and the parallel-ticket prompts hand-retype the worktree contract
(including "NEVER run `git stash`, `git checkout`, `git restore`, `git reset`, `git clean`").

The repo already declined automatic worktree removal once, on the grounds recorded in
`.scratch/_closed/ui-verification-redesign/issues/13-branch-temporary-code-sweep.md`: it would
delete another agent's in-flight build. That was the right call; this ticket supplies the
missing follow-through.

Changes:

- Add a prune wrapper that refuses a worktree with uncommitted changes, a live process rooted
  in it, or unpushed commits, and prints why it refused. Never prune on an age rule.
- Document one worktree home and naming scheme.
- Keep `git worktree remove` out of any automatic housekeeping path.

---

<!-- was: issues/03-agent-config-audit.md -->

## Print the effective merged agent definition, per agent

Type: task (AFK)
Status: needs-triage
Blocked by: —

### Question

Can one command print, for each agent definition, the effective model, thinking level and
extension access after the bundled-frontmatter / user-override / extension-discovery merge, so
drift is visible without interrogating a live agent?

### Context

Agent behaviour is a merge product, so the file you edit is not always the config that runs.
The drift stays invisible until someone asks or a rollout fails.

Observed on 2026-10-03: the per-definition `thinking` values (`reviewer` xhigh, `scout` medium,
generalist/implementer/scout high) were discovered only because the operator asked "What is the
model set of all of the subagents now?". At the same time `researcher` was found to run without
`--no-extensions` and therefore discover every global package. A provider 429 was diagnosed
only by discharging a live agent, and an earlier plugin update had to be re-applied from memory.

Changes:

- Print the effective merged definition per agent: source files, model, thinking level,
  `noExtensions` state, and the resolved values of any global maps that apply.
- Diff it against intent, or at least against the previous run.
- Keep it out of the gate; run it after any plugin, agent-definition, or settings change.

---

<!-- was: issues/04-comment-class-lint.md -->

## Enforce the recurring comment-slop classes in the linter, not by repeated sweeps

Type: task (AFK)
Status: needs-triage
Blocked by: —

### Question

Which comment classes that `chronicler-comment-fixer` re-finds every sweep can be detected by
a rule instead of by a whole-file review pass?

### Context

Comment hygiene deployed as a post-hoc sweeper is a mop: the rules live where slop is *found*,
never where comments are *authored*, so the same classes recur.

`chronicler-comment-fixer` ran six or more times in three days (2026-10-03 at 20:46, 22:03,
22:09; 2026-10-04 at 18:14, 18:21, 18:25). Each pass noted that an earlier session had already
trimmed most of the same material. The recurring classes are stable and mechanical:

- narration of what the code does,
- comments that paraphrase the code,
- stale cross-file references,
- task references in comments (`(ticket 11)`, "See ticket 30") — already banned by
  `CODING_STANDARDS.md`,
- separator/decorative comments.

The last class is already enforced; the others are found by reading. `CODING_STANDARDS.md`
§ Code comments states the rules; nothing checks them.

Changes:

- Pick the classes that are mechanically detectable without false positives.
- Add them to an existing linter/pre-commit path rather than a new tool.
- Leave taste-dependent classes (why-comment quality) to the review pass.

---

<!-- was: issues/05-ignore-chromium-core-dumps.md -->

## Ignore Chromium `core.<pid>` crash dumps

Type: task (AFK)
Status: needs-triage
Blocked by: —

### Question

Should `.gitignore` exclude the `core.<pid>` files the managed browser leaves at the repo root?

### Context

The managed/ssh Chrome launch in this container leaves core dumps at the repo root, roughly
27 MB each. The cause is already documented in
`.agents/skills/chronicler-ui-investigator/ENVIRONMENT.md`: "the managed browser leaves core
dumps behind (`core.<pid>` in the repo root, ~27 MB each …)".

Two commit sessions on 2026-10-04 hit one and did not know what it was or who owned it. One
excluded it by hand from staging; the clean `git add -A` path would have staged it.

Changes:

- Add a `core.*` rule to `.gitignore`.
- Confirm it does not mask a directory or source file the repo actually tracks.

---

<!-- was: issues/06-build-cleanup-scope-note.md -->

## State the `build.py --cleanup` scope so worktrees never route through it

Type: task (AFK)
Status: needs-triage
Blocked by: —

### Question

Can the `--cleanup` docstring and `ENVIRONMENT.md` say plainly that cleanup ages out
`target/<name>` trees only, and never a worktree?

### Context

A stale `--target-dir` tree is rebuildable debris and safe to age out. A worktree is not
rebuildable — uncommitted work there is unique.

On 2026-10-04 the operator asked whether cleanup could be folded into `build.py`, and settled
on: "it's fine to leave the standard build alone and focus on the randomly generated builds
from `--target-dir`". The agreed retention is 3 days, skipping the default `target/debug`.

`build.py`'s cleanup path already removes only the target dir and stale port locks, so the
behaviour is right. The gap is that nothing states the scope, which invites a future change to
route worktree removal through it.

Changes:

- State the scope in the `build.py --cleanup` docstring.
- State it in the `ENVIRONMENT.md` section on what forces a rebuild.
- Cross-reference the safe-prune ticket for worktrees.

---

<!-- was: issues/07-precommit-collision-path.md -->

## Give the pre-commit hook a non-destructive path when its generated files are dirty

Type: task (AFK)
Status: needs-triage
Blocked by: —

### Question

When a generated file already carries another agent's unstaged edits, can the pre-commit hook
resolve that itself instead of aborting and leaving the agent to improvise?

### Context

`scripts/git-hooks/pre-commit` regenerates four generated files (`AGENTS.md`,
`tests/AGENTS.md`, `docs/AGENTS.md`,
`docs/diataxis/reference/coding_standards/guardrails.md`) and re-stages them. If any of the
four already has unstaged changes, it aborts with "cannot safely re-stage it", because the
regenerated output would sweep unrelated prose into the commit.

In a shared checkout with concurrent agents this collision is routine, and the hook offers no
resolution. Two sessions on 2026-10-04 resolved the same collision in opposite ways:

- one ran `git commit --no-verify`, deliberately, because satisfying the hook would have swept
  the other work into the commit;
- one staged the other agent's unstaged work as well, on the reasoning that it belonged in the
  commit.

Both reasoned correctly from the evidence available. The hook just has no answer for the case it
detects.

Changes:

- Consider regenerating into a temp path and comparing, so a dirty generated file is reported
  rather than blocking.
- Or add a documented flag/env var that skips regeneration for the commit and says what it
  skipped.
- Whatever the choice, document it in `.agents/skills/commit-and-push/SKILL.md` so the next
  agent does not improvise.
- Keep the safety property: never sweep an unrelated agent's unstaged edits into a commit.

---

<!-- was: issues/08-benchmark-contention-condition.md -->

## Record the contention condition and effect size in the benchmark report

Type: task (AFK)
Status: needs-triage
Blocked by: —

### Question

Can `scripts/diagnostic_benchmark.py` (and the gate's step-timing output) record what else was
running on the box and the run-to-run spread, so a reader can tell a real change from noise?

### Context

The gate runs on a shared, contended host. Single runs are not comparable across sessions.

Observed in the window: a 663 s gate with clippy at 220 s and architecture tests at 236 s, all
described as "the only thing running on the box" when it was not. A retro finding was
subsequently retracted as hand-waving ("My 'slot acquisition' framing was hand-waving …
Candidate 4 is not a finding. Drop it"). The operator had to ask for repeated runs before the
numbers meant anything, and remarked that single runs on a busy box should be read for ranking,
not for exact seconds.

Meanwhile the cheapest wins were not measured at all, because measuring them cost more than the
fix.

Changes:

- Emit the contention condition with each timing: concurrent build-slot holders, load average,
  and the run-to-run spread when a step is repeated.
- Flag any delta below a stated threshold as within noise rather than reporting it as a
  speedup.
- Note the measurement-cost-versus-fix-cost rule in the report so cheap fixes are not blocked
  on a benchmark.
