# Plan: Unify Lock-Poison Recovery on Strategy A (Recover + Log)

## Goal
Every poisoned-lock site in the codebase recovers via `into_inner()` and logs a warning. No silent defaults, no user-facing errors.

## Files to Change

| File | Line(s) | Current Behavior | New Behavior |
|------|---------|------------------|--------------|
| `src/server/settings_fragment/handlers.rs` | 10-21 | `try_lock!` macro returns HTML error on poison | New `try_lock!` macro recovers via `into_inner()` + `log::warn!` |
| `src/server/settings_fragment/handlers.rs` | 40, 73, 90, 112, 151, 169, 185, 222, 255, 275 | All 12 call sites use old `try_lock!` | Same call sites, now resilient |
| `src/server/mod.rs` | 298 | `settings()` returns `unwrap_or_default()` on poison | Recover via `into_inner().clone()` + `log::warn!` |
| `src/server/mod.rs` | 279, 288, 362 | Already recover (cancel token) | Add `log::warn!` for observability |

## Implementation Details

### Step 1: Rewrite `try_lock!` macro
```rust
macro_rules! try_lock {
    ($lock:expr) => {
        match $lock {
            Ok(g) => g,
            Err(p) => {
                log::warn!("Poisoned lock recovered in settings handler");
                p.into_inner()
            }
        }
    };
}
```

### Step 2: Fix `AppState::settings()`
```rust
pub fn settings(&self) -> AppSettings {
    match self.settings.read() {
        Ok(g) => g.clone(),
        Err(p) => {
            log::warn!("Poisoned settings lock recovered");
            p.into_inner().clone()
        }
    }
}
```

### Step 3: Add logging to existing recovery sites
- `server/mod.rs` `current_cancel_token()` — add `log::warn!("Poisoned cancel_token read lock recovered");`
- `server/mod.rs` `replace_cancel_token()` — add `log::warn!("Poisoned cancel_token write lock recovered");`
- `server/mod.rs` `shutdown_signal` — add `log::warn!("Poisoned cancel_token read lock recovered during shutdown");`

### Step 4: Verify no other inconsistent sites remain
Search pattern: `.read().map(` or `.read().unwrap_or_default` or `.write().map(` or `Err(_) =>` near lock calls.

### Step 5: Run validation
```bash
cd chronicler_engine && python build.py
```

## Success Criteria
- [ ] `settings_fragment` handlers no longer return poison error HTML
- [ ] `AppState::settings()` no longer returns defaults on poison
- [ ] All `RwLock`/`Mutex` poison sites recover via `into_inner()`
- [ ] All recovery sites log a warning
- [ ] `python build.py` passes (fmt + clippy + tests)
