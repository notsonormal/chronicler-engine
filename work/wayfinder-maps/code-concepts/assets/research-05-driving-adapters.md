# Research 05: the concepts in the driving adapters

Ticket: [05-research-driving-adapters.md](../issues/05-research-driving-adapters.md)

## What was read

**Path rule.** A path that starts with `http/` means `src/adapters/driving/http/`. Every other path is relative to the repo root.

**Read in full (every line):** all 60 non-test `.rs` files of `src/adapters/driving/` (4,816 lines). They are `mod.rs`, `http/mod.rs`, `error.rs`, `app_state.rs`, `bootstrap/` (2 files), `builders/` (5 files), `utils/` (9 files), `templates.rs`, `view_models.rs`, and the `action`, `autoplay`, `chat_window`, `debug`, `games`, `history`, `layout`, `prompt_presets`, `settings` and `worlds` modules (handlers, templates and `mod.rs` files). The 57 `pub async fn` handlers are all in these files.

**Sampled, with the reason:**
- `tests/infrastructure/guardrails/layers.rs` and `free_fn.rs` (module header and the rule text only) and `arch-lint.toml:85-166`. Reason: they state rules about this area, so they show which rules exist next to the code.
- `src/application/errors.rs`, `src/application/generation/gate.rs` (the `Refusal` lines only), `src/application/games/catalogue.rs` (`active_game_id`). Reason: to see what the handlers call. Not part of the area.
- `grep` over `src/` for WebSocket and Server-Sent Events code. Reason: `AGENTS.md` says "HTTP/WebSocket".

**Not read:** the 22 `*_tests.rs` unit-test files in `src/adapters/driving/` (only their `mod` registration lines), `tests/http/*`, `tests/browser/*`, and the JavaScript and CSS in `assets/` (out of scope). Where a row depends on client behaviour, the label says *inferred*.

**Not opened:** `assets/research-01-lens.md` and every other `research-*.md` asset.

**Labels.** *known* = the cited lines show it. *inferred* = the lines point to it, but one step is my reading. *guessed* = weak evidence.

---

## 1. Concepts

Columns: concept, its opposite(s), evidence, how far it holds in the area, label. Evidence cites 2-3 sites.

