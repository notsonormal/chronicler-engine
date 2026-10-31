# Research: a structural duplicate report beside jscpd

Date: 2026-10-31. Ticket: [02 — what a structural duplicate report shows beside jscpd](../issues/02-research-duplicate-report.md).
Scratch evidence: `tmp/unclebob/` (source copies, run output, the two join scripts; git-ignored).

## Answer

At its defaults, dryer adds **two** actionable production pairs that jscpd misses, and **42** idiom pairs that no one will act on. On the same scope jscpd finds 184 clone fragments. So jscpd stays the default and dryer earns its place only with structural ignore rules: 42 of the 52 dryer pairs fall into seven idioms, and one rule per idiom removes them. The seven rules cost one real pair (`from_config`) and leave nine, all real.

Test code is where both tools find most of the duplication: dryer returns 409 pairs for `*_tests.rs` + `src/test_support/` and 1,016 for `tests/`, against 52 for production, and 92.5% of the gate's own jscpd report is test clones. So the first decision is scope: is the report about production code, or about all code?

## How the runs were made

dryer 0.1.0 (`tmp/unclebob/.venv`), jscpd 5.4.2 — the version `scripts/healthcheck.py` pins. Both tools got a fresh copy of the current `src/` (2026-10-31 12:52: 318 `.rs` files, 42,124 lines), not the 2026-10-25 copy in `tmp/unclebob/ce/`.

| Scope | Copy | Removed |
|---|---|---|
| production | `tmp/unclebob/prod/` | `*_tests.rs` (86 files), `src/test_support/` (10 files) |
| unit tests | `tmp/unclebob/testscope/` | everything except `*_tests.rs` and `src/test_support/` |
| integration tests | `tmp/unclebob/itests/` | nothing (a copy of `tests/`) |

Four traps:

1. **`--root ..` rescopes the walk.** `dryer --root .. -s .` from a copy directory walked the sibling clones and reported 786 pairs of crapper, mutator and dryer code. Run dryer from inside the copy and pass no paths.
2. **dryer skips any directory named `tests`.** `discover.py` holds `TEST_DIRS = {"test", "tests", "spec", "specs", "__tests__"}`. A copy of `tests/` must sit in a differently named directory. The run above uses `itests/`.
3. **dryer does not skip `*_tests.rs`.** The per-language test-file patterns cover `_test.go`, `_test.clj`, `test_*.py` and `.spec.ts`; the Rust pattern is absent. So the production scope needs the copy with `*_tests.rs` deleted, as the map says. The 409-pair test run is what happens when it is not.
4. **A third test-only location.** `src/adapters/driven/storage/test_support.rs` is `#[cfg(feature = "testing")]` (`src/adapters/driven/storage/mod.rs:25-29`). The map names `*_tests.rs` and `src/test_support/` only. The 52-pair list below drops that file; keeping it adds two idiom pairs (`set`/`set_write`, `config`/`internal`).

jscpd ran with the `healthcheck.py` settings — `--min-lines 3 --min-tokens 30 --format rust` — on each scope separately, so the counts are not mixed. Output is under `tmp/unclebob/out/`, not `report/`, because the full gate rewrites `report/jscpd-report.json`.

## Counts side by side

| Scope | `.rs` files | Lines | dryer pairs (defaults) | dryer real (A+B) | jscpd clones | jscpd duplicated lines |
|---|---|---|---|---|---|---|
| production | 221 | 20,799 | **52** | **10** | 184 | 5.19% (1,072) |
| unit tests (`*_tests.rs` + `src/test_support/`) | 96 | 21,242 | **409** | not classified | 667 | 23.22% (4,933) |
| integration tests (`tests/`) | 137 | 52,354 | **1,016** | not classified | 1,492 | 19.69% (10,305) |
| repo-wide, the gate's own report (2026-10-31) | 412 sources | 94,231 | — | — | 2,427 | 17.81% |

jscpd ran on the 222-file copy, so its production figures include `src/adapters/driven/storage/test_support.rs`; dryer's 52 excludes it. jscpd's own line count for that scope is 20,663 across "184 sources". 222 files were in scope, so "sources" appears to count the files that hold clones (inferred: the figure does not change when the ignored file is restored to the copy).

**The gate's report is 92.5% test clones.** Of the 2,427 clones in `report/jscpd-report.json`, 2,246 have at least one side under `tests/`, `src/test_support/` or a `*_tests.rs` file. The production list inside it is 181 clones — the same order as the 184 above. So the tool lacks nothing on the test side; it buries the production side.

