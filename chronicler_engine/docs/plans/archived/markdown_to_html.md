# Plan: Markdown to HTML Text Formatting

**Created:** 2026-04-13
**Status:** completed

## Problem

The Chronicler Engine receives narrative text from the LLM that includes markdown formatting (italics with `*`, bold with `**`, quotes with `"`). Currently, text is rendered using `format_text_with_newlines()` which only handles line breaks - markdown formatting is not converted to HTML.

This results in raw markdown appearing in the chat UI instead of styled HTML.

## Solution

Integrate `pulldown-cmark` library to parse markdown text from LLM and convert to HTML before rendering to the UI.

### Goals

1. Parse markdown in narrative text (italics `*`, bold `**`, quotes `"`) 
2. Convert to semantic HTML (`<em>`, `<strong>`, `<q>`)
3. Maintain existing paragraph/newline handling
4. Convert quotes to `<q>` tags for easy CSS styling (not `<blockquote>`)

## Files Changed

| File | Change |
|------|--------|
| `src/server/templates.rs` | Added `parse_markdown()` function with `<q>` tag post-processing, integrated in `LogEntryView::from()` |
| `src/server/fragments.rs` | Removed unused `parse_markdown()` and `format_text_with_newlines()` functions, cleaned up imports |

## Implementation Summary

1. Added `pulldown-cmark` import to templates.rs
2. Created `parse_markdown()` function that:
   - Parses markdown via pulldown_cmark (converts `"..."` → Unicode curly quotes)
   - Post-processes curly quotes to `<q>` tags for dialogue display
3. Modified `LogEntryView::from()` to call `parse_markdown()` on text
4. Added 8 tests validating markdown parsing (quotes, italic, bold, blockquote, mixed)
5. Removed dead code from fragments.rs (unused `parse_markdown` and `format_text_with_newlines`)
6. Cleaned up unused imports in fragments.rs

## Test Results

All 46 library tests pass, 3 binary tests pass, 4 behavior tests pass.

## Notes

- Quotes are converted to `<q>` (inline) not `<blockquote>` (block) - allows easy CSS styling
- LLM example: `"Well, well," Gabriella remarks` → `<q>Well, well,</q> Gabriella remarks`