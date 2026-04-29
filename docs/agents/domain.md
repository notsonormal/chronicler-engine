# Domain Docs

This is a **multi-context** repo with architecture docs in multiple subdirectories.

## Layout

| Context | Location | Docs |
|---------|----------|------|
| chronicler_engine | `chronicler_engine/docs/` | Architecture, ADRs, plans |
| docker | N/A | See root `docs/` |
| scripts | N/A | See root `docs/` |
| .opencode | N/A | Skills in `.opencode/skills/` |

## Consumer Rules

The following skills read from these locations:

- `improve-codebase-architecture` - Reads `CONTEXT.md` and `docs/adr/` - checks each context directory
- `diagnose` - Reads domain docs for terminology
- `tdd` - Reads domain docs for context

For multi-context, skills should look in the relevant subdirectory based on task context.