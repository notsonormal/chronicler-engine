# Code Map: `.agents/` Directory

## Overview

The `.agents/` directory contains agent rules, conventions, and workflows that govern AI agent behavior in the MRN-General workspace. These files define how agents should operate, what constraints they must follow, and how they integrate with the OpenCode configuration system.

---

## Responsibility

The `.agents/` directory is responsible for:

1. **Defining Agent Behavior** - Establishing rules and conventions that all AI agents must follow when working in the workspace
2. **Providing Context** - Supplying agents with essential information about the environment, project structure, and domain-specific requirements
3. **Enforcing Constraints** - Implementing security, access, and operational boundaries for agent operations
4. **Standardizing Workflows** - Defining repeatable processes for common tasks like debugging, planning, and implementation

---

## Design

The directory is organized into two main subdirectories:

```
.agents/
├── rules/           # Rule files with trigger conditions
│   ├── rules.md
│   ├── environment.md
│   ├── rust_conventions.md
│   ├── chronicler_engine.md
│   └── antigravity_ide_rules.md
└── workflows/       # Pre-approved command workflows
    └── ai_stack.md
```

### Rule Categories

#### 1. Core Rules (`rules.md`)

**Trigger**: `always_on`

**Purpose**: Foundational rules that apply to every agent session regardless of context.

**Key Points**:
- Memory Bank usage (`.ag-memory/`)
- Task progress tracking via `TODO.md`
- Thought capture via `SCRATCHPAD.md`
- Behavioral habits in `MEMORY.json`

**Location**: `.ag-memory/` for operational state, `docs/` for project facts

---

#### 2. Environment & Security (`environment.md`)

**Trigger**: `always_on`

**Purpose**: Defines the runtime environment and security constraints.

**Key Points**:
- Linux-based devcontainer environment
- Docker access via restricted proxy at `host.docker.internal:2375`
- Workspace scoping: agents restricted to `/workspaces` directory
- No root access; sensitive file access denied
- Path mapping: host paths (e.g., `D:/...`) map to container paths (`/workspaces/...`)

---

#### 3. Rust Conventions (`rust_conventions.md`)

**Trigger**: Context-dependent (applies when working with Rust code)

**Purpose**: Coding standards and best practices for the Chronicler Engine Rust codebase.

**Key Areas**:
- **Error Handling**: Prefer `Result` over `panic!`, use `EngineError` enum, propagate with `?`
- **Naming Conventions**: `snake_case` for functions/variables, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants
- **Struct Design**: `pub` fields for DTOs, getter methods for computed values, derive traits where appropriate
- **Imports**: Group in order: std/lib → external crates → local modules
- **Tests**: `#[cfg(test)]` modules, descriptive names (`test_<function>_<scenario>`)
- **Thread Safety**: Clone data before `move` closures, use `Arc` for shared ownership
- **Documentation**: `#[doc = "..."]` for public APIs, document error variants

---

#### 4. Chronicler Engine (`chronicler_engine.md`)

**Trigger**: `when_working_in_chronicler_engine`

**Purpose**: Domain-specific workflow for the Rust game engine project.

**Key Principles**:
- Architecture as single source of truth (update `docs/architecture/system.md` before implementing)
- Plans update architecture first
- Plan in `docs/plans/`
- Validate with tests, format, clippy

**Workflow**:
1. Create/update plan in `docs/plans/`
2. Update architecture in `architecture/system.md`
3. Implement code
4. Validate (cargo fmt, cargo clippy, cargo test)
5. Archive completed plans to `plans/archived/`

**Visual Verification Rule**: For UI/CSS changes, agents must rebuild, restart server, navigate to page, take screenshot, and visually confirm the result.

---

#### 5. Antigravity IDE (`antigravity_ide_rules.md`)

**Trigger**: `always_on`

**Purpose**: IDE-specific context and memory bank integration.

**Key Points**:
- Running in Google Antigravity IDE (specialized VS Code fork for agentic AI coding)
- Memory Bank usage integrated with task workflow
- Read `.ag-memory/MEMORY.json`, `.ag-memory/TODO.md`, and `docs/` at session start
- Write progress to `TODO.md`, thoughts to `SCRATCHPAD.md`, habits to `MEMORY.json`

---

#### 6. AI Stack Workflow (`ai_stack.md`)

**Trigger**: Pre-approved commands via `// turbo-all` directive

**Purpose**: Standard diagnostic and management commands for local AI stack (Ollama, Open Notebook, SurrealDB).

