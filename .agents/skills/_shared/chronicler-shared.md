# Chronicler Workflow — Shared Blocks

## Documentation Sync

When implementing changes, ALWAYS update these documents BEFORE writing code:

1. **Plan** (`chronicler_engine/docs/plans/<name>.md`) - Document problem, solution, files to change
2. **Architecture** (`chronicler_engine/docs/diataxis/explanation/architecture.md`) - Core module structure changes
3. **System docs** (`chronicler_engine/docs/diataxis/reference/`) - Domain-specific docs (frontend/dashboard.md, game_flow.md, etc.)
4. **Reference docs** (`chronicler_engine/docs/diataxis/reference/coding_standards/`) - Data schemas, API specs
5. **ADRs** (`chronicler_engine/docs/adr/*.md`) - If making architectural decisions
6. **CHANGELOG.md** - Record the change (Date the change was made, not released. Also remove concat and eventually remove older entries if the file gets too large)

Example: If adding a new HTMX polling endpoint, update:
- Plan document
- diataxis/explanation/architecture.md (Server tier)
- diataxis/reference/frontend/dashboard.md (frontend section)
- diataxis/reference/coding_standards/testing.md (add test)
- CHANGELOG.md

## Visual/UI Verification (MANDATORY)

For any CSS, layout, or visual changes:

1. **Rebuild** the project after CSS changes
2. **Restart** the server with the new build
3. **Navigate** to the affected page in the browser
4. **Take a screenshot** and **actually look at it** — do not skip this step
5. **Confirm visually** that the change renders correctly before claiming done

**NEVER claim "verified" or "browser verification complete" without a screenshot that you have personally reviewed.** Subagent verification does not count — you must see the rendered result yourself.

Common CSS pitfalls that pass build but break visually:
- `flex: 1` on containers causing unwanted expansion
- `align-items: center` centering content in unexpectedly large spaces
- Missing `overflow` properties causing content to spill or disappear
- Z-index conflicts hiding elements behind others
