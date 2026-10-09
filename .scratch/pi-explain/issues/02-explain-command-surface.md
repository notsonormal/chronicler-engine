# 02 — The /explain command surface and STE scope

Type: grilling
Status: claimed
Blocked by: (none)
Assignee: pi (this session)

## Question

What commands does v1 expose, and is Simplified Technical English in scope?

`explain-claude-mod` offers `/explain`, `/explain html`, `/explain off`, and
`/explain diagram how git rebase works`, plus a picker band above the prompt.
Decide how much of that v1 carries.

## Options

- **Option A — formats only.** `/explain [md|diagram|html|off] [topic]`, plus a
  picker band. The band is `ctx.ui.setWidget()`, the pi primitive for persistent
  content near the editor.
- **Option B — A plus Simplified Technical English.** `/explain ste` changes how
  the main agent answers. It produces no artifact and opens no panel. It is one
  system-prompt section on `before_agent_start`.
- **Option C — B plus named presets**, such as `/explain diagram how git rebase
  works` with a stored topic.

## Background

- STE is not an artifact. It is answer style, and it is the cheapest half of the
  explain-claude-mod idea.
- Mode state must survive a session. `pi.appendEntry()` stores session data
  without putting it in model context, and `ctx.sessionManager.getBranch()`
  rebuilds it on `session_start`.
- Open question carried in the map's fog: whether the picker band is a widget or
  part of the panel header.

## Recommendation

Option B.
