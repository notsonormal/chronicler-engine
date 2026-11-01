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

SillyTavern uses a layered prompt construction system where each layer serves a specific purpose. One OpenAI preset file holds the layers (`prompts` and `prompt_order`), the connection, the model, the samplers and the utility prompts. An author exports, imports and shares the preset as that file. Each layer can be configured with:
- **Role**: System, User, or Assistant
- **Position**: Relative (placed by its index in the prompt list) or In-chat (placed at a depth in the chat history)
- **Depth**: How far from end of history to inject (in-chat prompts only)
- **Order**: Numeric ordering within same depth/role (in-chat prompts only)
- **Triggers**: Conditional inclusion based on the generation type (`normal`, `continue`, `impersonate`, `swipe`, `regenerate`, `quiet`)

The Main Prompt and the Post-History Instructions have a lock, `forbid_overrides`. The lock stops a character card from replacing the text of these two prompts. The on/off switch of a prompt is the `enabled` flag of its entry in `prompt_order`.

The preset also holds the utility prompts: `impersonation_prompt`, `wi_format`, `scenario_format`, `personality_format`, `group_nudge_prompt`, `new_chat_prompt`, `new_group_chat_prompt`, `new_example_chat_prompt`, `continue_nudge_prompt` and `send_if_empty`. It also holds `continue_postfix`, `assistant_prefill` and `assistant_impersonation`.

The Author's Note and the depth prompt of a character card are extension prompts. The Author's Note has a position, a depth, a role and an interval. Its default position is in-chat. The depth prompt is always in-chat, and it has a depth and a role.

### Layer Types

#### Main Prompt (System)
- **Role**: System
- **Position**: Relative by default. It is a normal entry of the prompt list.
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
- **Trigger**: Keyword matching in conversation (the default)
- **Content**: World lore, setting facts, background information
- **Features**:
  - `name`: Entry identifier
  - `content`: The lore text
  - `keywords`: Array of trigger words
  - `order`: Order if multiple match
  - Regex support for advanced pattern matching
  - Format templates to wrap content in specific tags
  - Eight positions, with a depth and a role for the `atDepth` position
  - Per-entry `sticky`, `cooldown`, `delay`, `group` and `probability`
  - A keyword is not the only trigger: `constant` and vectorized entries also exist

#### Chat History
- **Role**: User/Assistant alternating
- **Content**: Full conversation history (up to token limit)
- **Features**:
  - `chatHistory` is a marker. The prompt order places it.
  - Prompts and extension prompts reach into it at a configurable "depth" relative to the end
  - Can use "Continue" nudges to extend conversations
  - Supports "squashing" consecutive system messages

#### User Message
- **Role**: User
- **Content**: The user's current message

#### Post-History Instructions (PHI / Jailbreak)
- **Role**: System by default. The author can change it.
- **Position**: Set by the prompt order. The default preset puts it after the history.
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

SillyTavern uses Handlebars-style blocks such as `{{#if}}` only in the text-completion story string. The macros below work in the prompt text of both APIs.
- `{{char}}` - Character name
- `{{user}}` - User name
- `{{description}}` - Character description
- `{{scenario}}` - Character scenario
- `{{personality}}` - Character personality

## Prompt Manager UI Concepts

SillyTavern provides drag-and-drop prompt ordering with:
- **Position**: Relative (placed by its index in the prompt list) or In-chat (placed at a depth in the chat history)
- **Depth**: How far from end of history to inject (in-chat prompts only)
- **Order**: Numeric ordering within same depth/role (in-chat prompts only)
- **Triggers**: Conditional inclusion based on the generation type

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

A card also carries `system_prompt`, `post_history_instructions`, `character_version` and `creator_notes`. It also carries `extensions.depth_prompt` with `prompt`, `depth` and `role`. `system_prompt` can replace the Main Prompt, `post_history_instructions` can replace the Post-History Instructions, and the depth prompt adds an in-chat prompt.

## References

- SillyTavern Prompt Manager: https://docs.sillytavern.app/usage/prompts/prompt-manager/
- SillyTavern Prompt Building: https://docs.sillytavern.app/usage/prompts/prompt-building/
- Prompt Assembly Pipeline: https://deepwiki.com/SillyTavern/SillyTavern/3.3-prompt-assembly-pipeline

## Document References

- [Prompt System Design](../diataxis/explanation/prompt_system_design.md) — chronicler layered prompt design
