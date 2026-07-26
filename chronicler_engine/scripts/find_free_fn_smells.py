"""Find module-level free functions whose first parameter looks like a receiver.

Heuristic: skip anything inside `impl` blocks (inherent or trait). For each top-level
`pub fn NAME(...)`, classify the first parameter. A first param whose bare type identifier
starts with an uppercase letter and is not a stdlib value wrapper is flagged as a likely
"method pretending to be a free function" smell.

Pure / not-smell categories:
  - No parameters
  - First param is a primitive or &str / &[T] / &Path / String / Path / PathBuf
  - First param is a lowercase generic type parameter (`T`, `T: Bound`)
  - Constructor: returns `Self`

Output: grouped by category (SMELL first), then by file. Each row shows file, line,
function name, first param, and bare type.

Usage:
  python scripts/find_free_fn_smells.py [ROOT]
  ROOT defaults to the chronicler_engine directory (scans both src/ and tests/).
  Pass any path to override; pass multiple paths to scan more than one root.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

SUPPRESSED_FREE_FNS: frozenset[tuple[str, str]] = frozenset(
    {
        ("adapters/driven/storage/mappers/message.rs", "model_swipes_to_db"),
        ("adapters/driven/storage/mappers/state_snapshot.rs", "snapshot_to_db"),
        ("bootstrap/load.rs", "seed_game_data"),
        ("test_support/context.rs", "seed_test_world_into_storage"),
        ("utils/settings.rs", "load_settings"),
        ("utils/settings.rs", "save_settings"),
        ("bootstrap/validate.rs", "validate_loaded_data"),
        ("domain/engine/action_processing.rs", "execute_freeaction_impl"),
        ("domain/engine/logic.rs", "attempt_semantic_walk"),
        ("domain/engine/state_diagnostics.rs", "assert_state_consistency"),
        ("domain/engine/trigger_eval.rs", "evaluate_triggers"),
        ("application/agents/quantifier/utils/orchestration.rs", "determine_npcs_in_room"),
        ("application/scenario.rs", "inject_scenario_logs"),
        ("adapters/driving/http/utils/fragment.rs", "render_fragment"),
        ("adapters/driving/http/utils/view_mappers.rs", "games_per_world"),
        ("adapters/driving/http/utils/locks.rs", "read_lock_or_recover"),
        ("adapters/driving/http/utils/locks.rs", "write_lock_or_recover"),
        ("application/utils/slot.rs", "release_owned_slot"),
    }
)

# Standard-library / wrapper types whose uppercase identifiers are NOT receiver smells.
STDLIB_WRAPPER_TYPES: frozenset[str] = frozenset(
    {
        "String",
        "Vec",
        "HashMap",
        "HashSet",
        "BTreeMap",
        "BTreeSet",
        "Option",
        "Result",
        "Box",
        "Rc",
        "Arc",
        "Weak",
        "Cow",
        "Cell",
        "RefCell",
        "Mutex",
        "RwLock",
        "OnceLock",
        "OnceCell",
        "Path",
        "PathBuf",
        "OsString",
        "OsStr",
        "CString",
        "CStr",
        "Duration",
        "SystemTime",
        "Instant",
        "IpAddr",
        "SocketAddr",
        "Error",
        "Formatter",
        "Args",
        "Command",
        "Child",
        "Stdio",
        "Sender",
        "Receiver",
        "JoinHandle",
        "Task",
        "Stream",
        "Sink",
        "Future",
        "Pin",
        "Context",
        "Waker",
    }
)

PRIMITIVE_TYPES: frozenset[str] = frozenset(
    {
        "bool",
        "char",
        "str",
        "u8",
        "u16",
        "u32",
        "u64",
        "u128",
        "usize",
        "i8",
        "i16",
        "i32",
        "i64",
        "i128",
        "isize",
        "f32",
        "f64",
        "Self",
    }
)


@dataclass(frozen=True)
class FreeFn:
    file: Path
    line: int
    name: str
    first_param_raw: str
    bare_type: str
    category: str


def is_suppressed(path: Path, name: str) -> bool:
    """Match suppression entries against paths relative to `src/`."""
    parts = path.parts
    relative = Path(*parts[parts.index("src") + 1 :]) if "src" in parts else path
    return (relative.as_posix(), name) in SUPPRESSED_FREE_FNS


def strip_string_and_comment_chars(source: str) -> str:
    """Replace the contents of string/char literals and line/block comments with spaces.

    Keeps brace counting honest: a `{` inside a string or comment must not push us into a
    fake impl block.
    """
    out: list[str] = []
    i = 0
    n = len(source)
    while i < n:
        c = source[i]
        # Line comment
        if c == "/" and i + 1 < n and source[i + 1] == "/":
            j = source.find("\n", i)
            if j == -1:
                j = n
            out.append(" " * (j - i))
            i = j
            continue
        # Block comment (nestable in Rust)
        if c == "/" and i + 1 < n and source[i + 1] == "*":
            depth = 1
            i += 2
            while i < n and depth > 0:
                if i + 1 < n and source[i] == "/" and source[i + 1] == "*":
                    depth += 1
                    i += 2
                elif i + 1 < n and source[i] == "*" and source[i + 1] == "/":
                    depth -= 1
                    i += 2
                else:
                    out.append(" ")
                    i += 1
            continue
        # Lifetime ('a, 'static) vs char literal ('x', '\n')
        if c == "'":
            # Lookahead: if next char is a letter or underscore, this is a LIFETIME, not a
            # char literal. Rust char literals are never followed by an identifier start.
            if i + 1 < n and (source[i + 1].isalpha() or source[i + 1] == "_"):
                out.append("'")
                i += 1
                # Consume identifier chars; do NOT eat a closing quote.
                while i < n and (source[i].isalnum() or source[i] == "_"):
                    out.append(source[i])
                    i += 1
                continue
            # Otherwise treat as a (possibly escaped) char literal.
            out.append(c)
            i += 1
            if i < n and source[i] == "\\":
                out.append(" ")
                i += 1
                if i < n:
                    out.append(" ")
                    i += 1
            while i < n and source[i] != "'":
                out.append(" ")
                i += 1
            if i < n:
                out.append(source[i])
                i += 1
            continue
        # String / byte / C-string literal
        if c == '"':
            out.append(c)
            i += 1
            while i < n and source[i] != '"':
                if source[i] == "\\" and i + 1 < n:
                    out.append(" ")
                    i += 1
                    out.append(" ")
                    i += 1
                    continue
                out.append(" ")
                i += 1
            if i < n:
                out.append(source[i])
                i += 1
            continue
        # Raw string r#"..."#
        if c == "r" and i + 1 < n and source[i + 1] in {'"', "#"}:
            j = i + 1
            hashes = 0
            while j < n and source[j] == "#":
                hashes += 1
                j += 1
            if j < n and source[j] == '"':
                out.append(" " * (j + 1 - i))
                i = j + 1
                closing = '"' + "#" * hashes
                end = source.find(closing, i)
                if end == -1:
                    i = n
                else:
                    out.append(" " * (end + len(closing) - i))
                    i = end + len(closing)
                continue
        out.append(c)
        i += 1
    return "".join(out)


def find_top_level_pub_fns(source: str) -> list[tuple[int, str]]:
    """Return [(line_no_1indexed, full_signature)] for module-level pub fns only.

    Skips pub fns inside any `impl ... { ... }` block by tracking brace depth and an
    `impl_depth` counter.
    """
    masked = strip_string_and_comment_chars(source)
    lines = masked.split("\n")
    results: list[tuple[int, str]] = []
    # `pending_impl` is set when we see `impl ...` at brace_depth 0 and is consumed
    # when the matching `{` opens.
    pending_impl = False
    brace_depth = 0
    # Stack of bool: was each open brace an impl-block opener?
    scope_is_impl: list[bool] = []
    impl_depth = 0

    head_re = re.compile(r"^\s*pub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")
    impl_re = re.compile(r"^\s*impl\b")

    line_starts: list[int] = []
    pos = 0
    for line in lines:
        line_starts.append(pos)
        pos += len(line) + 1

    for line_no, line in enumerate(lines, start=1):
        # 1. Detect `impl` header at brace depth 0 — queue it for the opening `{`.
        if brace_depth == 0 and impl_re.match(line):
            pending_impl = True
        m = head_re.search(line)
        # 2. Capture whether THIS line's pub fn is inside an impl block — evaluated
        #    BEFORE we process the body braces, so the `pub fn` is attributed to the
        #    scope it was declared in (not the scope its body opens).
        in_impl_now = impl_depth > 0
        # 3. Walk braces on this line, maintaining `impl_depth` as count of currently
        #    open impl scopes (derived from the stack).
        for ch in line:
            if ch == "{":
                scope_is_impl.append(pending_impl)
                if pending_impl:
                    impl_depth += 1
                pending_impl = False
                brace_depth += 1
            elif ch == "}":
                if brace_depth > 0:
                    was_impl = scope_is_impl.pop() if scope_is_impl else False
                    if was_impl:
                        impl_depth -= 1
                brace_depth = max(brace_depth - 1, 0)
        if m is not None and not in_impl_now:
            # Found a top-level pub fn. Collect the full signature starting at the
            # function name's first character. Walk forward, skipping generics `<...>`,
            # until we find the `(...)` param list, then take it.
            abs_start = line_starts[line_no - 1] + m.start()
            i = abs_start
            # Skip until we are past any generics `< ... >`
            # m already matched up to and including the name; m.end() points at `<` or
            # `(` (the `[<(]` group). Walk through generics if present.
            # Advance past whitespace
            while i < len(masked) and masked[i].isspace():
                i += 1
            angle_depth = 0
            paren_depth = 0
            while i < len(masked):
                ch = masked[i]
                if ch == "<":
                    angle_depth += 1
                elif ch == ">":
                    angle_depth -= 1
                elif ch == "(" and angle_depth == 0:
                    paren_start = i
                    paren_depth = 1
                    i += 1
                    while i < len(masked) and paren_depth > 0:
                        ch = masked[i]
                        if ch == "(":
                            paren_depth += 1
                        elif ch == ")":
                            paren_depth -= 1
                        i += 1
                    sig = masked[paren_start : i]
                    results.append((line_no, sig))
                    break
                elif ch.isspace():
                    # Between generics and parens — keep walking
                    pass
                i += 1
    return results


def split_params(sig_body: str) -> list[str]:
    """Split the parameter list body on top-level commas."""
    out: list[str] = []
    depth = 0
    buf: list[str] = []
    for ch in sig_body:
        if ch == "," and depth == 0:
            out.append("".join(buf).strip())
            buf = []
            continue
        if ch in "([{<":
            depth += 1
        elif ch in ")]}>":
            depth -= 1
        buf.append(ch)
    tail = "".join(buf).strip()
    if tail:
        out.append(tail)
    return out


def is_borrowed_param(param: str) -> bool:
    """True if the first param type starts with `&` (or `&mut` / `&'a`)."""
    p = param.strip()
    if p.startswith("mut "):
        p = p[4:].strip()
    if p == "self" or p.startswith("self:"):
        return False
    colon = p.find(":")
    type_part = p[colon + 1 :].strip() if colon != -1 else p
    return type_part.startswith("&")


def extract_bare_type(param: str) -> str:
    """Reduce a parameter declaration to its bare type identifier for smell testing.

    Example: `&mut self` -> `Self`; `app: &DefaultApplicationService` ->
    `DefaultApplicationService`; `t: T` -> `T`; `text: &str` -> `str`.
    """
    # Drop leading `mut`
    p = param.strip()
    if p.startswith("mut "):
        p = p[4:].strip()
    # Drop `self` -> `Self`
    if p == "self" or p == "&self" or p == "&mut self" or p == "mut self":
        return "Self"
    # Find the colon that separates name from type
    colon = p.find(":")
    if colon == -1:
        # Pattern form: just the type
        type_part = p
    else:
        type_part = p[colon + 1 :].strip()
    # Strip outer `&` / `&mut` / `&'a mut` / `&'a` but DO NOT eat the type name itself.
    # `&reqwest::blocking::Client` should leave the type untouched (only `&` removed),
    # not eat `reqwest` along with the `&`.
    type_part = re.sub(r"^\s*&('?\w+\s+)?\s*mut\s+", "", type_part)
    type_part = re.sub(r"^\s*&('?\w+\s+)?", "", type_part)
    # Strip leading `Arc<...>`, `Option<&...>` etc. to get the innermost receiver type.
    for _ in range(6):
        m = re.match(
            r"^\s*(?:Arc|Rc|Box|Cow|Option|Result|Vec|HashMap|HashSet|BTreeMap|"
            r"BTreeSet|RefCell|Mutex|RwLock|OnceLock|OnceCell|Pin)\s*<",
            type_part,
        )
        if not m:
            break
        # m.end() points at the char after the wrapper's `<`. Walk to the matching
        # closing `>` (counting nested generics).
        depth = 0
        i = m.end() - 1  # at the `<` itself
        while i < len(type_part):
            if type_part[i] == "<":
                depth += 1
            elif type_part[i] == ">":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        # Keep the content INSIDE the wrapper, not what's after the `>`.
        type_part = type_part[m.end() : i].strip()
    # Strip `&` if it survived (e.g. from `Option<&T>`)
    type_part = re.sub(r"^\s*&('?\w+\s+)?\s*mut\s+", "", type_part)
    type_part = re.sub(r"^\s*&('?\w+\s+)?", "", type_part)
    # Strip leading generic wrappers to get the bare identifier
    bare = type_part
    # The remaining leading identifier (could be a generic param like T or T: Bound)
    m = re.match(r"\s*([A-Za-z_][A-Za-z0-9_]*)", bare)
    if not m:
        return ""
    return m.group(1)


def classify(first_param_raw: str, bare_type: str, returns_self: bool, is_borrowed: bool) -> str:
    if not first_param_raw:
        return "NO_PARAMS"
    if returns_self:
        return "CONSTRUCTOR"
    # Complex wrapper containing an uppercase identifier (e.g. `Arc<RwLock<HashMap<...>>>`)
    # is itself a receiver-like type — flag as SMELL when borrowed, CONSTRUCTOR when owned.
    raw = first_param_raw.split(":", 1)[1].strip() if ":" in first_param_raw else first_param_raw
    has_uppercase_in_wrapper = bool(re.search(r"[A-Z]", raw)) and bool(re.search(r"<", raw))
    if bare_type in PRIMITIVE_TYPES:
        if has_uppercase_in_wrapper:
            return "SMELL" if is_borrowed else "CONSTRUCTOR"
        return "PURE"
    if bare_type in STDLIB_WRAPPER_TYPES:
        return "PURE"
    if bare_type == "":
        return "UNKNOWN"
    # Lowercase first letter usually means a generic type parameter
    if bare_type[0].islower():
        return "PURE"
    # Uppercase, not in our denylist, not primitive.
    # BORROWED (`&T` / `&mut T`) on a domain type → smell (should be a method).
    # OWNED (`T` / `Arc<T>`) on a domain type → factory/constructor, fine as free fn.
    if is_borrowed:
        return "SMELL"
    return "CONSTRUCTOR"


def returns_self(sig: str) -> bool:
    """Detect `-> Self`, `-> Self<T>`, `-> &'a Self`, etc."""
    arrow = sig.find("->")
    if arrow == -1:
        return False
    tail = sig[arrow + 2 :].strip()
    # Strip leading `&` / `&'a` / `&mut` etc
    tail = re.sub(r"^\s*&(\s*'?[a-zA-Z_]\w*\s*)?\s*mut\s+", "", tail)
    tail = re.sub(r"^\s*&(\s*'?[a-zA-Z_]\w*\s*)?\s*", "", tail)
    m = re.match(r"\s*Self\b", tail)
    return m is not None


def scan(root: Path) -> list[FreeFn]:
    findings: list[FreeFn] = []
    for path in sorted(root.rglob("*.rs")):
        # Skip target/ and similar build output if scanning from a crate root
        if any(part == "target" for part in path.parts):
            continue
        text = path.read_text(encoding="utf-8")
        for line_no, sig in find_top_level_pub_fns(text):
            # sig is the parenthesised param list "(...)"
            assert sig.startswith("(") and sig.endswith(")")
            body = sig[1:-1]
            params = split_params(body)
            first = params[0] if params else ""
            bare = extract_bare_type(first) if first else ""
            cat = classify(first, bare, returns_self(sig), is_borrowed_param(first))
            findings.append(
                FreeFn(
                    file=path,
                    line=line_no,
                    name=sig_line_name(text, line_no),
                    first_param_raw=first,
                    bare_type=bare,
                    category=classify(first, bare, returns_self(sig), is_borrowed_param(first)),
                )
            )
    return findings


def sig_line_name(text: str, line_no: int) -> str:
    """Recover the function name from the source line at `line_no`."""
    line = text.split("\n")[line_no - 1]
    m = re.search(r"\bpub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)", line)
    return m.group(1) if m else "?"


def render(findings: list[FreeFn]) -> str:
    smells = [f for f in findings if f.category == "SMELL"]
    pure = [f for f in findings if f.category in {"PURE", "CONSTRUCTOR", "NO_PARAMS"}]
    other = [f for f in findings if f.category not in {"SMELL", "PURE", "CONSTRUCTOR", "NO_PARAMS"}]

    def short(path: Path) -> str:
        try:
            return str(path.relative_to(Path.cwd()))
        except ValueError:
            return str(path)

    def fmt_row(f: FreeFn) -> str:
        raw = f.first_param_raw if len(f.first_param_raw) < 60 else f.first_param_raw[:57] + "..."
        return f"  {short(f.file)}:{f.line}  {f.name}  type={f.bare_type}  param={raw}"

    def group_by_file(items: list[FreeFn]) -> list[tuple[str, list[FreeFn]]]:
        grouped: dict[str, list[FreeFn]] = {}
        for f in items:
            grouped.setdefault(short(f.file), []).append(f)
        return sorted(grouped.items())

    def emit(title: str, items: list[FreeFn]) -> list[str]:
        if not items:
            return []
        out: list[str] = []
        out.append("=" * 80)
        out.append(f"{title}  ({len(items)})")
        out.append("=" * 80)
        for path, group in group_by_file(items):
            out.append(f"{path}  ({len(group)})")
            for f in group:
                out.append(fmt_row(f))
            out.append("")
        return out

    lines: list[str] = []
    lines.append(f"Total module-level pub fns: {len(findings)}")
    lines.append(f"  SMELL (receiver-like first arg): {len(smells)}")
    lines.append(f"  PURE / CONSTRUCTOR / NO_PARAMS: {len(pure)}")
    lines.append(f"  UNKNOWN: {len(other)}")
    lines.append("")
    lines.extend(
        emit("SMELLS (first arg is a domain type — likely should be a method)", smells)
    )
    lines.extend(emit("OTHER (uncertain / unknown)", other))
    lines.extend(emit("PURE / CONSTRUCTOR / NO_PARAMS (kept as free fn, NOT flagged)", pure))
    return "\n".join(lines)


def main(argv: list[str]) -> int:
    if len(argv) > 1:
        roots = [Path(a).resolve() for a in argv[1:]]
    else:
        crate_root = Path(__file__).resolve().parent.parent
        roots = [
            (crate_root / "src").resolve(),
            (crate_root / "tests").resolve(),
        ]
    roots = [r for r in roots if r.exists()]
    if not roots:
        print("error: no valid roots to scan", file=sys.stderr)
        return 2
    findings: list[FreeFn] = []
    for root in roots:
        findings.extend(scan(root))
    findings.sort(key=lambda f: (str(f.file), f.line))
    findings = [
        finding
        for finding in findings
        if finding.category != "SMELL" or not is_suppressed(finding.file, finding.name)
    ]
    print(render(findings))
    return 1 if any(finding.category == "SMELL" for finding in findings) else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))