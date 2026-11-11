# B3 — Implicit protocols: pairs of parts that must agree, where nothing forces the agreement

Lens: "find every contract my code assumes but never verifies". One row = two parts that
must agree on a shape, a name, an order or a timing, where no type, schema or gate test
holds both sides.

Labels: **known** = I read the code of both parts; **inferred** = the code of one side is
read and the other side is an external rule I cannot run; **guessed** = neither side run.

Read first: `work/wayfinder-maps/invariant-audit/assets/REPORT.md` (53 findings) and the
already-known list in the brief. Rows below are outside both.

---

## R1 [known] The Quantifier's `movement_type` is parsed and logged, and no rule reads it

**Part A** — `src/application/agents/quantifier/utils/parser.rs:168-173`

```rust
movement_type: m
    .movement_type
    .as_ref()
    .and_then(|t| match t.to_lowercase().as_str() {
        "entering" => Some(MovementType::Entering),
        "leaving" => Some(MovementType::Leaving),
        _ => None,
    }),
```

**Part B** — `src/domain/model/utils/game_rules.rs:57-62`

```rust
    if !is_event {
        movement(
            &mut state,
            result.destination.as_deref(),
            &result.present,
            world.map,
        );
    }
```

**The agreement.** `MovementType` carries the meaning "the player is leaving the current
room" (`types.rs:35-40`). The rule that moves the player takes **only** `destination`.
`movement_type` has exactly two readers in `src/`: the parser that builds it and a
`tracing::info!` in `orchestration.rs:119`. So "leaving + destination = the room I came
from" moves the Persona *into* that room.

**Test that pins both sides.** No test found.
`rg -n "movement_type" src/ tests/` → 6 hits, all parser/orchestration/log.
`src/application/agents/quantifier/utils/parser_tests.rs:175-184` even asserts the shape
that produces the wrong result: `{"type": "teleporting", "destination": "castle"}` gives
`movement_type: None, destination: Some("castle")`, which `game_rules` applies as a move.

**One-sided change.** Silent wrong result. A model answer of `"leaving"` with a
destination sends the player to the room they are leaving, or to any Room of the map the
model names. A change that makes `movement_type` authoritative would also change the
behaviour of every stored Swipe on a Retry.

---

## R2 [known] The Quantifier preset's `output_format` is the only contract with the JSON parser, and it is author-editable

**Part A** — `src/adapters/driving/http/prompt_presets/templates/prompt_presets.rs:119`

```html
<textarea id="{{ preset_type }}-preset-output" name="output_format" rows="6" placeholder="Enter output format..."></textarea>
```

**Part B** — `src/application/agents/quantifier/utils/parser.rs:11-18`

```rust
#[derive(Deserialize, Debug)]
struct QuantifierJsonResponse {
    /// An edited prompt preset can ask for `npcs_in_room`.
    #[serde(default, alias = "npcs_in_room")]
    characters_present: Vec<String>,
```

**The agreement.** The seeded preset asks for exactly one key spelling and one movement
shape (`data/prompt_presets/quantifier/default.json:6`:
`Respond ONLY with a JSON object in this exact format:\n{"characters_present": ["id1", "id2"], ...`).
The parser accepts `characters_present` plus one alias. The alias comment proves the pair
has already broken once.

**Test that pins both sides.** No test found. `rg -n "output_format" src/ tests/` gives
form, storage and render hits only; no test loads a preset's `output_format` and runs it
through `try_parse_json_full`. `src/application/agents/quantifier/prompt_tests.rs:15-37`
hard-copies an override string instead of reading `data/prompt_presets/quantifier/default.json`.

**One-sided change.** Silent wrong result, in the same shape as the known llm-01 but by a
different cause: an author who edits the format to `"npcs"`, to a nested object, or to
prose makes all three parse attempts fail (`parser.rs:62-133`), then `parse_with_movement`
falls to the substring fallback (`parser.rs:190-201`) with `MovementType::None` and no
movement. `orchestration.rs:184-188` treats Low confidence as "keep the static Characters",
so the scene freezes with no error and no dashboard signal. The known llm-01 is about
names instead of ids; this one is about a shape the dashboard invites the author to change.