**Commands**:
- Diagnostics: container status, resource usage, Ollama model list
- Logs: Open Notebook, Ollama, SurrealDB
- Management: start/stop stack

---

## Flow: How Rules Are Applied at Session Start

### Session Initialization Sequence

```
1. Agent receives task
   │
   ▼
2. Load always_on rules (rules.md, environment.md, antigravity_ide_rules.md)
   │
   ▼
3. Check Memory Bank
   ├── Read .ag-memory/MEMORY.json (behavioral habits)
   ├── Read .ag-memory/TODO.md (task progress)
   └── Read relevant docs/ files (project facts)
   │
   ▼
4. Evaluate context triggers
   ├── If working in chronicler_engine → load chronicler_engine.md
   ├── If writing Rust code → load rust_conventions.md
   └── If using AI stack → load ai_stack.md workflow
   │
   ▼
5. Apply rules throughout session
   ├── Follow naming conventions
   ├── Use proper error handling
   ├── Update Memory Bank as needed
   └── Validate outputs
```

### Trigger Evaluation

Rules use YAML frontmatter to define activation conditions:

| Trigger | When Applied |
|---------|--------------|
| `always_on` | Every session, regardless of context |
| `when_working_in_chronicler_engine` | When task involves `chronicler_engine/` directory |

---

## Integration with OpenCode Config

### Dual Configuration System

MRN-General uses two parallel agent configuration systems:

1. **`.agents/`** - Custom rules and conventions (this directory)
2. **`.opencode/`** - OpenCode agent definitions and skills

### How They Connect

#### OpenCode Agent Definitions (`.opencode/agents/`)

The `.opencode/agents/` directory contains agent profiles that reference the rules:

| Agent | Role | Focus |
|-------|------|-------|
| `@chronicler-dev` | Engine Specialist | Rust dev in `chronicler_engine/` |
| `@ops-expert` | System & Workflow | Docker, automation, maintenance |
| `@coder` | General Implementation | Cross-project refactoring, bug fixes |
| `@reviewer` | Quality Assurance | Code review, security |
| `@debugger` | Problem Solver | Memory leaks, sync failures |
| `@planner` | Architect | Task decomposition, design |

#### OpenCode Skills (`.opencode/skills/`)

The `.opencode/skills/` directory provides specialized skills that complement the rules:

- `chronicler-dev-workflow` - Engine-specific implementation workflow
- `chronicler-after-plan-workflow` - Post-planning execution
- `coding-guidelines` - General coding standards
- `unsafe-checker` - Rust unsafe code review
- Domain skills: `domain-cli`, `domain-fintech`, `domain-iot`, `domain-ml`

#### Skill Metadata Integration

Each skill includes metadata that references relevant rules:

```json
{
  "name": "chronicler-dev-workflow",
  "triggers": ["when_working_in_chronicler_engine"],
  "references": [
    ".agents/rules/chronicler_engine.md",
    ".agents/rules/rust_conventions.md"
  ]
}
```

### Rule Loading in OpenCode

When OpenCode initializes an agent session:

1. **Agent profile** is loaded from `.opencode/agents/<agent>.md`
2. **Trigger evaluation** identifies applicable rules from `.agents/rules/`
3. **Rules are injected** into the agent context
4. **Skills are loaded** based on task domain and trigger conditions
5. **Memory Bank** is synchronized with session state

---

## File Reference

| File | Purpose | Trigger |
|------|---------|---------|
| `rules/rules.md` | Core agent rules, Memory Bank | `always_on` |
| `rules/environment.md` | Devcontainer, Docker, security | `always_on` |
| `rules/rust_conventions.md` | Rust coding standards | Context-dependent |
| `rules/chronicler_engine.md` | Engine workflow | `when_working_in_chronicler_engine` |
| `rules/antigravity_ide_rules.md` | IDE context, Memory Bank | `always_on` |
| `workflows/ai_stack.md` | AI stack diagnostics | `// turbo-all` |

---

## Best Practices

1. **Always check Memory Bank first** - Read `.ag-memory/MEMORY.json` and `.ag-memory/TODO.md` at session start
2. **Evaluate triggers before applying rules** - Not all rules apply to every task
3. **Update Memory Bank throughout** - Track progress in `TODO.md`, capture thoughts in `SCRATCHPAD.md`
4. **Follow Rust conventions** when working in `chronicler_engine/`
5. **Use visual verification** for UI/CSS changes - Never claim verification without reviewing a screenshot
6. **Validate outputs** - Run tests, format, clippy before completing tasks