# Make every grilling question name its lever and restate its fact

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Grilling rounds asked the user about outcomes they cannot control, and cited opaque finding ids with no underlying fact. The user replied "Q4 -> What is F17 and F18?" and "Q4 -> I don't understand what the problem you want to solve here is", and earlier "a lot of the questions on grilling me on things that I can't control? … no point in a grilling section on exactly what I want to happen, unless I can meaningfully make a change".

`grilling/SKILL.md:12` already says "Each round must carry all the context needed to answer its questions". It does not say what "context" means, nor who owns the lever.

What should a question carry?

- **A restated fact.** Every question inlines the concrete behaviour or rule at stake. No ticket number, finding id, or slug without its content.
- **A named lever.** Every question names what the answer moves, and the agent states the skill/script edit the answer would translate into.
- **Both, plus a lever-ownership test.** If the answerer cannot enforce the change, the question is the proposed skill/script edit, not a request to change habits.

## Context

- Source: `/reflect` 2026-10-07, sessions `01a10892`, `01a108c9`, `01a108d1`, `01a10db9`, `01a10e1a`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted rows 2 and 7, merged).
- `grilling/SKILL.md:12` — "Each round must carry all the context needed to answer its questions"; `:18` — the `❓ **Q1**` template.
- The rejected-alternative cost is already required by the 2026-10-04 reflect fix ([bucket map](../map.md) → "Options carry a one-line cost"); check whether that landed before re-proposing it.