---

## R3 [known] The Options preset's shape is a contract with three regexes, and a fourth shape fails as "the model failed"

**Part A** — `src/adapters/driving/http/prompt_presets/templates/prompt_presets.rs:119`
(the same editable field, for `preset_type == "options"`).

**Part B** — `src/application/agents/options/utils/parser.rs:19-26`

```rust
static RE_SUGGESTION_TAG: Lazy<Regex> =
    Lazy::new(|| compile(r"(?is)<suggestion>(.*?)</suggestion>"));
static RE_NUMBERED_LINE: Lazy<Regex> = Lazy::new(|| compile(r"(?m)^\s*\d+[.)]\s+(.+?)\s*$"));
static RE_SUGGESTION_PREFIX: Lazy<Regex> =
    Lazy::new(|| compile(r"(?im)suggestion\s+\d+\s*:\s*(.+?)\s*$"));
```

**The agreement.** Two seeded presets happen to match two of the three shapes
(`options/default.json` asks for `<suggestion>` tags; `options/roadway_domains.json` asks
for "ONLY a numbered list"). An author who writes `- action` bullets, a JSON array, or
`**1.** action` (markdown bold) matches none of the three.

**Test that pins both sides.** No test found. `rg -n "parse_options" src/ tests/` → the
parser, its caller, and unit tests of `parse_options` with literal strings; no test reads a
`data/prompt_presets/options/*.json` and feeds its stated shape to the parser.

**One-sided change.** Not silent at the end, but wrongly attributed.
`src/application/agents/options/utils/orchestration.rs:53-68` retries
`MAX_OPTIONS_ATTEMPTS` times, then returns
`EngineError::Parse("options response contained no parseable options")`. The two paid model
calls and an Options failure line that blames the provider for an author's edit.

---

## R4 [known] The `{{...}}` placeholders fill differently in preset text and in World text

**Part A** — `src/application/prompting/assembler.rs:91-99` (preset text)

```rust
    /// Sole writer of the narrative-voice template vars.
    fn apply_posture(
        template_vars: &mut TemplateVars,
        perspective: NarrativePerspective,
        tense: NarrativeTense,
    ) {
        template_vars.narrative_perspective = perspective.as_str().to_string();
```

**Part B** — `src/domain/model/utils/opening.rs:18` and
`src/domain/model/utils/game_rules.rs:99-100` (World text)

```rust
        .map(|scenario| TemplateVars::new(persona_name).render(&scenario.text))
```

```rust
            direction: TemplateVars::new(world.persona_name)
                .render(&trigger.narration.narration_prompt),
```

**The agreement.** `TemplateVars::render` (`template.rs:58-69`) substitutes seven names.
`from_persona` fills three of them from the Persona; `new` leaves them empty and hardcodes
`narrative_perspective: "third"`, `narrative_tense: "past"` (`template.rs:35-36`). The
Scenario text and the Trigger narration are authored text that the dashboard also lets an
author write, and both use `new`.

**Test that pins both sides.** No test found. `rg -n "persona_personality|narrative_perspective}"
tests/` → no hit. `src/domain/model/template_tests.rs:44-52` asserts the *empty* result of
`new` on purpose, so the difference is a decision on one side and an unstated contract on
the other; the doc comment "Sole writer of the narrative-voice template vars" at
`assembler.rs:92` is false, because `TemplateVars::new` writes them too.

**One-sided change.** Silent wrong result, and the empty case is silent even today: a
Scenario text with `{{persona_personality}}` renders an empty line into the opening
Narration, and one with `{{narrative_perspective}}` says "third" in a second-person Game.
The same placeholder in a preset renders correctly. `{{char}}` stays literal
(`template_tests.rs:15-20`), which is at least visible.

