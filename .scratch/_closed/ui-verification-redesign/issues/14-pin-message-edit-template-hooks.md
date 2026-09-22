# Pin the message-edit template hooks the edit JavaScript reads

Type: task
Status: resolved
Blocked by: (none — graduated from ticket 11's Answer, section 5)

## Question

Ticket 07 recorded a drift-tax gap on the story-log edit path, and ticket 11's
sweep corrected the record before graduating it here. The corrected gap:

- The edit JavaScript (`showEditForm`, `assets/index.html:242`) reads
  `data-raw-text` from the rendered entry to seed the edit textarea; it also
  selects the entry by `data-id` and swaps content into the `class="text"` span.
- **No gated test pinned any of the three on the real template.** The unit tier
  pins `edit-btn`/`delete-btn`/`message-actions`/`check-btn`
  (`templates_tests.rs`) but not the data attributes; the gated HTTP tier counts
  `class="log-entry"` only; tier 2 reads the canned fixture
  (`tests/test_utils/stub_fixtures/story_log.html`), which is a separate file —
  so a template change cannot fail a fixture-reading test.
- Mutation proof (pre-fix): renaming `data-raw-text` in the real template left
  the full gate **green** (1604 integration / 137 guardrails / 20 browser /
  1 architecture, 0 failed) while the product broke — Edit opens an empty
  textarea and a save writes an empty value. The failure is silent, not the
  loud failure ticket 07 predicted.

Question: pin the hooks at the right tier, with evidence that each assertion
actually fires on the break it claims to catch.

## Answer

Resolved 2026-09-20, same session as ticket 11 (user-directed "finish now"
override of the one-ticket-per-session stop).

### 1. The fix — one unit test on the real template

`test_story_log_template_edit_path_hooks` in
`src/adapters/driving/http/templates_tests.rs` (alongside its 40 sibling
template tests; the template is Rust, so no browser and no route are needed).
It renders the real `NarrativeLogTemplate` and asserts:

| Assert | Pins |
|---|---|
| `data-id="1"` | the entry selector `showEditForm`/`cancelEdit` resolve by |
| `data-raw-text="Raw body text"` | the textarea seed the edit JS reads |
| `<span class="text">` | the span the textarea replaces into |
| `showEditForm(1)` | the edit button's onclick wiring carries the id |
| `data-raw-text="He said &#34;hi&#34;"` | the `\| escape` wire form — a raw quote must not terminate the attribute |

No doc comment on the test: sibling tests carry none, and the
fixture-vs-template separation insight lives in `tests/STRATEGY.md`, not in a
comment that would rot.

### 2. Every assert is mutation-proven, not assumed

Each break was applied to the real template
(`src/adapters/driving/http/templates.rs`) and the test run in isolation:

| Mutation | Result |
|---|---|
| `data-raw-text` renamed | fails — diagnostic shows `data-raw-text-mutated` in the render |
| `data-id` renamed | fails |
| `<span class="text">` renamed | fails |
| `showEditForm(` renamed | fails |
| `\| escape` → `\| safe` | fails — diagnostic shows the broken wire form `data-raw-text="He said "hi""` |

Two review corrections recorded along the way:

- The first draft's escape mutation was "remove `| escape`" — a **no-op**:
  askama auto-escapes templates with `ext = "html"`, so the default escaper
  still runs. Replaced with the real risk, the `| safe` bypass.
- The `&#34;` expectation was initially taken from the hand-written tier-2
  fixture, not template output. The clean run confirmed it against real
  template output; the `| safe` mutation run then showed the unescaped form
  directly.

### 3. Accepted residual

Substring asserts cannot pin **nesting** (`.text` inside `.log-entry`). A
change that moves the span outside the entry div still passes. Accepted: the
rename risk is the real one, and a nesting check needs an HTML parser or a
browser page — machinery this project prefers not to add for one assertion.

### 4. Verification

| Check | Result |
|---|---|
| Full gate (`python build.py`) | **green, exit 0** |
| integration tests | 1605 passed, 0 failed, 2 skipped (was 1604 — +1 is this test) |
| guardrail tests | 137 passed, 0 failed |
| browser tests | 20 passed, 0 failed |
| architecture tests | 1 passed, 0 failed |
| `validate_feature_spec.py` | 141 declared, 141 covered, 0 gap(s), 0 orphan(s), 0 untagged, 0 surface mismatch(es), quarantine 85/85 — unchanged, no scenario changed |

Files touched: `src/adapters/driving/http/templates_tests.rs` (+32). No
production source changed — the template was already correct; the test makes
its contract load-bearing. All mutation backups restored; `git diff` on
`templates.rs` is empty.

### 5. Ticket 07's "loud failure" reasoning — corrected for the record

Ticket 07 argued the tier-2 fixture keeps the structural hooks, so a template
rename "fails loudly (element not found), not silently". That holds only in one
direction: the fixture test fails if the **fixture** loses a hook. It says
nothing about the **template** losing one — the two artifacts are separate
files and nothing compares them. The pre-fix mutation run is the proof: green
gate, broken product. This correction is recorded in ticket 11's Answer
(section 5) and in the map's ticket-11 line.
