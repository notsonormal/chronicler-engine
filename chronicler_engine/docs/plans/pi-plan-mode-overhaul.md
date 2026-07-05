# Plan: pi-plan-mode Extension Overhaul

**Date:** 2026-07-05
**Status:** Planning
**Scope:** `.pi/extensions/pi-plan-mode/` (extension), `.pi/plan-mode.json` (config), `.agents/skills/grill-with-docs/SKILL.md` (skill)
**Investigation sources:** 2-week session log analysis (`/tmp/plan-manifest.json`, 240 plan-mode sessions), scout A report (`context.md`), explorer pass on pi-subagents extension

## Objective

Overhaul the existing `pi-plan-mode` extension to (a) make the default tool set configurable (already implemented), (b) integrate file-based plan persistence + scratch-folder write allowances, (c) extend the proposed plan format to support phases/tasks/subtasks with story points, (d) integrate `grill-with-docs` skill, and (e) replace the Codex-like wall-of-text system prompt with a brief nudge.

## Decisions Already Locked (from grilling session 2026-07-05)

1. **Config schema additions**: `planFolder` (string, cwd-relative) and `scratchFolders` (string array, cwd-relative). No `.pi/scratch`.
2. **Skill integration via (A) + (B)**:
   - (A) Drop `disable-model-invocation: true` from `grill-with-docs/SKILL.md` and broaden its description so pi auto-loads it in plan-mode contexts.
   - (B) New `/plan grill` extension command calls `pi.sendUserMessage("/grill-with-docs")` for explicit invocation.
3. **Plan format**: `## Implementation` section supports `### Phase N` → `#### Task N.M (N SP)` → `##### SubTask N.M.K (N SP)`. Phases optional for single-stage work. SubTasks optional for atomic tasks, required for tasks >5 SP. SP mandatory on every Task.
4. **`planning-and-task-breakdown` skill dropped from workflow**. Agent handles task breakdown directly in the proposed plan format. Existing skill remains installed but not referenced by the extension.
5. **Plan detection unchanged**: extension continues to parse `<proposed_plan>` block from assistant message (existing `extractProposedPlan`). No mtime polling, no folder watching.
6. **Plan-file persistence**: on "implement" or "exit" from plan mode, extension writes the extracted `<proposed_plan>` content to `<planFolder>/<slug>.md`. Slug from first `# Title` line; fallback to `plan-<timestamp>.md`.
7. **Tool gating**: in plan mode, allow `write`/`edit` to `planFolder` + each `scratchFolder`; block elsewhere. Bash stays read-only (existing SAFE/MUTATING patterns).
8. **System prompt**: replace the existing Codex-like wall of text with a brief nudge that mentions: plan folder path, scratch folders, and recommended skills (mention-only, no auto-dispatch).

## Architecture

### Config schema (`.pi/plan-mode.json`)

```json
{
  "defaultTools": [
    "read", "bash", "grep", "find", "ls",
    "describe_image", "fetch_content", "get_search_content",
    "intercom", "subagent", "recall",
    "rag_index", "rag_query", "rag_status",
    "visual_explainer", "web_search"
  ],
  "planFolder": "chronicler_engine/docs/plans",
  "scratchFolders": ["tmp"]
}
```

- `planFolder`: cwd-relative. Where approved plans get written.
- `scratchFolders`: cwd-relative. Additional write targets during plan mode.
- `defaultTools`: already implemented (project > user config precedence; CLI `--plan-tools` override).

### Skill integration

**`grill-with-docs/SKILL.md` (edit):**

Current:
```yaml
---
name: grill-with-docs
description: A relentless interview to sharpen a plan or design, which also creates docs (ADR's and glossary) as we go.
disable-model-invocation: true
---
```

New:
```yaml
---
name: grill-with-docs
description: A relentless interview to sharpen a plan or design, which also creates docs (ADR's and glossary) as we go. Use in plan mode or when stress-testing a plan before building.
---
```

Drop `disable-model-invocation: true` so pi can auto-load by description match. Broaden description to mention plan mode. No other changes to skill body.

### Proposed plan format (extension system prompt)

Replace the finalization block in `buildPlanModePrompt()` (lines ~1154-1168 of `src/index.ts`) with:

```markdown
<proposed_plan>
# Title

## Summary
...

## Key Changes
...

## Implementation

Use phases when the work spans multiple distinct stages. Skip the Phase
heading entirely for single-stage work.

### Phase 1: [Stage Name]

- [ ] #### Task 1.1: [Title] (N SP)
  - [ ] ##### SubTask 1.1.1: [Title] (N SP)
  - [ ] ##### SubTask 1.1.2: [Title] (N SP)
- [ ] #### Task 1.2: [Title] (N SP)

### Phase 2: [Stage Name]
...

## Test Plan
...

## Assumptions
...
</proposed_plan>
```