---

## R5 [known] A broken `world.json` loses the World with a log line; a broken `map.json` stops the boot

**Part A** — `src/bootstrap/load.rs:102-112`

```rust
    let manifest: WorldManifest = match read_json_file(&world_json) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(
                "Failed to parse world manifest {}: {e}",
                world_json.display()
            );
            return Ok(());
        }
    };
```

**Part B** — `src/bootstrap/load.rs:121-122`

```rust
            let map_path = world_dir.join(&map_file);
            let map: MapDef = read_json_file(&map_path)?;
```

**The agreement.** Both files are author-editable JSON for one World, both are validated by
`scripts/validate_data.py`, and both are read by the same seeding function. The manifest
error is swallowed; the map error propagates and fails the boot. `load.rs:154-157` swallows
a Character error the same way as the manifest.

**Test that pins both sides.** No test found. `rg -n "Failed to parse world manifest" src/ tests/`
→ one hit, the code itself. The two error policies have no test that states which one is
intended.

**One-sided change.** A visible error on one path and a silent wrong result on the other. A
World whose manifest has one wrong field simply does not exist after boot (no Game can be
created from it, and the dashboard lists it as missing) with nothing but a WARN. A World
whose map has one wrong exit fails the whole server, so the same class of authoring error
takes down every other World.

---

## R6 [known] `validate_data.py` checks the Character file name; the engine seeds the card's `id` field

**Part A** — `scripts/validate_data.py:43-52`

```python
    for scenario in scenarios:
        for character_id in scenario_character_ids(scenario):
            character_file = chars_dir / f"{character_id}.json"
            if not character_file.exists():
                missing.append((scenario.get("id"), character_id, character_file))
```

**Part B** — `src/bootstrap/load.rs:147-153`

```rust
        match read_json_file::<CharacterCard>(&path) {
            Ok(character) => {
                if !existing_chars.contains(&character.id) {
                    storage.seed_character(world_id, &character)?;
```

**The agreement.** The gate proves that a file named `<id>.json` exists. The engine keys the
Character by the `id` **inside** the file and never compares it to the file name (the table
comment says `key TEXT NOT NULL,  -- from CharacterCard.id`). Nothing checks the two agree.

**Test that pins both sides.** No test found. `rg -n "characters_dir|seed_character" tests/`
→ tests that seed a card through the Rust API, never through `data/characters/**` file names.

**One-sided change.** Silent wrong result that passes the gate: `carla.json` whose `id` is
`carlaa`, with a Scenario naming `carla`, validates clean and produces a World whose opening
scene is missing the Character, while the Character exists under a key nothing refers to.
The known world-03 covers "Scenario names a Character that does not exist"; this row is why
the gate cannot see it.

---

## R7 [known] The character schema does not require `id`, and the Rust model does

**Part A** — `data/schemas/character.schema.json:57`

```json
  "required": ["name", "description", "personality", "scenario"]
```

**Part B** — `src/domain/model/character.rs:60-62`

```rust
pub struct CharacterCard {
    pub id: String,
```

**The agreement.** `CharacterSheet` and the schema agree on the four required fields.
`CharacterCard.id` has no `#[serde(default)]`, so a card without `id` fails deserialization,
while the schema calls that card valid.

**Test that pins both sides.** No test found. `rg -n '"required"' data/schemas/character.schema.json`
vs `rg -n "pub id: String" src/domain/model/character.rs` — no test loads a schema at all;
`validate_data.py` runs in `build.py` (`validate-data` step) and no Rust test invokes it.

**One-sided change.** The gate passes and the engine skips the Character with one WARN
(`load.rs:154-157`), so the World silently loses a Character. The file is also valid for
`data/personas/*.json`, which shares the schema.

---

## R8 [known] The map schema accepts any exit key; the Rust `Direction` enum accepts six

**Part A** — `data/schemas/map.schema.json:27-29`

