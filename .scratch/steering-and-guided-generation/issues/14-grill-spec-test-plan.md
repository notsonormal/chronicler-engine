# Grill: spec and integration-test plan for AI steering

Type: grilling
Status: pending
Blocked by: 11

## Question

Review the research asset `research/11-specs-and-integration-tests.md` (audit findings + proposed `docs/specs/steering.md` + proposed `tests/http/steering.rs`) and resolve the open questions so the spec and tests can be committed as gating artifacts for the implementation tickets.

## Open questions

1. **Scenario ID range.** The proposed spec uses `22.x`–`25.x`. Confirm this allocation or pick a different range.
2. **Impersonate preset activation.** The tests assume a new `AppSettings::active_impersonate_prompt_preset_id` field. Is this the intended mechanism?
3. **Unknown slash commands.** Should `/unknown hello` fall back to a plain player action, or should unknown slash commands be rejected?
4. **Mutual-exclusivity enforcement.** Should the parser reject a combined command, or is exclusivity only a per-turn replay-blob property?
5. **Narrator message placement.** Is `/narrator` a permanent history note that can later be edited/deleted like other messages?
6. **Impersonate output type.** ~~Should impersonate output be `MessageType::Input` (as proposed) or a distinct type such as `Dialogue`?~~ **Resolved ahead of grilling (2026-08-15): `MessageType::Input`.** Impersonate's purpose is to produce text that reads as player input — to pretend to be the player. Distinguishing it with a separate type or metadata would defeat that purpose; the output must be indistinguishable from a typed `Input` by everything downstream. The replay blob (ticket 06) still holds `impersonate=true` for retry mechanics, but that is not a visible type distinction.

## Answer

Q6 resolved ahead of grilling (2026-08-15): impersonate output is `MessageType::Input` — see the question above. Remaining questions (1–5) to be resolved during the grilling session.