| # | Concept | Opposite(s) | Evidence | How far | Label |
|---|---|---|---|---|---|
| C1 | **Ports and adapters, driving side** (primary adapter calls inward) | Handler that reaches storage or an outbound system itself | `src/adapters/driving/mod.rs:2`; `http/app_state.rs:31-43` (state holds application services only); `arch-lint.toml:164-166` (server may not import storage) | Everywhere for storage. `debug.rs` reaches `job_deps.narrator` (see MP-23) | known |
| C2 | **Dependency rule** (imports point inward; domain and application never import the adapter) | Inner layer imports the adapter | `arch-lint.toml:148-150` (server may not use storage models); `http/app_state.rs:16-29` (imports only `application`, `domain`, `error`, plus `bootstrap::wiring`) | Everywhere in the area. The one inward-to-outer import is `bootstrap::wiring::WiredApp` (`http/app_state.rs:26`, `http/bootstrap/server.rs:13`) | known |
| C3 | **Thin controller over an application service** | **Transaction script in the handler** (fat controller) | Thin: `http/games/handlers/games.rs:113-116,120,136`; `http/history/handlers/history.rs:25,35`. Fat: `http/settings/handlers/settings.rs:317-335`; `http/worlds/handlers/worlds.rs:172-190` | Thin in games, history, chat window, action. Fat in settings, worlds and prompt presets (MP-1) | known |
| C4 | **Front controller** (one central route table) | One router per feature, merged | `http/builders/router.rs:22-224` (all 57 routes in one function); `http/mod.rs:4-22` (one `pub mod` per feature) | Everywhere | known |
| C5 | **Hypermedia-driven application** (server sends HTML fragments, the markup carries the next action) | JSON API with client-side rendering (SPA) | `http/builders/router.rs:62-90` (`/fragment/*` routes); `http/templates.rs:16`; `http/games/templates/games.rs:61` (`hx-post`, `hx-swap`) | Everywhere. JSON only on `/debug/state` and `/debug/backend` (`http/debug/handlers/debug.rs:19,40`) | known |
| C6 | **RPC-style endpoints** (verb in the URL, POST for every write) | **REST** (resource URL, `PUT` and `DELETE` verbs) | `http/builders/router.rs:92-103` (`/games/:id/rename`, `/delete`, `/switch`); `http/builders/router.rs:135-138`; `http/builders/router.rs:207-217` | Everywhere. No `put`, `patch` or `delete` route | known |
| C7 | **Command-query separation at the route** (GET reads, POST writes) | One verb for both | `http/builders/router.rs:62-90` (GET fragments); `http/builders/router.rs:147-154` (POST `connections/test` calls a provider but writes no record) | Mostly. `GET /worlds/:key/edit` is outside `/fragment/` (`http/builders/router.rs:131-134`); `POST /action/check` can return a preview (`http/action/handlers/actions.rs:111-118`) | known |
| C8 | **Asynchronous request-reply** (accept, answer at once, report by status) | Synchronous request that waits for the work | `http/action/handlers/actions.rs:42-45` (`Job::start`, then `THINKING_STATUS`); `http/layout/handlers/endpoints.rs:70-89` (status endpoint); `http/templates.rs:76` (comment: "polling container") | Job and Autoplay routes. Connection test awaits (`http/settings/handlers/settings.rs:279-290`). The poll side is client code | known for the server half, inferred for the poll |
| C9 | **Command object** (text → `Action` → `JobRequest` → executor) | Handler that branches on the raw string and does the work | `http/action/handlers/actions.rs:35-46`; `http/chat_window/handlers/chat_window.rs:33,40`; `http/autoplay/handlers/autoplay.rs:41` | Action, Retry, Retrigger and Autoplay routes | known |
| C10 | **Parameter object** (a named change-set for the service) | Closure over mutable state; whole entity read, edited and written back | `http/prompt_presets/handlers/prompt_presets.rs:216-223` (`PresetEdit`); `http/games/handlers/games.rs:213-221` (`PresetSelection::new`) | In presets and per-game presets only (MP-17) | known |
| C11 | **Parse at the edge** (extractors and typed `Form` turn bytes into typed input before the handler body) | Validate deep inside; pass strings inward | `http/utils/current_game.rs:17-39` (`CurrentGame`); `http/chat_window/handlers/chat_window.rs:58` (`Path<(u64, usize)>`); `http/games/handlers/games.rs:103-107` | Mostly. Many form fields stay `String` and the handler parses them (`http/games/handlers/games.rs:163-165,186`; `http/worlds/handlers/worlds.rs:30-31,42-49`) | known |
| C12 | **Newtype extractor** (`CurrentGame(GameId)`) | Raw `u64` id | `http/utils/current_game.rs:17`; `http/games/handlers/games.rs:92` (`GameId(game.id)`) | The id of the current Game is a newtype; the id in a `Path` is a raw `u64` (MP-13) | known |
| C13 | **Refusal is not failure** (an expected rejection and a fault take two paths) | One error class, one status | `http/error.rs:13-27` (`Validation` → 400; other → generic 500); `http/utils/error.rs:41-46,48-51,63-67`; `http/error.rs:30-43` (`Refusal`) | Everywhere that `ApplicationError` or `Refusal` shows. Status mapping differs by site (MP-2) | known |
| C14 | **Error translation at the boundary through a trait impl and `?`** | Hand-written `match` ladder at each call site | `http/error.rs:13` (`impl IntoResponse for ApplicationError`); `http/games/handlers/games.rs:109-117`. Opposite: `http/worlds/handlers/worlds.rs:96-100,139-143` | 3 of 57 handlers return `Result<Response, ApplicationError>` (`http/games/handlers/games.rs:112,135,144`) plus `debug_state_handler` (`http/debug/handlers/debug.rs:19`). The rest use `match` | known |
| C15 | **Secure error messages** (hide internals from the response) | Verbose error text to the client | `http/error.rs:20-23` (`"Internal Server Error"` for a fault) | One place. Other sites print the raw error (MP-3) | known |
| C16 | **Progressive disclosure of failure** (short line, raw text behind "Details") | Raw text only, or short line only | `http/utils/error.rs:24-31`; `http/layout/handlers/endpoints.rs:105-111`; `http/builders/headers.rs:185-187` | Status, banner, Settings. Not in `render_error` fragments (`http/utils/error.rs:18-20`) | known |
| C17 | **Graceful degradation** (fail soft, keep the feature working) | **Fail fast** | `http/action/handlers/actions.rs:87-93,104-108` (text-check fault → send the command); `http/app_state.rs:88-91` (`"Unknown"` game name); `http/games/handlers/games.rs:238-244` (preset load fault → controls still render) | Several sites. Fail-fast sites sit next to them (MP-5) | known |
| C18 | **Postel's law** (liberal on input: unknown value → default) | **Strict parsing** (unknown value → refusal) | Liberal: `http/worlds/handlers/worlds.rs:74-88`; `http/settings/handlers/settings.rs:132-137`; `http/prompt_presets/handlers/prompt_presets.rs:283`. Strict: `http/games/handlers/games.rs:186-189`; `http/settings/handlers/settings.rs:180-183` | Both, even in one file (MP-4) | known |
| C19 | **Server-side rendering with compile-time templates** (Askama) | Runtime template engine; HTML built with `format!` | Askama: `http/templates.rs:14-19`; `http/games/templates/games.rs:25-31`. `format!` HTML: `http/builders/forms.rs:14-24`; `http/builders/headers.rs:104-108` | Both (MP-6) | known |
| C20 | **Co-located (inline) templates** (markup in a Rust string next to its view struct) | Template files in a `templates/` folder (MVC split) | `http/templates.rs:16`; `http/prompt_presets/templates/prompt_presets.rs:13-169`. No `.html` file under `src/`. | All fragments. The page shell is a file: `http/chat_window/handlers/chat_window.rs:19` (`include_str!` of `assets/index.html`) | known |
| C21 | **Presentation model / view model** (a type shaped for the template) | Domain type goes straight into the template | View model: `http/view_models.rs:126-155` (`StoryRowView`), `191-217` (`LlmMessageView`). Domain type: `http/games/templates/games.rs:124` (`Vec<WorldCard>`); `http/builders/presets.rs:138` (`PromptPreset`); `http/settings/templates/settings.rs:92` | Story log, dock, sidebar, LLM messages use view models. Presets, Settings, Games panel mix (MP-8) | known |
| C22 | **Escape by default, with a marker type for trusted markup** (`SafeHtml`) | Manual escaping in string builders; `\|safe` filter on `String` | Marker: `http/view_models.rs:15-25`; `http/games/templates/games.rs:155`. Manual: `http/builders/forms.rs:19-23`; `http/utils/response.rs:35-41`. Filter: `http/games/templates/games.rs:50,94` | Three mechanisms in one area (MP-7) | known |
| C23 | **Sanitize untrusted markup** (raw HTML from Markdown becomes text) | Pass Markdown output through unchanged | `http/utils/view_models.rs:14-22`; `http/view_models.rs:129` | One place: the story text | known |
| C24 | **Composite view** (render the parts to strings, embed them in the parent) | Template `include` or macro; one flat template | `http/prompt_presets/templates/prompt_presets.rs:18-20,188-190`; `http/settings/templates/settings.rs:120-122`; `http/games/templates/games.rs:127,50`. No `{% include %}` in the area | Everywhere a panel has a part | known |
| C25 | **Logic in the view model** (the template only prints) | **Logic in the template** | View model: `http/view_models.rs:146-151`; `http/builders/presets.rs:152-164,167-204`. Template: `http/templates.rs:25` (`loop.last && entries.len() > 1`, `.as_ref().unwrap()`); `http/settings/templates/settings.rs:287` (`text_check_mode == "disabled"`) | Both (MP-9) | known |
| C26 | **Single source of truth** for a fragment | Copy the markup | Claimed in comments: `http/games/templates/games.rs:128-130`; `http/prompt_presets/handlers/prompt_presets.rs:25-27`; `http/view_models.rs:27-28`; `http/utils/template_helpers.rs:10-12` | Where a comment names it. Copies exist beside it (MP-11) | known |
| C27 | **Table-driven code** (a `const` array drives the loop) | Unrolled, repeated calls | Table: `http/games/handlers/games.rs:256-270`; `http/builders/headers.rs:18,157-164`; `http/settings/templates/settings.rs:126-139` | Both for the same four preset types (MP-10) | known |
| C28 | **Closed-world enum dispatch** (exhaustive `match`, states as enum variants) | String tag or boolean flag | Enum: `http/builders/headers.rs:20-36,56-79` (`RoleHealthState`); `http/settings/handlers/settings.rs:230-245` (`ConnectionTestSurface`). String: `http/view_models.rs:137-141` (`log_type: String`); `http/games/templates/games.rs:160` (`perspective == "second"`) | Both (MP-12) | known |
| C29 | **Mapper at the edge** (form → domain; domain → view) | Deserialize straight into the domain type; no mapper | `http/worlds/handlers/worlds.rs:59-93`; `http/prompt_presets/handlers/prompt_presets.rs:146-165`; `http/settings/handlers/settings.rs:96-113`; `http/utils/view_mappers.rs:7-15` | Every form. Four constructor shapes (MP-12 and shapes S5) | known |
| C30 | **CQRS-style read facade** (`GameViewQuery` reads; `GameWriter` writes) | One CRUD service for read and write | Split: `http/app_state.rs:39-42`; `http/layout/handlers/endpoints.rs:77`; `http/debug/handlers/debug.rs:21-22`. CRUD: `http/worlds/handlers/worlds.rs:97,117`; `http/settings/handlers/settings.rs:30` | Game state is split. World, Preset, Persona, Settings are CRUD services (MP-24) | inferred (the split is in the field names; I did not read the services) |
| C31 | **Dependency injection through one shared state bag** (`State<AppState>`) | Narrow dependencies per handler (`FromRef` sub-state); service locator | `http/app_state.rs:46-59` (`from_wired`); `http/history/handlers/history.rs:20` (takes the whole bag, uses one field); `http/worlds/handlers/worlds.rs:131` (`State(_state)`, unused) | Every handler | known for the shape, inferred for the name "service locator" |
| C32 | **Test seam by conditional compilation** | Trait object injected by wiring | `http/app_state.rs:65-74` (`#[cfg(feature = "testing")]`); `http/bootstrap/server.rs:21-29` (`#[cfg(test)] impl Default`) | Two sites in the area | known |
| C33 | **Co-located unit tests in a sibling `*_tests.rs`** | Inline `mod tests` | `http/mod.rs:6-7,27-34`; `http/action/handlers/mod.rs:11-12`; `http/utils/mod.rs:13-26` | 22 files. `http/autoplay/handlers/mod.rs:1-6` registers none | known (registration only; files not read) |
| C34 | **Package by feature** (folder per feature, `handlers/` and `templates/` inside) | Package by layer or by kind | Feature: `http/games/mod.rs:4-5`; `http/worlds/mod.rs:4-5`. Kind: `http/templates.rs`, `http/view_models.rs`, `http/builders/`, `http/utils/` (`http/mod.rs:10,19,20,21`) | Mixed (MP-21) | known |
| C35 | **Module as facade** (private file, `pub use` of chosen names) | `pub mod` that exposes the file | Facade: `http/action/handlers/mod.rs:4-9`; `http/games/handlers/mod.rs:4-11`. Open: `http/chat_window/handlers/mod.rs:4-8` (`pub mod` plus `pub use`); `http/worlds/templates/mod.rs:4` | Mostly facade. Templates modules differ | known |
| C36 | **Utility module** (free functions in `utils/`, `builders/`, `handlers/`) | Behaviour on the type (`impl`, `From`) | Free fn: `http/utils/view_mappers.rs:7`; `http/utils/handler_helpers.rs:7,19`. On the type: `http/view_models.rs:126` (`impl From`). Rule text: `tests/infrastructure/guardrails/free_fn.rs:1` | Both (MP-12) | known |
| C37 | **Logging in the handler** (log where the fault is turned into a response) | Cross-cutting middleware only | Handler: `http/utils/fragment.rs:20`; `http/action/handlers/actions.rs:90,106`; `http/layout/handlers/endpoints.rs:82,123`. Middleware: `http/builders/router.rs:222` (`TraceLayer`) | Both. Some conversions log nothing (MP-3) | known |
| C38 | **Runtime resource binding** (`ServeDir` on a path relative to the working directory) | Compile-time embedding | Runtime: `http/builders/router.rs:219-221`. Compile-time: `http/chat_window/handlers/chat_window.rs:19` | Both, in two files (MP-26) | known |
| C39 | **Trusted-network assumption** (no authentication, authorization or CSRF token in the adapter) | Defence in depth | `http/builders/router.rs:222` (the only layer is tracing); no auth code in the 60 files | Whole area | inferred (absence of code; the intent is not stated here) |
| C40 | **Immediate apply (autosave on change)** | **Explicit save** (submit, then commit) | Autosave: `http/games/templates/games.rs:154,159`; `http/worlds/templates/worlds.rs:222-223`; `http/settings/templates/settings.rs:50,283`. Explicit: `http/worlds/templates/worlds.rs:78,108-109`; `http/settings/templates/settings.rs:169,204` | Both, on one World form (MP-14) | known |
| C41 | **Edge-minted identity** (the adapter makes the id) | Id assigned by storage or by the user | Minted: `http/utils/handler_helpers.rs:19-28`; `http/prompt_presets/handlers/prompt_presets.rs:184,255`; `http/settings/handlers/settings.rs:185`. Storage: `http/games/handlers/games.rs:113-115`. User: `http/worlds/handlers/worlds.rs:25` | Three sources (MP-15) | known |
| C42 | **Graceful shutdown; the server returns its handle** | Process exit on signal | `http/bootstrap/server.rs:50-62`; `http/bootstrap/server.rs:31-34` (returns `(SocketAddr, JoinHandle)`) | One place | known |
| C43 | **Blocking work off the async runtime** (`spawn_blocking`) | Blocking call inside `async fn` | `http/settings/handlers/settings.rs:264-277`. Opposite (if the service blocks): `http/action/handlers/actions.rs:99-103`; `http/games/handlers/games.rs:86` | One site offloads | inferred (I did not read whether the services block) |
| C44 | **HTTP status as a UI control channel** (non-2xx stops the htmx swap) | Status for HTTP meaning only | `http/utils/error.rs:48-50,63-65`; `http/utils/fragment.rs:26-37` (`HX-Reswap: none`); `http/games/handlers/games.rs:225-226` ("house style", 200 for validation) | Everywhere that an error fragment shows (MP-2) | known |
| C45 | **Out-of-band swaps and header-driven swaps** | One response, one target | `http/builders/headers.rs:152-154,189-192` (`hx-swap-oob`); `http/builders/headers.rs:195-201` (`HX-Retarget`, `HX-Reswap`); `http/utils/response.rs:19-25` (`HX-Refresh`) | Header, Settings health, action status | known |
| C46 | **Full refresh after a write** (`HX-Refresh`, close to Post/Redirect/Get) | Return the changed fragment | Refresh: `http/games/handlers/games.rs:116,121,137`; `http/chat_window/handlers/chat_window.rs:24`. Fragment: `http/worlds/handlers/worlds.rs:122-125`; `http/prompt_presets/handlers/prompt_presets.rs:190` | Both (MP-18) | known |
| C47 | **Declarative behaviour in markup** (`hx-*` attributes) | **Imperative inline JavaScript** (`onclick`) | Declarative: `http/games/templates/games.rs:61`; `http/builders/presets.rs:134`. Imperative: `http/templates.rs:25` (`onclick="showEditForm(...)"`), `63`, `79` | Both, in one template (MP-20) | known |
| C48 | **Accessible markup** (ARIA labels, roles) | Unlabelled icon buttons, mouse-only targets | Labelled: `http/templates.rs:25,79,94` (`aria-label`, `role`). Mouse-only: `http/templates.rs:63` (`div` with `onclick`) | Most icon buttons carry a label | inferred (keyboard behaviour is client code) |
| C49 | **Null object for "no Game"** (`Option<CurrentGame>`; answer "idle") | Refuse with 400 | Null: `http/layout/handlers/endpoints.rs:72-76,115-119`; `http/games/handlers/games.rs:30-33`. Refuse: `http/utils/current_game.rs:36-37` | Both (MP-13) | known |