```json
                    "exits": {
                      "type": "object",
                      "additionalProperties": { "type": "string" }
                    },
```

**Part B** — `src/domain/model/map.rs:42` and `:73-81`

```rust
    pub exits: HashMap<Direction, String>,
```

```rust
#[serde(rename_all = "lowercase")]
pub enum Direction {
    North,
```

**The agreement.** `HashMap<Direction, String>` makes every key a `Direction`, so the only
legal keys are `north south east west up down`. The schema says any key with a string value
is fine.

**Test that pins both sides.** No test found. `rg -n "exits" data/schemas/map.schema.json scripts/validate_data.py`
→ the schema only; `scripts/validate_data.py` has no line that names a direction.

**One-sided change.** An author who writes `"north-east"` or `"in"` gets a PASS from the
gate and then an `EngineError::DataLoad` from `read_json_file(&map_path)?` at boot
(`load.rs:122`), which takes down the whole server — not only that World. A silent variant
exists too: an unknown key that is dropped nowhere, because the whole parse fails.

---

## R9 [known] `scripts/trial.py` reads the stored JSON keys of `SwipeState` and of `message_type`, and its own test fixtures copy those keys

**Part A** — `scripts/trial.py:965-970`

```python
def state_line(entry):
    try:
        state = json.loads(entry["state"])
    except (TypeError, json.JSONDecodeError):
        return "?", []
    room = (state.get("movement") or {}).get("current_room_id", "?")
    present = [npc.get("id") for npc in (state.get("scene") or {}).get("npcs_in_area", [])]
```

**Part B** — `src/domain/model/state/swipe_state.rs:20-21` and
`src/domain/model/state/scene_state.rs:9-10`

```rust
    /// The stored key is `npc_encounter_log`.
    #[serde(rename = "npc_encounter_log")]
    pub encounters: Encounters,
```

```rust
    /// The stored key is `npcs_in_area`.
    #[serde(rename = "npcs_in_area")]
    pub characters_present: Vec<CharacterCard>,
```

**The agreement.** The Python reader and the Rust serde names must spell the same keys.
`scripts/tests/test_trial.py:165-175` writes the fixtures with `json.dumps(message_type)`
and the same column list, so the Python test proves only that trial.py agrees with itself.

**Test that pins both sides.** No test found that runs the Python reader against a database
the Rust code wrote. `rg -n "npcs_in_area" scripts/ tests/ src/` → the Rust model, the Rust
storage test (`tests/storage/story_storage.rs:967-978`, which pins the *Rust* side) and
`scripts/trial.py`. `rg -n "trial.py" tests/` → nothing.

**One-sided change.** `state_line` returns `[]` for the Characters present, and the reading
pack prints an empty scene for every entry: a silent wrong result in the artifact a trial
review reads. `message_type_of` (`trial.py:276-283`) has the same shape for the
`message_type` column and feeds `cycles_played`, so a serde change silently reports 0 or
the wrong number of Cycles.

---

## R10 [known] The scripts name screen parts, and the guardrail that ties names to `SCREEN_PARTS` is vacuous once a name changes

**Part A** — `assets/dashboard-action-area.js:202-205`

```javascript
  if (parts.includes("status-display")) applyStatusDisplay();
  else syncActionState();
  if (parts.includes("story-log")) announceNewNarration();
  if (parts.includes("options-dock")) announceOptions();
```

**Part B** — `src/adapters/driving/http/screen/handlers/parts.rs:31-40`

```rust
pub const SCREEN_PARTS: [(&str, PartRenderer); 13] = [
    ("header", render_header_part as PartRenderer),
    ("failure-banner", render_failure_banner_part as PartRenderer),
    ("story-log", render_story_log_part as PartRenderer),
```

**The agreement.** A part name in a script must be a name in `SCREEN_PARTS`, and
`tests/infrastructure/guardrails/screen.rs` ties `SCREEN_PARTS` to `assets/index.html`
(`check_shell_parts_match_the_server`) and to the stub (`check_stub_answers_the_parts_of_the_server`).

