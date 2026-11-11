# A delegated agent is told to run `/simple-after-plan`, which it cannot run

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A ticket said "End with `/simple-after-plan`". The coordinator put this line in the implementer brief. A delegated agent gets no `delegate` tool, so it cannot start the subagents of steps 2 to 4 of the skill. Where must the rule "the top session runs `/simple-after-plan`" live?

## Context

Source: `/retro` 2026-11-01, the coordinator session of layout-trials ticket 09 (`work/wayfinder-maps/layout-trials/issues/09-move-changing-blocks-after-history.md`), implementer session `f6c26d0f-a623-4a5e-87e2-6596387b6f7a`.

- **known** `.pi/extensions/delegate/index.ts` (the header comment): "Child (PI_DELEGATE_JOB set by the parent): registers no tools." This is by design.
- **known** The implementer ran the comment pass and a doc re-read by itself. It skipped the comment-fixer and document-review subagents, `/code-review` and `/retro`, and said so in its report.
- **known** The implementer also ran the full `python build.py` (about 5 minutes). The fixes of the later reviews made that run out of date, so the coordinator ran the full gate again. Decision 03 of this map says "the bare full gate runs once, at the end".
- **known** The layout-trials map Note "Decide, then build" says a kept candidate's task ticket "ends with `/simple-after-plan`". It does not say which agent runs it.
- **known** `/simple-after-plan` has a "Slice mode" for a slice built by an implementer subagent. The default mode does not say who runs it.
- This is not [The delegate tool is missing, so the subagent steps of the after-plan skills cannot run](93-delegate-tool-missing-from-the-session.md). In that issue the top session had no `delegate` tool. Here the top session had the tool, and the brief gave the work to a child.

## Options to decide

- **A.** One line at the top of `.agents/skills/simple-after-plan/SKILL.md`: "Run this skill in the top session. A delegated agent gets no `delegate` tool, so it cannot run steps 2 to 4. An implementer brief ends at `python build.py --no-browser`."
- **B.** A as well as a line in the "Decide, then build" Note of each map that names `/simple-after-plan`.
- **C.** A line in the `AGENTS.md` section "Subagents and delegation extra rules": a brief never names a skill that starts subagents.
- **D.** The `delegate` extension adds a note to each child's task: "You cannot start subagents."