**Counts:** 49 concept rows. Strongest (many sites, little doubt): C3, C4, C5, C6, C8, C13, C19, C20, C22, C24, C40, C44, C45.

---

## 2. Meeting points

Each one is a question. The evidence is what I saw. I make no ruling.

**MP-1. Where does a business rule live: in the handler or in a service?**
`delete_connection_handler` holds three rules (`Connection not found`, `Cannot delete the last connection`, `role in use`) inside an `update_settings` closure in the handler (`http/settings/handlers/settings.rs:317-335`). `update_world_handler` merges the form into a `WorldCard` and drops a stale `default_scenario_id` in the handler (`http/worlds/handlers/worlds.rs:172-190`). `duplicate_preset_handler` builds the copy, clears `is_default` and sets the id (`http/prompt_presets/handlers/prompt_presets.rs:254-260`). `activate_preset_handler` refuses "not allowed for mode" in the handler (`http/prompt_presets/handlers/prompt_presets.rs:287-289`). Next to them, `rename_game` and `delete_game` are one call to `GameCatalogue` (`http/games/handlers/games.rs:136,145`). Which of these steps is a rule of the application, and which is a form-to-entity step?

**MP-2. Which HTTP status does a refusal carry?**
`ApplicationError::Validation` maps to 400 in `http/error.rs:16` and `http/utils/error.rs:69-74`. The same kind maps to 200 in `render_posture_result` ("house style", `http/games/handlers/games.rs:225-226,233`), in the delete-World branch (`http/worlds/handlers/worlds.rs:215-218`) and in the posture-value branch (`http/worlds/handlers/worlds.rs:254-256`). `Refusal::StillThinking` is 200 with markup (`http/error.rs:33`) while `Refusal::BadRequest` is 400 (`http/error.rs:34-36`). In one file, a missing World is 400 (`http/worlds/handlers/worlds.rs:141`) and a bad posture value is 200 (`http/worlds/handlers/worlds.rs:255`). Is the rule "the status says whether htmx swaps", "the status says what happened", or both?

