# Add a connection test

Type: task (AFK)
Status: resolved
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

## Answer

Resolved. A Test control tests a connection on a click and reports inline.

- **What it does.** Sends one fixed short prompt to a connection and shows the
backend, the model and the reply time; a failure renders through the shared
short-message + anchored-popover shape into the surface's result slot.
- **Where.** On each Connections-list row (the saved values) and on the shared
Add/Edit form (the values typed before Save).
- **No recording.** The new `ConnectionTestService`
(`src/application/connection_test_service.rs`) builds the provider from the
connection config through an injected factory and calls `LlmProvider::complete`
directly — never `LlmCallRecorder`. No `llm_messages` row is written, so role
health and the failure banner cannot move.
- **Routes.** `POST /connections/:id/test` (row) and `POST /connections/test`
(form). Both wrap the sync provider call in `tokio::task::spawn_blocking`. A
test failure answers 200 because its target is its own `.connection-test-slot`,
so it cannot replace the row or the form.
- **Blank API key.** Blank means "no key", matching what Save stores, so the test
cannot pass on credentials Save would discard.
- **Specs and docs.** `settings.md` 20.19–20.21 and 20.1 extended;
`http_routes.md` regenerated; `dashboard.md`, `ui_design.md`.
- **Tests.** Tier 1 (`tests/http/settings.rs`) with the mock provider: pass,
failure shape, typed form values, no new forensic row and unchanged
`role_health`. Unit tests in `connection_test_service_tests.rs`. No browser test:
the server behaviour is tier-1 and the button's `hx-include` is declarative.
- **Build.** Full gate green: architecture 1, guardrails 165, integration 1567
(2 skipped), browser 68, 0 failed (`logs/build_20261007_195200.log`).
- **State.** Committed as `6077371b` on `dashboard-ui-issues-2`, together with
  ticket 64. Development commits: `46c9fbb1`, `54371f99` on `wf/t69`.
