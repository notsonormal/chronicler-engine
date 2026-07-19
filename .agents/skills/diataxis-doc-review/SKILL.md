---
name: diataxis-doc-review
description: Reviews the diataxis-doc-review
disable-model-invocation: true
---

When LLM writes documentation, it tend to write what it knows. Either from it's own knowledgebase or by reading the code. The problem is that this kind of documentation is useless, because it obvious stuff that it already knows and because reading code is a far better way to understand the code.

Getting an LLM to write good documentation is a struggle against the initial instincts of every LLM.

Another issue the LLM will look at nearby documentation to see what documentation should look like. This works for code because code at least has to compile for it's usually in a decent state. However, documentation easily rots so the entire suite of documentation can easily be trash. The problem is causes is that LLM will constantly justify the existance of bad documentation because it matches the existing document which is also bad. This can make it insanely difficult to get an LLM to fix documentation without stating exactly what the problems are in super specific detail. 

In general, it can be a struggle to get LLM to think about documentation in a systematic and holistic way. Like, I often ask it to fix something and it says "oh that makes sense and goes and fixes half of it" and then I have to constantly follow up to get it to keep fixing, repeating the exact same principles I started with. 

# Diataxis

Read `chronicler_engine/docs/diataxis/explanation/diataxis.md` to gain an understanding of the Diataxis framework. 

One of the purpose of the Diataxis framework more clearly organise the split between "understanding" (explaintion docs) and "information" (reference). However, that is just the starting point.

There are no use cases for tutorial and how-to documentation.

## Reference

When it comes to reference, the first instinct is to simply read the code and describe the code. But that is pointless because the LLM is going to read the code anything when it does work and the documentation is going to constantly become out of date.

So the real question is what value does each reference doc, and all of it's content, provide when the AI is going to read the code anyway and the code is reasonably self-documenting. 

Exactly what counts as 'good' reference docs and 'bad' reference docs hasn't been completely narrowed down yet.

The `system_prompt.md` doc is valuable because it something intrinsic to LLM chat engines. 

The `agent_system.md` and `narration_engine.md` docs are valuabl because they roviders an overarching frame, a way to view the system. For example, this should stop different systems from leaking into each other, or keep them focused on specific pillar.

The `unit_test_standards.md` and `integration_test_standards.md` are valuable for coding standards. As it's really difficult to keep thousands of LLM-generated tests consistent and aligned.

The `game_flow.md` and `action_pipeline.md` docs are valuable because they describe a step-by-step flow that is hard to understand without reading through the code end-to-end.

## Explaination

In some way, explaination docs are an 'overflow' for refrences. We need to write more detail on certain topics, but don't want to bloat the information docs. 

What counts as a 'good' explaination docs and 'bad' is even more vague then reference docs. At very least they (a) shouldn't explain obvious things, (b) shouldn't explain code closely (which be even futher away from the code than reference documents), (c) shouldn't reiterate what the reference docs already say except to expand on them.

# How to review

Consider the things mentioned here when doing this review. Also load the following skills

- `.agents/skills/chronicler-docs-hygiene/SKILL.md`
- `.agents/skills/domain-modeling`

Fundamentally, the goal is determine "is this documentation worth the cost". A line of documentation has a much high maintance cost than a line of code, the value can also be hard to define. 