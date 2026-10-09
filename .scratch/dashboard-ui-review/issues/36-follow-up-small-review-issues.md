# Follow up on small issues found during review

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Code reviews and implementers on this map keep finding small issues that don't block a merge: stale references, dead code, weak assertions, stale comments. Which are still true, and what fixes them?

## Context

- This ticket collects them so they don't get lost. The coordinator appends each new item under `## Items` with its source ticket. Check each item still holds before fixing it, because later tickets may have fixed it. Drop any that no longer apply, and say so in the answer.
- Anything that grows beyond a small fix becomes its own ticket instead, added to the final re-review's `Blocked by:`.

## Items

From [Stop world creation from silently overwriting an existing key](29-world-key-silent-overwrite.md):
- **Done in [Keep the world id stable when SQLite re-seeds a world](35-sqlite-reseed-keeps-world-id.md).** ~~SQLite `create_world` refuses with a pre-check before `INSERT OR REPLACE`. It is atomic within one process only. Consider letting the `worlds.key UNIQUE` constraint (`src/adapters/driven/storage/utils/plumbing.rs` ~150) refuse, mapped to `WorldAlreadyExists`. May fold into [Keep the world id stable when SQLite re-seeds a world](35-sqlite-reseed-keeps-world-id.md) if that changes the same insert.~~
- Scenario 25.7 (`docs/specs/worlds.md`) asserts only name and key are unchanged after a refused create. Also assert description and map. Its When omits the endpoint, unlike 25.1–25.5.
- `tests/http/worlds.rs` file header does not mention create.
- `tests/storage/world_storage.rs`: the new InMemory/SQLite test pairs vary literals (`duplicate`/`dup_map` vs `sql_dup`/`sql_map`), against `unit_test_standards.md` Pattern 2. The file already deviates the same way.

From [Make the test standards docs match the code](33-fix-test-standards-drift.md) (implementer findings, unverified):
- `check_no_legacy_test_context` (`tests/infrastructure/guardrails/structure.rs`) is dead: it gates on `path.starts_with("integration/")`, and `tests/integration/` no longer exists.
- `arch-lint.toml` ~66 still lists `src/application/narrative_prompt/**` (now `src/application/prompting/`).
- `docs/diataxis/how-to/debugging.md` ~78 still points at `src/application/narrative_prompt/`.
- `PipelineHelpers` in `tests/helpers/application_ext.rs` is declared by no test binary and carries `#![allow(dead_code)]`. Delete it or wire it in.
- `docs/plans/t9-00-follow-up-3-apply-now-review-fixes-revised.md` mentions the removed `SqliteTestAppBuilder`. Likely a stale plan: archive or leave, don't rewrite history.

From [Recover from a failed message save instead of freezing the story log](27-recover-failed-message-save.md):
- The `/retrigger` 500 route in `tests/test_utils/stub_server.rs` is undriven: no stub fixture renders a retrigger button. Add the button and a 30.5-style case, or drop the route.
- `resetStatusToReady()` in `assets/index.html` hardcodes the Ready span that `onStatusPoll` and `updateToThinking` also write. One `setStatus(label, cls)` helper.
- `error_toast_text` in `tests/browser/stub/story_log.rs` duplicates `read_error_toast` in `tests/browser/stub/dashboard.rs`. Share one helper.
- Scenarios 30.4/30.5 (`docs/specs/browser_story_log.md`) use element ids in Givens and Thens, against the map's "CONTEXT.md terms until ticket 31" note; they match 30.1–30.3. Reword the section in CONTEXT.md terms, or leave it for the tier-1 migration.
- Scenario 30.5 observes dashboard chrome (status display, toast) but lives in `browser_story_log.md`; `tests/STRATEGY.md` places chrome scenarios in `browser_dashboard.md`. Arguable, since the retry control belongs to the story log.