**Test that pins both sides.** The guardrail only fires while the name exists. Its matcher
`names_a_part` (`tests/infrastructure/guardrails/screen.rs:287-303`) tests whether a literal contains a name from `parts`;
rename the part in `SCREEN_PARTS` and in `index.html` together and a stale
`parts.includes("story-log")` names no part any more, so `check_scripts_write_no_part`
finds no violation. `rg -n "dashboard-action-area.js|assets/" tests/infrastructure/guardrails/screen.rs`
→ no test loads the scripts and lists their part names.

**One-sided change.** Silent wrong result: the live region stops announcing a new Narration
or new Options, and the action area stops following the status display, with no test in the
gate.

---

## R11 [known] The Game header name is a literal in the script and a `const` in the Rust layer

**Part A** — `assets/dashboard-screen.js:6` and `:427`

```javascript
const GAME_HEADER = "X-Chronicler-Game";
```

```javascript
  if (game) evt.detail.headers[GAME_HEADER] = game;
```

**Part B** — `src/adapters/driving/http/utils/request_layers.rs:14`

```rust
const REQUEST_GAME_HEADER: &str = "x-chronicler-game";
```

**The agreement.** Every non-GET request carries this header only because the script adds
it; the refusal in `refuse_stale_game` (`request_layers.rs:42-50`) reads the name and does
nothing when the header is absent (`A request with no header acts on the active Game.`).

**Test that pins both sides.** No test found. `rg -n "X-Chronicler-Game|x-chronicler-game" tests/`
→ `tests/http/screen.rs:23` (a Rust copy), and the stub's own records
(`tests/test_utils/stub_request_layers.rs:85`). No test reads the script's literal.

**One-sided change.** Silent wrong result of high cost: a rename on either side makes the
stale-Game refusal never fire, so an Action, a Retry or a Reset from an old tab acts on the
newly active Game and answers `2xx`. The known gate-02/http-01 is about two reads of the
active pointer inside one request; this row is about the header that makes the check exist.

---

## R12 [known] The `/changes` pair and the `X-Revision` stamp are one format written in three places

**Part A** — `src/domain/model/change.rs:150-153` and `:179`, `:189`

```rust
    pub fn stamp(&self) -> String {
        format!("{}:{}", self.server, self.revision)
    }
```

**Part B** — `assets/dashboard-screen.js:174-176` and `:333`

```javascript
function splitStamp(stamp) {
  const at = stamp.lastIndexOf(":");
  return { server: stamp.slice(0, at), revision: Number(stamp.slice(at + 1)) };
```

```javascript
  if (answer.server + ":" + answer.revision === screenRoot.dataset.revision) {
```

**The agreement.** The header value, the JSON pair and the page's own comparison must spell
the same string. The third copy is in the test itself
(`tests/http/screen_load.rs:21-27`, `format!("{}:{}", answer["server"], answer["revision"])`).

**Test that pins both sides.** No Rust test reads the script, and no browser test runs
against the Rust server for this: `tests/browser/**` asserts the format against the stub
(`tests/browser/stub/drafts.rs:83`, `answer.server + ':' + answer.revision`). The Rust test
at `screen_load.rs:184` compares the header with a string it builds itself, so it pins the
Rust pair and not the script.

**One-sided change.** Loud but confusing on one side, silent on the other: a new separator
in `stamp()` fails the Rust test, and a separator changed in both Rust places passes the
gate while `answer.server + ":" + answer.revision` never equals the page stamp, so `/changes`
loads the whole screen on every poll forever.

---

## R13 [known] `data-status` values are read by the script and rendered by the server

**Part A** — `assets/dashboard-action-area.js:39-53`

```javascript
function statusShows(state) {
  const statusDisplay = document.getElementById("status-display");
  return !!(
    statusDisplay && statusDisplay.querySelector('[data-status="' + state + '"]')
  );
}

function statusIsGenerating() {
  return statusShows("thinking");
}
```