## Overlap with jscpd

A dryer pair counts as "covered" on one side when a jscpd fragment in that file overlaps the function's line range.

| Scope | Pairs | jscpd covers both | covers one | covers neither |
|---|---|---|---|---|
| production | 52 | 14 | 6 | 32 |
| unit tests | 409 | 207 | 20 | 182 |
| integration tests | 1,016 | 184 | 143 | 689 |

Only 2 of the production 32 are real duplicates. Of the 14 "both covered", 8 are real duplicates and 6 are enum-to-text idiom pairs that jscpd reaches only through a 7-line fragment spanning the boundary between two functions — `domain/model/settings.rs:50-56` runs from the end of `display_label` into the start of `parse_or_default` (36 tokens). So "covered" here means a fragment touched the lines, not that jscpd saw the same thing dryer saw.

## The production list, classified (52 pairs)

Class **A** = real duplicate that jscpd misses on both sides. **B** = real duplicate that jscpd also finds. **C** = idiom noise.

| Class | Score | Left | Right | What it is | Ignore rule |
|---|---|---|---|---|---|
| A | 1.00 | `src/application/agents/options/agent.rs:21`<br>`fmt` | `src/application/agents/quantifier/agent.rs:24`<br>`fmt` | `Debug` impl; struct name differs | — |
| A | 1.00 | `src/application/prompting/assembler.rs:196`<br>`render_game_state_layer` | `src/application/prompting/assembler.rs:290`<br>`render_world_info_layer` | XML layer renderers | — |
| B | 1.00 | `src/adapters/driven/storage/mappers/message.rs:16`<br>`column_result` | `src/adapters/driven/storage/mappers/message.rs:23`<br>`column_result` | same `FromSql` body for two newtypes | — |
| B | 1.00 | `src/adapters/driving/http/settings/handlers/settings.rs:423`<br>`set_narrator_handler` | `src/adapters/driving/http/settings/handlers/settings.rs:433`<br>`set_quantifier_handler` | near-clone HTTP handlers | — |
| B | 1.00 | `src/application/agents/options/agent.rs:29`<br>`from_config` | `src/application/agents/quantifier/agent.rs:32`<br>`from_config` | constructor; name literal differs | — |
| B | 1.00 | `src/application/agents/options/agent.rs:40`<br>`with_provider` | `src/application/agents/quantifier/agent.rs:43`<br>`with_provider` | identical test-only constructor | — |
| B | 0.97 | `src/application/agents/options/prompt.rs:40`<br>`build_user_prompt` | `src/application/agents/quantifier/prompt.rs:54`<br>`build_user_prompt` | near-clone prompt builders | — |
| B | 0.96 | `src/adapters/driven/storage/worlds.rs:84`<br>`get_world` | `src/adapters/driven/storage/worlds.rs:338`<br>`get_world_by_id` | near-clone row readers | — |
| B | 0.89 | `src/adapters/driven/storage/models/character.rs:49`<br>`to_card` | `src/adapters/driven/storage/models/persona.rs:42`<br>`to_card` | near-clone row → card readers | — |
| B | 0.84 | `src/adapters/driven/storage/games.rs:10`<br>`list_games` | `src/adapters/driven/storage/personas.rs:12`<br>`list_personas` | near-clone query pipelines | — |
| C | 1.00 | `src/adapters/driven/llm/providers/mock.rs:34`<br>`with_fail` | `src/adapters/driven/llm/providers/mock.rs:45`<br>`with_empty_response` | builder setter | R1 |
| C | 1.00 | `src/adapters/driven/llm/providers/mock.rs:34`<br>`with_fail` | `src/adapters/driven/llm/providers/mock.rs:50`<br>`with_trigger_narration_fail` | builder setter | R1 |
| C | 1.00 | `src/adapters/driven/llm/providers/mock.rs:45`<br>`with_empty_response` | `src/adapters/driven/llm/providers/mock.rs:50`<br>`with_trigger_narration_fail` | builder setter | R1 |
| C | 1.00 | `src/adapters/driven/llm/providers/mock.rs:55`<br>`with_narrations` | `src/adapters/driven/llm/providers/mock.rs:60`<br>`with_prompt_responses` | builder setter | R1 |
| C | 1.00 | `src/adapters/driven/llm/providers/mock.rs:55`<br>`with_narrations` | `src/adapters/driving/http/utils/answer_words.rs:85`<br>`with_item` | builder setter | R1 |
| C | 1.00 | `src/adapters/driven/llm/providers/mock.rs:55`<br>`with_narrations` | `src/application/prompting/assembler.rs:134`<br>`with_guide` | builder setter | R1 |
| C | 1.00 | `src/adapters/driven/llm/providers/mock.rs:60`<br>`with_prompt_responses` | `src/adapters/driving/http/utils/answer_words.rs:85`<br>`with_item` | builder setter | R1 |
| C | 1.00 | `src/adapters/driven/llm/providers/mock.rs:60`<br>`with_prompt_responses` | `src/application/prompting/assembler.rs:134`<br>`with_guide` | builder setter | R1 |
| C | 1.00 | `src/adapters/driven/storage/models/llm_message.rs:19`<br>`from_row` | `src/adapters/driven/storage/models/prompt_preset.rs:23`<br>`from_row` | struct literal of column reads | R2 |
| C | 1.00 | `src/adapters/driven/storage/models/llm_message.rs:19`<br>`from_row` | `src/adapters/driven/storage/models/settings.rs:25`<br>`from_row` | struct literal of column reads | R2 |
| C | 1.00 | `src/adapters/driven/storage/models/prompt_preset.rs:23`<br>`from_row` | `src/adapters/driven/storage/models/settings.rs:25`<br>`from_row` | struct literal of column reads | R2 |
| C | 1.00 | `src/adapters/driven/storage/settings.rs:95`<br>`save_settings` | `src/adapters/driven/storage/settings.rs:119`<br>`seed_settings` | thin delegation; mode argument differs | R7 |
| C | 1.00 | `src/adapters/driven/storage/worlds.rs:146`<br>`seed_world` | `src/adapters/driven/storage/worlds.rs:155`<br>`create_world` | thin delegation; mode argument differs | R7 |
| C | 1.00 | `src/adapters/driving/http/builders/headers.rs:18`<br>`role_label` | `src/adapters/driving/http/builders/headers.rs:27`<br>`role_failure_effect` | enum → text match | R3 |
| C | 1.00 | `src/adapters/driving/http/utils/answer_words.rs:85`<br>`with_item` | `src/application/prompting/assembler.rs:134`<br>`with_guide` | builder setter | R1 |
| C | 1.00 | `src/domain/model/agent.rs:57`<br>`step` | `src/domain/model/agent.rs:65`<br>`role` | accessor match over an enum | R5 |
| C | 1.00 | `src/domain/model/llm_backend.rs:41`<br>`wire_id` | `src/domain/model/llm_backend.rs:52`<br>`display_label` | enum → text match | R3 |
| C | 1.00 | `src/domain/model/llm_backend.rs:41`<br>`wire_id` | `src/domain/model/llm_backend.rs:61`<br>`default_base_url` | enum → text match | R3 |
| C | 1.00 | `src/domain/model/llm_backend.rs:52`<br>`display_label` | `src/domain/model/llm_backend.rs:61`<br>`default_base_url` | enum → text match | R3 |
| C | 1.00 | `src/domain/model/llm_backend.rs:80`<br>`attribution_title` | `src/domain/model/llm_backend.rs:92`<br>`session_header` | enum → text match | R3 |
| C | 1.00 | `src/domain/model/settings.rs:38`<br>`as_str` | `src/domain/model/settings.rs:47`<br>`display_label` | enum → text match | R3 |
| C | 1.00 | `src/domain/model/settings.rs:38`<br>`as_str` | `src/domain/model/settings.rs:233`<br>`as_str` | enum → text match | R3 |
| C | 1.00 | `src/domain/model/settings.rs:38`<br>`as_str` | `src/domain/model/settings.rs:271`<br>`as_str` | enum → text match | R3 |
| C | 1.00 | `src/domain/model/settings.rs:47`<br>`display_label` | `src/domain/model/settings.rs:233`<br>`as_str` | enum → text match | R3 |
| C | 1.00 | `src/domain/model/settings.rs:47`<br>`display_label` | `src/domain/model/settings.rs:271`<br>`as_str` | enum → text match | R3 |
| C | 1.00 | `src/domain/model/settings.rs:54`<br>`parse_or_default` | `src/domain/model/settings.rs:240`<br>`parse_or_default` | `FromStr` / `parse_or_default` impl | R4 |
| C | 1.00 | `src/domain/model/settings.rs:54`<br>`parse_or_default` | `src/domain/model/settings.rs:278`<br>`parse_or_default` | `FromStr` / `parse_or_default` impl | R4 |
| C | 1.00 | `src/domain/model/settings.rs:75`<br>`from_str` | `src/domain/model/settings.rs:251`<br>`from_str` | `FromStr` / `parse_or_default` impl | R4 |
| C | 1.00 | `src/domain/model/settings.rs:75`<br>`from_str` | `src/domain/model/settings.rs:289`<br>`from_str` | `FromStr` / `parse_or_default` impl | R4 |
| C | 1.00 | `src/domain/model/settings.rs:233`<br>`as_str` | `src/domain/model/settings.rs:271`<br>`as_str` | enum → text match | R3 |
| C | 1.00 | `src/domain/model/settings.rs:240`<br>`parse_or_default` | `src/domain/model/settings.rs:278`<br>`parse_or_default` | `FromStr` / `parse_or_default` impl | R4 |
| C | 1.00 | `src/domain/model/settings.rs:251`<br>`from_str` | `src/domain/model/settings.rs:289`<br>`from_str` | `FromStr` / `parse_or_default` impl | R4 |
| C | 1.00 | `src/domain/model/trigger.rs:72`<br>`get_times_met` | `src/domain/model/trigger.rs:87`<br>`is_currently_meeting` | `map(..).unwrap_or(..)` accessor | R6 |
| C | 0.94 | `src/application/generation/gate.rs:141`<br>`job` | `src/application/generation/gate.rs:278`<br>`step` | accessor match over an enum | R5 |
| C | 0.90 | `src/domain/model/agent.rs:33`<br>`agent_name` | `src/domain/model/llm_backend.rs:41`<br>`wire_id` | enum → text match | R3 |
| C | 0.90 | `src/domain/model/agent.rs:33`<br>`agent_name` | `src/domain/model/llm_backend.rs:52`<br>`display_label` | enum → text match | R3 |
| C | 0.90 | `src/domain/model/agent.rs:33`<br>`agent_name` | `src/domain/model/llm_backend.rs:61`<br>`default_base_url` | enum → text match | R3 |
| C | 0.85 | `src/adapters/driving/http/utils/answer_words.rs:26`<br>`reason` | `src/domain/model/change.rs:22`<br>`token` | enum → text match | R3 |
| C | 0.85 | `src/domain/model/llm_backend.rs:41`<br>`wire_id` | `src/domain/model/prompt_preset.rs:22`<br>`as_str` | enum → text match | R3 |
| C | 0.85 | `src/domain/model/llm_backend.rs:52`<br>`display_label` | `src/domain/model/prompt_preset.rs:22`<br>`as_str` | enum → text match | R3 |
| C | 0.85 | `src/domain/model/llm_backend.rs:61`<br>`default_base_url` | `src/domain/model/prompt_preset.rs:22`<br>`as_str` | enum → text match | R3 |
| C | 0.83 | `src/adapters/driven/storage/models/game.rs:30`<br>`from_row` | `src/adapters/driven/storage/models/world.rs:25`<br>`from_row` | struct literal of column reads | R2 |

