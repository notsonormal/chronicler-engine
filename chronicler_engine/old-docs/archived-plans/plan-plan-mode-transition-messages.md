# Plan: Plan-mode transition messages

## Summary
Replace the disabled-marker scan with hidden `<pi-plan>` transition messages sent on enter/exit (and session_start when enabled). Keep the planning prompt injection in the system prompt untouched. Keep the earlier `BLOCKED_BUILTIN_TOOLS` fix (it was a separate bug).

## Key Changes
- Remove `PLAN_CONTEXT_MARKER_DISABLED` constant and the assistant-message scan in `before_agent_start`.
- Restore `before_agent_start` disabled branch to its original `if (!state.enabled) return;`.
- Add constant `PLAN_MODE_TRANSITION_MESSAGE_TYPE = "pi-plan"`.
- In `enterPlanMode`, after `persistState()`, send a hidden message:
  `<pi-plan>Entering plan mode</pi-plan>`
- In `exitPlanMode`, after `persistState()`, send:
  `<pi-plan>Exiting plan mode</pi-plan>`
- In `session_start`, when `state.enabled` becomes true, send the entering message once.
- All messages use `display: false` and `{ triggerTurn: false }`, matching the existing `proposed-plan` pattern.
- Do not filter `pi-plan` messages in the `context` handler.

## Implementation

### Phase 1: Revert marker/scan

- [ ] #### Task 1.1: Remove `PLAN_CONTEXT_MARKER_DISABLED` and scan block (1 SP)
  - [ ] Delete the `PLAN_CONTEXT_MARKER_DISABLED` constant.
  - [ ] Replace the disabled branch in `before_agent_start` with `if (!state.enabled) return;`.

### Phase 2: Add transition messages

- [ ] #### Task 2.1: Add transition constant (1 SP)
  - [ ] Add `const PLAN_MODE_TRANSITION_MESSAGE_TYPE = "pi-plan";` near the other constants.
- [ ] #### Task 2.2: Emit entering message (1 SP)
  - [ ] In `enterPlanMode`, after `persistState()`, call `pi.sendMessage` with the entering content.
- [ ] #### Task 2.3: Emit exiting message (1 SP)
  - [ ] In `exitPlanMode`, after `persistState()`, call `pi.sendMessage` with the exiting content.
- [ ] #### Task 2.4: Emit entering on session_start when enabled (1 SP)
  - [ ] In `session_start`, when `state.enabled`, send the entering message after `activatePlanModeTools()`.

## Test Plan
- `npm test` in `.pi/extensions/pi-plan-mode` must still pass (33/33).
- Manually: enable plan mode → confirm a hidden `<pi-plan>Entering plan mode</pi-plan>` message appears in the session log (model-visible, not in UI). Exit → confirm `<pi-plan>Exiting plan mode</pi-plan>` appears.
- Verify the planning prompt is still injected in the system prompt on every turn while enabled.
- Verify the `BLOCKED_BUILTIN_TOOLS` fix still allows `edit`/`write` inside allowed folders.

## Assumptions
- The `pi-plan` customType will not be filtered by any other extension.
- Hidden messages (`display: false`) remain in the conversation history and are visible to the model.
- The planning prompt injection in the system prompt is preserved exactly as before.