From [Keep the world id stable when SQLite re-seeds a world](35-sqlite-reseed-keeps-world-id.md):
- The worlds column list now appears three times in `src/adapters/driven/storage/worlds.rs` (the INSERT, the `ON CONFLICT … DO UPDATE SET`, and `update_world`'s SET). One shared column list.
- `seed_world`'s world-row and map-row writes are two autocommit statements; a failed map write leaves a refreshed world with a stale map. Wrap both in `conn.unchecked_transaction()` as `utils/plumbing.rs` does. Not a regression.
- `tests/storage/world_storage.rs` header ("create/list/delete") does not mention re-seed.
- InMemory world id allocation is `last().world_id + 1`, so deleting the newest world can reuse its id. [implementer, pre-existing]

From [Fix edit mode: size, focus, keys and locked controls](11-fix-edit-mode.md):
- Clicking Edit on a different entry while one is being edited re-runs `showEditForm` without reverting: `pausePolling` overwrites `originalTrigger` with `"none"`, so the resumed poll freezes and the first entry stays in edit mode. [implementer, pre-existing]
- Four lockstep edit-mode globals (`currentEditId`, `originalText`, and the two HTML snapshots) could be one `editState` bundle.
- `tests/browser/stub/story_log.rs` copies the `.text` innerText capture three times; the `window.fetch` patch in 30.8 records every fetch (route interception would be cleaner).
- `check_edit_textarea_matches_original_height` is only valid below the 50vh cap.

From [Show Character names in the visual sidebar](22-show-npc-names.md):
- `display:flex; flex-direction:column` on `.image-container.npc-portrait` is redundant; `min-height:80px` does the work.
- The label duplicates `alt`, so a screen reader announces the name twice (`aria-hidden="true"` on the label, or empty alt).
- The escaping test pins Askama's numeric entity form (`&#38;`); assert only that no raw markup survives.
- Scenario 32.1's "one portrait per Character in the Room" is unasserted (the test seeds one NPC).
- Long names truncate at 80px with the full name in a `title` tooltip. A wrap/two-line decision belongs to ticket 19.

## Done when

- Each item is fixed or dropped with a one-line reason in `## Answer`.
- Tests follow the map's Notes. Pure comment and docs fixes need no test.
- `python build.py` is green. Commit after review.

## Answer

Every item was checked against the code first. Done unless noted.

**Ticket 29:** 25.7's When names `POST /worlds` and its Then covers description and map (test strengthened); test-file header fixed; the 29 storage pairs use identical literals. The SQLite pre-check item was already closed by 35.
**Ticket 33:** deleted the dead `check_no_legacy_test_context` guardrail and the unused `tests/helpers/application_ext.rs`; `arch-lint.toml` and `debugging.md` now say `prompting/`. *Dropped:* the stale `docs/plans/t9-00-…` doc, because it still has a pending action item and is history, not an orphan.
**Ticket 27:** one `setStatus(label, cls)` in `index.html`; one shared `read_error_toast`. *Dropped:* the `/retrigger` 500 stub route is removed rather than driven, since its failure path is the shared `submitGenerationRequest` that 30.5 already covers; the "ids vs CONTEXT.md terms" wording waits for ticket 31, and the 30.5 placement waits for ticket 32.
**Ticket 35:** one `WORLD_COLUMNS` list drives the three SQL statements; `seed_world` writes world and map in one transaction (so `create_world` is transactional too); InMemory world ids come from a monotonic counter, with a test pair; header fixed.
**Ticket 11:** one `editState` bundle replaces four globals; one `narration_text` test helper; the height assertion holds above the 50vh cap. *Dropped:* editing a second entry while one is open (needs a design, now ticket 38) and route interception for the fetch patch (needs playwright-rs plumbing).
**Ticket 22:** label is `aria-hidden`; the escaping test asserts no raw markup; 32.1 seeds two NPCs. *Dropped:* the redundant flex declarations (visual-sidebar CSS belongs to ticket 19) and name truncation (ticket 19).

**Gate:** worktree on `6db6e4d9`: `nextest: 1641 passed`, browser 30 (`build_20261001_193300.log`). Then on main with 20 and 28 applied: `nextest: 1661 passed, 0 failed, 2 skipped`, browser 30 passed (`build_20261001_195556.log`).

**Code review** (`/code-review`, no hard findings; the only must-fix was this answer). Left as judgement calls, now in ticket 37: `cls` is an abbreviation in `setStatus`; the escaping assertion no longer pins the escaped form; 32.1 counts labels, not portraits.

