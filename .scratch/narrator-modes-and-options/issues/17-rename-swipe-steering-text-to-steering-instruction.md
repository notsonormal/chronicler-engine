Type: task
Status: resolved
Blocked by: (none)

# 17 Task: rename the swipe's merged steering text to `steering_instruction`

## Question

Ticket 16 merged the guide text and the impersonate direction into one Swipe property and named it `direction`. The post-merge code review flagged the name: neither `guide` nor `direction` says what the field holds, and `direction` collides with the domain's map `Direction` enum (room-exit compass directions, `src/domain/model/map.rs`).

### Scope

1. Rename the merged-value chain to `steering_instruction` — compound per the source vocabulary (Marinara wraps the typed text as "Guided generation instruction"; the GuidedGenerations extension calls it "a transient, per-generation instruction"): `Swipe.steering_instruction`, the accessor, `set_stored_inputs`, `ImpersonateInputs.steering_instruction`, and the merged-pair params (`run_from_input`, `execute_action_with_inputs`, `push_message`, `add_message_with_inputs`, `seed_swipe_with_stored_inputs`).
2. Storage: `DbSwipe`, mappers, INSERT/SELECT column lists, migration v23 (guarded `RENAME COLUMN direction TO steering_instruction`), `user_version` 23. v22's SQL and the historical blob JSON keys are untouched.
3. `GenerationInputs.guide` keeps its name — it is the guide-mode slot paired with the `impersonate` slot; the merged name belongs to the Swipe shape only.
4. Docs: CONTEXT.md Swipe entry plus a Steering deprecated-term carve-out, `ai_steering.md`, `storage.md`. The `/impersonate <direction>` command-grammar placeholder is unchanged.
5. New unit test: simulated v22-shape database renamed forward; chain-end version assertions move 22 to 23.

### Notes-for-the-session

- The user directed the source-vocabulary re-read (Marinara + GuidedGenerations extension) and explicitly relaxed the glossary's Steering ban for this usage; the ban stays for the feature umbrella.
- v23's guard skips the rename when `steering_instruction` already exists: the simulated-old-DB tests re-run the chain on tables already at the final shape, and v22's presence-guard re-adds `direction` there.

## Answer

Shipped as scoped. `python build.py` fully green.

### Facts for later tickets

- The v22 backfill test's simulation must rename `steering_instruction` back to `direction` before adding the replay column — after v23, the fresh-run table has no `direction` column to backfill into.
- The `/impersonate <direction>` slash-command grammar and the `Action::Impersonate` payload binding keep the `direction` name; only the stored/staged value chain renamed.
