# Issue Tracker: Local Markdown

Issues for this repo are tracked as **markdown files** in `.scratch/`.

- **Location:** `.scratch/<feature>/*.md`
- **CLI:** None (file-based)

## Format

```markdown
---
title: Issue title
status: open|closed
tags: []
---

## Description

Issue body...

## Notes

- ...
```

## Common Commands

```bash
# List issues
ls .scratch/*/

# Create issue
mkdir -p .scratch/feature-name
echo "..." > .scratch/feature-name/issue.md
```

## Workflows Using This

- `to-issues` - Convert plans to markdown files
- `triage` - Triage markdown issues
- `to-prd` - Create PRD as markdown