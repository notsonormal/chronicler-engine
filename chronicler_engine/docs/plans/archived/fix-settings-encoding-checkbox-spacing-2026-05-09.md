# Plan: Fix Settings Panel Encoding and Checkbox Spacing

## Problem

Two UI defects in the settings panel:

1. **Encoding corruption**: The em-dash character between provider name and model name renders as `â€"` (UTF-8 bytes misinterpreted as Windows-1252). Example: `OpenRouter â€" openai/gpt-4o-mini`.

2. **Checkbox spacing too tight**: Checkbox labels use `label:has(> input[type="checkbox"])` with `gap: var(--spacing-xs)` (4px). The `:has()` pseudo-class has limited browser support, and 4px is barely visible even when it works.

## Solution

1. Replace corrupted `â€"` with simple hyphen ` - ` in all occurrences.
2. Add explicit `.checkbox-label` class to checkbox labels.
3. Update CSS to target `.checkbox-label` instead of `:has()`, increase gap from 4px to 8px.

## Files to Change

- `src/server/settings_fragment.rs` — fix 3 corrupted dashes, add `class="checkbox-label"` to 3 labels
- `assets/styles.css` — replace `:has()` selector with `.checkbox-label`, bump gap

## Acceptance Criteria

- [ ] No `â€"` characters remain in `settings_fragment.rs`
- [ ] All 3 checkbox inputs are inside `.checkbox-label`
- [ ] CSS uses `.checkbox-label` not `label:has(> input[type="checkbox"])`
- [ ] Checkbox label gap = `var(--spacing-sm)` (8px)
- [ ] `python build.py` passes (fmt, clippy, tests)
- [ ] Visual verification: screenshot shows correct dashes and checkbox spacing

## Dependencies

None. Self-contained UI fix.
