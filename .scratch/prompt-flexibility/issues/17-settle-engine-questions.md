# Settle the open questions of the engine tickets

Type: grilling
Status: resolved
Blocked by: 09

## Question

Ticketing the engine work ([Write the build plan for the engine changes](09-build-plan.md)) left five owner decisions (O1 to O5). The plan review of the tickets (2026-10-31) found one more (O6). For each one, does the owner keep the recommended answer or choose another? Each engine ticket applies the recommended answer until this ticket says otherwise.

## Context

- **O1. The banner on the `Err` path.** Ticket 15 rule 3 says the `Err` path keeps the state "and shows the banner". The banner reads only LLM messages, and the `Err` path makes no model call (known). A Job cannot reach the path (inferred).
  - (a) Keep the state and log the fault, with no banner. **Recommended**: a record of a call that never happened would make the debug record false.
  - (b) Write a failed LLM message with no prompts, so the banner shows.
  - (c) Remove the `Err` path by type: give the after-narration hook its own input, with a text and a Room that are not optional. The cost is a second input type beside `HookInput`, because Options passes no text.
  - Changes [Keep the state when a state writer fails](23-keep-state-when-a-writer-fails.md).
- **O2. No Quantifier registered.** `after_narration` then returns the empty result, and every state update clears the Characters present (known). `agent_system.md` says the engine works with zero agents. **Recommended**: the same keep rule. A missing writer leaves the state as it was. Effect: a Game whose `agents` setting disables the Quantifier keeps its Characters present for ever, and no test covers that setting today. Changes [Keep the state when a state writer fails](23-keep-state-when-a-writer-fails.md).
- **O3. Routes for the Impersonate and Options slots.** Ticket 07 puts the pickers in the map's fog item "A UI for the new changeable fields". **Recommended**: no route now. The trial script reads the slots, and the UI item adds the routes and the Settings rows together. Effects:
  - On an existing database, the two slots stay empty and fall back to the narration Connection, because the settings seed writes only an absent row.
  - The preset `parameters` and `option_count` reach an existing database only through a seed file with a new preset id. Ticket 07 says "can be set through `data/` JSON": that holds for a fresh database and for new ids.
  - Changes [Add the connection slots and remove the global response length](21-connection-slots-and-response-length.md) and [Put the call parameters on the presets](22-call-parameters-on-presets.md).
- **O4. The fallback sentences.** Ticket 06 says "one rule for every layer". The empty card lists and an empty history render a sentence, not empty tags. **Recommended**: keep the sentences. They tell the model a fact (nobody is here, the story starts now), so the layer has content. Changes [Fix the prompt text](19-fix-the-prompt-text.md).
- **O5. The meaning of the Connection `max_tokens` value.** Ticket 07 calls it "a `max_tokens` ceiling (ME `maxTokensOverride`)". A ceiling and an override act in opposite ways when only the Connection sets a value.
  - (a) A cap: send the preset value or 2048, never above the Connection value. **Recommended**: it matches the word "ceiling", and the Quantifier and Options keep 2048 on every Connection. Effect: Narration on `ollama-gemma-4-26B` asks for 2048, not 8192, unless its preset sets more.
  - (b) A fallback: when the preset sets nothing, send the Connection value. Effect: the Quantifier and Options on `ollama-gemma-4-26B` ask for 8192 on an 8192-token window, and no clamp protects them.
  - Changes [Put the call parameters on the presets](22-call-parameters-on-presets.md).
- **O6. A glossary term for the connection slot.** [Add the connection slots and remove the global response length](21-connection-slots-and-response-length.md) adds four connection slots. Impersonate gets a slot and has no Role. Then the `CONTEXT.md` entry **Connection** ("One or more Roles use each Connection.") is false, and the new spec scenario needs a glossary noun (`tests/STRATEGY.md`, "Spec vocabulary"). The map Notes say "No `CONTEXT.md` changes for now" (owner, charting).
  - (a) Add a **Connection slot** entry, and change the **Connection** entry, when ticket 21 lands, with `/domain-modeling`. **Recommended**: the code then has a concept with a name, and the glossary stays true.
  - (b) Keep `CONTEXT.md` unchanged. The scenario says "the Connection of Impersonate", and the **Connection** entry stays slightly wrong until a later change.
  - Changes [Add the connection slots and remove the global response length](21-connection-slots-and-response-length.md).

## Output

A resolution with the six answers, and the edits to the engine tickets that they need.

## Answer

The owner answered all six questions on 2026-10-31, and kept the recommended answer of each one.

- **O1: (a).** The `Err` path keeps the state and logs the fault, with no banner. Applied in [Keep the state when a state writer fails](23-keep-state-when-a-writer-fails.md).
- **O2: the keep rule.** A missing writer leaves the state as it was. Applied in ticket 23, with a second test.
- **O3: (1), no new routes.** No route sets the Impersonate or Options slot. The trial script only reads the two slots. On an existing database they stay empty and use the Narration Connection. Preset parameters and the Options count reach an existing database only through a seed file with a new preset id. The routes and the Settings rows stay in the map's fog item "A UI for the new changeable fields". Applied in tickets 21 and 22.
- **O4: keep the sentences.** The fallback sentences are content, and stay. Applied in [Fix the prompt text](19-fix-the-prompt-text.md).
- **O5: (a), a cap.** Send the preset value or 2048, and never more than the Connection value. Applied in [Put the call parameters on the presets](22-call-parameters-on-presets.md).
- **O6: (a).** Ticket 21 adds a **Connection slot** entry to `CONTEXT.md` and changes the **Connection** entry, with `/domain-modeling`. The map note "Names" records this exception.

Each ticket now states the answer it applies, and no ticket waits for this one.
