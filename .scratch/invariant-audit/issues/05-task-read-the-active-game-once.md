# Read the active Game once per request

Type: task (AFK, decide and do)
Status: open
Blocked by: —

## Question

Make the Game that the header check verifies the same Game that the handler writes (gate-02 = http-01). Choose the fix under the enforcement order of the map Notes, and put it in place.

Today `refuse_stale_game` reads the active Game pointer (`src/adapters/driving/http/utils/request_layers.rs:45`). `CurrentGame` reads it again (`src/adapters/driving/http/utils/current_game.rs:34`). A switch or a create on another thread in between makes `/action`, `/reset` or `/status/reset-busy` act on the new Game.

The candidate fixes: `refuse_stale_game` puts its one read into a request extension, and `CurrentGame` reads that extension; or the writes check the pointer under the `PointerHold`.

## Context

- Findings in full: gate-02 in [the gate report](../assets/gate.md), http-01 in [the http report](../assets/http.md). Two scouts found it on their own, and the coordinator confirmed it in the code.
- The layer is a layer on purpose: axum turns a rejected `Option<CurrentGame>` into `None` (the doc comment of `refuse_stale_game`). Keep that reason.
- The window between the two reads is small, so the test needs a seam that holds the request between the layer and the extractor. If no seam exists, say so (map Notes).
- The six end conditions of the map Notes apply. A change to `catalogue.rs` or `active.rs` runs `/verify-gate`.

## Answer
