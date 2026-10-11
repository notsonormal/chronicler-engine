# Drop the `reviewer`-has-no-shell premise from the review skills

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`.agents/skills/code-review/SKILL.md:62` states a fact about the roster: "The `reviewer` definition has no shell, so the bundle is the only diff they can read." The definition in force is `/home/node/.pi/agent/agents/reviewer.md` → `tools: ["read", "ls", "find", "grep", "bash"]` (the shipped pi-herdsman default is `[read, ls, find, grep]`, so the sentence was true before the override added `bash`). The claim is now wrong, and worse, it invites a reviewer to treat the bundle as its only source: in this run both axes read the repo directly, and the pass had already built the bundle only because the skill ordered it. `chronicler-after-plan-workflow-plus-review/SKILL.md:24` repeats the premise ("the read-only three … hold no shell").

How should the skills word it?

- **State the requirement, not the roster.** "Pass the bundle; a reviewer holding bash may also read the repository, but the bundle is the frozen snapshot both axes cite." The instruction survives a roster change either way.
- **Keep the bundle mandatory, delete the reason.** The bundle stays the review's fixed input (stable diff, frozen tree) — just stop justifying it with a tool grant that no longer holds.
- **Drop the bundle step when the reviewer holds bash.** The reviewers fetch their own diff; the bundle is then a second, drifting copy. Say whether it is still worth its step.

## Context

- Source: `/retro` 2026-10-11 on the phase2-after-plan pass; the sentence was read and quoted by the review sessions and by ticket 10's reviewer.
- `.agents/skills/code-review/SKILL.md:62` — the sentence, in the §4 prompt-template step. `.agents/skills/chronicler-after-plan-workflow-plus-review/SKILL.md:24` — "because they hold no shell".
- Roster evidence: all six Lead sessions list `reviewer — … | tools read, ls, find, grep, bash`.
- Direct-read evidence: review-standards (`01a12c70-8313-7732-98d7-d5b2d07ac530`) read `CODING_STANDARDS.md`, `tests/STRATEGY.md`, `CONTEXT.md`, `docs/AGENTS.md`, `.agents/skills/*/SKILL.md` from the tree plus one bundle patch; review-spec (`01a12c70-90fa-72fb-844d-ee138e30611f`) ran `git diff c739932f..HEAD`, `grep -rn … src/ / assets/`, and read `tests/http/swipe_new.rs`, `assets/index.html` — never the bundle.
- Related: [Let the review bundle take a path list](32-review-bundle-path-scope.md), [Never tell a review axis to skip `tooling.patch`](21-review-bundle-tooling-patch.md).
