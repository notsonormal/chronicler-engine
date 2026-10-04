# Decide whether duplicate connection and preset names are allowed

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Adding two connections with the identical name creates two identical-looking cards (only the model line distinguishes them), and adding a preset with an existing name is equally accepted. Should duplicate names be rejected on create, or accepted with a visible disambiguator? Which of the two surfaces do it?

## Context

- Finding 05.F4 (P2). DOM probe after a duplicate add: two cards titled `QA T05 Conn A` with details `qa/throwaway-a` and `qa/throwaway-a2`; screenshot 57; preset duplicate on screenshot 50; ticket 05 answer.
- Connection ids are timestamp-derived (`conn-<millis>`), so nothing breaks technically — the cost is telling two cards apart, including which one holds the Narrator/Quantifier role.
- Related: [Decide one save model for panel forms](16-decide-save-model.md) and the Games copy sweep (21) if the decision is a naming/display convention.

## Done when

- The decision is in the ticket answer, and the implementation either lands here or graduates into a new ticket.
- `python build.py` is green. Commit after user approval.

## Answer

**Decision: reject a duplicate name on add, for connections and for prompt presets. Names compare trimmed and case-insensitively. Editing an item may keep its own name. Connection names are unique across all connections; preset names are unique within each preset category.**

Why: two cards with the same title can't be told apart, and the Narrator/Quantifier role badge is the one thing a user needs to find. Names differing only in case or edge whitespace look identical, so they collide too. A System preset and a Quantifier preset called "Alpha" sit in different sections, so they stay allowed. The edit form re-posts the stored name, so an edit must keep its own.

**Where:** the application layer (`SettingsService::{add_connection, update_connection}`, `PromptPresetService::save_preset`), not a database constraint. An existing user database may already hold duplicates, and a `UNIQUE` migration would fail with no way to dedupe. Startup seeding is untouched.

**What the user sees:** the existing validation path, as in ticket 29. A 400 whose body names the duplicate shows in the existing toast, and the panel is not wiped. No new error surface (08 and 25 still decide that).

**Duplicate action:** it now picks the first free name, `(Copy)`, `(Copy 2)`, and so on, so duplicating the same preset twice still works. The review caught this: without it a second Duplicate returned a 400.

**Tests (tier 1, `tests/http/`):** settings 20.9–20.11 (add refused, case/space variant refused, edit keeps its own name); presets 21.29–21.34 (add refused, variant refused, same name in another category allowed, update keeps its own name, duplicate twice gives distinct copies, rename onto a sibling refused). Unit tests in `settings_service_tests.rs` and `prompt_preset_service_tests.rs`. About 31 handler unit tests changed to read the `Response` body.

**Gate:** worktree on `7daf645b`, nothing merged since: `nextest: 1664 passed, 0 failed, 2 skipped`, browser 30 passed (`build_20261001_195140.log`). The implementer's first run hit one stub-browser timing test that passes alone and on re-run; same family as the earlier parallel-load flakes.

**Code review** (`/code-review`, ISSUES → fixed): the Duplicate regression, an untested rename refusal, and the Validation→400 mapping now reuses `bad_request`.

**Not fixed (judgement):** the preset check lists siblings and then writes under two storage-lock acquisitions, so a concurrent same-category create could pass both (connections check inside one lock); the trim-and-lowercase scan is written twice (`connection_name_available`, `preset_name_available`); preset uniqueness is service-only, so seeding could still introduce a duplicate if the data files ever did.

