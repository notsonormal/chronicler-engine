# Narrator generate-then-add path

Type: task
Status: pending

## Question

Wire `/narrator <text>` to persist the narrator message AND trigger a narration generation (a continue).

Per the design synthesis (`../research/04-design-synthesis.md`, Q11):

1. `/narrator <text>` (from ticket 07's parser) persists a `MessageType::Narrator` `MessageEntry` (ticket 05) into history, then immediately triggers a narration generation.
2. The generation is a continue — no player action precedes it. Verified continue path exists (`continue_narration`, `actions.rs:27-28` → `action.rs:55`). The narrator message is in history and shapes the generation as a permanent directive (rendered bare, no prefix, per ticket 05).
3. This shares the "no-player-input generation" shape with guide (ticket 08) and plain continue.

Grounding: ST splits `/sys` (add-only) from `/sysgen` (generate-then-add); chronicler chose generate-then-add (Q11=B) on UX — a narrator direction that produces no response leaves the user wondering if it registered. Marinara has no manual narrator slash command (narrator rows are automated scene/game flows only), so it does not decide this.

Blocked by: 05 (narrator type), 07 (slash parser), 14 (specs/tests grilled and committed).
