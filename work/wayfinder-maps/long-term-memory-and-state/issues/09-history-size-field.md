# Make the history size a Connection field

Type: task (AFK)
Status: resolved
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
- the cuts fall as in the `lt15` trial (`tmp/lt15/new/trial-1`): about 5 cuts, the first at about Cycle 40 (corrected on resolution: this line said 7, and the reference has 5);
- the cached share of the Narrator-role calls is near the 86 % of that trial (a report, not a gate).

This trial is also the first reference trial of this map at 16K on the new build.

## Not in scope

- A logical cut point (a scene end), and a change of the 75 % block.
- A trial at 64K. The default is chosen now, from the evidence of ticket 08.
- The scroll-back of the story in the dashboard ([the seed](../../../issues/pending/19-story-scroll-back.md)).

## Answer

Resolved on 2026-11-03. The build is done on the branch `long-term-memory` (not committed yet), and the check trial passes.

**What was built (known).**
- `LlmConnection::max_history_tokens: Option<u32>` and `resolve_max_history_tokens()`. The default is `DEFAULT_MAX_HISTORY_TOKENS` = 65,536 in `src/domain/model/settings.rs`, not `MAX_HISTORY_TOKENS` in `token_budget.rs`: the domain may not import the application layer (`arch-lint.toml`). `MAX_HISTORY_TOKENS` is gone.
- `TokenBudget` carries `max_history_tokens`. `Job::call_plan` builds the `TokenBudget`, and `Narrator::narrate` takes it. `history_space` is the smaller of the history size and the free space in the window.
- The Settings Connection form has "History (tokens)" under "Context window (tokens)" (specs 20.56 to 20.58). A blank field stores nothing.
- `data/settings.json`: `opencode-go-deepseek-v4.1-flash` has `max_context_tokens` 1,000,000. This reaches a fresh DB only. The probe DB Connection keeps no window, so 32,768.
- `scripts/trial.py`: the setup field `history_tokens` is the only place that sets the size. The script writes `max_history_tokens` in the probe DB on the Narration Connection, and on the Impersonate slot when it names its own. When the field is absent, it removes the key. The pack shows "History size". `redmist_julian.json` has `history_tokens: 16000`.
- No migration: Connections are one JSON column, and the key uses `serde(default)`.
- `CONTEXT.md` has **History space**: the tokens of story history that one call may hold when it writes a Narration, an Impersonate Input or an event Narration.
- Docs: the History section of `prompt_system.md`, and `docs/specs/settings.md`. The ticket named a Settings reference of the form and a trial doc; neither exists, and the `trial.py` docstring holds the setup fields.

**Review (known).** `/simple-after-plan`: the comment pass deleted 8 comments; the doc review gave 4 fixes; `/code-review` gave 9 fixes (the most important: `trial.py` left 16,000 in the probe DB for a later setup with no field). The full `python build.py` passes (`logs/build_20261103_220637.log`). The retro added evidence to retro issues 95 and 96.

**The check trial (known).** Pack `tmp/ltm09/redmist_julian/trial-1`, against `tmp/lt15/new/trial-1`.

| Figure | Reference | New |
|---|---|---|
| History size in the pack | — | 16,000 |
| Cuts (Cycle of each) | 5 (44, 55, 67, 78, 91) | 6 (41, 52, 64, 75, 87, 98) |
| Cached share, Narration and Impersonate | 86.0 % | 85.6 % |
| Cycles, restarts, failed calls | 100, 0, 0 | 100, 0, 0 |
| Input tokens | 2,917,705 | 2,949,300 |
| Wall time | 1054 s | 968 s |

A cut comes every 11 to 12 Cycles and drops about 21 entries in both packs. The sixth cut is there only because the first cut came 3 Cycles earlier. This pack is the first reference trial of this map at 16K on the new build.