**Part B** — `src/adapters/driving/http/layout/handlers/endpoints.rs:86-96`

```rust
        Busy::Job(step) => status_element("thinking", "thinking", step_label(step)),
        Busy::Autoplay { progress, step } => {
            let mut label = format!("Autoplay: cycle {}/{}", progress.cycle(), progress.budget);
```

**The agreement.** `status_element` writes `data-status="{state}"` (`endpoints.rs:72`, `:88`, `:95`) from a
free `&str` with no type or constant, and the script matches two of the four values it can
write (`thinking`, `autoplay`; the others are `idle` and `failed`).

**Test that pins both sides.** The server side is pinned and the script side is not.
`tests/http/dashboard.rs:47` asserts `data-status="thinking"` as a literal, so a server
rename fails a Rust test only if the test is kept; nothing in `tests/` reads the script's
`"thinking"`. `rg -n 'data-status' tests/` → Rust tests and stub browser tests only.

**One-sided change.** If the server renders another word, `actionIsGenerating()` returns
false while a Job runs: the action area shows the "generating" controls as idle and the send
control stays available. The busy refusal (`409`) still stops the second request, so the
result is a misleading UI, not a lost write.

---

## R14 [inferred] The transport assumes `choices[0].message.content` is a JSON string

**Part A** — `src/adapters/driven/llm/transport/utils/response.rs:62-76`

```rust
            // `reasoning` and `reasoning_content` are separate channels: only
            // `choices[0].message.content` can become the answer.
            if let Some(content) = message
                .get("content")
                .and_then(|c| c.as_str())
```

**Part B** — the hosts behind one provider: `src/adapters/driven/llm/providers/chat_completions.rs:2`
(`//! The chat-completions provider: one backend for every OpenAI-compatible host.`) serves
OpenRouter, Ollama and OpenCode Go through the same parse (`llm_backend.rs:11-14`).

**The agreement.** The OpenAI-compatible schema allows `content` to be a string **or** an
array of parts (`[{"type":"text","text":"..."}]`). The parser accepts only the string. An
answer in the array form is not a parse error with a clear cause: `as_str()` returns `None`,
so the caller falls through to `finish_reason` and returns `LlmFailure::EmptyResponse`
(`response.rs:92-99`) or `TokenBudgetSpent` when the reason is `length`.

**Test that pins both sides.** No test found. `rg -n "content" src/adapters/driven/llm/transport/utils/response_tests.rs`
→ string-shaped fixtures only, and `src/test_support/chat_completions_mock.rs` answers the
string form because the test writes it.

**One-sided change.** A provider that switches to the array form turns every generation into
"The model answered nothing", which reads as a model fault and not as a transport fault.
Label: inferred — I did not call a host that answers the array form.

---

## R15 [inferred] The value shape of `x-opencode-session` is an external rule that nothing checks

**Part A** — `src/domain/model/llm_backend.rs:95-101`

```rust
    /// OpenCode Go refuses a call without its session header.
    pub fn session_header(self) -> Option<&'static str> {
        match self {
            Self::OpenCodeGo => Some("x-opencode-session"),
```

**Part B** — `src/adapters/driven/llm/providers/chat_completions.rs:108-114`

```rust
    fn session_header_value(&self, agent_name: &str, game: Option<GameId>) -> String {
        let owner = game.map_or_else(|| self.connection_id.clone(), Self::game_session);
        format!("{owner}:{agent_name}")
    }
```

**The agreement.** The header name and a value the provider accepts. The value is
`<uuid>:<role>` for a Game and `<connection id>:<role>` with no Game, where the role is a
`Role` name or, for the Quantifier, the literal `"quantifier"` (known llm-09). The comment
states that the provider refuses a call without the header; nothing states what the value
must look like beyond that.

**Test that pins both sides.** The name is pinned, the value is not.
`rg -n "x-opencode-session" src/ tests/` → `llm_backend.rs` and the provider tests that
assert the header is present; no test can assert that the host accepts the value.

