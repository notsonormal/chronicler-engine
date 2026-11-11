# Find the tools that evaluate characters and prompts

Type: research
Status: resolved
Blocked by: —

## Question

Which existing tools evaluate character cards, prompts or roleplay output, and what does each one measure? The owner recalls an ST extension that evaluates characters and tries to fix them, and something similar in Marinara Engine.

## Context

- The owner said that it is too early to define what a good output is. This research gives the later decision about criteria something real to start from. It does not decide the criteria.
- Faults seen in the first Autoplay run, as examples only: Inputs that are too long for one beat, the same pattern in every Input, a beat done twice, fact errors from the card, a stale game state ([review](../assets/impersonate-output-review.md), [transcript](../assets/autoplay-story-transcript.md)).
- Search: SillyTavern extensions (official and community lists), Marinara Engine features (agents and others), roleplay benchmarks and LLM-judge rubrics for character consistency.

## Output

An asset `assets/evaluation-tools.md`: for each tool, what it is, what it measures, how (rule, LLM judge, human), and a source link. Mark known and inferred.

## Answer

Tools exist for both halves of the question, and the owner's memory is right.

- SillyTavern has third-party card evaluators. CharacterTools, replaced by CardRefinery, scores each card field with an LLM judge, rewrites it, and then checks for drift. CardQualifier and Character Card Studio use a local rubric, with an LLM review on top. Character Card Suggestions audits a live chat for card-worthy facts. ST-jeved is the one output judge: user-written sensors plus deterministic rules that nudge, reroll or run a script.
- Marinara Engine has no story-level scorer. Its per-reply agents do the work: Card Evolution Auditor (card edits), Continuity Checker (spatial and timeline faults), Prose Guardian (banned words and repetition), and the tracker agents that re-extract state.
- The benchmarks are CharacterEval, CharacterBench, InCharacter, PersonaGym, RoleEval and RoleLLM/RoleBench. G-Eval is the rubric-plus-reasoning method, and IFEval-style checks are the rule method.

No found tool scores a whole multi-cycle story. The criteria decision stays open.

Details and a fault-to-tool mapping: [evaluation tools](../assets/evaluation-tools.md).