**MP-3. What error text reaches the player?**
`http/error.rs:20-23` sends `"Internal Server Error"` for a fault. `internal_error(format!("Failed to load worlds: {e}"))` sends the raw error (`http/worlds/handlers/worlds.rs:99,142,205`). `render_error(&e.to_string())` does the same in Settings (`http/settings/handlers/settings.rs:35-37`) and `failed_fragment_response(&e.to_string())` in the fragment helper (`http/utils/fragment.rs:21`). `list_games_fragment` sends `"Failed to list games"` and drops the error without a log (`http/games/handlers/games.rs:33-35`). `debug_state_handler` drops it with `map_err(|_| ...)` (`http/debug/handlers/debug.rs:22-23`). Which failures show the raw text, which show a fixed line, and which log?

**MP-4. Strict or liberal parsing of the same fields?**
In `worlds.rs`, create and update read `narrator_mode`, `narrative_perspective` and `narrative_tense` with `parse_or_default` (`http/worlds/handlers/worlds.rs:74-88,181-189`). The posture handler for the same three fields uses `from_str` and answers an error fragment (`http/worlds/handlers/worlds.rs:238-257`). In Settings, an unknown `check_mode` becomes `Disabled` (`http/settings/handlers/settings.rs:132-137`), but an unknown provider is a refusal (`http/settings/handlers/settings.rs:180-183`). Does a bad value mean "use the default" or "refuse", and what decides?

