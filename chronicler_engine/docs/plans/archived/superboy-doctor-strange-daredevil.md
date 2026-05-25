# Plan: Phase 3 UI — Swipe Navigation & Retrigger

## Current State

Phase 3 is **~70% complete**. The following already exists and works:

- `POST /message/:id/swipe/:index` endpoint — switches active swipe, restores snapshot
- `StoryLogTemplate` — renders swipe arrows + counter on the last message
- `switchSwipe()` JS — calls the endpoint, refreshes story log
- `submitRetry()` JS — calls `POST /retry`, which now creates new swipes via `post_retry_swipe_migration()`
- Swipe CSS — complete styling for controls

## What's Missing

### 1. Right arrow on latest swipe generates a new swipe (replaces "Retry" button)

**Current behavior:** The right arrow (▶) is **disabled** when on the latest swipe. There's a separate "Retry" button (↻) in the message actions area.

**Required behavior:** 
- Remove the separate "Retry" button from the template entirely.
- The right arrow on the latest swipe should call a new JS function `submitNewSwipe()` that triggers generation of a new swipe variant.
- The right arrow when NOT on the latest swipe continues to call `switchSwipe()` to navigate to the next existing swipe.
- The backend can still use the existing `/retry` endpoint (or we rename it to `/swipe/new` — see options below).

**Files:**
- `src/server/templates.rs` — remove retry button, update right arrow logic
- `assets/index.html` — new `submitNewSwipe()` JS function

### 2. `switch_swipe_handler` must validate message is the last message

**Current behavior:** The handler accepts swipe switches on **any** message.

**Required behavior:** Return 400 if the message is not the last message. Only the last message is swipeable per design constraint.

**File:** `src/server/fragments/misc.rs`

### 3. `POST /retrigger` endpoint + "Retrigger Event" button

**Use case:** When the user swipes back to an old narration whose snapshot has `last_trigger` data, they can click "Retrigger Event" to run trigger continuation from that restored state and generate the event.

**Components:**
- `src/server/fragments/misc.rs` — new `retrigger_handler`
- `src/server/mod.rs` — register `POST /retrigger` route
- `src/server/templates.rs` — add `show_retrigger` field to `LogEntryView`, render button
- `assets/index.html` — new `submitRetrigger()` JS function
- `assets/styles.css` — `.retrigger-btn` styles

**Handler logic:**
1. Load current state
2. Verify `state.narrative.last_trigger.is_some()`
3. Verify last message is narration (not event)
4. Set status to `Generating`, save generating snapshot
5. Spawn blocking task: `pipeline.run_trigger_continuation(state, trigger, input_text)`
6. Return "Retriggering..."

**Template logic:** Show "Retrigger Event" button on the last narration message when `show_retrigger == true` (passed from handler based on `last_trigger` presence).

## Endpoint Naming Options

The user noted: **"It shouldn't be 'retry' anymore."**

| Approach | UI Function | Endpoint | Backend |
|----------|-------------|----------|---------|
| **A. Keep `/retry`, hide the name** | `submitNewSwipe()` calls `/retry` | `/retry` (unchanged) | `retry_last_response_impl()` (unchanged) |
| **B. Rename endpoint to `/swipe/new`** | `submitNewSwipe()` calls `/swipe/new` | `/swipe/new` | Same handler body, new route name |
| **C. Full rename** | `submitNewSwipe()` calls `/swipe/new` | `/swipe/new` | Rename `retry_last_response_impl` to `generate_new_swipe_impl` throughout |

**Recommendation: Option B** — minimal code change, no "retry" terminology exposed in the UI or API.

## Files to Modify

1. `src/server/templates.rs` — right arrow behavior, remove retry button, retrigger button
2. `src/server/fragments/misc.rs` — last-message validation, retrigger handler
3. `src/server/mod.rs` — register `/retrigger`, optionally rename `/retry` to `/swipe/new`
4. `assets/index.html` — `submitNewSwipe()` and `submitRetrigger()` JS
5. `assets/styles.css` — `.retrigger-btn` styles
6. `tests/components/misc.rs` — add retrigger test + last-message validation test

## Risks

- **Retrigger complexity:** The retrigger handler needs access to `pipeline.run_trigger_continuation`, which is currently `pub(crate)` in `retry.rs`. We may need to expose it via `GameService` trait or call it differently.
- **GameService trait change:** Adding `retrigger_event()` to `GameService` trait affects all implementations (mock, test, etc.).

## Success Criteria

- Right arrow on latest swipe triggers new swipe generation (no "retry" button visible)
- Swipe switching rejected with 400 for non-last messages
- Retrigger endpoint runs trigger continuation when `last_trigger` exists
- All existing tests pass; new tests cover retrigger and validation
