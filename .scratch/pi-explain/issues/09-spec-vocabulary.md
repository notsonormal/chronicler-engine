# 09 — The compact spec vocabulary

Type: grilling
Status: open
Blocked by: 03
Assignee: (unclaimed)

## Question

If a compact spec drives any rendered artifact, what can the spec express?

This ticket is meaningless if ticket 03 chooses model-authored HTML only. It
opens only after that answer.

## What the vocabulary must cover

Judge against the requests the extension is meant to serve: explain a system,
show a flow, compare options, summarise a sequence over time.

Candidate primitives:

- Labelled boxes with a role or colour.
- Arrows between boxes, labelled.
- Grouping into regions or layers.
- A vertical or horizontal sequence, for timelines and pipelines.
- A two-column comparison table.
- A short caption block.

## What to decide

1. Which primitives make v1.
2. Whether the spec is JSON with a schema, and whether the schema is validated
   before render. `visual-explainer`'s `quick/schema.json` is prior art.
3. How the model learns the vocabulary. A tool parameter description is the
   smallest surface; a skill file is the most complete.
4. What happens when the model emits an out-of-vocabulary spec: reject, or
   degrade to Markdown.

## Background

- `visual-explainer` validates a compact JSON spec and renders it with a bundled
  local renderer. That is the closest prior art.
- The spec path exists to keep the artifact body small, so the spec must stay
  small. A vocabulary that needs a large spec defeats its own purpose.

## Recommendation

Six primitives, JSON, schema validated, description carried on the tool
parameter. Reject an invalid spec and fall back to Markdown.