**MP-5. A failed read: an empty default, or an error?**
In `list_games_fragment`, a failed `list_worlds` is a 500 (`http/games/handlers/games.rs:52-55`) and a failed `list_personas` is a warning and an empty list (`http/games/handlers/games.rs:56-62`). In `worlds.rs`, a failed `list_games` or `list_worlds` becomes `unwrap_or_default()` (`http/worlds/handlers/worlds.rs:102,119-123,198-201`). In `presets_template`, four failed reads become empty lists (`http/prompt_presets/handlers/prompt_presets.rs:32-47`). Which reads may fail soft, and what does the player see when they do (an empty list reads as "No worlds defined", `http/worlds/templates/worlds.rs:27`)?

**MP-6. Askama template or `format!` for HTML?**
Panels use Askama (`http/games/templates/games.rs:25-31`). The preset card is Askama (`http/builders/presets.rs:132-142`), but the preset view and edit forms in the same file are `format!` strings (`http/builders/presets.rs:25-42,105-127`). Error fragments, the header banner and the form fields are `format!` (`http/utils/error.rs:11-16,24-31`; `http/builders/headers.rs:104-108`; `http/builders/forms.rs:14-24`). The World posture selects are a `format!` that embeds Askama-rendered options (`http/worlds/templates/worlds.rs:237-253`). What decides which markup is a template and which is a string?

**MP-7. Which mechanism marks markup as trusted, and who escapes?**
Three marks exist: the `SafeHtml` type (`http/view_models.rs:15-25`), the `|safe` filter on a `String` (`http/games/templates/games.rs:50`) and raw `format!` arguments (`http/utils/error.rs:26-29`, where `message` is escaped and `detail_html` is not). `saved_games_empty` is a `SafeHtml` and the template still applies `|safe` (`http/games/templates/games.rs:94,130`). Two escape functions exist: Askama's and `html_escape` (`http/utils/response.rs:35-41`). Which one is the escape rule for a hand-built fragment?

**MP-8. View model or domain type in the template?**
`StoryRowView` and `LlmMessageView` decouple the template (`http/view_models.rs:2,126,191`). `GamesPanelTemplate` holds `Vec<WorldCard>` (`http/games/templates/games.rs:124`), `PresetCardTemplate` holds `PromptPreset` (`http/builders/presets.rs:138`), `SettingsTemplate` holds `Vec<LlmProviderConfig>` (`http/settings/templates/settings.rs:92`), and `PromptPresetsTemplate` holds a whole `AppSettings` (`http/prompt_presets/templates/prompt_presets.rs:177`). The file header says view models "decouple templates from domain types". Which templates are meant to follow that?

**MP-9. Logic in the template or in the view?**
The story-log template decides `loop.last && entries.len() > 1` and calls `.as_ref().unwrap()` after `is_some()` (`http/templates.rs:25`), while `StoryRowView` already computes `prev_swipe_index` and `next_swipe_index` (`http/view_models.rs:146-151`). `PresetCardTemplate` precomputes badges and buttons (`http/builders/presets.rs:152-164`). The Settings template compares strings: `text_check_mode == "disabled"` (`http/settings/templates/settings.rs:287`), while the Rust side holds the `TextCheckMode` enum (`http/settings/templates/settings.rs:316-321`). Where is the line between "decide" and "print"?

**MP-10. One table or four calls for the four preset types?**
`PRESET_PICKER_TYPES` drives a loop over the four `PresetType`s (`http/games/handlers/games.rs:256-270`). `presets_template` makes four separate `list_presets` calls (`http/prompt_presets/handlers/prompt_presets.rs:32-47`), the panel struct has four fields and four near-equal `*_cards` methods (`http/prompt_presets/templates/prompt_presets.rs:170-202`), and the template repeats a block of about 35 lines four times (`http/prompt_presets/templates/prompt_presets.rs:15-165`). `GamePostureTemplate` has four option fields (`http/games/templates/games.rs:207-210`). Is the number of preset types a fixed fact, or a list that can grow?

