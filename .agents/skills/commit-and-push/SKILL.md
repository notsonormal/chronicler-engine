---
name: commit-and-push
description: Generate commit message, run pre-commit hooks, stage changes, commit, and push. Handles docs index regeneration automatically.
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
>
> **This is an ACTION skill, not a DOCUMENTATION skill.**

---

## Pre-commit Hook Behavior

The chronicler_engine project has a pre-commit hook that:
1. Runs `generate_docs_index.py` to update `docs/README.md`
2. **Aborts the commit** if README.md was modified
3. Requires you to stage the updated README.md and commit again

**This is by design** — the hook ensures the docs index is always current, but means you may need two commits when docs change.

---

## Execution Workflow

### Step 1: Run Pre-commit Hooks Manually (Optional but Recommended)

Run the docs index generator BEFORE committing to catch changes early:

```bash
cd chronicler_engine
python scripts/generate_docs_index.py
```

If this updates `docs/README.md`, stage it now:
```bash
git add docs/README.md
```

**Why do this first?** Running the hook manually before staging means the README.md change is included in your single commit, avoiding the two-commit dance.

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

**If pre-commit hook blocks:**

The hook updated README.md. Just stage and commit again:

```bash
git add docs/README.md
git commit
```

Git will reuse the previous commit message automatically.

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
# 1. Run docs index generator first (catches changes early)
cd chronicler_engine
python scripts/generate_docs_index.py
# Output: "Generated index with 47 entries"

# 2. Check status
git status
# Shows modified files + updated README.md

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

## Two-Commit Flow (When Pre-commit Hook Fires)

**Why it happens:**
1. Your code changes trigger the pre-commit hook
2. Hook runs `generate_docs_index.py` and updates README.md
3. Commit aborts because working tree is now dirty
4. You stage README.md and run `git commit` again

The second commit reuses the same message automatically — this is expected and fine.

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
- [ ] Pre-commit hooks satisfied (or README.md staged for second commit)
- [ ] `git status` shows clean working tree (or only expected untracked files)

After pushing:
- [ ] `git push` succeeded
- [ ] Remote branch updated (verify on GitHub if needed)

---

## Common Mistakes

| Mistake | Result | Fix |
|---------|--------|-----|
| Skipping pre-commit hook run | Two commits required | Run `generate_docs_index.py` first or just commit again after hook updates README |
| Committing without reviewing diff | Accidental debug code, TODOs | Always `git diff --staged` first |
| Using `git push --force` on shared branches | May overwrite others' work | Use force only on personal branches |
| Ignoring merge conflicts | Push fails, remote unchanged | Resolve conflicts, complete merge commit |
| Arbitrary deleting or reverting unexpected files | Valid code changes being lost | Leave files are is or ask for permission |