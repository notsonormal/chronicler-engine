# The comment finder flags comments that narrate a change

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`CODING_STANDARDS.md` says a comment never references the task. In the glossary alignment, comments such as "World files written before the Character rename", "The stored key stays `npc_id`" and "The tag … became `<available_character_ids>`" passed the comment pass (`/chronicler-comment-fixer`) and reached the code review or the lead. Should a check flag them?

- **A. `comment_finder.py` flags change words** ("rename", "renamed", "became", "stays", "no longer", "used to", "was changed") in changed comments, so the fixer sees each one. Report-only; the fixer judges.
- **B. A guardrail in the gate** that fails on the same words in added comment lines. Noisier: "stays" also states an invariant.
- **C. Leave it** to the standards review.

## Context

- Source: lead session of 2026-10-24/25 (glossary alignment, `docs/plans/glossary-alignment-plan.md`).
- Standards review `a10ad421` finding 3: `parser.rs:13`, `scenario.rs:17`, `world.schema.json:46`, and "The stored key stays …" in `swipe_state.rs`, `scene_state.rs`, `trigger.rs`, `event_record.rs`.
- Comment fixer `f32aeab7` kept the "stored key stays" lines as compatibility comments.
- Migration v31 comment in `src/adapters/driven/storage/utils/plumbing.rs` ("became"), fixed by the lead after the review.
- The script: `.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py`.
