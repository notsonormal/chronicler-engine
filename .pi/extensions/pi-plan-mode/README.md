# 🧭 pi-plan-mode — Codex-like Plan Mode for Pi

[![npm](https://img.shields.io/npm/v/@notsonormal/pi-plan-mode)](https://www.npmjs.com/package/@notsonormal/pi-plan-mode) [![Pi extension](https://img.shields.io/badge/Pi-extension-blue)](https://pi.dev) [![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

`@notsonormal/pi-plan-mode` adds a Codex-like `/plan` collaboration mode to Pi. Plan mode is for read-only exploration, clarifying questions, and a final implementation-ready `<proposed_plan>` block before any code mutation happens.

Pi core intentionally does not ship a built-in plan mode; this package provides one as an independently installable extension.

## ✨ Features

- Adds `/plan` to enter or manage Plan mode.
- Adds `--plan` to start a session in Plan mode.
- Enables built-in read-only tools by default while Plan mode is active.
- Disables extension and custom tools by default, with a `/plan tools` selector for explicit user-risk opt-in.
- Blocks mutating built-in tools and bash commands such as `rm`, `git commit`, dependency installs, redirects, and editor launches.
- Injects Codex-like Plan mode instructions: explore first, ask decision questions for high-impact ambiguity, do not mutate files, and finish with `<proposed_plan>` only when decision-complete.
- Adds a required `plan_mode_question` tool so the agent can ask structured Plan-mode questions before finalizing a plan.
- Detects proposed plan blocks and prompts you to implement, stay in Plan mode, or exit and discard the plan.
- Shows Plan mode state in Pi's statusline as `plan active` or `plan ready`; `@narumitw/pi-statusline` adds the default `📝` icon unless configured otherwise.
- Persists Plan mode state in the Pi session so resume restores the mode.

## 📦 Install

```bash
pi install npm:@notsonormal/pi-plan-mode
```

Try without installing permanently:

```bash
pi -e npm:@notsonormal/pi-plan-mode
```

Try this package locally from the repository root:

```bash
pi -e ./extensions/pi-plan-mode
```

## 🚀 Usage

```text
/plan
/plan <prompt>
/plan tools
/plan grill
/plan folder <relative-path>
```

Use `/plan` to enter Plan mode before writing your planning prompt. Use `/plan <prompt>` to enter Plan mode and immediately submit `<prompt>` as the first Plan-mode user message. Use `/plan tools` to choose which tools are active while Plan mode is enabled; the selector is paginated at 10 tools per page. Use `/plan grill` to stress-test the current plan with the `grill-with-docs` skill (loads `/grilling` + `/domain-modeling`). Use `/plan folder <path>` to override the plan folder for this session (cwd-relative, no `..` escape).

When Plan mode is active, ask the agent to design the change. The agent may inspect files and run read-only commands, but it should not edit files or execute the implementation. It should explore first, then use structured questions when your preference or a tradeoff materially changes the plan.

When Plan mode is active, the agent may inspect files and run read-only commands. Writes and edits are blocked except inside the `planFolder` and any `scratchFolders`. Bash is restricted to read-only / non-mutating commands. Extension and custom tools are disabled by default because Pi tools do not expose standardized mutability metadata; enable them from `/plan tools` only when you accept the risk for that session.

### Configuring the default tool set

The initial tool selection (used before any per-session `/plan tools` choice) can be customized two ways. Both only affect the initial defaults; once you use `/plan tools`, your selections persist for the rest of that session.

**Config file** — `<cwd>/.pi/plan-mode.json` (project) overrides `~/.pi/plan-mode.json` (user):

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

- `planFolder` (string, cwd-relative): where approved plans get written when Plan mode exits. Default: `chronicler_engine/docs/plans`.
- `scratchFolders` (string array, cwd-relative): additional folders where `write`/`edit` are allowed during Plan mode (e.g. for draft notes). Default: `["tmp"]`.
- Both fields reject absolute paths and `..` traversal with a warning.

Names not present in the loaded tool set, or that fail Plan-mode selection rules, are silently dropped. You can also point at an arbitrary path with the `PLAN_MODE_CONFIG` env var.

**CLI flag** — pass `--plan-tools read,bash,grep,find,ls,describe_image` to override the config file for the current session.

`plan_mode_question` follows Codex's `request_user_input` pattern: the agent can ask 1-3 concise questions, each with meaningful options and a free-form Other path. If you cancel or no interactive UI is available, the agent should ask a concise plain-text question or proceed only with a clearly stated low-risk assumption instead of prematurely producing a final plan.

Pi activates tools by tool name. The `/plan tools` selector stores selections by name and shows each currently effective tool's source from Pi metadata, such as `built-in`, a user extension path, or a project extension path. If an extension overrides a built-in tool with the same name, Pi exposes the effective tool for that name and the selector shows that source.

A complete Plan mode answer should appear only after the agent has resolved discoverable facts and any high-impact user decisions. It should include exactly one block like this:

```xml
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

### Story point rules

- Sizes: 1, 3, 5, 8, 13
- 8 SP or larger must break into subtasks
- 5 SP = single worker session; primary agent must verify output
- SubTasks optional for atomic tasks <=5 SP; required for tasks >5 SP
- SP mandatory on every Task line

## Test Plan
...

## Assumptions
...
</proposed_plan>
```

When Plan mode exits (via implementation or direct exit), the extension writes the extracted plan content to `<planFolder>/<slug>.md`. The slug is derived from the first `# Title` line (lowercase, non-alphanumeric to `-`, max 60 chars), with `-2`, `-3`, ... suffixes on collision. Falls back to `plan-<timestamp>.md` if no title is found.

After a proposed plan is detected, `/plan` lets you choose whether to implement the plan, stay in Plan mode, or exit Plan mode. Choosing implementation disables Plan mode, restores full tool access, and immediately starts an implementation turn with the proposed plan. Choosing Stay keeps the plan ready while you decide what to do next; to revise the plan, choose Stay and type your revision feedback in the normal prompt. When that next Plan-mode turn starts, the previous plan is no longer treated as the latest implementable plan unless the agent produces an updated `<proposed_plan>`. Choosing exit/off disables Plan mode and discards the proposed plan so it is not carried into later non-plan turns.

While Plan mode is enabled, the extension also publishes a compact status for Pi statuslines. With `@narumitw/pi-statusline`, this appears in the extension status area:

- `plan active`: Plan mode is enabled and still gathering context or drafting a plan.
- `plan ready`: A `<proposed_plan>` was detected and remains ready until you implement it, continue planning, or exit Plan mode.

You can also exit directly. Direct exit discards the latest proposed plan instead of treating it as an implementation request:

```text
/plan exit
```

## 🧠 Codex-like behavior

This extension maps Codex's `ModeKind::Plan` behavior onto Pi's extension API:

- Plan mode is a conversational collaboration mode, not TODO/progress tracking.
- `/plan <prompt>` follows Codex behavior by switching to Plan mode before submitting the inline prompt.
- The agent should use `plan_mode_question` for important non-discoverable preferences or tradeoffs before finalizing.
- `update_plan`-style checklist use is discouraged while Plan mode is active.
- The implementation boundary is explicit: Plan mode restores tools before starting implementation, choosing implementation immediately triggers a normal agent turn with full tool access, and plain exit/off discards the proposed plan.
- Pi extension safety is approximated with built-in tool restriction plus bash filtering; non-built-in tools are user-selected at user risk because Plan mode does not classify extension/custom tool behavior.

## 🗂️ Package layout

```txt
extensions/pi-plan-mode/
├── src/
│   └── plan-mode.ts
├── README.md
├── LICENSE
├── tsconfig.json
└── package.json
```

The package exposes its Pi extension through `package.json`:

```json
{
  "pi": {
    "extensions": ["./src/plan-mode.ts"]
  }
}
```

## 🔎 Keywords

Pi extension, Pi coding agent, plan mode, Codex-like plan mode, AI coding workflow, read-only planning, implementation plan.

## 📄 License

MIT. See [`LICENSE`](./LICENSE).
