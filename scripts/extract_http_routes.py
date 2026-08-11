"""Generate `docs/diataxis/reference/frontend/http_routes.md` from `router.rs`.

Covers `.route()` calls only; `.nest_service` / `fallback_service` are static
mounts and are not enumerated.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

# (handler_module_prefix, area_h2). Order matters — the script emits H2s in
# this sequence. A new handler-module prefix surfaces as a warning to the
# operator, who decides whether to add a new area or fold it into an existing
# one.
AREA_GROUPS: tuple[tuple[str, str], ...] = (
    ("chat_window::", "Core"),
    ("debug::", "Core"),
    ("action::", "Action"),
    ("history::", "History"),
    ("layout::", "Layout fragments"),
    ("games::", "Games"),
    ("worlds::", "Worlds"),
    ("settings::", "Settings & connections"),
    ("prompt_presets::", "Prompt presets"),
)

# Paths (relative to repo root) the script must find.
ROUTER_REL = "src/adapters/driving/http/builders/router.rs"
OUTPUT_REL = "docs/diataxis/reference/frontend/http_routes.md"



@dataclass(frozen=True)
class Route:
    verb: str  # 'GET' or 'POST'
    path: str  # literal path, e.g. '/message/:id/swipe/:index'
    handler: str  # fully-qualified handler, e.g. 'fragments::action_handler'

    @property
    def module_prefix(self) -> str:
        idx = self.handler.find("::")
        if idx == -1:
            return ""
        return self.handler[: idx + 2]



def _collect_use_imports(source: str) -> dict[str, str]:
    # Accepts both `use super::...` and `use crate::...` paths. Router source
    # uses fully-qualified `use crate::...`; handler files use `use super::...`.
    # Intermediate `crate::` segments are discarded because router imports
    # all start at `crate::adapters::driving::http::`.
    # ponytail: grouped `use ...::{a, b, c};` not supported; add if it appears.
    imports: dict[str, str] = {}
    pattern = re.compile(
        r"^\s*use\s+(?:super|crate)::(?P<path>[A-Za-z_][A-Za-z0-9_]*"
        r"(?:::[A-Za-z_][A-Za-z0-9_]*)*)\s*;"
    )
    for line in source.splitlines():
        m = pattern.match(line)
        if not m:
            continue
        parts = m.group("path").split("::")
        if len(parts) == 1:
            # Module import: local reference `module` -> `module::`.
            module = parts[0]
            imports[module] = f"{module}::"
        else:
            module = parts[-2]
            name = parts[-1]
            imports[name] = f"{module}::{name}"
    return imports


def _find_outer_args(text: str, start: int) -> tuple[str, int] | None:
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
    # Expected shape:  "<path>", <verb>(<handler>)
    # Verb is `get` or `post`; handler may contain `::` (module separator)
    # and must not contain parens. Path is a double-quoted string; the path
    # itself may contain `:` (axum path parameters) but not `"`.
    path_match = re.match(r'\s*"([^"]*)"\s*,\s*', args)
    if not path_match:
        return None
    path = path_match.group(1)

    rest = args[path_match.end() :]
    verb_match = re.match(r"(get|post)\(", rest)
    if not verb_match:
        return None
    verb = verb_match.group(1).upper()

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
    # Unknown prefixes land in a sentinel bucket "_unknown" so the operator
    # sees them and decides whether to add a new area.
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



_OVERVIEW_PROSE = (
    "Generated from `src/adapters/driving/http/builders/router.rs` — "
    "re-run `python scripts/extract_http_routes.py` after any `router.rs` change. "
    "Covers `.route()` calls only; `.nest_service` (assets/data) and "
    "`fallback_service` live in `router.rs` but aren't enumerated."
)


def _strip_module_prefix(handler: str) -> str:
    return handler.rsplit("::", 1)[-1]


def _render_table(routes: list[Route]) -> str:
    lines = [
        "| Method | Path | Handler |",
        "|--------|------|---------|",
    ]
    for r in routes:
        lines.append(f"| {r.verb} | `{r.path}` | `{_strip_module_prefix(r.handler)}` |")
    return "\n".join(lines)


def _render_document(grouped: dict[str, list[Route]]) -> str:
    unknown = grouped.pop("_unknown", [])

    parts: list[str] = []
    parts.append("---")
    parts.append("diataxis: reference")
    parts.append("title: HTTP Routes")
    parts.append("---")
    parts.append("")
    parts.append("## Overview")
    parts.append("")
    parts.append(_OVERVIEW_PROSE)
    parts.append("")

    rendered_areas: set[str] = set()
    for _prefix, area in AREA_GROUPS:
        if area in rendered_areas:
            continue
        rendered_areas.add(area)
        rows = grouped.get(area, [])
        if not rows:
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
            "`scripts/extract_http_routes.py`:"
        )
        parts.append("")
        parts.append(_render_table(unknown))
        parts.append("")

    return "\n".join(parts)


def write_document(engine_root: Path, text: str) -> Path:
    out_path = engine_root / OUTPUT_REL
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(text, encoding="utf-8")
    return out_path



def _parser(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Extract .route(...) calls from router.rs and emit "
            "docs/diataxis/reference/frontend/http_routes.md."
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
    rendered = _render_document(grouped)

    output_path = (
        args.output.resolve()
        if args.output
        else engine_root / OUTPUT_REL
    )

    if args.stdout:
        sys.stdout.write(rendered)
        return 0

    if args.check:
        if not output_path.exists():
            print(
                f"Error: --check: output file does not exist: {output_path}",
                file=sys.stderr,
            )
            return 1
        on_disk = output_path.read_text(encoding="utf-8")
        if on_disk != rendered:
            print(
                f"Error: --check: {output_path} is stale; "
                "regenerate with `python scripts/extract_http_routes.py`.",
                file=sys.stderr,
            )
            return 1
        return 0

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(rendered, encoding="utf-8")
    print(f"Wrote {output_path} ({len(routes)} routes).")
    return 0


if __name__ == "__main__":
    sys.exit(main())