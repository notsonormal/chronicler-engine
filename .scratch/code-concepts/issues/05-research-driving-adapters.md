# Research: the concepts in the driving adapters

Type: research
Status: resolved

## Question

Which concepts does the code in `src/adapters/driving/` (HTTP handlers, the inline Askama templates, view models and builders; about 4,800 lines without the unit tests) follow, and where do opposing or overlapping concepts meet in it?

Write the asset in the shape that the map's Notes give (Concepts, Meeting points, Shapes with no name, Questions). Do not rule on a concept or label a meeting point; write it as a question.

Dig the concepts out of the code. You get no list of concepts: do not open `assets/research-01-lens.md` or the other research assets. Read the code, and find the shapes that it repeats and the choices that it makes. Give each one the name that software-engineering books and well-known essays give it (a principle, an architecture style, a design style, a pattern or an idiom), and its opposite. When no such name fits, put the shape under Shapes with no name.

The HTMX fragment shape is in scope as it shows in the Rust code and the templates. The JavaScript and CSS files in `assets/` are out of scope.

## Done when

Every module in the scope is read, or is named in the asset as sampled with the reason. Every row cites `path:line` evidence and has a known, inferred or guessed label.

## Answer

Asset: [research-05-driving-adapters.md](../assets/research-05-driving-adapters.md). The researcher read all 60 non-test `.rs` files in `src/adapters/driving/` (4,816 lines, 57 handlers, 57 routes) and sampled nothing in the area. The researcher did not open the other research assets. The researcher read none of the 22 `*_tests.rs` files (only their `mod` lines), and none of `tests/http`, `tests/browser` or the `assets/` JavaScript and CSS. Outside the area, the researcher opened only `arch-lint.toml`, the rule text of `tests/infrastructure/guardrails/layers.rs` and `free_fn.rs`, and a few lines of `application/errors.rs`, `application/generation/gate.rs` and `application/games/catalogue.rs`. The researcher checked the 450 `path:line` citations with a script (all in range, the first line of each range read) and corrected 5 wording or path items. No line number was wrong.

- **49 concepts.** The strongest (known): hypermedia-driven HTML fragments (the opposite is a JSON API); a front controller with one route table of 57 RPC-style POST and GET routes; ports and adapters on the driving side with a lint that bars storage imports; asynchronous request-reply (a Job starts, the client reads status); refusal kept apart from failure at the boundary; compile-time Askama templates written inline in Rust; escape by default with a `SafeHtml` marker type; composite views built from pre-rendered strings; immediate apply (autosave) next to explicit save; HTTP status used as an htmx swap control.
- **26 meeting points, each written as a question.** Examples: the same `Validation` refusal answers 400 in one place and 200 in another, in one file (MP-2); the same three World fields parse with a default in create and update but strictly in the posture handler (MP-4); "an unchecked checkbox" means false, preserve or both-modes in three handlers, although a comment says they share one grammar (MP-16); three ways to mark markup as trusted, and two escape functions (MP-7); the four preset types are looped from one table in one file and unrolled four times in another (MP-10); a request names its Game by the server pointer in most routes and by the URL in others (MP-13).
- **10 shapes with no name, 11 questions.** The shapes include a text status protocol (`idle`, `failed`, `autoplay 2/5 narrating`), markup used as a contract with client script, and an error passed into a re-rendered panel as template input.
- **Hazards to check (inferred, not tested):**
  - `http/worlds/handlers/worlds.rs:119-125`: after a successful create, a failed `list_worlds` renders an empty panel, "No worlds defined".
  - `http/games/handlers/games.rs:56-62`: a failed `list_personas` shows "No personas available".
  - `http/worlds/handlers/worlds.rs:157-195` and `259-277`: the details save and the posture autosave each read and write the whole World, so two requests close in time can overwrite each other.
  - `http/utils/response.rs:7-16`: `ok()` sets no `Content-Type`, but `Html(...)` does.
  - `http/utils/error.rs:24-31`: `error_disclosure` escapes `message` but not `detail_html`, so a new caller can miss the escape.
  - `http/chat_window/handlers/chat_window.rs:19` against `http/builders/router.rs:219-221`: `index.html` is embedded at compile time and the other assets are read from disk.
  - `http/builders/router.rs:220`: the `data/` folder is served over HTTP; its content is unknown.
- **Also noted (known):** `AGENTS.md` says "HTTP/WebSocket server", and the search of `src/` finds no WebSocket or SSE code (Q1 in the asset).

- **Check by the coordinator:** spot check of four citations (`http/error.rs:16`, `http/worlds/handlers/worlds.rs:254-256`, `http/utils/response.rs:7-16`, `http/builders/router.rs:220`) against the code. All four match. A case-insensitive search of `src/` for "websocket" finds no file, so Q1 holds.