**Story point rules** (also injected into the prompt):

- Sizes: 1, 3, 5, 8, 13
- 8 SP or larger → must break into subtasks
- 5 SP = single worker session; primary agent must verify output
- SubTasks optional for atomic tasks ≤5 SP
- SubTasks required for tasks >5 SP
- SP mandatory on every Task line

### Tool gating (write/edit interception)

Use `pi.on("tool_call", handler)` to intercept `write` and `edit` tool calls while plan mode is active.

```
if (!planMode.enabled) → allow (not our concern)
if toolName not in ["write", "edit"] → allow
extract file path from event.input
resolvedPath = resolve(cwd, filePath)  // normalize
for each allowed folder (planFolder + scratchFolders, resolved against cwd):
  if resolvedPath starts with allowedFolder + sep → allow
block with reason: "Plan mode is active. Writes allowed only in: <list>"
```

File path extraction: `write` tool uses `path` field; `edit` tool uses `path` field. (Per pi tool schema, both take a `path` string parameter.)

### Plan-file persistence

On plan-mode exit via "implement" or "exit" action:

1. Extract `<proposed_plan>` block content (already parsed by `extractProposedPlan`)
2. Strip the `<proposed_plan>` / `</proposed_plan>` tags from stored content
3. Derive slug from first `# Title` line: lowercase, replace non-alphanumeric with `-`, collapse repeats, trim leading/trailing `-`, max 60 chars. Fallback: `plan-<YYYYMMDD-HHMMSS>`.
4. Resolve full path: `join(cwd, planFolder, slug + ".md")`
5. If file already exists, suffix `-2`, `-3`, etc.
6. Write file with content
7. Log to user: "Plan written to: `<path>`"
8. Proceed with existing exit flow (restore tools, send user message if "implement")

### System prompt replacement

Strip the long Codex-like prompt in `buildPlanModePrompt()`. Replace with a brief version:

```
# Plan Mode

You are in Plan Mode. Chat your way to a decision-complete implementation plan, then emit a <proposed_plan> block.

Rules:
- No mutating actions outside {planFolder} and scratch folders {scratchFolders}.
- Do not implement. Producing the plan is the task.
- Use /grill-me-with-docs to stress-test the plan before finalizing if the user requests grilling.

Plan folder: {planFolder}
Scratch folders: {scratchFolders}

[Existing Phase 1/2/3 guidance stays — environment grounding → intent → implementation chat]
```

