# Research: outside check, a second reader

Type: research
Status: open
Blocked by: 05, 06, 07

## Question

Does the framework note give the same answers to a reader who is not the owner? This is the map's second outside check (map Notes).

- A fresh agent from another model family (role `judge-glm` or `judge-qwen`) reads only the draft note (`assets/framework-note-draft.md`) and the `## Question` sections of [Prototype: worked case, `CONTEXT.md`](06-prototype-worked-case-context.md) and [Prototype: worked case, the concept catalogue](07-prototype-worked-case-catalogue.md).
- It does not see the answers of those tickets, their assets, or the charting notes. It applies the note and writes its own answers.
- Then the session compares those answers with the worked cases: where they agree, where they differ, and why. A difference can come from a note that is not clear, a reader that differs, or a case that is still open.

## Done when

`assets/research-08-second-reader.md` holds the second reader's answers, the comparison, and what the comparison suggests about the note. It does not rule.
