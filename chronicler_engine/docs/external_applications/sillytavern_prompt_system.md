# SillyTavern Prompt System Reference

> **Status:** historical/reference, not authoritative. Comparison source for the layered prompt system; not a chronicler design source.

## Overview

SillyTavern is a popular open-source frontend for LLMs, famous for its sophisticated prompt management system.

This document explains SillyTavern's system as a reference for understanding its prompt construction approach.

## Source

- **Official Docs**: https://docs.sillytavern.app/usage/prompts/prompt-manager/
- **GitHub**: https://github.com/SillyTavern/SillyTavern
- **Source Code**: `public/scripts/PromptManager.js`

## Layered Prompt Construction

SillyTavern uses a layered prompt construction system where each layer serves a specific purpose. Each layer can be configured with:
- **Role**: System, User, or Assistant
- **Position**: Absolute (top/bottom) or Relative (depth in history)
- **Depth**: How far from end of history to inject
- **Order**: Numeric ordering within same depth/role
- **Triggers**: Conditional inclusion based on context

### Layer Types

#### Main Prompt (System)
- **Role**: System
- **Position**: Absolute (top)
- **Content**: Global rules, AI persona, narrative style guidelines

#### Character Description
- **Role**: System
- **Content**: Character sheets for active characters
- **Includes**: name, description, personality, scenario, example_dialogue

#### Persona Description
- **Role**: System
- **Content**: User's persona and description

#### World Info / Lorebook
- **Role**: System
- **Trigger**: Keyword matching in conversation
- **Content**: World lore, setting facts, background information
- **Features**:
  - `name`: Entry identifier
  - `content`: The lore text
  - `keywords`: Array of trigger words
  - `priority`: Order if multiple match
  - Regex support for advanced pattern matching
  - Format templates to wrap content in specific tags

#### Chat History
- **Role**: User/Assistant alternating
- **Content**: Full conversation history (up to token limit)
- **Features**:
  - Injected at configurable "depth" relative to end
  - Can use "Continue" nudges to extend conversations
  - Supports "squashing" consecutive system messages

#### User Message
- **Role**: User
- **Content**: The user's current message

#### Post-History Instructions (PHI / Jailbreak)
- **Role**: System (injected as user)
- **Position**: After history, before response
- **Content**: Final behavioral instructions

## Token Budget Management

- **maxContext**: Maximum tokens in context window (e.g., 8192)
- **maxResponse**: Tokens reserved for LLM response (e.g., 1024)
- **availableForPrompt**: maxContext - maxResponse
- **Strategies**:
  1. Truncation (remove oldest messages)
  2. Summarization (compress history)
  3. Hierarchical memory

## Context Templates

SillyTavern uses template variables (Handlebars-style):
- `{{char}}` - Character name
- `{{user}}` - User name
- `{{description}}` - Character description
- `{{scenario}}` - Character scenario
- `{{personality}}` - Character personality

## Prompt Manager UI Concepts

SillyTavern provides drag-and-drop prompt ordering with:
- **Position**: Absolute (top/bottom) or Relative (depth in history)
- **Depth**: How far from end of history to inject
- **Order**: Numeric ordering within same depth/role
- **Triggers**: Conditional inclusion based on context

## Character Card Format

SillyTavern character cards (Jailbreak format):
```json
{
  "name": "Character Name",
  "description": "Physical appearance, personality",
  "personality": "Behavior traits",
  "scenario": "Setting context",
  "example_dialogue": "Sample conversations"
}
```

## References

- SillyTavern Prompt Manager: https://docs.sillytavern.app/usage/prompts/prompt-manager/
- SillyTavern Prompt Building: https://docs.sillytavern.app/usage/prompts/prompt-building/
- Prompt Assembly Pipeline: https://deepwiki.com/SillyTavern/SillyTavern/3.3-prompt-assembly-pipeline

## Document References

- [ADR-005: SillyTavern-Style Layered Prompt System](../adr/adr-005-layered-prompts.md) — chronicler layered prompt design
