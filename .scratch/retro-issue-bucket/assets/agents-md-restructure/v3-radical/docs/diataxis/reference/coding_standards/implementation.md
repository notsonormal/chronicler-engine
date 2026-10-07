---
diataxis: reference
title: Implementation
---

## Backward compatibility

Do not preserve backward compatibility unless the user asks for it.

## Naming

Symbols (functions, types, variables) must use verbose, domain-aligned names that map 1-to-1 with concepts in the `docs/` (where such symbols are present).

## HTTP form structs

`axum::extract::Form` payloads must deserialize from partial urlencoded bodies: mark fields a legacy poster may omit with `#[serde(default)]`, and encode checkbox groups as one boolean field per value.
