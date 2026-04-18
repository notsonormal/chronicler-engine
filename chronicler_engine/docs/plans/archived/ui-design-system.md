# Plan: Extract CSS to Design System tokens

## TL;DR

> **Quick Summary**: Convert hardcoded CSS in `assets/index.html` to use CSS custom properties (design tokens) defined in an external `assets/styles.css` file, enabling consistent theming and easier visual maintenance without visual checking.

> **Deliverables**:
> - `assets/styles.css` - External CSS file with design tokens as CSS variables
> - Updated `assets/index.html` - References external CSS, uses CSS variables

> **Estimated Effort**: Short (1-2 hours)
> **Parallel Execution**: NO (sequential)
> **Critical Path**: styles.css → index.html update → verify

---

## Context

### User's Problem
- Doesn't have "UI sense" - hard to judge if UI looks good visually
- Wants to make UI decisions logically, not visually

### What Already Exists
- **Design tokens documented**: `docs/system/ui_design.md` has full token specs
- **HTMX web UI**: `assets/index.html` with inline CSS

### Gap
- Design tokens exist in docs but are NOT USED in the actual CSS
- Colors are hardcoded (e.g., `#0a0a0a`) instead of using variables (e.g., `var(--color-bg-primary)`)

---

## Work Objectives

### Core Objective
Extract inline CSS to external file using CSS custom properties (design tokens), making the UI themable and consistent without visual checking.

### Concrete Deliverables

1. **`assets/styles.css`** - New external CSS file with:
   - Design tokens as CSS custom properties (`:root` block)
   - All component styles using variables
   - Responsive breakpoint
   - Mobile-friendly adjustments

2. **`assets/index.html`** - Updated to:
   - Reference external `styles.css`
   - Replace hardcoded colors with CSS variable references

### Definition of Done

- [ ] `docs/system/ui_design.md` updated with implementation notes
- [ ] `assets/styles.css` exists with all design tokens in `:root`
- [ ] `assets/index.html` loads `styles.css` via `<link>`
- [ ] No hardcoded colors remain in HTML (all use `var(--...)`)
- [ ] Dashboard renders correctly (server starts, page loads)
- [ ] Responsive: works on 375px mobile width
- [ ] Tests added to `e2e_tests.rs` and passing

### Must Have

- External CSS file (not inline)
- All design tokens from `docs/system/ui_design.md` converted to CSS variables
- No visual degradation from current state

### Must NOT Have

- Tailwind CSS or other framework (keep it simple)
- JavaScript changes (only CSS)
- Breaking changes to current layout

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: Tests after (e2e_tests.rs)
- **Framework**: Playwright (via e2e_tests)

### QA Policy
Every task MUST include agent-executed QA scenarios.

**QA Scenarios:**

```
Scenario: Dashboard loads correctly
  Tool: Bash (curl)
  Preconditions: Server running on port 3000
  Steps:
    1. curl -s http://localhost:3000/ | grep -q "Chronicler Engine"
    2. curl -s http://localhost:3000/styles.css | grep -q ":root"
  Expected Result: Both return 200, content present
  Evidence: curl output captured

Scenario: CSS variables are used (not hardcoded)
  Tool: Bash
  Preconditions: None
  Steps:
    1. curl -s http://localhost:3000/styles.css | grep -c "var(--"
    2. curl -s http://localhost:3000/ | grep -c "#0a0a0a\|#111\|#333"
  Expected Result: Task 1 returns >10 (variables present), Task 2 returns 0 (no hardcoded)
  Evidence: grep output counts

Scenario: Mobile responsive
  Tool: Bash (curl)
  Preconditions: None
  Steps:
    1. curl -s http://localhost:3000/styles.css | grep -q "@media"
  Expected Result: Media query found
  Evidence: grep output
```

---

## TODOs

- [x] 1. Update `docs/system/ui_design.md` with CSS variable implementation notes

  **What to do**:
  - Add section documenting that CSS custom properties (variables) are the implementation approach
  - Document the file location: `assets/styles.css`
  - Add note about responsive breakpoints
  - Link this spec to the actual implementation

  **References**:
  - `docs/system/ui_design.md` - Existing spec to update

  **Acceptance Criteria**:
  - [ ] Implementation note added to spec
  - [ ] File location documented
  - [ ] Links to actual CSS file

- [x] 2. Create `assets/styles.css` with design tokens

  **What to do**:
  - Create new file `assets/styles.css`
  - Add `:root` block with all design tokens as CSS custom properties
  - Copy component styles from `index.html` `<style>` block, replacing hardcoded values with `var(--...)`
  - Add `@media` queries for responsive design

  **References**:
  - `docs/system/ui_design.md` - Design token specifications (source of truth)
  - `assets/index.html` - Current inline CSS to convert

  **Acceptance Criteria**:
  - [ ] `assets/styles.css` exists
  - [ ] Contains `:root` block with >20 CSS custom properties
  - [ ] All component styles use `var(--...)` not hardcoded hex

- [x] 3. Update `assets/index.html` to use external CSS

  **What to do**:
  - Add `<link rel="stylesheet" href="/assets/styles.css">` in `<head>`
  - Remove entire `<style>` block (lines 9-245)
  - Verify page still renders correctly

  **References**:
  - `assets/index.html` - Current structure
  - `assets/styles.css` - New external styles

  **Acceptance Criteria**:
  - [ ] `<link>` tag present in `<head>`
  - [ ] No `<style>` tag remains
  - [ ] `curl localhost:3000/` shows expected content

- [x] 4. Add CSS/UI tests to e2e_tests.rs

  **What to do**:
  - Add test that verifies `styles.css` loads correctly
  - Add test that verifies no hardcoded colors remain in HTML
  - Add test for responsive breakpoint (mobile width)
  - Ensure tests follow existing pattern from `e2e_tests.rs`

  **References**:
  - `chronicler_engine/tests/e2e_tests.rs` - Existing test patterns
  - This task uses Playwright - ensure browser skill is loaded

  **Acceptance Criteria**:
  - [ ] New test for CSS file loading
  - [ ] New test for CSS variables (not hardcoded)
  - [ ] Tests pass with `cargo test --test e2e_tests`

---

## Commit Strategy

- **1**: `feat(ui): extract CSS to external design tokens file` - assets/styles.css, assets/index.html
- **Pre-commit**: `cargo build --quiet` (verify server still compiles)

---

## Success Criteria

### Verification Commands
```bash
curl -s http://localhost:3000/ | grep -o 'styles.css'  # Should find link tag
curl -s http://localhost:3000/styles.css | grep -c "var(--"  # Should be >10
```

### Final Checklist
- [ ] External CSS file exists
- [ ] Design tokens used throughout
- [ ] No hardcoded colors in HTML
- [ ] Dashboard still renders
- [ ] Responsive breakpoint added
- [ ] Docs updated with implementation notes
- [ ] Tests added and passing