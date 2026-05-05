# Issue Tracker: Local Markdown

Issues for this repo are tracked as **markdown files** in `.scratch/issues/`.

- **Location:** `.scratch/issues/*.md`
- **CLI:** `scripts/issue_tracker/cli.py`
- **Format:** Markdown with YAML frontmatter

## Why Local Markdown?

- **Offline-first:** No external SaaS dependency
- **Git-versioned:** Issues live in the repo and follow the same branching/merging workflow
- **Agent-readable:** Kimi/OpenCode agents can read and write markdown natively
- **Obsidian-compatible:** View and edit issues in Obsidian alongside your vault

---

## Issue File Format

Each issue is a single markdown file:

```markdown
---
id: 1
identifier: MRN-001
title: "Issue title"
status: needs-triage
priority: 2
labels:
  - infrastructure
  - devcontainer
created_at: 2026-05-01T10:00:00Z
updated_at: 2026-05-05T14:30:00Z
---

# Issue title

## Description

What needs to be done...

## Acceptance Criteria

- [ ] Criterion one
- [ ] Criterion two

## Notes

- Any additional context
```

### Field Reference

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Auto-incrementing numeric ID |
| `identifier` | string | Human-readable key (`MRN-001`) |
| `title` | string | Short issue title |
| `status` | string | One of the [triage labels](triage-labels.md) |
| `priority` | integer | `1` (highest) to `4` (lowest); default `2` |
| `labels` | list | Arbitrary string tags |
| `created_at` | ISO-8601 | Creation timestamp |
| `updated_at` | ISO-8601 | Last modification timestamp |

### Status Values

See [triage-labels.md](triage-labels.md) for definitions:

- `needs-triage`
- `needs-info`
- `ready-for-agent`
- `ready-for-human`
- `wontfix`

---

## CLI Usage

### Create an issue

```bash
python scripts/issue_tracker/cli.py create --title "Fix devcontainer network"
python scripts/issue_tracker/cli.py create --title "Add feature X" --description "Details..." --label backend --priority 1
```

### List issues

```bash
# All issues
python scripts/issue_tracker/cli.py list

# Filter by status
python scripts/issue_tracker/cli.py list --status ready-for-agent

# Filter by label
python scripts/issue_tracker/cli.py list --label chronicler

# Sort by priority descending
python scripts/issue_tracker/cli.py list --sort -priority
```

### Show issue details

```bash
python scripts/issue_tracker/cli.py show MRN-001
```

### Transition status

```bash
python scripts/issue_tracker/cli.py transition MRN-001 ready-for-human
```

### Search

```bash
python scripts/issue_tracker/cli.py search "network"
```

### Edit in $EDITOR

```bash
python scripts/issue_tracker/cli.py edit MRN-001
```

---

## Directory Layout

```
.scratch/
├── issues/
│   ├── 001-fix-devcontainer-network.md
│   ├── 002-refactor-engine.md
│   └── ...
└── templates/
    └── issue.md
```

---

## Agent Workflows

The following skills read from and write to `.scratch/issues/`:

- `to-issues` — Convert plans into markdown issue files
- `triage` — Triage issues, update labels and status
- `to-prd` — Create PRD documents as issues

Agents can interact with issues directly via:
- **Read:** `ReadFile(".scratch/issues/001-*.md")`
- **Write:** `WriteFile(".scratch/issues/004-new-issue.md", ...)`
- **CLI:** `Shell("python scripts/issue_tracker/cli.py ...")`

---

## Migration Notes

The old `.scratch/<feature>/*.md` layout (status: `open|closed`, no CLI) has been replaced by the flat `.scratch/issues/*.md` layout with the Python CLI. If you have existing issues in the old format, migrate them by copying the markdown body into the new template and assigning a proper `MRN-###` identifier.
