# Design the AI steering feature for chronicler_engine

Type: grilling
Status: claimed
Blocked by: 01, 02, 03

## Question

Synthesize the three research summaries (tickets 01–03) into one coherent design for chronicler_engine covering all three steering surfaces, ready for implementation tickets to graduate. Resolve each via `/grilling` + `/domain-modeling` with the human, one question at a time:

1. **Guided generation** — the prompt block format and its position in the `LayerRenderer` order (the old plan's recency-bias "after Output Format" claim is open and may be overturned by research); the threading path (`ActionForm` → `process_action`/`retry` → `pipeline_run` → `PromptContext` → `assemble`); how transience is enforced (not persisted to history); whether it applies on retry only or also new generation, and its interaction with the swipe system.
2. **Narrator Action** — whether to add a `MessageType::Narrator` variant or reuse `System`; the history-rendering format sent to the model; persistence as a permanent `MessageEntry`; the slash-command (`/narrator <text>`) entry path and how it triggers a generation.
3. **Impersonate** — how it forces the AI to write as a persona; interaction with the persona-card layer and output-format instructions (replace or augment); the slash-command (`/impersonate <persona> [text]`) entry path; threading.
4. **Prompt-layer coordination** — resolve any conflict between the three features' placements in the `LayerRenderer` order so they compose, not collide.
5. **UI** — the affordance (toggle vs slash command vs both) for each feature.

Ground each decision in the three research summaries; cite which repo's mechanism you're porting and why. The resolution is the design; implementation tickets graduate from it into the map's Not-yet-specified section.
