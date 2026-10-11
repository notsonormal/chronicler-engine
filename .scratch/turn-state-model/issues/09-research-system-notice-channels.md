# Research: route each System notice to an existing channel

Type: research
Status: claimed
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