### The two A pairs

**A1 — `src/application/prompting/assembler.rs:196` and `:290`, score 1.00.** Two ten-line XML layer renderers that differ only in the tag, the label and the field they read:

```rust
fn render_game_state_layer(&self) -> String {          // :196
    let mut output = String::from("<GameState>\n");
    output.push_str("Current Room: ");
    output.push_str(&self.room.name);
    ...
    output.push_str("</GameState>\n");
    output
}
fn render_world_info_layer(&self) -> String {          // :290
    let mut output = String::from("<WorldLore>\n");
    output.push_str("World: ");
    output.push_str(&self.world.name);
    ...
    output.push_str("</WorldLore>\n");
    output
}
```

jscpd cannot see it: every line carries a different literal. A helper `render_tagged_layer(tag, label, name, body)` collapses the pair.

**A2 — `src/application/agents/options/agent.rs:21` and `src/application/agents/quantifier/agent.rs:24`, score 1.00.** Two `Debug` impls that differ in the struct name:

```rust
f.debug_struct("OptionsGeneratorAgent").field("name", &self.name).finish_non_exhaustive()
f.debug_struct("QuantifierAgent").field("name", &self.name).finish_non_exhaustive()
```

Real, and small. A macro removes it. This is a judgement call, not a defect.

### The B pairs

