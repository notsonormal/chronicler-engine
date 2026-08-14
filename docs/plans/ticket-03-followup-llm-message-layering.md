# Follow-up: Remove storage → application `LlmMessage` dependency

## Problem
`src/adapters/driven/storage/in_memory_data.rs` and `src/adapters/driven/storage/llm_messages.rs` import `crate::application::llm_message::LlmMessage`. This is an inward dependency from a driven adapter (`storage`) into the `application` layer, violating the hexagonal dependency invariant (`domain` → `application` → `adapters`; adapters depend only on domain/ports).

This is pre-existing; Ticket 03's flatten made it visible, not new.

## TODO
1. Investigate why `LlmMessage` lives in `application/llm_message.rs` and whether it belongs there or in `domain/model/message.rs` / a shared DTO.
2. List every file that imports `application::llm_message::LlmMessage`.
3. Decide fix: move the type down to `domain`, or split into a port DTO in `application/ports/` and a storage DTO in `domain/model/state/`.
4. Apply the move and update all import sites.
5. Run `python build.py`.

## Scope note
Do not change storage behavior. Pure relocation of the type / import paths.
