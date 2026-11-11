# 03 — Who authors an HTML page

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

For the HTML format, does the model author complete HTML, does a bundled
renderer build it from a compact spec, or both?

## Options

- **Option A — the model authors complete HTML.** Highest ceiling, covers any
  topic. Cost: a large artifact body, so it needs the isolation that ticket 04
  measures.
- **Option B — the model authors a compact spec; a bundled renderer emits the
  HTML.** Deterministic, cheap, and small in the transcript. Limited to the
  spec's vocabulary: boxes, arrows, timelines, tables.
- **Option C — both, chosen per format.** Spec driven for diagrams, model
  authored for the open-ended case.

## Background

- `visual-explainer` ships a JSON-spec renderer (`quick/`) next to its
  model-authored path, which is evidence both paths are useful.
- The charting recommendation was Option C. Confirm or change it.
- This ticket unblocks ticket 09, which fixes the spec vocabulary. Ticket 09 is
  meaningless if Option A wins.
- This ticket and ticket 04 inform each other. If Option B wins outright, the
  subagent question loses most of its force.

## Recommendation

Option C.
