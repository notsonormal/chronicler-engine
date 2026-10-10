# Frontend spec gaps and dashboard.md anchors after the doc rewrite

Status: needs-triage
Type: task

The frontend docs rewrite (plan `docs/plans/frontend-docs-rewrite-spec-boundary-and-document-review-cali.md`) moved behaviour out of `docs/diataxis/reference/frontend/dashboard.md` and deleted `ui_design.md`. The boundary is `docs/AGENTS.md` §Where a fact lives. This issue holds the follow-ups that the rewrite did not do.

## 1. Feature-level behaviour with no spec

- **Text-check preflight over HTTP** (`POST /action/check`, `POST /action/confirm`). `6916556a` removed `docs/specs/text_check.md`. The tests sit in `tests/http/requires_migration/text_check.rs` (quarantined, untagged). Browser scenarios 16.9–16.15 and HTTP scenarios 1.12–1.13 cover parts. The fail-open rule lives in `docs/diataxis/reference/game_flow.md` §Text-Check Branch.

## 2. The "Complete specs" rule

`tests/STRATEGY.md` §Spec scenarios says: "A spec covers each distinct response its route can return." The owner's position is that specs cover features and do not need to cover every single scenario. The two can both hold, because the rule is per route and says nothing about fine-grained UI behaviour. Decide whether the rule needs a sentence that says so.

## 3. Application-layer DOC anchors to `dashboard.md`

These files carry `//! [DOC: docs/diataxis/reference/frontend/dashboard.md]`, but their subject is not the dashboard. Decide the doc each one should point at, or drop the anchor:

- `src/application/settings_service.rs`
- `src/application/prompt_preset_service.rs`
- `src/application/connection_test_service.rs`
- `src/application/generation/guard.rs`
- `src/utils/settings.rs`