These are the "we agree the code is duplicated" list. The user already has them from jscpd, but dryer names them at function level and gives a score, which is what a reviewer needs: `column_result` (1.00, same `FromSql` body for two newtypes), `set_narrator_handler`/`set_quantifier_handler` (1.00), `from_config` (1.00), `with_provider` (1.00, test-only), `build_user_prompt` (0.97, 56 and 65 lines), `get_world`/`get_world_by_id` (0.96), `to_card` (0.89), `list_games`/`list_personas` (0.84).

## The seven ignore rules

dryer has **no ignore mechanism**: `--help` and `src/dryer/cli.py` offer `--threshold`, `--min-lines`, `--min-nodes`, `--format`, `--root` and `--source-root` only. `discover.py` holds the one hard-coded skip list. So each rule below is a feature of a port or a vendored patch — a decision for the map's *Form of each kept tool*.

| Rule | Structural predicate | Pairs removed | What else it hides |
|---|---|---|---|
| R1 | body is assignments to `self.<field>`, ending in `self` | 9 | nothing. `with_provider` builds `Self` through a call, so it stays. A `with_*` name rule would hide both. |
| R2 | body is one struct literal (optionally inside `Ok`) whose fields are column reads, constants, or calls on a literal | 4 | `from_config` (B3) — the same boilerplate the rule targets |
| R3 | body is one `match` whose arms are literals, `Some(..)`/`None`, or a field read | 18 | nothing real in this run |
| R4 | `FromStr` impl, or a `parse_or_default` wrapper (literal match + `format!` error, or a warn fallback) | 6 | nothing |
| R5 | body is a two-arm `match` returning a payload of the variant | 2 | nothing |
| R6 | body is `expr.map(\|x\| x.field).unwrap_or(default)` | 1 | nothing |
| R7 | body is one call to a private helper, differing only in an enum argument | 2 | `set_*_handler` (B2) if the rule also swallows the closure argument. Keep R7 narrow: the call target must be private and the differing argument an enum literal. |

