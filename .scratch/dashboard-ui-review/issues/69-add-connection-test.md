# Add a connection test

Type: task (AFK)
Status: open
Blocked by: 68

## Question

Add the connection test decided in [Settings panel: roles, buttons and text-check controls](15-settings-panel-prototype.md) (Q3 → A, Q4 → C).

- **What it does.** Test sends one short fixed prompt to a connection and shows the result inline: success with the reply time, or the error through the shared short-message-plus-popover shape (`error_disclosure`).
- **No recording.** A test writes **no** `llm_messages` row, so it never changes role health or the failure banner. Ticket 08 defines health as a role's newest real attempt, and a short prompt can pass where a real call fails. So the test must not go through `LlmCallRecorder`, which saves every attempt; call the provider built from the connection config directly.
- **Where.** On each row of the Connections list (tests the saved values), and on the shared Add/Edit form (tests the values typed in the form, before Save).
- **Cost.** It runs only on a click. Never on page load, never on a timer.

## Context

- Needs the Connections page shape from [Split Settings into Connections and Text Check sub-tabs](68-split-settings-sub-tabs.md).
- Wiring: `provider_from_config` and `recorder_with_storage` in `src/bootstrap/wiring.rs`; the port `LlmProvider::complete` in `src/application/ports/llm_provider.rs`. The handler belongs behind an application service, not in the driving adapter. Check `arch-lint.toml`.
- Form-values test: the API key field may be blank on Edit while a key is saved. Decide whether a blank key means "use the saved key".

## Tests

- Tier 1 with the mock provider: a passing test returns the success result; a failing test returns the error shape; after a test, the LLM Messages read seam (`GET /fragment/llm-messages` or `GameViewQuery::list_latest_llm_messages`) shows no new row, and `role_health` is unchanged.

## Done when

- `python build.py` is green, the user has reviewed the diff, and it is committed through `/commit-and-push`.