**MP-11. Single source, or copies, for a fragment?**
Comments claim one source for the empty saved-games line, the card response and the select options (`http/games/templates/games.rs:128-130`; `http/prompt_presets/handlers/prompt_presets.rs:25-27`; `http/view_models.rs:27-28`). Copies sit beside them: the rename form is written twice (`http/games/templates/games.rs:40-46,102-108`); the World list refresh (`list_worlds`, `list_games`, `games_per_world`, render) is written three times (`http/worlds/handlers/worlds.rs:96-105,117-125,196-203`); the `NpcPortraitView` mapping is written twice (`http/app_state.rs:123-126,158-161`). Where does the "one source" rule stop?

**MP-12. One mapping between an enum and its text, or one per site?**
`NarratorMode` carries `as_str`, `from_str` and `display_label` in the domain and the adapter uses them (`http/builders/presets.rs:90,180`; `http/games/handlers/games.rs:186`). `TextCheckMode` is mapped to strings by a local `match` twice (`http/settings/handlers/settings.rs:132-137`; `http/settings/templates/settings.rs:316-321`) and a third time in the template's `<option>` values (`http/settings/templates/settings.rs:287-290`). `LlmBackendType` is matched locally (`http/settings/templates/settings.rs:228-233`) and the provider list is a literal array (`http/view_models.rs:48-53`) that omits `Mock`, which the match has. Perspective and tense have `as_str` (`http/games/templates/games.rs:226-227`) and also literal `("second", "Second person")` pairs (`http/worlds/templates/worlds.rs:228-235`; `http/games/templates/games.rs:160-161`). Who owns the text form of a domain enum?

**MP-13. How does a request name its Game: by the server pointer or by the URL?**
`CurrentGame` reads one process-wide pointer (`http/utils/current_game.rs:34`), and most routes use it: `/action`, `/retrigger`, `/history/:id`, `/autoplay` (`http/builders/router.rs:24-31,41,50-57`). Other routes carry the id in the path: `/games/:id/switch`, `/mode`, `/posture`, `/presets` (`http/builders/router.rs:92-115`). With no active Game, `CurrentGame` refuses with 400 `"No active game"` (`http/utils/current_game.rs:36-37`); `Option<CurrentGame>` answers `idle`, `failed` or a list instead (`http/layout/handlers/endpoints.rs:74-76,117-119`; `http/games/handlers/games.rs:30-33`). The id in a path is a raw `u64` (`http/games/handlers/games.rs:119,131`) and the pointer id is `GameId`. Can two browser tabs on two Games meet here (inferred, not tested)?

**MP-14. Autosave and explicit save on one form?**
The World edit form has a submit button for details and an autosave group for posture. A note on the page says so: "Cancel applies to the details fields only. Posture saves automatically." (`http/worlds/templates/worlds.rs:114,119-123`). The posture group posts on `change` to its own route (`http/worlds/templates/worlds.rs:222-223`). The Games panel and the text-check card also save on `change` (`http/games/templates/games.rs:154`; `http/settings/templates/settings.rs:283`). What decides which field saves at once and which waits for a button?

**MP-15. Who makes the id: the adapter, storage or the user?**
Preset and Connection ids come from `generate_storage_id` in the adapter, which reads the system clock and a UUID (`http/utils/handler_helpers.rs:19-28`; `http/prompt_presets/handlers/prompt_presets.rs:184,255`; `http/settings/handlers/settings.rs:185`). `create_game` takes keys and returns no id (`http/games/handlers/games.rs:113-115`). A World key is typed by the user in the form (`http/worlds/handlers/worlds.rs:25`). Is the id an adapter concern, a service concern or a user fact?

**MP-16. What does an unchecked checkbox mean?**
A comment says the World flag uses "the same grammar as the PresetForm flags" (`http/worlds/handlers/worlds.rs:35-36`). On World update, an absent `options_always_on` means false (`http/worlds/handlers/worlds.rs:190`). On preset update, both mode flags absent means "preserve" (`http/prompt_presets/handlers/prompt_presets.rs:211-215`). On preset create, both absent means "both modes" (`http/prompt_presets/handlers/prompt_presets.rs:176-182`). On text check, absent `enable_auto_check` means false (`http/settings/handlers/settings.rs:119-120`). Is "absent" one rule or three?

**MP-17. How does a handler tell a service what to change?**
Three ways: a named change-set (`PresetEdit`, `http/prompt_presets/handlers/prompt_presets.rs:216-225`), a closure over the stored value (`update_settings`, `update_connection`; `http/settings/handlers/settings.rs:139-144,212-222`) and a read of the whole entity, edit in the handler, and write of the whole entity (`http/worlds/handlers/worlds.rs:157-195,259-277`). Which one is the default for a new edit route?

**MP-18. After a write: full refresh, or the changed fragment?**
Game create, switch, rename and reset answer `HX-Refresh: true` (`http/games/handlers/games.rs:116,121,137`; `http/chat_window/handlers/chat_window.rs:24`). World create and update, and preset create and activate, answer the re-rendered panel (`http/worlds/handlers/worlds.rs:122-125,200-203`; `http/prompt_presets/handlers/prompt_presets.rs:190,301-304`). `switch_swipe_handler` renders the story log in the handler (`http/chat_window/handlers/chat_window.rs:64-69`) but the edit and delete routes answer a status span or an empty body (`http/history/handlers/history.rs:26,36`). What decides refresh, fragment or status-only?