42 of the 52 pairs go, and 1 real pair (B3, `from_config`) goes with them. A2 survives R1–R7 because it is a chained call and not a helper call — a wider "body is one call" rule would take it. Net after the seven rules: **9 pairs, all real** — A1, A2, B1, B2, B4, B5, B6, B7, B8.

## Settings

Same production copy, one knob at a time.

| Settings | Pairs | A | B | C | What binding changed |
|---|---|---|---|---|---|
| defaults: score ≥ 0.82, ≥ 4 lines, ≥ 20 nodes | 52 | 2 | 8 | 42 | — |
| `--min-nodes 40` | 52 | 2 | 8 | 42 | nothing: no reported form has fewer than 40 nodes |
| `--min-nodes 60` | 37 | — | — | — | drops 15 pairs (not classified) |
| `--min-nodes 100` | 14 | 1 | 6 | 7 | best ratio of the bare settings: 7 real, 7 idiom |
| `--threshold 0.90` | 42 | 2 | 6 | 34 | **loses 2 real pairs** (`to_card` 0.89, `list_*` 0.84) and keeps every enum idiom, which sit at 1.00 |
| `--threshold 0.90 --min-nodes 40` | 42 | 2 | 6 | 34 | same as the line above |
| `--min-lines 8` | 21 | 1 | 6 | 14 | drops the 4- and 5-line setters and accessors, keeps the enum matches |

`--threshold` is the wrong knob for this repo: the noise scores 1.00 because the normalizer drops exactly the literals that make each enum arm different, while the real near-clones sit at 0.84–0.97. `--min-nodes 100` is the best single knob without a port, and it still leaves 7 idiom pairs. `--min-lines` and `--threshold` are independent of both.

## What each tool sees

| | dryer | jscpd |
|---|---|---|
| Unit | one function body, normalized: call names, operators, tree shape kept; locals, fields, literals dropped | tokens, unchanged |
| Unit of report | a pair of functions with a Jaccard score | a fragment pair with a line count and byte offsets |
| Production output | 52 pairs, 22 across files | 184 clones, median 5 lines, max 24, 124 across files |
| Sees | whole functions, so a diff at the edges still scores high | declarations, signatures, `struct` field lists, and spans that cross function boundaries |
| Misses | a clone split across two functions, a `struct` field list, a lone function signature | name and literal changes, which is most of A1 |
| Cost | 7.8 s on the production copy, 14 s on the unit-test copy, 45 s on `tests/` (three runs each; the walk is quadratic in function count) | 0.2 s on `src`, 0.16 s on the unit-test copy, 0.7 s on `tests/` |

