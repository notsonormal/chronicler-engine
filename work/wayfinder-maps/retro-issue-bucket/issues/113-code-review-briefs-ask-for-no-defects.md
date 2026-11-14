# The briefs of `/code-review` do not ask for defects

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The Standards brief asks for breaches of documented standards and for smells; the Spec brief checks what the spec says. Must one brief also ask for defects (wrong behaviour) that neither covers?

## Context

Source: `/retro` 2026-11-13, the long-term-memory-and-state session `01a1cc56` (base story memory, ticket 14, and its `/simple-after-plan` pass), with the review sessions `6eb3f633` (Standards, round 1), `3a9ef9b0` (Standards, round 2) and `d4875b7c` (Spec, round 1).

- **known** Two defects were in the round-1 bundle: the story memory writer call skipped the Connection's ceiling (`..CallSettings::default()`), and one writer call took every uncovered Message with no size limit. Round 1 did not report either. The Standards reviewer quoted the first hunk only as the smell Refused Bequest.
- **known** In round 2 the lead added "Also report any real defect you see (wrong behaviour), with file:line and why". That reviewer found the unbounded writer prompt, and its test failed before the fix.
- **inferred** The spec named neither case, so the Spec review could not catch them.
- **known** The brief texts: `.agents/skills/code-review/SKILL.md:66` (Standards) and `:72` (Spec). The lead pastes them word for word.

## Options to decide

- **A.** Add the defect sentence to the Standards brief at `:66`.
- **B.** Add a third sub-agent for defects only.
- **C.** Nothing: defects are the spec's job.
