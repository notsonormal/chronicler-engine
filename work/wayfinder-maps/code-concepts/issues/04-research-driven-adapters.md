# Research: the concepts in the driven adapters

Type: research
Status: resolved

## Question

Which concepts does the code in `src/adapters/driven/` (storage, LLM providers and transport, text check; about 4,800 lines without the unit tests) follow, and where do opposing or overlapping concepts meet in it?

Write the asset in the shape that the map's Notes give (Concepts, Meeting points, Shapes with no name, Questions). Do not rule on a concept or label a meeting point; write it as a question.

Dig the concepts out of the code. You get no list of concepts: do not open `assets/research-01-lens.md` or the other research assets. Read the code, and find the shapes that it repeats and the choices that it makes. Give each one the name that software-engineering books and well-known essays give it (a principle, an architecture style, a design style, a pattern or an idiom), and its opposite. When no such name fits, put the shape under Shapes with no name.

## Done when

Every module in the scope is read, or is named in the asset as sampled with the reason. Every row cites `path:line` evidence and has a known, inferred or guessed label.

## Answer

Asset: [research-04-driven-adapters.md](../assets/research-04-driven-adapters.md). The researcher read all 44 non-test files in `src/adapters/driven/` (4,815 lines) and sampled nothing. The researcher did not open the other research assets. The researcher read no test files as evidence. Outside the area, the researcher opened only the two ports, `bootstrap/wiring.rs`, `error.rs` and `domain/model/settings.rs` to confirm claims. The researcher checked the 267 citations with a script and corrected 4 of them.

- **51 concepts.** The five with the strongest evidence (known): ports and adapters on the driven side, for the LLM provider and text check only (storage has no port); Data Mapper with `Db*` row structs and domain serde shapes stored in JSON columns; Unit of Work with a capability token, scoped to one Game (`storage/transaction.rs:32-53,280-328`); one coarse lock around one SQLite connection, and the held lock gives the atomicity (`storage/db.rs:12-14`); forward-only `PRAGMA user_version` migrations that can be re-run (`storage/utils/plumbing.rs:17-61`), two of which delete every Game.
- **18 meeting points, each written as a question.** Examples: the LLM provider has a `MockBackend` fake behind the port, shipped in the production tree and selectable by config, but the storage fake is a test layer inside the concrete `Storage`, and 9 application files import it; atomicity comes from three mechanisms (a closure under the lock, an explicit SQL transaction, the `Tx` unit), and the "name taken" rules use a `UNIQUE` constraint, a scan under the lock, or a count check; the domain serde shape is the stored format, so migrations v28 and v30 deleted all Games when the shape changed.
- **8 shapes with no name, 7 questions.** One question (MP-13): after a World is re-seeded, an edited Character file probably does not reach a database that already holds that Character (inferred).
- **Hazards to check (inferred, not tested):** `llm/transport/utils/response.rs:140-142` slices `&error_body[..500]` by byte, which can panic on a multi-byte character; `utils/harper.rs:25-40` maps a lint span that starts at the end of the text to byte 0; `response.rs:46-56` records an API error in an HTTP 200 body as `Http { status: 200 }`; `utils/mock.rs:78` builds the raw response JSON with `format!` and unescaped text; `storage/worlds.rs:314-334` writes the world and the map in two statements with no SQL transaction (the backend lock is held); `storage/transaction.rs:47-48` a `Storage` call inside a unit closure deadlocks, and only a comment prevents it; `llm/providers/deepseek.rs:25-30` DeepSeek can be configured and then fails on every call.
- **Check by the coordinator:** spot check of five citations (`response.rs:140-142`, `db.rs:12-14`, `transaction.rs:47-48`, `worlds.rs` `update_world`, `deepseek.rs:25-30`) against the code. All five match. The `update_world` range is the world write; the map write is at line 334.
