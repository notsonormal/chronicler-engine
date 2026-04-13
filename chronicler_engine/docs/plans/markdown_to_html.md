# Plan: Markdown to HTML Text Formatting

**Created:** 2026-04-13
**Status:** in_progress

## Problem

The Chronicler Engine receives narrative text from the LLM that includes markdown formatting (italics with `*`, bold with `**`, quotes with `"`). Currently, text is rendered using `format_text_with_newlines()` which only handles line breaks - markdown formatting is not converted to HTML.

This results in raw markdown appearing in the chat UI instead of styled HTML.

## Solution

Integrate `pulldown-cmark` library to parse markdown text from LLM and convert to HTML before rendering to the UI.

### Goals

1. Parse markdown in narrative text (italics `*`, bold `**`, quotes `"`) 
2. Convert to semantic HTML (`<em>`, `<strong>`, `<blockquote>`)
3. Maintain existing paragraph/newline handling
4. Escape HTML for security before markdown parsing

## Files to Change

| File | Change |
|------|--------|
| `Cargo.toml` | Add `pulldown-cmark` dependency |
| `src/server/fragments.rs` | Add markdown parsing function, integrate with `render_log_entry()` |

## Implementation Steps

1. Add `pulldown-cmark = "0.13"` to Cargo.toml dependencies
2. Create `parse_markdown()` function in fragments.rs
3. Update `render_log_entry()` to call `parse_markdown()` after `html_escape()`
4. Validate with cargo fmt, clippy, test

## Trade-offs Considered

- **pulldown-cmark**: Fast, lightweight, minimal allocations ✓ (chosen)
- **comrak**: More features, security sanitization built-in, but heavier and requires Rust 1.85+

## Test Cases

- Input: `*italic text*` → Output: `<em>italic text</em>`
- Input: `**bold text**` → Output: `<strong>bold text</strong>`
- Input: `"quoted text"` → Output: `<blockquote>quoted text</blockquote>`
- Input: `Normal *italic* and **bold** mixed.` → Output: `<p>Normal <em>italic</em> and <strong>bold</strong> mixed.</p>`

## Notes

- The LLM returns markdown-formatted prose, so we need to parse it before displaying
- Must HTML-escape BEFORE parsing to prevent XSS
- Keep existing `format_text_with_newlines()` behavior for non-markdown text