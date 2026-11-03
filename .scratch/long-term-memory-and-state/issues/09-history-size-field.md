# Make the history size a Connection field

Type: task (AFK)
Status: open
Blocked by: 08

## Question

Build the decisions of [Decide the size and the point of the history cut](08-history-cut.md), test first: the history size becomes a field of the Connection with a default of 65,536 tokens, a trial sets it from its setup file, and a person sets it in the Settings form.

Done when the full `python build.py` passes and the check trial below passes.

## Decisions (from ticket 08)

- **The field.** `max_history_tokens: Option<u32>` on the Connection (`src/domain/model/settings.rs`), next to `max_context_tokens`, with `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- **The default.** `MAX_HISTORY_TOKENS` (`src/application/prompting/token_budget.rs`) becomes 65,536. An empty field uses it.
- **The rule stays.** The history space is the smaller of the Connection size and the room in the window (`TokenBudget::history_space`). A cut still drops whole Messages from the front, in blocks, down to 75 % (`HistoryCut`). No logical cut point.
- **The OpenCode Go window.** `data/settings.json`: `opencode-go-deepseek-v4.1-flash` gets `max_context_tokens` 1,000,000.
- **The form.** A "History (tokens)" field in the Settings Connection form, under "Context window (tokens)", with the placeholder "Empty uses the default (65,536)". A blank field stores nothing, as the window field does.
- **The trial.** A `history_tokens` field in the setup file. `scripts/trial.py` writes it on the Connection of the probe server before the run, and the pack shows it. `scripts/trial_setups/redmist_julian.json` gets `history_tokens: 16000`. A trial size is separate from the default.

## Facts (known on 2026-11-03)

- Connections are stored as one JSON column of the `settings` table (`src/adapters/driven/storage/settings.rs`). Every optional Connection key uses `serde(default)`. So the new key needs no migration (inferred: confirm it while you build).
- `TokenBudget` carries `max_context_tokens` and `max_tokens`. `Job` builds it from `connection.resolve_max_context_tokens()` (`src/application/pipeline/job.rs`). The history size reaches `history_space` the same way.
- The form already has `conn_max_context_tokens` (`src/adapters/driving/http/settings/templates/settings.rs`, `handlers/settings.rs`, `resolved_max_context_tokens`). Follow that shape.
- `trial.py` names its Connections by id (`connections` in the setup file) and checks them in the probe DB (`check_connections`). It changes no field today.
- The Narrator seam cases of the cut are in `narrator_tests.rs` (`CASE narrator-history-cap`, `narrator-history-small-budget`, the `narrator-history-cut-*` cases). `narrator-history-cap` uses `MAX_HISTORY_TOKENS`, so the new default changes what it builds.
- Specs 20.25 and 20.26 (`tests/http/settings_connection_form.rs`, `docs/specs/settings.md`) cover the window field of the form.

## Steps

1. Read `tests/STRATEGY.md` ("The one rule") before you add a test. Add a seam case: a Connection with `max_history_tokens` set cuts at that size, and an empty field cuts at the default. Then a form case for the new field (save, blank, invalid). Then a `trial.py` Python test for `history_tokens` (read, write, shown in the pack).
2. Build the field, the default, the form, `data/settings.json` and `trial.py`.
3. **Glossary.** "History space" is not in `CONTEXT.md`. Add it with `/domain-modeling` when the field lands: the tokens of history that one Narrator-role call may hold.
4. **Docs.** Change what the change makes wrong (`docs/AGENTS.md`, "Direction of edits"): the History section of `docs/diataxis/reference/narrative/prompt_system.md`, the settings spec and the Settings reference of the Connection form, and the setup-file fields of the trial in its docs.
5. `/simple-after-plan`: the comment pass, the doc review, `/code-review`, the full `python build.py`, and the retro.

## Check

Run one 100-Cycle trial of `scripts/trial_setups/redmist_julian.json` (`history_tokens: 16000`) on the built change. The setup file is the only place that sets the size. Pass when:

- the pack shows the history size 16,000;
- the cuts fall as in the `lt15` trial (`tmp/lt15/new/trial-1`): about 7 cuts, the first at about Cycle 40;
- the cached share of the Narrator-role calls is near the 86 % of that trial (a report, not a gate).

This trial is also the first reference trial of this map at 16K on the new build.

## Not in scope

- A logical cut point (a scene end), and a change of the 75 % block.
- A trial at 64K. The default is chosen now, from the evidence of ticket 08.
- The scroll-back of the story in the dashboard ([the seed](../../story-scroll-back/idea.md)).
