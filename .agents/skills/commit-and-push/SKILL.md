---
name: commit-and-push
description: Generate commit message, run pre-commit hooks, stage changes, commit, and push. Use for any commit or push — keeps each commit scoped to the current task's changes and asks before including unrelated or untracked files.
argument-hint: "[commit message hints...]"
action-required: EXECUTES — runs actual git commands (stage, commit, push) when invoked
---
# Commit and Push Skill

> **Purpose:** Automate commit message generation and git workflow with pre-commit hook handling
>
> **CRITICAL: THIS SKILL EXECUTES THE COMMIT**
>
> When the user invokes this skill (says "commit and push", "git commit", etc.), you MUST:
> 1. **Actually run** the git commands — do NOT just describe the steps
> 2. **Do NOT ask for confirmation** — the user's invocation IS the confirmation
> 3. **Do NOT output a tutorial** — execute the workflow immediately
> 4. **Do NOT delete, stash, revert or change existing files** - you should not overthink
> 5. **Do not try to build or tests to validate the that application is working* - that's not part of this workflow
>
> **This is an ACTION skill, not a DOCUMENTATION skill.**

---

## Pre-commit Hook Behavior

The chronicler_engine project has a pre-commit hook that regenerates four generated files and stages them into the commit being made:

- `AGENTS.md`
- `tests/AGENTS.md`
- `docs/AGENTS.md`
- `docs/diataxis/reference/coding_standards/guardrails.md`

The hook stages them itself, precisely so you do not have to commit twice — this is by design.

The hook aborts, naming the file, only when one of those four already had unstaged changes before it ran. The generated files carry hand-written prose beside their generated blocks, so the hook cannot re-stage the file without sweeping that prose into the commit. Stage or stash the file, then commit again — Git reuses the previous commit message automatically:

```bash
git add <file>
git commit
```

---

## Execution Workflow

### Step 1: Run the Generators Manually (Optional)

The hook runs the four generators for you and stages the result. Running them first is optional — it surfaces a generator failure before you have staged everything:

```bash
python scripts/generate_docs_index.py
python scripts/generate_guardrails_doc.py
python scripts/generate_structure_index.py
python scripts/generate_tests_structure_index.py
```

Do not stage their output by hand — the hook regenerates and stages it anyway.

### Step 2: Check Git Status

```bash
git status
```

Identify:
- Modified files
- Untracked files (should they be committed?)
- Files modified by pre-commit hooks

### Step 3: Stage All Changes

```bash
git add -A
```

Or selectively:
```bash
git add path/to/file1 path/to/file2
```

### Step 4: Generate Commit Message

Inspect the changes:

```bash
git diff --staged --stat
git diff --staged <important-file>
```

Generate a conventional commit message following the project's conventions:

**Format:**
```
<type>(<scope>): <subject>

<body - optional, explains WHY not WHAT>

Fixes #<issue-number> (if applicable)
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code restructuring (no behavior change)
- `docs`: Documentation only
- `chore`: Maintenance, no production code change
- `inv`: Invariant/rule addition
- `test`: Adding or updating tests

**Example:**
```
refactor(action_processing): Extract composable pure functions from handle_movement

- Extracted attempt_movement() for semantic walk + dynamic room creation
- Extracted update_npc_encounters_on_room_change() for NPC state updates
- Extracted log_movement_completion() for narrative pending location
- handle_movement() now composes helpers in linear flow
- Each helper has single responsibility, testable in isolation
- No behavioral changes — all 947 tests pass; clippy clean
```

### Step 5: Commit

```bash
git commit -m "type(scope): subject"
# Or with multi-line message:
git commit -F /path/to/message.txt
```

**If the hook aborts:** it found unstaged changes in one of the generated files. See **Pre-commit Hook Behavior** above for the fix.

### Step 6: Push

```bash
git push
```

**If remote has diverged (push rejected):**
```bash
git pull
# Resolve any merge conflicts if they arise
git push
```

Just a normal merge — nothing fancy needed.

---

## Complete Example Session

```bash
# 1. Optionally run the four generators first (catches changes early)
python scripts/generate_docs_index.py
# Output: "Generated index with 47 entries"

# 2. Check status
git status
# Shows modified files, untracked files, and any staged generated indexes

# 3. Stage everything
git add -A

# 4. Review changes
git diff --staged --stat
# 5 files changed, 120 insertions(+), 30 deletions(-)

# 5. Commit
git commit -m "refactor(action_processing): Extract composable pure functions"

# 6. Push
git push
```

---

## Edge Cases

### Untracked Files
Verify untracked files should be committed:
```bash
git status --untracked-files=all
```
Add to `.gitignore` if they shouldn't be tracked.

### Merge Conflicts on Pull
```bash
git pull
# Edit conflicted files to resolve
git add <resolved-files>
git commit  # Completes the merge
git push
```

### Large Binary Files
Git LFS may be required for files >100MB. Check project guidelines.

---

## Verification Checklist

Before pushing:
- [ ] All intended files staged
- [ ] Commit message follows conventional format
- [ ] Pre-commit hook ran clean (the four generated files are staged by the hook)
- [ ] `git status` shows clean working tree (or only expected untracked files)

After pushing:
- [ ] `git push` succeeded
- [ ] Remote branch updated (verify on GitHub if needed)

---

## Common Mistakes

| Mistake | Result | Fix |
|---------|--------|-----|
| Committing with unstaged edits to a generated file | Hook aborts the commit | Stage the file, then commit again — see Pre-commit Hook Behavior |
| Committing without reviewing diff | Accidental debug code, TODOs | Always `git diff --staged` first |
| Using `git push --force` on shared branches | May overwrite others' work | Use force only on personal branches |
| Ignoring merge conflicts | Push fails, remote unchanged | Resolve conflicts, complete merge commit |
| Arbitrary deleting or reverting unexpected files | Valid code changes being lost | Leave files are is or ask for permission |