**MP-19. A busy Game: 200 with markup, or a refusal?**
`Refusal::StillThinking` answers 200 with `BUSY_STATUS` markup, and the client keys on class `.status.wait` (`http/error.rs:33`; `http/action/handlers/actions.rs:24-26`). Other refusals use `render_error` with 400 or 500 (`http/error.rs:34-41`). Why is "busy" the one refusal that is a normal reply?

**MP-20. `hx-*` attributes or inline `onclick`?**
`OptionsDockTemplate` uses both in one template: an `hx-post` form for regenerate and `onclick="useOption(this)"` for each option (`http/templates.rs:79`). The story log uses `onclick="showEditForm(...)"`, `submitNewSwipe()` and `switchSwipe(...)` (`http/templates.rs:25`). Games, Worlds, presets and Settings use `hx-*` only (`http/games/templates/games.rs:109-110`; `http/builders/presets.rs:134`). What decides which interaction is a hypermedia attribute and which is a script call?

**MP-21. Folder per feature, or folder per kind?**
Games, Worlds, presets and Settings own `handlers/` and `templates/` (`http/games/mod.rs:4-5`). The story log, dock, sidebar, headshots, header, LLM messages and text-check preview templates share one file, `http/templates.rs:14-216`, and their view models share `http/view_models.rs`. Their handlers are spread over `layout/`, `chat_window/`, `action/` and `history/`. The free-function guardrail names `builders`, `utils` and `handlers` as places for free functions (`tests/infrastructure/guardrails/free_fn.rs:1`). Where does a new fragment go?

**MP-22. What does a handler return?**
The route guardrail says handlers return `Response<Body>` and not `(StatusCode, String)` (`tests/infrastructure/guardrails/layers.rs:57`). Handlers return `Response<Body>` (`http/action/handlers/actions.rs:52`), `Response` (`http/history/handlers/history.rs:24`), `Html<String>` (`http/layout/handlers/endpoints.rs:73`; `http/settings/handlers/settings.rs:24`), `Result<Response, ApplicationError>` (`http/games/handlers/games.rs:112`), `Result<Json<_>, StatusCode>` (`http/debug/handlers/debug.rs:19`), `String` (`http/debug/handlers/debug.rs:35`) and `Json<_>` (`http/debug/handlers/debug.rs:40`). The guardrail skips `debug.rs` (`tests/infrastructure/guardrails/layers.rs:66`). Is `Html<String>` a case of the rule, or a second shape?

**MP-23. Facade or direct reach into the application?**
Read-side queries go through `GameViewQuery` (`http/app_state.rs:86-141`). The debug routes read `state.job_deps.narrator.backend_info()` (`http/debug/handlers/debug.rs:26,42`); one carries a lint exemption comment (`http/debug/handlers/debug.rs:41`). The state also offers two ways to get one service: the public field `text_check_service` and the method `text_check_service()` (`http/app_state.rs:37,76-78`), and a `settings()` shortcut (`http/app_state.rs:61-63`) next to direct `settings_service` calls (`http/settings/handlers/settings.rs:139,190`). Which path is the intended one?

**MP-24. Split read and write for a Game, CRUD service for the rest?**
For a Game: `GameViewQuery` reads, `GameWriter` and `GameCatalogue` write (`http/app_state.rs:39-42`). For World, Preset, Persona and Settings, one service reads and writes (`http/worlds/handlers/worlds.rs:97,117`; `http/settings/handlers/settings.rs:30,139`). Role health, which belongs to Settings, is read through `GameViewQuery` (`http/settings/handlers/settings.rs:31`). Which reads go to which service?

**MP-25. Who fetches the data for a fragment?**
Four places do it: `AppState::render_*` fetch and render (`http/app_state.rs:99-110`); a builder takes data and renders (`http/builders/headers.rs:175`); a handler-local helper fetches and renders (`http/games/handlers/games.rs:240-252`; `http/prompt_presets/handlers/prompt_presets.rs:31-56`); and a `Template::from_*` constructor maps data (`http/settings/templates/settings.rs:100`). `AppState::render_*` report a failed read as `EngineError::Config` (`http/app_state.rs:95,103,116,121,139`) and a failed render as `EngineError::Template` (`http/app_state.rs:109`). Why `Config` for a read failure?

**MP-26. Page shell embedded, assets from disk?**
`index.html` is embedded at compile time (`http/chat_window/handlers/chat_window.rs:19`). The `assets` and `data` folders and the fallback are served from the working directory at run time (`http/builders/router.rs:219-221`). Which of these needs a rebuild after an edit, and does a server started from another directory still find them (inferred)?

**Count: 26 meeting points.**

---

## 3. Shapes with no name