Keep the existing Phase 1/2/3 process guidance (it's useful and grounded). Only the finalization template + mode rules get trimmed.

## Task List

### Phase 1: Config + skill changes

- [ ] #### Task 1.1: Extend `PlanModeDefaultsConfig` interface + load logic to include `planFolder` and `scratchFolders` (3 SP)
  - Config validation: strings, cwd-relative, no `..` escape, trimmed
  - No hardcoded defaults in the extension (config is single source of truth; absent config → strictest behavior, all writes blocked)
  - Update `resolveDefaultToolsConfig` to return all three fields
  - Update `.pi/plan-mode.json` with new fields
- [ ] #### Task 1.2: Edit `grill-with-docs/SKILL.md` (1 SP)
  - Drop `disable-model-invocation: true`
  - Broaden description to mention plan mode

### Phase 2: Tool gating

- [ ] #### Task 2.1: Add `tool_call` interceptor for `write`/`edit` (5 SP)
  - SubTask 2.1.1: Path-extraction helper (extract `path` from `event.input` for write/edit tools) (3 SP)
  - SubTask 2.1.2: Folder-resolution helper (resolve cwd-relative paths, normalize, check prefix against allowed folders) (3 SP)
  - SubTask 2.1.3: Block handler returning `{block: true, reason: "..."}` with allowed-folders list (1 SP)
  - SubTask 2.1.4: Tests for gating (paths inside allowed, outside, subdirectory traversal, symlinks) (3 SP)

### Phase 3: Plan format extension

- [ ] #### Task 3.1: Update `buildPlanModePrompt()` finalization template (3 SP)
  - Replace 4-section block with extended Implementation section
  - Add story-point rules
  - Add phase/subtask optionality guidance
- [ ] #### Task 3.2: Update README to document new plan format (1 SP)

### Phase 4: Plan-file persistence

- [ ] #### Task 4.1: Slug derivation helper (3 SP)
  - SubTask 4.1.1: Parse first `# Title` line from plan content (1 SP)
  - SubTask 4.1.2: Slugify (lowercase, non-alphanumeric → `-`, collapse repeats, trim, max 60 chars) (1 SP)
  - SubTask 4.1.3: Collision handling (suffix `-2`, `-3`, etc.) (1 SP)
- [ ] #### Task 4.2: Wire persistence into exit flow (3 SP)
  - SubTask 4.2.1: Hook into existing exit handler (where `awaitingAction` is cleared) (1 SP)
  - SubTask 4.2.2: Write file, log path to user (1 SP)
  - SubTask 4.2.3: Tests for slug + persistence (3 SP)

### Phase 5: Slash commands + system prompt

- [ ] #### Task 5.1: Strip Codex-like wall-of-text from `buildPlanModePrompt()` (3 SP)
  - Keep Phase 1/2/3 process guidance
  - Inject dynamic `planFolder` / `scratchFolders` values
  - Mention grill-with-docs by name (mention-only, no auto-dispatch)
- [ ] #### Task 5.2: New `/plan grill` extension command (3 SP)
  - SubTask 5.2.1: Register `pi.registerCommand("grill", ...)` (1 SP)
  - SubTask 5.2.2: Subcommand parsing on existing `/plan` command (1 SP)
  - SubTask 5.2.3: Verify sendUserMessage pipeline expands skill correctly (1 SP)

### Phase 6: Validation

- [ ] #### Task 6.1: `tsc --noEmit` clean on `src/index.ts` (1 SP)
- [ ] #### Task 6.2: `biome check` clean (excluding pre-existing unused `PlanModeQuestionParams`) (1 SP)
- [ ] #### Task 6.3: README updates for all new features (3 SP)

## Test Plan

- Unit tests for:
  - Config validation (planFolder, scratchFolders)
  - Path extraction from write/edit tool inputs
  - Folder resolution + prefix check (allow/block)
  - Slug derivation (title → slug, edge cases: empty, special chars, length cap)
  - Collision handling (existing file → suffix increment)
- Integration tests:
  - `/plan grill` triggers `sendUserMessage("/grill-with-docs")`
  - Plan-file written on exit with correct slug
- Manual verification:
  - Real plan-mode session produces extended-format plan
  - Write to `tmp/` allowed in plan mode, write to `src/` blocked with clear reason
  - Skill auto-loads by description match in plan mode

## Assumptions

1. Pi's `tool_call` event handler signature supports returning `{block: true, reason: string}` for write/edit tools (verified against docs line 746-749 example).
2. Pi skill loader will auto-load `grill-with-docs` once `disable-model-invocation: true` is dropped, provided description matches the conversation context. (Verified `pi-subagents` uses this pattern successfully.)
3. `pi.sendUserMessage("/grill-with-docs")` triggers the same skill-expansion pipeline as user-typed slash commands (verified per docs line 855-858 + 1376-1378).
4. `write` and `edit` tools use a `path` string parameter for the target file. (Will verify against pi tool definitions before implementing Task 2.1.)
5. Existing `extractProposedPlan` regex continues to work for the extended plan format — the `<proposed_plan>` tags are unchanged, only the markdown content inside.
6. `grill-with-docs` body is a thin wrapper (`Run a /grilling session, using the /domain-modeling skill.`) — no body changes needed; the wrapper itself loads nested skills via pi's expansion.

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| `tool_call` interception of write/edit blocks legitimate writes agent needs during planning (e.g., writing ADRs to a non-plan folder) | Medium | Allow `planFolder` + `scratchFolders` + their subdirectories. To change the plan folder, update `.pi/plan-mode.json` and restart. |
| `grill-with-docs` auto-loads too eagerly in non-plan contexts now that `disable-model-invocation` is dropped | Medium | Broadened description explicitly mentions "plan mode or when stress-testing a plan" — should scope matching. Monitor and tighten if false-positives. |
| Slug derivation produces collisions or unclear names | Low | Collision handling via `-2`, `-3` suffix. Fallback to timestamp if no title. |
| Extended plan format confuses agent (too many conditionals) | Medium | Keep format example explicit in prompt. Test with real session. |
| `extractProposedPlan` regex breaks on deeply nested headings | Low | Regex matches `<proposed_plan>...</proposed_plan>` tags only, not markdown inside. No change needed. |

## Out of Scope

- Fixing the broken `test/plan-mode.test.ts` (pre-existing rot; separate 3-5 SP task)
- Multi-plan support (multiple plans per session)
- Plan archival / move semantics
- Auto-trigger of grilling (user invokes `/plan grill` or types the skill manually)
- Changes to `planning-and-task-breakdown` skill (remains installed, just not referenced)
- Plan templates (skeleton `.md` files in planFolder)

## Plan Adherence

This plan **does not dictate implementation order beyond the phase numbering**. Where a sub-plan deviates, stop and report per `AGENTS.md` "Plan Adherence" rule.
