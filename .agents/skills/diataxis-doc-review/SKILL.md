---
name: diataxis-doc-review
description: Break the LLM's status-quo bias when reviewing chronicler_engine docs. Judge whether each diataxis doc earns its place or is merely inherited from what's already there.
disable-model-invocation: true
---

When LLM writes documentation, it tend to write what it knows. Either from it's own knowledgebase or by reading the code. The problem is that this kind of documentation is useless, because it obvious stuff that it already knows and because reading code is a far better way to understand the code.

Getting an LLM to write good documentation is a struggle against the initial instincts of every LLM.

Another issue the LLM will look at nearby documentation to see what documentation should look like. This works for code because code at least has to compile for it's usually in a decent state. However, documentation easily rots so the entire suite of documentation can easily be trash. The problem is causes is that LLM will constantly justify the existance of bad documentation because it matches the existing document which is also bad. This can make it insanely difficult to get an LLM to fix documentation without stating exactly what the problems are in super specific detail. 

In general, it can be a struggle to get LLM to think about documentation in a systematic and holistic way. Like, I often ask it to fix something and it says "oh that makes sense and goes and fixes half of it" and then I have to constantly follow up to get it to keep fixing, repeating the exact same principles I started with. 

# Diátaxis

Read `chronicler_engine/docs/diataxis/explanation/diataxis.md` for the framework's two-axis compass (action/cognition × study/work) and the four modes it produces (Tutorial, How-to, Reference, Explanation). The compass test ("what problem does this solve for the reader?") is the basic existence check — if no reader need registers under any mode, the doc fails the test.

# What earns a doc's place

A reference doc earns its place only if it carries something the code can't. The first instinct is to read the code and describe it — pointless, because the LLM reads the code anyway when doing work, and code is reasonably self-documenting, so the description starts rotting the moment it's written.

The working test for the chronicler tree:

- `diataxis/reference/narrative/prompt_system.md` — earns its place because it describes something intrinsic to LLM chat engines: how the engine assembles the system/user message split for each call.
- `diataxis/reference/narrative/agent_system.md` and `diataxis/reference/narrative/narration_system.md` — earn their place because they provide an overarching frame, a way to view the system. This stops different systems from leaking into each other, keeping each focused on its pillar.
- `diataxis/reference/coding_standards/unit_test_standards.md` and `diataxis/reference/coding_standards/integration_test_standards.md` — earn their place because coding standards need to be consistent across thousands of LLM-generated tests.
- `diataxis/reference/game_flow.md` — earns its place because it describes a step-by-step flow that's hard to grasp without reading the code end-to-end. (`docs/specs/action_pipeline.md` covers a similar flow but lives under `docs/specs/`; it is a component spec, not a Diátaxis reference doc — do not hold it up as a diátaxis example.)

The pattern: a doc earns its place when it carries **purpose, invariants, or an overarching frame** the code does not say directly. Re-stating what the code already says — field lists, function signatures, "how the system works" summaries — is the failure mode. That is what the code does, by definition, better.

# Explanation docs

Explanation docs are the overflow for reference: more detail on certain topics without bloating the reference docs. The bar for what earns a place is vaguer than for reference, but at minimum an explanation doc should not (a) explain obvious things, (b) restate what the code does (explanation sits further from the code than reference, not closer), or (c) reiterate what the reference docs already say except to genuinely expand on them.

# How to review

Load these skills as input:

- `.agents/skills/chronicler-docs-hygiene/SKILL.md` — rule-compliance findings against `AGENTS.md ## Writing Conventions` and drift against `src/`. This is the lint layer; its findings are evidence, not the verdict.
- `.agents/skills/domain-modeling` — for the project's ubiquitous language; do not redefine glossary terms in the docs under review.

The verdict is yours, not the lint layer's: a doc can pass every rule and still fail the status-quo test. "Matches the rules and matches what's there" is not the same as "earns its place".

**Completion criterion:** every doc or section under review audited against (1) would I write this from scratch today? (2) what does it carry that the code can't? (3) does the maintenance cost over time stay less than the value it provides? Anything that fails (1) or (2) is inherited, not earned — flag for removal or rewrite.

A line of documentation carries a much higher maintenance cost than a line of code, and the value is harder to define. The status quo bias is what makes the cost invisible — the doc is already there, so keeping it feels free. It isn't.
