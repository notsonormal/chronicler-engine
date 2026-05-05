# Plan: Auto-Generated Index for `chronicler_engine/docs`

## Problem
`chronicler_engine/docs/` contains 52+ markdown files across 7 subdirectories (`architecture/`, `system/`, `plans/`, `adr/`, `reference/`, `diagnostics/`, `CHANGELOG.md`, `ROADMAP.md`). The existing `docs/README.md` is a manually-maintained index that is already stale (missing 3 ADRs, several system docs, all diagnostics docs, and more).

## Goal
Keep `docs/README.md` automatically up-to-date with a complete, structured listing of all documentation files, including a timestamp of when the index was last regenerated.

## Clarifications

### Project-Specific Kimi Hooks?
Kimi Code CLI hooks are configured in **global** `~/.kimi/config.toml`. There is no per-project `kimi.toml` or similar for hooks. However, the hook command receives the session `cwd` in its stdin JSON context, so the hook script can be **project-aware**: it checks if `cwd` is inside `mrn-general/chronicler_engine/` before running the generator. This is the standard pattern for making global hooks project-scoped.

### Timestamp in README.md
Yes. The generator script will include a line like:
> *Index last generated: 2026-05-05 18:59 UTC*

This will appear inside the `<!-- AUTO-INDEX START -->` block and update every time the script runs.

## Options

### Option A: Python Script + Git Pre-Commit + Kimi SessionStart (Recommended)

**Components:**
1. **`chronicler_engine/scripts/generate_docs_index.py`**
   - Scans `docs/**/*.md` recursively.
   - Extracts the H1 title from each file (falling back to filename if no H1).
   - Regenerates `docs/README.md`, preserving a manually-written preamble (delimited by `<!-- AUTO-INDEX START -->` / `<!-- AUTO-INDEX END -->` markers).
   - Groups files by subdirectory with relative links.
   - Includes a timestamp of generation.
   - Only writes if the generated content differs from the current file (idempotent).
   - Accepts `--check` flag for CI (exit non-zero if stale).

2. **Git Pre-Commit Hook** (project-local)
   - Runs the Python script before every commit.
   - If `docs/README.md` is modified by the script, the commit is blocked until the user stages the update.
   - **Advantages:** Project-local, catches edits from ALL editors (Kimi, VS Code, Obsidian), handles file deletions cleanly, no recursion risk.

3. **Kimi `SessionStart` Hook** (global `~/.kimi/config.toml`)
   - A single global hook entry that calls a project-aware wrapper script.
   - The wrapper checks `cwd`; if inside `mrn-general/chronicler_engine/`, it runs the generator.
   - Ensures agents always see a fresh index at the beginning of a session.
   - Low-risk, no recursion issues.

**Trade-offs:**
- (+) Most reliable; catches human edits outside Kimi.
- (+) No recursion; `README.md` updates happen during git commit, not inside the agent loop.
- (-) Requires one-time setup of the pre-commit hook (can be automated with a setup script).

### Option B: Python Script + Kimi PostToolUse + SessionStart Hooks Only

**Components:**
1. Same Python script as Option A (with timestamp).
2. **Kimi `PostToolUse` Hook** (global `~/.kimi/config.toml`)
   - Matcher: `WriteFile|StrReplaceFile`
   - Command: A project-aware wrapper script that checks `cwd` and whether the changed file is inside `chronicler_engine/docs/` (and is not `README.md` itself), then runs the generator.
   - **Caveats:**
     - Does NOT catch file deletions (those typically use the `Shell` tool, e.g. `rm`/`del`).
     - Does NOT catch edits made in VS Code, Obsidian, or other editors.
     - Requires global `~/.kimi/config.toml` configuration.
     - Writing `README.md` can re-trigger the hook; must be guarded against loops.
3. Same `SessionStart` hook as Option A.

**Trade-offs:**
- (+) Immediate feedback during Kimi sessions.
- (-) Misses deletions and non-Kimi edits.
- (-) Requires careful loop-avoidance logic.
- (-) Global hook config affects all Kimi sessions (mitigated by project-aware wrapper).

## Recommendation
**Option A** is the better foundation because documentation is often edited outside Kimi (in Obsidian, VS Code, etc.). The git pre-commit hook is the industry-standard pattern for this exact problem. The Kimi `SessionStart` hook can still be added as a convenience layer on top.

## Implementation Steps (Option A)

1. **Create `chronicler_engine/scripts/generate_docs_index.py`**
   - Accept `--check` flag (exit non-zero if index is stale, useful for CI).
   - Accept `--docs-dir` defaulting to `chronicler_engine/docs`.
   - Sort files within each directory alphabetically.
   - Skip `README.md` itself when building the file list.
   - Add timestamp comment: `*Index last generated: <ISO timestamp>*`.

2. **Update `chronicler_engine/docs/README.md`**
   - Add `<!-- AUTO-INDEX START -->` and `<!-- AUTO-INDEX END -->` markers.
   - Move the existing manual file lists inside the auto-generated section (or let the script replace them).

3. **Create git pre-commit hook**
   - Add a script at `chronicler_engine/scripts/install-git-hooks.py` (or a shell script) that symlinks the pre-commit hook into `.git/hooks/`.
   - The hook itself runs `python chronicler_engine/scripts/generate_docs_index.py` and aborts the commit if the working tree becomes dirty (forcing the user to stage the updated index).

4. **Document in `chronicler_engine/AGENTS.md`**
   - Add a note that `docs/README.md` is auto-generated and should not be edited manually inside the `<!-- AUTO-INDEX -->` block.
   - Include the Kimi `SessionStart` hook TOML snippet for optional setup.

5. **Optional: Provide Kimi hook wrapper**
   - Add `chronicler_engine/scripts/kimi_hook_wrapper.py` that reads stdin JSON, checks `cwd`, and conditionally runs the generator. This makes the global hook config a one-liner.

## Files to Modify
- `chronicler_engine/scripts/generate_docs_index.py` (new)
- `chronicler_engine/scripts/kimi_hook_wrapper.py` (new, optional)
- `chronicler_engine/docs/README.md` (restructure with markers)
- `chronicler_engine/AGENTS.md` (add reference)
- `chronicler_engine/scripts/install-git-hooks.py` (new, optional)
