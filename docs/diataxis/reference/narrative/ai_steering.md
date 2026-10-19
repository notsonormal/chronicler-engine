---
diataxis: reference
title: AI Steering
---

# AI Steering

## Overview

Two steering commands let the player shape what the narrator generates. Both enter through the single command input as slash commands (`/guide`, `/impersonate`) and differ in what they steer.

| Command | What it steers | Output message type | Retry re-applies via |
|---|---|---|---|
| Guided generation | content — what the AI says | `Narration` | the swipe's stored guide text |
| Impersonate | speaker — who says it | `Input` | the swipe's stored impersonation flag and impersonate direction |

Both commands are transient: their steering lives for one generation and never becomes a history entry. A generation resolves one steering source at a time. On an impersonated generation, the impersonate preset replaces the system preset.
