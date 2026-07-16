"""Generate `docs-diataxis/reference/http_routes.md` from `router.rs`.

Parses every `.route(<path>, <verb>(<handler>))` call in
`chronicler_engine/src/adapters/driving/http/router.rs`, groups routes by
handler-module prefix, and emits a single 7-area Reference doc.

Standalone Python 3.12 — stdlib only (mirrors `validate_docs_diataxis.py`).

The router's `.route()` calls are syntactically regular: a path string,
followed by a verb wrapper (`get(...)` or `post(...)`), followed by a fully
qualified handler reference. The pattern spans one to several lines per call,
so a balanced-paren walk (not a single regex) is used to extract each call as
a contiguous substring; from there the four fields (verb, path, handler)
are pulled with a small inner parser.

Output: writes the doc to
`chronicler_engine/docs-diataxis/reference/http_routes.md`.

Validator-clean: the doc satisfies every `validate_docs_diataxis.py` rule
that applies to a `diataxis: reference` doc.

Usage:
    python scripts/extract_http_routes.py
    python scripts/extract_http_routes.py --check     # exit 1 if doc is stale
    python scripts/extract_http_routes.py --stdout    # write doc to stdout
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

# ---------------------------------------------------------------------------
# Handler-module → area mapping.
# ---------------------------------------------------------------------------

# (handler_module_prefix, area_h2). Order matters — the script emits H2s in
# this sequence. A new handler-module prefix surfaces as a warning to the
# operator, who decides whether to add a new area or fold it into an existing
# one.
AREA_GROUPS: tuple[tuple[str, str], ...] = (
    ("handlers::", "Index"),
    ("fragments::", "Action / Status / History / Lifecycle"),
    ("games_fragment::", "Games"),
    ("worlds_fragment::", "Worlds"),
    ("settings_fragment::", "Settings & connections"),
    ("prompt_presets_fragment::", "Prompt presets"),
    ("debug::", "Debug"),
)

# Paths (relative to chronicler_engine/) the script must find.
ROUTER_REL = "src/adapters/driving/http/router.rs"
OUTPUT_REL = "docs-diataxis/reference/http_routes.md"


# ---------------------------------------------------------------------------
# Parsed route.
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Route:
    verb: str  # 'GET' or 'POST'
    path: str  # literal path, e.g. '/message/:id/swipe/:index'
    handler: str  # fully-qualified handler, e.g. 'fragments::action_handler'

    @property
    def module_prefix(self) -> str:
        """Module-prefix portion of the handler (before `::`)."""
        idx = self.handler.find("::")
        if idx == -1:
            return ""
        return self.handler[: idx + 2]


# ---------------------------------------------------------------------------
# Parser.
# ---------------------------------------------------------------------------


def _collect_use_imports(source: str) -> dict[str, str]:
    """Build a {local_name: qualified_path} map from `use ...;` lines.

    Only `use` imports of the shape `use super::module::name;` (single-segment
    trailing path) and `use super::module;` (whole-module import, mapped to
    `module::`) are honored — those are the shapes that appear in router.rs.
    `use super::module::{a, b, c};` (grouped) is not currently present and
    is left as a TODO if it ever shows up.

    Examples resolved:
        use super::handlers::index_handler;     -> {"index_handler": "handlers::index_handler"}
        use super::fragments;                    -> {"fragments": "fragments::"}
        use super::games_fragment;               -> {"games_fragment": "games_fragment::"}
    """
    imports: dict[str, str] = {}
    pattern = re.compile(
        r"^\s*use\s+super::([A-Za-z_][A-Za-z0-9_]*)"
        r"(?:::([A-Za-z_][A-Za-z0-9_]*))?\s*;"
    )
    for line in source.splitlines():
        m = pattern.match(line)
        if not m:
            continue
        module = m.group(1)
        name = m.group(2)
        if name is None:
            # Module import: local reference `module` -> `module::`.
            imports[module] = f"{module}::"
        else:
            imports[name] = f"{module}::{name}"
    return imports


def _find_outer_args(text: str, start: int) -> tuple[str, int] | None:
    """Walk text from `start` (the `(` of `.route(`) tracking paren depth.

    Returns (args_text, end_index) where end_index is the position of the
    matching `)` (exclusive after slicing). Returns None if parens don't
    balance.
    """
    if start >= len(text) or text[start] != "(":
        return None
    depth = 0
    for j in range(start, len(text)):
        c = text[j]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return text[start + 1 : j], j + 1
    return None


def _parse_route_args(args: str) -> Route | None:
    """Parse the inside of a `.route(...)` call into a Route.

    Expected shape:  "<path>", <verb>(<handler>)
    Verb is `get` or `post`; handler may contain `::` (module separator)
    and must not contain parens. Path is a double-quoted string; the path
    itself may contain `:` (axum path parameters) but not `"`.
    """
    path_match = re.match(r'\s*"([^"]*)"\s*,\s*', args)
    if not path_match:
        return None
    path = path_match.group(1)

    rest = args[path_match.end() :]
    verb_match = re.match(r"(get|post)\(", rest)
    if not verb_match:
        return None
    verb = verb_match.group(1).upper()

    # Walk from the `(` after the verb to find its matching `)`.
    paren_start = verb_match.end() - 1
    depth = 0
    for k in range(paren_start, len(rest)):
        c = rest[k]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                handler = rest[paren_start + 1 : k].strip()
                if not handler:
                    return None
                return Route(verb=verb, path=path, handler=handler)
    return None


def extract_routes(source: str) -> list[Route]:
    """Extract every `.route(...)` call from `source` (the router file body).

    Walks the file once, finding `.route(` and slicing out the balanced-args
    substring for each match. The args substring is then parsed for the
    verb / path / handler triple.

    Bare handler references (no `::`, e.g. `index_handler`) are normalized
    via the file's `use super::...` imports so the doc carries fully-qualified
    handler paths (e.g. `handlers::index_handler`) per the ticket spec.
    """
    imports = _collect_use_imports(source)
    routes: list[Route] = []
    needle = ".route("
    i = 0
    while i < len(source):
        idx = source.find(needle, i)
        if idx == -1:
            break
        paren_pos = idx + len(needle) - 1
        found = _find_outer_args(source, paren_pos)
        if found is None:
            # Malformed — skip past this match to avoid an infinite loop.
            i = idx + len(needle)
            continue
        args_text, end_pos = found
        route = _parse_route_args(args_text)
        if route is not None:
            handler = route.handler
            if "::" not in handler and handler in imports:
                handler = imports[handler]
                route = Route(verb=route.verb, path=route.path, handler=handler)
            routes.append(route)
        i = end_pos
    return routes


def group_routes_by_area(routes: list[Route]) -> dict[str, list[Route]]:
    """Group routes by their handler-module prefix → area name.

    Unknown prefixes land in a sentinel bucket "_unknown" so the operator
    sees them and decides whether to add a new area.
    """
    prefix_to_area: dict[str, str] = dict(AREA_GROUPS)
    grouped: dict[str, list[Route]] = {area: [] for _, area in AREA_GROUPS}
    grouped["_unknown"] = []
    for route in routes:
        prefix = route.module_prefix
        area = prefix_to_area.get(prefix)
        if area is None:
            grouped["_unknown"].append(route)
        else:
            grouped[area].append(route)
    return grouped


# ---------------------------------------------------------------------------
# Doc emitter.
# ---------------------------------------------------------------------------


_OVERVIEW_PROSE = (
    "This doc is generated from `src/adapters/driving/http/router.rs` and is "
    "the canonical map of HTTP route to handler for the engine's HTTP server. "
    "Re-run `python scripts/extract_http_routes.py` after any change to "
    "`router.rs`. The seven areas below match the seven handler-module "
    "prefixes discovered from the router file (one prefix per source-tree "
    "module under `src/adapters/driving/http/`). Static-asset behaviour — "
    "`.nest_service` for `/assets` and `/data`, plus the `fallback_service` "
    "for unmatched paths — lives in `router.rs` but is not enumerated here; "
    "the generator handles `.route()` calls only."
)


def _render_table(routes: list[Route]) -> str:
    """Render one per-area table. Routes are emitted in input order (source order)."""
    if not routes:
        return ""
    lines = [
        "| Method | Path | Handler |",
        "|--------|------|---------|",
    ]
    for r in routes:
        lines.append(f"| {r.verb} | `{r.path}` | `{r.handler}` |")
    return "\n".join(lines)


def _render_document(
    routes: list[Route],
    grouped: dict[str, list[Route]],
) -> str:
    """Render the full markdown document."""
    unknown = grouped.pop("_unknown", [])

    parts: list[str] = []
    parts.append("---")
    parts.append("diataxis: reference")
    parts.append("title: HTTP Routes")
    parts.append("---")
    parts.append("")
    parts.append(
        "> **Diátaxis mode:** Reference. The reader problem this solves is "
        "*look-up*: wiring a client, testing an endpoint, or debugging a "
        "routing issue. The doc is a map of path to handler; it is not a "
        "tutorial on axum and not the LLM-call forensics workflow (see "
        "`./llm_processing.md`)."
    )
    parts.append("")
    parts.append("## Overview")
    parts.append("")
    parts.append(_OVERVIEW_PROSE)
    parts.append("")

    for _prefix, area in AREA_GROUPS:
        rows = grouped.get(area, [])
        if not rows:
            # No routes in this area; still emit the H2 + a one-line note so
            # the seven-area shape stays stable if a future router change
            # empties an area.
            parts.append(f"## {area}")
            parts.append("")
            parts.append(
                "_No routes in this area in the current `router.rs`._"
            )
            parts.append("")
            continue
        parts.append(f"## {area}")
        parts.append("")
        parts.append(_render_table(rows))
        parts.append("")

    if unknown:
        parts.append("## Unknown areas")
        parts.append("")
        parts.append(
            "The following routes have handler-module prefixes not listed "
            "in the script's `AREA_GROUPS`. Add a new area (or fold them "
            "into an existing one) in "
            "`chronicler_engine/scripts/extract_http_routes.py`:"
        )
        parts.append("")
        parts.append(_render_table(unknown))
        parts.append("")

    parts.append("## Document References")
    parts.append("")
    parts.append(
        "- [`src/adapters/driving/http/router.rs`](../../src/adapters/driving/http/router.rs) "
        "— the canonical route table; this doc is generated from it."
    )
    parts.append(
        "- [`docs-diataxis/AGENTS.md`](../../docs-diataxis/AGENTS.md) "
        "— writing conventions the generator applies (seam identifiers, "
        "no mechanics leaks)."
    )
    parts.append(
        "- [`chronicler_engine/scripts/extract_http_routes.py`](../../chronicler_engine/scripts/extract_http_routes.py) "
        "— the generator."
    )
    parts.append("")

    return "\n".join(parts)


def write_document(engine_root: Path, text: str) -> Path:
    """Write the rendered document to disk; return the path written."""
    out_path = engine_root / OUTPUT_REL
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(text, encoding="utf-8")
    return out_path


# ---------------------------------------------------------------------------
# CLI.
# ---------------------------------------------------------------------------


def _parser(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Extract .route(...) calls from router.rs and emit "
            "docs-diataxis/reference/http_routes.md."
        ),
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help=(
            "Exit 1 if the on-disk doc does not match what would be "
            "generated; otherwise exit 0. Does not modify files."
        ),
    )
    parser.add_argument(
        "--stdout",
        action="store_true",
        help="Write the rendered doc to stdout instead of a file.",
    )
    parser.add_argument(
        "--router",
        type=Path,
        default=None,
        help="Override the router.rs path (relative to engine root or absolute).",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Override the output path (relative to engine root or absolute).",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parser(argv)
    engine_root = Path(__file__).resolve().parent.parent

    router_path = (
        args.router.resolve()
        if args.router
        else engine_root / ROUTER_REL
    )
    if not router_path.exists():
        print(f"Error: router file not found: {router_path}", file=sys.stderr)
        return 2

    source = router_path.read_text(encoding="utf-8")
    routes = extract_routes(source)
    if not routes:
        print(
            f"Error: parsed zero routes from {router_path}; "
            "parser may be broken.",
            file=sys.stderr,
        )
        return 2
    grouped = group_routes_by_area(routes)
    rendered = _render_document(routes, grouped)

    if args.stdout:
        sys.stdout.write(rendered)
        return 0

    if args.check:
        existing = (
            args.output.resolve()
            if args.output
            else engine_root / OUTPUT_REL
        )
        if not existing.exists():
            print(
                f"Error: --check: output file does not exist: {existing}",
                file=sys.stderr,
            )
            return 1
        on_disk = existing.read_text(encoding="utf-8")
        if on_disk != rendered:
            print(
                f"Error: --check: {existing} is stale; "
                "regenerate with `python scripts/extract_http_routes.py`.",
                file=sys.stderr,
            )
            return 1
        return 0

    output_path = (
        args.output.resolve()
        if args.output
        else engine_root / OUTPUT_REL
    )
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(rendered, encoding="utf-8")
    print(f"Wrote {output_path} ({len(routes)} routes).")
    return 0


if __name__ == "__main__":
    sys.exit(main())