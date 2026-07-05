# pi-permissions

Project-local pi extension. Intercepts dangerous bash commands and prompts for confirmation before execution. Per-session grants let you pre-approve rules you trust.

## What it does

- Matches every `bash` tool call against a project-local rule list
- On match, prompts via `ctx.ui.confirm` (or blocks silently in non-interactive mode)
- Pre-grant rules for the session with `/permissions grant <name>`
- Config at `.pi/permissions.json` (seeded with git + `/mnt/` defaults on first run)

## Default rules

| Name | Pattern | Why |
|------|---------|-----|
| `git-commit` | `\bgit\s+commit\b` | History mutation |
| `git-push` | `\bgit\s+push\b` | Remote history mutation |
| `git-tag` | `\bgit\s+tag\b` | History mutation |
| `git-merge` | `\bgit\s+merge\b` | History mutation |
| `git-rebase` | `\bgit\s+rebase\b` | History rewrite |
| `git-reset` | `\bgit\s+reset\b` | Local history / working tree |
| `git-pull` | `\bgit\s+pull\b` | Fetch + possible merge |
| `mnt-access` | `(^\|\s)/mnt/` | WSL mount — Windows filesystem from Linux session |

## Install / dev

```bash
cd .pi/extensions/pi-permissions
npm install
npm test        # node:test + tsx
npm run check   # tsc --noEmit
```

Auto-discovered by pi (`.pi/extensions/*/index.ts`). No build step at runtime — pi uses jiti.

## Configuration

`.pi/permissions.json` (tracked — team-shared defaults):

```
```json
{
  "rules": {
    "git-push":       "\\bgit\\s+push\\b",
    "git-reset-hard": "\\bgit\\s+reset\\s+--hard\\b",
    "rm-rf":          "\\brm\\s+-rf\\b"
  }
}
```

Each rule is `name → regex`. Names surface in the confirm dialog and `/permissions list`. Add or remove freely — `/permissions reload` picks up changes.

The seeded file is committed so the team shares the same baseline. Personal customizations go in the same file; if you'd rather keep some rules local, drop them in a project-level `.gitignore` exception or split into a second untracked file (future feature).

## Commands

```
/permissions                       # show status: rules + grants + recent blocks
/permissions grant <name>          # bypass confirm for <name> this session
/permissions revoke <name>         # remove session grant
/permissions list                  # rules + which are granted
/permissions reload                # re-read .pi/permissions.json
```

Session grants are in-memory and lost on session restart.

## Out of scope

- **PATH shim for nested scripts** — `git push` inside `npm run release` is not caught. Documented limitation, same as the regex-match-only caveat in `subagent-guardrails/commit-veto.ts`.
- **File-tool guards** — bash only. For sensitive file protection, see `pi-secret-guard` or `pi-sensitive-guard`.
- **Audit log persistence** — only in-session ring buffer (last 20 blocks).

## Subagent behavior

Forked subagents run in their own `pi -p` (print-mode) process via `pi-subagents`, so `ctx.hasUI === false` in every worker. Consequence: **matched commands in subagents are hard-blocked with no UI path to grant.** No confirm dialog surfaces, and session grants from the parent session don't carry across the process boundary.

This is intentional. Workers shouldn't mutate history or touch `/mnt/`. If a worker needs git access, it should stage changes locally and return a summary for the parent session to commit.

### Hook ordering with `subagent-guardrails`

Both this extension and `subagent-guardrails/commit-veto.ts` register `tool_call` handlers. pi runs them in extension load order (`fs.readdirSync` order, filesystem-dependent — not guaranteed alphabetical). The first handler to return `{ block: true }` wins; the other never executes. In practice both end up blocking the same git verbs in subagents, so the user-visible result is identical regardless of order.

## Sibling extension

`subagent-guardrails/commit-veto.ts` blocks a parallel set of git verbs (commit/push/tag/merge/rebase) inside forked subagents. The two extensions do not coordinate — both will fire on overlapping commands in subagent contexts, which is intentional belt-and-braces.