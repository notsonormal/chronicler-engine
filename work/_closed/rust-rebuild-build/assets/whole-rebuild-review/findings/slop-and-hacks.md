# AI slop, hacks, dead code (step 6)

Source: scout `slop-and-hacks` (session 699961ee). Lead verified S-1, S-2, S-5, S-6 by reading the code.
Full evidence column is in the scout session; ids S-n follow the scout's row numbers.

## To fix in this pass (brief `fix-code`)

| # | Finding | Where |
|---|---|---|
| S-1 | Swipe-index repair: both branches no-ops (lead: verified) | `storage/mappers/message.rs` |
| S-2 | `CancellationToken` in `JobEntry` is vestigial: every canceller removes the entry in the same hold, and `writer.step()` already fails when the entry is gone (lead: verified) | `gate.rs` `cancel`/`cancelled`, `job.rs::require_alive` |
| S-3, S-27 | Comments say "its model call stops" / "observes around a model call"; false since ticket 12 option C | `gate.rs`, `job_tasks.rs` |
| S-5 | `Job::preset_text` turns a settings/preset read failure into "no preset" silently (lead: verified) | `job.rs::preset_text` |
| S-8, S-9, S-12, S-19 (part) | Dead after deletion: `GenerationStatus::is_generating`, `failure`, `GenerationFailure::with_context`, `ApplicationError::internal`, `response::service_unavailable`, `test_support::clear_write`, `GameViewQuery::active_quantifier_prompt` | various |
| S-11 | Plan vocabulary in production error text ("the rebuild", "tier 1") | `storage/utils/records.rs` |
| S-13 / K-3 | `generation_failure_summary` one-line forwarder | `http/utils/error.rs` |
| S-14 | Two preset-fallback helpers, one generic with one caller | `storage/games.rs` |
| S-15 | `reset` re-implements `create_game`'s request build (~25 lines) | `games/catalogue.rs` |
| S-21 | `render_header` hides a storage error behind "Unknown" | `app_state.rs` |
| S-22 | `ScriptedModel::hold` silently releases a first hold on a second call | `test_support/scripted_model.rs` |
| S-23, S-31 | Stale `#[allow]` attributes | `actions.rs`, `view_models.rs` |
| S-24 | `async fn dispatch_action` with no `await` | `actions.rs` |
| S-25, S-26 | "Pipeline ✓" log text; "turn" in doc comments | `narrator.rs`, `agent.rs`, `message_types.rs`, `gate.rs` |
| S-29 | Duplicate doc sentence | `http/utils/error.rs` |

## Left for the report (not fixed here)

| # | Finding | Why left |
|---|---|---|
| S-4 | `GameRulesOutcome::unknown_location`, `trigger_narration_landed`, `GameRuleRequest::NpcEvents` read only by tests | removing them rewrites the P3 property tests; owner call |
| S-6 | `games.rs` handlers `render().unwrap_or_default()` → empty `200` | pre-existing at `bc7d6cd4` |
| S-7 | Most `Message` Swipe accessors are test-only | large API surface; owner call |
| S-10, S-17, S-18, S-28, S-30 | Test-only / deliberate items recorded in ticket 07 | recorded decisions |
| S-16 | `max_tokens` threaded through 5 layers with no producer | ticket 10 option B keeps the stored field read at run time |
| S-19 (rest) | `TokenBudget::fallback`, `AgentRunner::is_empty`, `get_current_options`, `clear_dock` test-only | low |
| S-20 | Debug DTO filled in two steps | low |
| S-32 | 16 repeated arch-lint allow lines in `scripted_model.rs` | tool limit, inferred |
| S-33 | A Message with zero Swipe rows loads silently with empty text | inferred; behaviour change |

Not found (scout): no TODO/FIXME/"for now"/slice ids in production code; `validate_feature_spec.py` has no `#[ignore]` anchor path any more (P-3 closed).