**One-sided change.** Every call to that host fails with an HTTP error that
`src/adapters/driving/http/error.rs` shows as "could not be reached" (known), so a session
format change is invisible in the dashboard. Label: inferred — the provider's rule is not in
this repo, and I did not call it.

---

## R16 [known] The Gemma 4 marker text is written by the provider and recognised by the sanitizer, with no shared constant

**Part A** — `src/adapters/driven/llm/providers/chat_completions.rs:124-132`

```rust
        if self.provider == LlmBackendType::Ollama && is_gemma4 {
            format!("{text}\n<|turn>model\n<|channel>thought\n<channel|>")
```

**Part B** — `src/application/prompting/sanitize.rs:13-16`

```rust
    static RE_CHANNEL_THOUGHT: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?s)<\|channel>thought.*?<channel\|>").expect("valid regex"));
    static RE_TURN_MARKERS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"<\|turn>model|<turn\|>|<\|turn>").expect("valid regex"));
```

**The agreement.** The literal markers appended to the last prompt part must be the markers
the sanitizer removes from the answer, including the newline layout the regex spans.

**Test that pins both sides.** No test found. `rg -n "channel>thought" src/ tests/` →
the provider test `chat_completions_ollama_closes_the_thinking_channel_of_a_gemma4_model`
(`chat_completions_tests.rs:186-200`) asserts the appended text, and
`narrator_tests.rs:1724-1739` asserts the stripping of a similar string typed by the test;
no test feeds the one string into the other.

**One-sided change.** A leaked ` <channel|>` or `<|turn>model` in the story text: the raw
markers are stored as Narration and shown in the log. Silent, since the story is valid text.

---

## Candidates I dropped, and why

- **`ANSWER_WORDS` vs the router paths** — a guardrail already forces it:
  `tests/infrastructure/guardrails/answer_words.rs` checks every `post` route has a row and
  every row is a route.
- **Screen part names in `assets/index.html` vs `SCREEN_PARTS`** — checked
  (`check_shell_parts_match_the_server`, `check_stub_answers_the_parts_of_the_server`).
- **`.log-entry`, `.edit-btn`, `#story-log`, `#command-input`, `.option-btn` in the scripts
  vs the askama templates** — covered by the browser tier against the real server
  (`tests/test_utils/browser.rs:413`, `tests/test_utils/wait.rs:77`,
  `tests/browser/options.rs:21`). Only `.swipe-counter`
  (`dashboard-story-log.js:151`) is stub-only, which is a weaker form of R10 and not a
  separate row.
- **`SwipeState` key renames (`npcs_in_area`, `npc_encounter_log`, `trigger_context`)** —
  pinned by `tests/storage/story_storage.rs:967-978` on the Rust side; R9 is the Python
  side, which that test cannot reach.
- **The settings JSON columns (`connections`, `text_check`, `agents`,
  `mode_preset_registry`)** — a renamed field fails loudly
  (`src/adapters/driven/storage/models/settings.rs:44-51` propagates the parse error), so
  the damage is a boot-time failure, not a wrong result.
- **The Quantifier value `"in"`** — the preset offers `"entering|in|leaving"` and the parser
  maps `"in"` to `None`, but the movement still happens through `destination`, so the row
  collapses into R1.
- **A Connection id in the session header** — the id is server-generated
  (`generate_storage_id("conn")`), so the header value cannot be malformed by an author.
- **The dashboard form field names vs the Rust form structs** — already known (http-04).
- **Name uniqueness, the active pointer, the lock order** — already known.

Total: 16 rows (14 known, 2 inferred). Dropped: 12 candidates, listed above. The five I
judge most serious: R11 (a stale tab writes to the new Game), R1 (a "leaving" answer
teleports the Persona), R6 (a gate-passing World silently loses a Character), R9 (trial
packs report an empty scene), R2 (an author's preset edit freezes the scene with no error).