Examples of what only jscpd sees on this tree: `fn complete(...)` signature against `llm_recorder` (7 lines), and `src/adapters/driven/storage/models/llm_message.rs:4-15` against `src/domain/model/llm_message.rs:8-19` (a `struct` field list). Members are not functions, so dryer never compares them. Example of what only dryer sees: A1, where no 3-line window matches.

The 30-token floor is what hides the setters from jscpd, not the 3-line floor. Same copy at `--min-tokens 10`: 852 clones, 19.8% duplicated lines, and the `mock.rs` setter pairs appear.

## The gate's own jscpd run (added by the coordinator, 2026-10-31)

The comparison above uses jscpd runs on copied scopes. This section checks the dryer pairs against the jscpd run that the build really makes.

**Command:** `python build.py duplicates --all` (log `logs/build_20261031_143959.log`, report `report/jscpd-report.json`). It runs `scripts/healthcheck.py duplicates --all`: jscpd 5.4.2 over the whole repository root, `--format rust`, `--min-lines 3`, `--min-tokens 30`, ignoring `target`, `report`, `tmp`, `docs/external_applications`. Run time 2.10 s. Exit 0.

**What it reports (known):**
- Full report: 2,427 clones, 16,778 duplicated lines of 94,231 (17.81 %), 412 source files. Production code, test code and `tests/` are all in scope.
- Printed summary: the step filters to clones of ≥ 5 lines that cross files: 681 clone pairs in 325 file pairs. It prints the top 25 file pairs. In 24 of the 25, both files are test code (`tests/`, `*_tests.rs`). Only one is production code: `options/prompt.rs` ↔ `quantifier/prompt.rs` (B5).
- The gate runs the same step, but the summary shows only pairs where one side is a file changed against `main`.

**The 10 real dryer pairs against the gate's report** (a pair counts as found when a jscpd clone overlaps both function spans; script `tmp/dup_gate_overlap.py`):

| Pair | Functions | jscpd clones | Largest clone | Passes the summary filter (≥ 5 lines, cross-file) | In the printed top 25 |
|---|---|---|---|---|---|
| A1 | `Debug` `fmt`, options / quantifier agent | 0 | — | no | no |
| A2 | `render_game_state_layer` / `render_world_info_layer` (same file) | 0 | — | no | no |
| B1 | `column_result` ×2 (same file) | 1 | 8 lines | no: same file | no |
| B2 | `set_narrator_handler` / `set_quantifier_handler` (same file) | 1 | 6 lines | no: same file | no |
| B3 | `from_config` | 2 | 16 lines | yes | no |
| B4 | `with_provider` | 1 | 16 lines (the same clone as B3) | yes | no |
| B5 | `build_user_prompt` | 3 | 24 lines | yes | **yes** |
| B6 | `get_world` / `get_world_by_id` (same file) | 2 | 10 lines | no: same file | no |
| B7 | `to_card` | 2 | 12 lines | yes | no |
| B8 | `list_games` / `list_personas` | **0** | — | no | no |

**Corrections to the sections above (known):**
- B8 is not in the gate's report. The class "jscpd also finds" is wrong for B8 with the gate's settings. So the gate's jscpd misses **3** real pairs (A1, A2, B8), not 2.
- 4 of the real pairs (B1, B2, B6 and A2) are in one file. The printed summary drops every same-file clone, so even a jscpd hit on them never reaches a reader of the build output. Only the JSON report has them.
- Of the 10 real pairs, jscpd's printed output shows **1** (B5). It shows that one only because it ranks in the top 25.

**What this means for the value question (inferred):** the claim "dryer adds nothing that the build's jscpd already shows" does not hold. Against what a reader of the build output sees, dryer adds 9 of 10 real pairs. Against the full JSON report, dryer adds 3 (A1, A2, B8). The difference comes from two causes: the summary filter of `scripts/healthcheck.py` (cross-file only, top 25, which test clones fill), and the token matching of jscpd (it misses A1, A2 and B8 because names and literals differ). The first cause is a jscpd setting, so a change to the filter could fix it without dryer. The second cause is the reason to add dryer.

### What dryer still finds after every jscpd clone is fixed (coordinator, 2026-10-31)

Fresh dryer runs (defaults) on copies of HEAD `38043843` (`tmp/dup_now/`), against a fresh `python build.py duplicates --all` report of the same tree. A dryer pair "survives" when no jscpd clone overlaps both of its functions. Script: `tmp/dup_now/residual.py`.

