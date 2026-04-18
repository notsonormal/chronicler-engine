# Add openrouter-image-mcp to OpenCode

## TL;DR

> **Quick Summary**: Add the openrouter-image-mcp MCP server to OpenCode to enable image analysis capabilities (analyze screenshots, extract text from images, debug UI).

> **Deliverables**:
> - MCP server configured in opencode.json
> - 3 new image analysis tools available to the LLM

> **Estimated Effort**: Quick (single JSON edit)
> **Parallel Execution**: NO (sequential - one task)

---

## Context

### Original Request
User wants to add openrouter-image-mcp to OpenCode so the LLM can see and understand images. User provided the repository: https://github.com/JonathanJude/openrouter-image-mcp

### Interview Summary
**Key Discussions**:
- Model choice: `google/gemini-2.0-flash-exp:free` (free default)
- User has OpenRouter API key already

### Metis Review
**Identified Gaps** (addressed):
- API key handling: User will provide key, will note both env var and config options
- Package verification: confirmed `openrouter-image-mcp` is the correct package

---

## Work Objectives

### Core Objective
Add the openrouter-image-mcp MCP server to OpenCode configuration, enabling 3 new image analysis tools.

### Concrete Deliverables
- `openrouter-image` MCP server entry in opencode.json

### Definition of Done
- [x] opencode.json updated with new MCP server
- [x] Configuration valid JSON syntax

### Must Have
- MCP server entry with npx command
- Model configuration set to `google/gemini-2.0-flash-exp:free`

### Must NOT Have
- DO NOT hardcode actual API key in config file (security risk)
- DO NOT use `npx openrouter-image-mcp` without pinned version

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO (configuration only)
- **No tests needed**: Simple JSON edit

### QA Policy
- Verify JSON is valid
- Verify file saves correctly

---

## Execution Strategy

### Single Task
This is a simple one-task configuration:
- Edit opencode.json to add MCP server

---

## TODOs

- [x] 1. Add openrouter-image MCP to opencode.json

  **What to do**:
  - Read current opencode.json
  - Add new MCP server entry under "mcp" section
  - Use npx to run openrouter-image-mcp
  - Configure model to google/gemini-2.0-flash-exp:free

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - **Reason**: Simple JSON configuration edit

  **Parallelization**:
  - NO - single task

  **References**:
  - opencode.json: Current MCP configuration

  **Acceptance Criteria**:
  - [ ] opencode.json updated successfully

  **QA Scenarios**:
  - Verify JSON is valid

---

## Final Verification Wave

None needed for this simple change.

---

## Commit Strategy

Not applicable - configuration file change only.

---

## Success Criteria

### Verification Commands
```bash
# Verify JSON is valid
cat opencode.json | python -m json.tool > /dev/null
```

### Final Checklist
- [ ] MCP server added to opencode.json
- [ ] JSON syntax valid