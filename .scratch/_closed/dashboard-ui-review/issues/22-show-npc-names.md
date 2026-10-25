# Show Character names in the visual sidebar

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

NPC portraits in the visual sidebar have no visible name and no tooltip, only `alt`. `ui_design.md` specifies portrait labels. Add a visible name to each Character portrait.

## Context

- Finding 3.5. Screenshot 01.
- The large empty area below the portraits is a layout question for [Decide the panel layout convention and supported viewports](19-decide-layout-convention.md).

## Done when

- Each portrait shows its Character's name.
- A test is placed by `tests/STRATEGY.md` (likely tier 1: the sidebar fragment is `curl`-observable).
- `python build.py` is green. Commit after user approval.

## Answer

Each portrait renders `<div class="image-label" title="name">name</div>` after the image, with Askama's default escaping. The alt stays the name. A `.image-label` style already existed and was unused: the label was in the pre-Askama implementation and got dropped in the template migration, so this reinstates it. The portrait tile is `min-height: 80px` instead of a fixed 80px height, which would have clipped the label; the label is single-line with an ellipsis and the full name in a `title` tooltip. The empty area below the portraits (ticket 19) is untouched. `ui_design.md` alignment is ticket 23, which is blocked by this ticket.

**Tests (tier 1, `tests/http/visual_sidebar.rs`):** `test_visual_sidebar_shows_character_name_label_http` (scenario 32.1) and `test_visual_sidebar_escapes_character_name_label_http` (32.2, a Character named `Ben & Jerry <Script>` renders escaped, no raw markup). Stub fixture mirrors the template.

**Spec:** new `docs/specs/visual_sidebar.md` with 32.1 and 32.2. No existing spec covered the sidebar fragment. `validate_feature_spec.py`: 152 declared / 152 covered.

**Gate:** worktree on `f1521a0f`: `nextest: 1643 passed, 0 failed`, browser 26 passed (`build_20260930_212643.log`). Then on main with ticket 11 applied: `nextest: 1644 passed, 0 failed, 2 skipped`, browser 30 passed (`build_20260930_214245.log`).

**Code review** (`/code-review`, verdict CLEAN, no must-fix). Judgement calls went to [Follow up on small issues found during review](36-follow-up-small-review-issues.md): a redundant flex declaration, the label duplicating `alt` for screen readers, a brittle escaping assertion, an unasserted "one portrait per Character" clause, and truncation of long names (a wrap decision for ticket 19).

