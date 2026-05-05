# Issue Tracker Implementation Plan

## Revised Understanding

The user does **not** have an existing issue tracker and is **not** looking for multi-agent orchestration. They want a **local issue tracker** for this repository, inspired by Symphony's issue-tracker integration patterns, but adapted for a Kimi Code / OpenCode workflow.

Current state: `.scratch/` (mentioned in AGENTS.md as the intended issue tracker location) does **not exist**. The user has never used it. The repo has planned-but-unimplemented issue tracking infrastructure:
- `docs/agents/issue-tracker.md` (referenced in AGENTS.md)
- `docs/agents/triage-labels.md` (defines: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix)
- Issue tracker skills: `to-issues`, `triage`, `to-prd` (these skills exist but have no data store to operate on)

The user also has Obsidian integration (`scripts/ObsidianVaultExport/`, `docker/obsidian_data/`), suggesting a markdown-friendly, local-first workflow.

---

## Recommendation: Local Markdown Issue Tracker (`.scratch/`)

**Implement the intended-but-missing local markdown issue tracker.** This is the path of least resistance because:
- The repo's AGENTS.md, skills, and docs already assume it exists
- It is 100% local, git-versionable, and works offline
- Markdown with YAML frontmatter is consistent with the repo's Obsidian workflow
- Kimi/OpenCode agents can read/write markdown files natively
- Zero external dependencies (no SaaS, no database server)

### What to Build

1. **`.scratch/` directory structure**
   ```
   .scratch/
   ├── issues/
   │   ├── 001-setup-devcontainer-network.md
   │   ├── 002-chronicler-engine-refactor.md
   │   └── 003-obsidian-export-bug.md
   ├── templates/
   │   └── issue.md
   └── config.yml
   ```

2. **Issue file format** (Markdown + YAML frontmatter)
   ```markdown
   ---
   id: 1
   identifier: MRN-001
   title: Setup devcontainer network isolation
   status: ready-for-agent
   priority: 1
   labels: [infrastructure, devcontainer]
   created_at: 2026-05-01T10:00:00Z
   updated_at: 2026-05-05T14:30:00Z
   ---
   # Setup devcontainer network isolation

   ## Description
   The current docker-compose.yml exposes too many ports. We should...

   ## Acceptance Criteria
   - [ ] Internal `no-internet` network works
   - [ ] Caddy proxy still accessible
   ```

3. **Simple Python CLI** (`scripts/issue_tracker/cli.py`)
   Commands:
   - `python scripts/issue_tracker/cli.py create --title "..." --label infrastructure`
   - `python scripts/issue_tracker/cli.py list --status ready-for-agent`
   - `python scripts/issue_tracker/cli.py transition MRN-001 ready-for-human`
   - `python scripts/issue_tracker/cli.py show MRN-001`
   - `python scripts/issue_tracker/cli.py search "network"`

   This is ~200-300 lines of Python using only stdlib (`argparse`, `pathlib`, `yaml`/`frontmatter`).

4. **MCP Server or Tool for Agents**
   Create a lightweight tool so Kimi/OpenCode agents can interact with the tracker:
   - `list_issues(status="ready-for-agent")`
   - `create_issue(title, description, labels)`
   - `update_issue(identifier, status=None, priority=None)`
   - `get_issue(identifier)`

   This could be:
   - A Python MCP server (if OpenCode/Kimi supports custom MCP servers)
   - Or simply document that agents should use the CLI via Shell tool
   - Or add as an OpenCode skill (`issue-manager`)

### Why This Over External Trackers

| Criterion | Local Markdown | GitHub Issues | Linear |
|-----------|---------------|---------------|--------|
| Offline | ✅ Yes | ❌ No | ❌ No |
| Git versioned | ✅ Yes | ⚠️ Partial | ❌ No |
| Agent-readable | ✅ Native | Via MCP | Via API |
| Cost | ✅ Free | ✅ Free | 💰 Paid |
| Setup effort | Low | None (if exists) | Medium |
| Query power | Low (CLI) | High | High |

Since this is a personal workspace repo (`mrn-general`) with heavy devcontainer/local-LLM usage, local-first makes the most sense.

### Integration with Kimi/OpenCode

Agents already have access to the filesystem. With `.scratch/issues/*.md`:
- **Reading**: Agent can `Glob('.scratch/issues/*.md')` and `ReadFile()` to find work
- **Writing**: Agent can `WriteFile()` to create or update issues
- **Context injection**: The `to-issues` and `triage` skills can finally function as designed
- **Workflow**: User creates issue → marks `ready-for-agent` → agent picks it up during a session

This is **manual dispatch** (user tells agent which issue to work on), not Symphony's **auto-dispatch** (scheduler picks issues). This matches the user's clarification that they don't need scheduling/orchestration.

---

## Alternative: SQLite + CLI

If markdown feels too limited for querying/filtering:
- Single SQLite file (`.scratch/issues.db`)
- Same Python CLI but with SQL-backed queries
- Issues still exportable to markdown for Obsidian/git viewing
- More complex, but better for >50 issues

**Verdict**: Start with markdown. Migrate to SQLite only if query performance or structured metadata becomes painful.

---

## Implementation Order

1. **Create `.scratch/` directory and issue template**
2. **Build the Python CLI** (create, list, show, transition, search)
3. **Write `docs/agents/issue-tracker.md`** documenting the schema and CLI usage
4. **Create sample issues** to test the workflow
5. **Optional**: Build MCP server or OpenCode skill for programmatic access

This delivers a functional issue tracker in one session, unlocking the existing `to-issues`, `triage`, and `to-prd` skills.
