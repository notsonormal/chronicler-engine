# Research: route each System notice to an existing channel

Type: research
Status: resolved
Blocked by: —

## Question

For each source of a `MessageType::System` Message, what does the player see today, what does the source report (a failure, a fallback, a world-data gap), and which existing channel (the status line, the degraded-role banner, the tracing log) could carry it? What do Games with stored `System` Messages need if the type goes away?

## Scope

1. Confirm the five sources in [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md) and find any others (grep `MessageType::System` and the `[System]` prefix).
2. For each source: the trigger condition, the text, whether a generation fails or continues, and whether `GenerationStatus::Error` or a `GenerationFailureKind` already reports the same event.
3. The existing channels: what each shows, how long it shows it, whether it survives a page reload, and how it is stored (`GenerationStatus`, LLM Messages rows, tracing). Use `docs/specs/failure_display.md` and the ticket-10 work in [Report a failed trigger continuation without a story-log Message](../../dashboard-ui-review-phase2/issues/10-report-failed-trigger-without-message.md) as the pattern.
4. Stored data: how `System` Messages are saved (with a Snapshot), how many code paths would need a render rule or a migration, and the migration pattern in `adapters/driven/storage/utils/plumbing.rs`.
5. Tests and specs that assert a `System` Message: list them.

## Output

An asset `../assets/research-09-system-notices.md`: one row per source (text, kind, current display, candidate channel, tests that assert it), a description of each channel, and the stored-data note. Label each fact known, inferred, or tested. Cite symbols, not line numbers.

## Feeds

[Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md).

## Answer

Asset: [research-09-system-notices.md](../assets/research-09-system-notices.md). Facts are known from the code unless marked. Five sources, three channels, one stored-data trap, 24 asserting tests and specs.

- **The five source list is complete.** Every site is `GameState::add_message(…, MessageType::System)`. No sixth exists: no action handler, bootstrap seed, arrival path or agent builds one. The five are the NPC-detection fallback (`PipelineRun::phase_post_generation`), `ActionPipeline::finalize_options_failure` (three flavours, the `/options` path), `ActionPipeline::rewrite_options_after_turn`'s empty result, `rewrite_options_after_turn`'s `Err(e)`, and `GameState::attempt_movement`'s unknown room.
- **No source sets `GenerationStatus::Error`.** A failed optional step finishes the turn as non-error by design. So the status line — the only per-Game, persisted channel — carries none of them today, and the ticket-10 pattern (remove the notice, rely on `GenerationStatus::Error` plus the banner's failed row) fits none of them as-is.
- **The banner is narrower than it looks.** It reads the newest `llm_messages` row per role, so it carries only a *failed LLM attempt*: it sees #1's and #2's and #4's transport flavour, and #1's failed-call flavour under a different sentence. It cannot carry a source that makes no LLM call (#3, #5) or one whose row says success (a zero-parse answer, or #1's low-confidence flavour). It also ages out: 50 rows engine-wide, then the sentence disappears.
- **The tracing log is the weakest channel.** #2 and #4 log at `warn!`; #3 logs nothing; #5 logs `debug!` under a default `info` filter; #1 logs nothing.
- **A render rule is not enough for stored Games.** The message mapper parses `message_type` with `serde_json::from_str(...)?` **inside** `load_message_rows`' row loop, and `MessageService::load_messages_into_state` swallows the error, so one legacy `System` row silently blanks the whole history (and `load_or_fresh` can rebuild from the scenario). Removing the variant needs a migration in the `run_migrations` shape of v22–v26, tested in `plumbing_tests.rs` the way `test_v22_backfills_swipe_inputs_from_replay_blob_and_drops_column` seeds its rows. Once legacy rows are migrated, only the deleted variant needs work: 4 exhaustive `match message_type` sites, the type-guarded predicates, 2 CSS rules and the mapper.
- **Ticket 02's list holds; two of its framings do not.** (a) The NPC-fallback notice is *not* the only player-visible record of that degradation for the failed-call flavour — the banner already says "Quantifier failed — using fallback NPC IDs"; only the low-confidence success flavour is unreported (this corrects ticket 10's Answer). (b) The fallback fires only when the room's in-area list is empty (inferred), so a quantifier failure in an occupied room is silent; ticket 02 treats it as one uniform notice.

Six open points go to the [grilling](02-replace-system-messages.md), not to a new ticket: whether each source needs the player to see it at all; which channel per source, and whether a non-fatal notice needs a new status value, a new `GenerationFailureKind`, or a slot beside `input_buffer.status`; the banner's role-effect wording if sources leave it; the migration target value for legacy rows; the `CONTEXT.md` and `storage.md` wording; and who owns the notice text (engine prose versus raw `EngineError` text).