| # | Shape | Evidence |
|---|---|---|
| S1 | **Text status protocol.** An endpoint answers a bare word or a one-line span that the client parses: `idle`, `failed`, `reset`, a step name, `autoplay 2/5 narrating`, `<span class="status ready">Edited</span>`. | `http/layout/handlers/endpoints.rs:75,91-103,121,124`; `http/history/handlers/history.rs:26`; `http/action/handlers/actions.rs:26-28` |
| S2 | **Markup as contract with the client script.** A Rust constant holds markup, and a comment names the script that depends on its class. | `http/action/handlers/actions.rs:24-26`; `http/builders/presets.rs:121`; `http/worlds/templates/worlds.rs:112` (`data-error-slot`, `inline-error-slot`) |
| S3 | **Dual-use fragment.** One fragment is part of a panel and also a swap target on its own, so the code renders it ahead of time and passes it as a string field. | `http/games/templates/games.rs:126-127`; `http/prompt_presets/handlers/prompt_presets.rs:25-27`; `http/games/handlers/games.rs:240-252` |
| S4 | **Error as view input.** A handler re-renders the whole panel and passes the error in as a template field. | `http/settings/handlers/settings.rs:29-39,337-340,356-359`; `http/settings/templates/settings.rs:113-119` |
| S5 | **Four constructor shapes for one job (data → view).** `impl From<&T>`, `Template::from_<source>`, `Template::new`, and a free `*_to_view` function. | `http/view_models.rs:126`; `http/settings/templates/settings.rs:100`; `http/view_models.rs:233`; `http/utils/view_mappers.rs:7` |
| S6 | **Twin guard macros.** Two `macro_rules!` return early with an error response. They differ only in status. | `http/prompt_presets/handlers/prompt_presets.rs:58-79` |
| S7 | **Anonymous tuples across a layer.** A service returns, or a handler passes, a tuple of strings or of three records. | `http/app_state.rs:113-125`; `http/worlds/handlers/worlds.rs:139,157,259`; `http/debug/handlers/debug.rs:26` |
| S8 | **Checkbox grammar.** One `bool` field per checkbox, `#[serde(default)]`, because the form decoder cannot read a list. A guardrail checks it. | `http/prompt_presets/handlers/prompt_presets.rs:115-133`; `tests/infrastructure/guardrails/layers.rs:96-100` |
| S9 | **Two error-fragment constructors that differ by a prefix.** `error_fragment` adds none and `render_error` adds `Error: `. Callers use one or the other. | `http/utils/error.rs:11-20`; `http/prompt_presets/handlers/prompt_presets.rs:65-66` (no prefix); `http/utils/fragment.rs:30` (prefix) |
| S10 | **Wrapper stack for one render.** `preset_card_html`, `preset_card_view` and `card_response` wrap one template each. | `http/builders/presets.rs:226-232`; `http/prompt_presets/handlers/prompt_presets.rs:27-29` |

**Count: 10 shapes.**

---

## 4. Questions

Other questions that the reading raised.

- **Q1.** `AGENTS.md` says "HTTP/WebSocket server". A search of `src/` finds no WebSocket or SSE code. The area uses plain `GET` fragments and status endpoints (`http/builders/router.rs:62-90`). Is "WebSocket" an old claim, or does a path outside the area carry it? (inferred)
- **Q2.** `/data` serves the `data/` folder over HTTP (`http/builders/router.rs:220`). Which files are in that folder at run time, and does any of them need to stay private? I do not know.
- **Q3.** `ok(...)` builds a response with no `Content-Type` (`http/utils/response.rs:7-16`). `Html(...)` sets `text/html`. Handlers use both for the same kind of body (`http/action/handlers/actions.rs:43` and `http/prompt_presets/handlers/prompt_presets.rs:28`). Do both reach htmx the same way? (inferred; not run)
- **Q4.** The debug routes are mounted in the one router (`http/builders/router.rs:32-40`). Are they present in every build, and is that on purpose?
- **Q5.** Is the lack of any authentication or CSRF layer (`http/builders/router.rs:222`) a stated rule of the product (a local, single-user tool), or only the state of the code?
- **Q6.** The adapter fixes presentation limits: 50 LLM messages (`http/app_state.rs:172`), a 120-character preview (`http/builders/presets.rs:209`), `maxlength="120"` (`http/games/templates/games.rs:43`). Where should such a limit live?
- **Q7.** Two modules are named `view_models`: `http/view_models.rs` (view structs) and `http/utils/view_models.rs` (the Markdown renderer, `http/utils/view_models.rs:6`). A third, `view_mappers`, maps `Game` to a row. Which name holds which job?
- **Q8.** `AppState::render_story_log` sets `view_only` from `run_holds` after it builds the template (`http/app_state.rs:105-107`). `posture_controls_html` does the same (`http/games/handlers/games.rs:249-251`). Each template has a `view_only` field that its constructor sets to `false` (`http/templates.rs:41`; `http/games/templates/games.rs:248`). Should the constructor take it?
- **Q9.** Handler names do not share one suffix: `*_handler`, `*_fragment`, `*_form`, `*_panel` (`http/builders/router.rs:62,141,157`). Is the suffix a signal, or only history?
- **Q10.** Every `*_tests.rs` file sits beside its module, but `http/autoplay/handlers/mod.rs:1-6` registers none. `tests/http/autoplay.rs` exists (per `tests/AGENTS.md`). Is the E2E file the only test for that handler by design?
- **Q11.** `delete_game_handler` receives `Option<CurrentGame>` only to compute the empty-state line (`http/games/handlers/games.rs:140-147,151-160`). If the deleted Game is the active one, what does the client show? I do not know.

**Count: 11 questions.**

---

## Hazards to check (inferred, not tested)

1. `http/worlds/handlers/worlds.rs:119-125`: after a successful create, a failed `list_worlds` renders an empty panel ("No worlds defined", `http/worlds/templates/worlds.rs:27`).
2. `http/games/handlers/games.rs:56-62` with `http/games/templates/games.rs:71-72`: a failed `list_personas` shows "No personas available. Create a persona first."
3. `http/worlds/handlers/worlds.rs:157-195` and `259-277`: a details save and a posture autosave each read the whole World and write it back. Two requests close in time can overwrite each other's fields.
4. `http/utils/response.rs:7-16`: `ok()` sets no `Content-Type` (see Q3).
5. `http/utils/error.rs:24-31`: `error_disclosure` escapes `message` but not `detail_html`. Callers must escape first (`raw_error_detail` does). A new caller can miss it.
6. `http/chat_window/handlers/chat_window.rs:19` versus `http/builders/router.rs:219-221`: an edit of `assets/index.html` needs a rebuild, but an edit of the other assets does not.
7. `http/settings/handlers/settings.rs:132-137`: a mistyped or missing `check_mode` silently turns text check off.
