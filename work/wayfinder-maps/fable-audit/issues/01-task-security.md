# Close the key leak and the links in the story log

Type: task
Status: open
Blocked by: —

## Question

Put three changes in place (owner, 2026-11-10, D6 and D17 of [the decisions](../decisions.md)). The listen address stays `0.0.0.0`.

### 1. A request from another site is refused (D6)

- **The fault (ran it).** The server has no login. No route checks the `Origin` or the `Host` header. So a web page that the player opens in the same browser can send a form to any route.
- **The change.** One route layer beside `refuse_stale_game` (`src/adapters/driving/http/utils/request_layers.rs:39`). It refuses a request that changes state when `Origin` or `Sec-Fetch-Site` shows another site. A request with neither header passes, because `curl` and the tests send none.
- **The `Host` header.** The report also recommends to refuse a `Host` that is not an address of the server. The owner accepted "a route layer that refuses a request from another site". Decide in the ticket whether the `Host` check is needed for that, and write the reason in `## Answer`. The container case must still work: the browser reaches the server through a mapped port.
- **The failing test first.** `POST /connections/test` with `Origin: http://evil.example` gets a refusal, and no provider call is made.

### 2. The key of the environment goes only to the default address of its provider (D6)

- **The fault (ran it).** `POST /connections/test` takes a Connection from form fields and calls it at once (`test_form_connection_handler`, `settings/handlers/settings.rs:364`). With no key in the form, `LlmConnection::resolve_api_key` (`src/domain/model/settings.rs:399`) reads the key of the environment. So the form can send `Authorization: Bearer <key>` to any `conn_base_url`.
- **The change.** `resolve_api_key` gives the key of the environment only when the Connection has the default address of its provider. A Connection with another address needs its own key.
- **The failing test first.** A Connection with provider OpenRouter, no key and a base address that is not the OpenRouter address resolves to no key.

### 3. The story log renders no `javascript:` link and no remote picture (D17)

- **The fault (ran it).** `sanitize_markdown_event` (`src/adapters/driving/http/utils/view_models.rs:14`) turns raw HTML into text, but it passes links and pictures. Model text gave `<a href="javascript:…">` and `<img src="http://…">` in the story log. The same renderer shows the opening text of a World.
- **The change.** Keep a link only when its address starts with `http://` or `https://`. Drop a picture, or keep it only for an address under `/data/`.
- **The failing test first.** Two cases of `markdown_to_html`: one with each of the two inputs above.

## Rules for every change

- Write the failing test first. Then make the change.
- Line numbers are from commit `e8d6512a`. Find the site again by its name before you edit.
- One change that turns out not to be local stops. Say so in `## Answer`, and do the other changes.
- Use `python build.py clippy` and `python build.py test-pattern` during the work. Run `python build.py` once at the end.
- Do not commit.

## Not in scope

- The listen address (D2: waits for a release).
- A stored key in the HTML of the Connection edit form, and `GET /data/settings.json`. The report names both. No decision accepted them.
- A `Content-Security-Policy` header.

## What already exists

- Findings 1 and 5 of [the engine report](../assets/engine-defects-and-risks.md), each with its probe.
- The spec of the Settings routes: `docs/specs/settings.md`. Add the scenarios of changes 1 and 2 there.