| Scope | dryer pairs | No linking jscpd clone (survives) | Clone covers < 50 % of a function | Clone covers ≥ 50 % |
|---|---|---|---|---|
| production | 54 | **43** | 3 | 8 |
| unit tests (`*_tests.rs` + `src/test_support/`) | 409 | 271 | 66 | 72 |
| `tests/` | 1,016 | 912 | 62 | 42 |

- Production has 54 pairs, not 52: two new idiom pairs in `src/adapters/driven/storage/test_support.rs` (`set`/`set_write`, `config`/`internal`).
- The 43 surviving production pairs: **3 real** (A1 `Debug` impls, A2 `assembler.rs` renderers, B8 `list_games`/`list_personas`) and 40 idiom (R1–R7 classes and the two new setter pairs).
- 3 real pairs have a partial jscpd clone only: B5 `build_user_prompt` 37 %, B6 `get_world` 21 %, B7 `to_card` 48 %. Whether dryer still reports them after the jscpd clone is fixed depends on the fix (guessed: a fix of the cloned part removes most of the shared body, so likely not).
- The test-scope survivors are not classified. Most dryer pairs on that scope are short helper wrappers (see *Test code* below), so the number of real duplicates among them is unknown.
- Limit: this is a static overlap check. A real refactor can also add or remove dryer pairs elsewhere.

## Test code: worth a separate run?

### Counts

| Scope | Lines | dryer pairs | at score 1.00 | cross-file | jscpd clones | jscpd duplicated lines |
|---|---|---|---|---|---|---|
| `*_tests.rs` + `src/test_support/` | 21,242 | 409 | 202 | 36 | 667 | 23.22% |
| `tests/` | 52,354 | 1,016 | 575 | **490** | 1,492 | 19.69% |

Test code is 73,596 lines against 20,799 production lines. Density is 19 pairs per 1,000 lines in `*_tests.rs`, 19 in `tests/`, against 2.5 in production. `tests/` is the one scope where most pairs cross a file boundary (48%), so they are not one file repeating itself.

### What the pairs are

- **`*_tests.rs`**: 373 of 409 are same-file. The top of the list is fixture shape: three near-identical test bodies in `chat_completions_tests.rs:160-210`, four in `response_tests.rs:45-71`, where the same arrange-act-assert shape carries a different JSON literal. Real duplication of setup, not of assertion.
- **`tests/`**: `browser/` contributes 856 of the 2,032 pair members, and 413 pairs are browser-to-browser. The idiom is a thin `page.evaluate(...).await.unwrap()` wrapper — `action_status_text`, `notice_source`, `send_takes_clicks`. The JS string and the type parameter differ, and the normalizer drops the string. `tests/browser/` holds 286 `page.evaluate` call sites. That is the same failure mode as the builder setters: the report is real, but the fix is a helper and there is one per call site.

So the tool is *more* productive here and *less* precise: the browser wrapper idiom needs its own rule (body is one call chain on `page`), which is a rule the production scope does not need.

### What the retros say (known, cited)

- `.scratch/_closed/test-strategy-execution/issues/05-retry-http-e2e.md:179-183` — a Standards review found **Duplicated Code**: `tests/http/actions.rs:176-228` and `tests/http/swipe_new.rs:183-215` hand-roll `NpcCard`/`CharacterSheet` when `src/test_support/fixtures.rs:84-160` already exports `TestNpc::with_times_met_trigger` and `TestNpc::with_room_scoped_trigger`. The fix is reuse, not extraction.
- `.scratch/_closed/test-strategy-execution/issues/05-retry-http-e2e.md:172` — a second duplication finding in the same review was **suppressed by plan**: "`#3 pipeline-boilerplate DRY vs. plan's 'one helper only' decision`". So the repo has already decided, once, to tolerate test-side duplication.
- `.scratch/_closed/test-strategy-execution/issues/13-lifecycle-http-e2e.md:89-91` and `.scratch/_closed/test-strategy-execution/assets/lifecycle-arrival-disposition.md:271-274` — the form-POST boilerplate was pinned as a choice: add `post_form(app, uri, body)`, "or inline the form-POST boilerplate (matches existing `games_fragment_handlers.rs` style)". It was inlined.
- `.scratch/retro-issue-bucket/assets/reflect-2026-10-07/reviewers/findings-tooling.md:11-14` — the jscpd report existed, "and no review in this window used it"; five review sessions hand-found duplicated code. That is the same failure the 92.5% figure above explains: the report the reviewer opens is nearly all test clones.

So the pain point is **known** and documented, and the existing response is deliberate: reuse a fixture when one exists, otherwise inline. dryer would have surfaced the `NpcCard` hand-roll without a reviewer noticing it. It would also have buried it under 1,000 browser wrappers.

