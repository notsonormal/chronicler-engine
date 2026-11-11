# Copy sweep

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Fix the UI copy problems from the review:

- "Add LlmProviderConfig" (a Rust type name) as a heading and button in Settings (finding 5.1).
- "(1 games)", and the leading "- " before world descriptions (finding 5.2).
- The jargon "Active · IF" and "(transient)" in the slash menu (finding 5.4).
- New Game world/persona selects with no labels. The "CURRENT" badge repeats the "Active Game" heading. "No saved games." shows while one game exists, because the list holds only non-active games (finding 5.5).

## Context

- Use `CONTEXT.md` terms in user-facing copy where they fit, for example "Guided Generation" rather than "transient".
- Game naming (finding 5.3) is in [Decide how games are named and renamed](21-decide-game-names.md), not here.

## Done when

- Each item is fixed. Existing test assertions on the old strings are updated.
- `python build.py` is green. Commit after user approval.

## Answer

Copy fixes, one per item:
- **5.1** `Add LlmProviderConfig` heading and button → `Add Connection`; the "LlmProviderConfig not found" error toasts (six sites) → "Connection not found".
- **5.2** World list: `(1 games)` → `(1 game)` (plural otherwise); the stray `-` before each description is gone.
- **5.4** `(transient)` → `(Guided Generation)` in the slash menu (CONTEXT.md term, matches `Action::Guide`). `Active · IF` → `Active · Interactive Fiction`, and `Set Active (IF)` → `Set Active (Interactive Fiction)` for consistency with the edit checkbox.
- **5.5** New Game World and Persona selects have `<label for>`; the redundant Current badge is gone (its CSS too); `No saved games.` → `No other saved games.`, which is true because the list holds only non-active games.

**Tests:** tier 1. New scenario `worlds.md` 25.8 with `test_worlds_fragment_pluralises_game_count_http` (singular and plural arms); `games.md` 20.8 and its test extended (no-badge, "No other saved games."). Pinned assertions, fixtures and specs for the old strings updated. `dashboard.md` Games section brought in line.

**Gate:** worktree on `6db6e4d9`, nothing merged since: `nextest: 1645 passed, 0 failed, 2 skipped`, browser 30 passed (`build_20261001_184748.log`).

**Code review** (`/code-review`, ISSUES → fixed): stale `dashboard.md`, the remaining type-name toasts, the orphaned `.game-badge` rule, the untested plural arm, and an undefined CSS variable in the new label rule.

**Left for a later follow-up pass (judgement calls):** 20.8 now also holds saved-games assertions, which is off-topic for "renders the posture controls" and should be its own scenario; the games test asserts the class `game-badge` is absent where the spec says "no status badge"; "No other saved games." has no referent when no game is active (unreachable in production); the "Interactive Fiction" literal is repeated at about 10 sites; `tests/http/games_fragment.rs` has a comment citing a `games_tests` module that does not exist.