### Verdict (inferred)

A separate test run is worth having only with a different rule set from production — and after the production run proves the tool's value. The order matters: production first, because it is 52 pairs, human-reviewable, and 10 of them are real.

## What the user must decide

These are the questions the grilling ticket needs answered. The evidence above settles none of them.

1. **Value.** Two production pairs that jscpd misses (A1, A2), against 42 idiom pairs at defaults. Is A1 alone — one 10-line renderer pair, a real refactor — enough to carry a tool step?
2. **Which rules.** Adopt R1–R7? Without them the report is 81% noise. R2 also hides `from_config` (B3), which is real. Is that acceptable?
3. **Settings and cost.** Defaults plus rules, or `--min-nodes 100` (14 pairs, 7 real) plus rules? Note that `--threshold` and `--min-nodes 40` do nothing here. Cost is a separate input: 7.8 s on production, 14 s on the unit tests, 45 s on `tests/`, against 0.2–0.7 s for jscpd. A full-gate step that runs twice per agent session pays that every time; an on-demand subcommand pays it only when asked.
4. **Test scope.** In or out? In, it needs a ninth rule for the `page.evaluate` wrapper and it makes the report 10× longer. Out, and the documented test-duplication pain stays with the reviewer.
5. **jscpd.** Keep `duplicates` as is, narrow it to production (the 92.5% finding), or run dryer beside it? Note the two tools disagree about granularity on purpose: 184 fragments against 52 functions, with 14 of the 52 shared.

## Evidence labels

- **Known** (ran or read): every count, every score, every line range; dryer's option list and skip lists; the `#[cfg(feature = "testing")]` gate; the four retro/issue citations; the 92.5% split of the gate's report; the `--min-tokens 10` result.
- **Inferred**: the class of each of the 52 pairs, and the rule that would remove each idiom. Each rests on reading the pair's body, not on running a rule — no ignore mechanism exists to run.
- **Guessed**: whether A1 and A2 are worth a tool step, and the ordering advice at the end of the test-code section. The value bar is the user's to set (map, *Note: the user sets each value bar after seeing real output*).

## Reproduction

```bash
# from /workspace/chronicler-engine
mkdir -p tmp/unclebob/prod && cp -a src tmp/unclebob/prod/src
find tmp/unclebob/prod -name '*_tests.rs' -delete && rm -rf tmp/unclebob/prod/src/test_support
(cd tmp/unclebob/prod && ../.venv/bin/dryer)                        # 52 pairs
(cd tmp/unclebob/prod && ../.venv/bin/dryer --min-nodes 100)        # 14 pairs
(cd tmp/unclebob/prod && ../.venv/bin/dryer --min-lines 8)          # 21 pairs
(cd tmp/unclebob/prod && ../.venv/bin/dryer --threshold 0.9)        # 42 pairs

# test scopes: the copy must not live in a directory named tests/
mkdir -p tmp/unclebob/testscope/src && cp -a src/test_support tmp/unclebob/testscope/src/
(cd src && find . -name '*_tests.rs' | while read -r f; do
   mkdir -p "../tmp/unclebob/testscope/src/$(dirname "$f")"
   cp "$f" "../tmp/unclebob/testscope/src/$f"; done)
(cd tmp/unclebob/testscope && ../.venv/bin/dryer)                   # 409 pairs
cp -a tests tmp/unclebob/itests
(cd tmp/unclebob/itests && ../.venv/bin/dryer)                      # 1,016 pairs

# jscpd, the healthcheck.py settings; output goes under tmp/, never to report/
npx --yes jscpd@5.4.2 src --min-lines 3 --min-tokens 30 --format rust \
  --ignore "**/*_tests.rs,**/test_support/**" \
  --output tmp/unclebob/out/jscpd-prod2 --reporters json
npx --yes jscpd@5.4.2 tmp/unclebob/itests --min-lines 3 --min-tokens 30 --format rust \
  --ignore "**/node_modules/**" --output tmp/unclebob/out/jscpd-itests --reporters json
```

Two more traps for a re-run. `--ignore` must not name `tmp` when the copy sits under `tmp/`, and jscpd reports paths relative to the directory it was given, so the join script needs that prefix. And `tmp/unclebob/` is shared scratch: a copy can lose files between runs (one did, mid-session). Re-copy before trusting a count.

The join and classification scripts are `tmp/unclebob/pairs.py`, `overlap2.py` and `classify.py`.
