"""Check the TLA+ model of the gate with TLC: every config in ``model/``, one after the other.

Usage: ``python3 .agents/skills/verify-gate/check.py [CONFIG ...]`` (default: every config),
``python3 .agents/skills/verify-gate/check.py --mutants [NAME ...]`` (default: every mutant), or
``python3 .agents/skills/verify-gate/check.py --diagrams``.

TLC and the JDK come from ``mise``, with the pins of ``mise.toml`` in this folder. The first run
installs them and needs the network. Each config prints one line: ``holds``, ``breaks: RULE ...``
or ``error``. TLC stops at the first rule that breaks. ``coverage.toml`` lists the rules that
break today (``known_breaks``). When a config breaks a known rule, the config runs again without
that rule, so that a second break cannot hide behind the first. The exit code is 0 when each
config breaks exactly its known rules, 1 when a rule breaks that is not known or a known break no
longer breaks, and 2 on a TLC error. The TLC logs go to
``tmp/verify-gate/<config>.out`` in the repo. For a rule that breaks and is not known, the
``/statechart`` skill draws the path to the break in ``tmp/verify-gate/paths/<config>/``.

A run of every config ends with the diagrams of ``diagrams/diagrams.toml`` (``SKILL.md``, *The
diagrams*). ``--diagrams`` writes them in every case, and checks no config.

``--mutants`` runs the mutants of ``model/mutants.toml``: each one is a copy of the model with one
text replaced, so that it mirrors one break of the code. A mutant passes when TLC breaks each rule
that it names (a ``fix`` mutant passes when its config holds). Because TLC stops at the first
break, a mutant names the rules that can break first, and it passes when one of them breaks. Its
``without`` list takes rules out of its config, so that a rule behind another rule can show. The exit code is 1 when a mutant
does not pass. The copies go to ``tmp/verify-gate/mutants/<name>/``.

Do not run it while a cargo build runs: memory is the limit of this host.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
import time
import tomllib
from pathlib import Path

SKILL = Path(__file__).resolve().parent
MODEL = SKILL / "model"
REPO = SKILL.parents[2]
OUT = REPO / "tmp" / "verify-gate"
SPEC = "VerifyGate.tla"
HEAP = "-Xmx2g"
DIAGRAMS = SKILL / "diagrams"
MODEL_WORDS = (
    "The TLA+ model of the gate (`.agents/skills/verify-gate/model/`) describes how one holder at a time"
    " writes the records of a Game. A holder is a Job or an Autoplay run."
)

sys.path.insert(0, str(REPO / "scripts"))
import verify_coverage  # noqa: E402

STATECHART = verify_coverage.statechart_scripts(SKILL.parent) / "statechart"

_BROKEN = re.compile(r"^Error: Invariant (\w+) is violated", re.M)
_BROKEN_PROPERTY = re.compile(r"^Error: Action property (\w+) is violated", re.M)
_DISTINCT = re.compile(r"([\d,]+) distinct states found")
_CLASS = re.compile(r"^  class \w+ (\w+)$", re.M)
# The light fills of the group colors of `/statechart` (its references/render.md). Its
# chart.mmd leaves them to the CSS of Pele, so a page adds them for every Mermaid viewer.
_FILLS = {
    "blue": "#c3e0ff", "orange": "#f0d3bf", "purple": "#e3d5ff", "yellow": "#f0dcbf",
    "teal": "#bfe3df", "tan": "#e5d9ce", "pink": "#ffcbe6",
}
_SCOPE = re.compile(r"^\\\* Scope: (.*)$", re.M)
_TITLE = re.compile(r'^title: "(.*)"$', re.M)
_DRAWN = re.compile(r"TLC reached (\d+) states and \d+ steps\. The chart draws (\d+) states")


def _mise(*args: str) -> str:
    env = dict(os.environ, MISE_TRUSTED_CONFIG_PATHS=str(SKILL))
    found = subprocess.run(
        ["mise", *args], cwd=SKILL, env=env, capture_output=True, text=True, check=False
    )
    if found.returncode != 0:
        install = subprocess.run(
            ["mise", "install"], cwd=SKILL, env=env, capture_output=True, text=True, check=False
        )
        if install.returncode != 0:
            sys.exit(f"mise install failed: {install.stderr.strip()}")
        found = subprocess.run(
            ["mise", *args], cwd=SKILL, env=env, capture_output=True, text=True, check=True
        )
    return found.stdout.strip()


def _tools() -> tuple[str, str]:
    jar = str(Path(_mise("where", "http:tla2tools")) / "tla2tools.jar")
    java = _mise("which", "java")
    return java, jar


def _run(
    java: str, jar: str, spec_dir: Path, config: str, name: str, out: Path
) -> tuple[str, set[str], str]:
    """One TLC run of the spec in ``spec_dir`` with the config text ``config``."""
    out.mkdir(parents=True, exist_ok=True)
    config_file = out / f"{name}.cfg"
    config_file.write_text(config, encoding="utf-8")
    command = [
        java, HEAP, "-XX:+UseParallelGC", "-cp", jar, "tlc2.TLC",
        "-workers", "2", "-metadir", str(out / f"{name}.states"),
        "-config", str(config_file), SPEC,
    ]
    done = subprocess.run(command, cwd=spec_dir, capture_output=True, text=True, check=False)
    text = done.stdout + done.stderr
    (out / f"{name}.out").write_text(text, encoding="utf-8")
    shutil.rmtree(out / f"{name}.states", ignore_errors=True)
    broken = set(_BROKEN.findall(text)) | set(_BROKEN_PROPERTY.findall(text))
    states = _DISTINCT.findall(text)
    count = states[-1] if states else "?"
    failed = "TLC threw an unexpected exception" in text or "Deadlock reached" in text
    finished = "Model checking completed" in text or "states left on queue" in text
    if failed or (not finished and not broken):
        return "error", broken, count
    return ("breaks" if broken else "holds"), broken, count


def _check(
    java: str, jar: str, spec_dir: Path, config: Path, known: set[str], out: Path
) -> tuple[str, set[str], str]:
    """Run one config. While it breaks only known rules, run it again without them."""
    text = config.read_text(encoding="utf-8")
    name = config.stem
    all_broken: set[str] = set()
    first_count = ""
    while True:
        outcome, broken, count = _run(java, jar, spec_dir, text, name, out)
        first_count = first_count or count
        all_broken |= broken
        if outcome != "breaks" or not broken <= known:
            break
        text = "\n".join(line for line in text.splitlines() if line.strip() not in broken)
        name = f"{config.stem}-without-{'-'.join(sorted(broken))}"
    if outcome == "error":
        return "error", all_broken, first_count
    return ("breaks" if all_broken else "holds"), all_broken, first_count


def _mutants(java: str, jar: str, names: list[str]) -> int:
    mutants = tomllib.loads((MODEL / "mutants.toml").read_text(encoding="utf-8"))["mutant"]
    unknown = set(names) - {m["name"] for m in mutants}
    if unknown:
        sys.exit(f"no such mutant: {' '.join(sorted(unknown))}")
    model = (MODEL / SPEC).read_text(encoding="utf-8")
    worst = 0
    for mutant in mutants:
        if names and mutant["name"] not in names:
            continue
        place = OUT / "mutants" / mutant["name"]
        shutil.rmtree(place, ignore_errors=True)
        place.mkdir(parents=True)
        if model.count(mutant["old"]) != 1:
            print(f"{mutant['name']}: error: its old text is not in the model once", flush=True)
            worst = 2
            continue
        (place / SPEC).write_text(model.replace(mutant["old"], mutant["new"]), encoding="utf-8")
        config = (MODEL / f"{mutant['config']}.cfg").read_text(encoding="utf-8")
        hidden = set(mutant.get("without", []))
        config = "\n".join(line for line in config.splitlines() if line.strip() not in hidden)
        outcome, broken, count = _run(java, jar, place, config, mutant["config"], place)
        is_fix = mutant.get("fix", False)
        if outcome == "error":
            passed, verdict = False, "ERROR"
        elif is_fix:
            passed = outcome == "holds"
            verdict = "holds" if passed else "STILL BREAKS"
        else:
            passed = bool(set(mutant["breaks"]) & broken)
            verdict = "caught" if passed else "MISSED"
        if not passed:
            worst = max(worst, 2 if outcome == "error" else 1)
        found = " ".join(sorted(broken)) or "nothing"
        print(f"{mutant['name']}: {verdict}: breaks {found} ({count} states)", flush=True)
    return worst


def _statechart(java: str, jar: str, config: Path, view: Path, out: Path, *more: str) -> int:
    """One run of the ``/statechart`` skill on the model, with the TLC of ``mise.toml``."""
    shutil.rmtree(out, ignore_errors=True)
    out.parent.mkdir(parents=True, exist_ok=True)
    command = [
        str(STATECHART), str(MODEL / SPEC), "--config", str(config), "--view", str(view),
        "--out", str(out), *more,
    ]
    env = dict(os.environ, TLA2TOOLS=jar, JAVA=java)
    done = subprocess.run(command, cwd=REPO, env=env, capture_output=True, text=True, check=False)
    out.with_suffix(".log").write_text(done.stdout + done.stderr, encoding="utf-8")
    return done.returncode


def _break_path(java: str, jar: str, config: Path) -> None:
    out = OUT / "paths" / config.stem
    code = _statechart(java, jar, config, MODEL / "VerifyGate.view.mjs", out)
    paths = sorted(out.glob("sequence-*.mmd")) if code != 2 else []
    for path in paths:
        print(f"  path to the break: {path.relative_to(REPO)}", flush=True)
    if not paths and code != 2 and (out / "report.md").is_file():
        # `/statechart` draws a sequence diagram for a broken INVARIANT only. A PROPERTY gets its path as steps.
        print(f"  path to the break, as steps: {(out / 'report.md').relative_to(REPO)}", flush=True)
    elif not paths:
        print(f"  no path to the break: see {out.with_suffix('.log').relative_to(REPO)}", flush=True)


def _invariants(config: Path) -> set[str]:
    text = config.read_text(encoding="utf-8")
    block = re.search(r"^INVARIANTS?\n((?:  \w+\n)+)", text, re.M)
    return set(block.group(1).split()) if block else set()


def _scenarios(file: Path) -> dict[str, dict]:
    script = "const m = await import(process.argv[1]); console.log(JSON.stringify(m.scenarios));"
    done = subprocess.run(
        ["node", "--input-type=module", "-e", script, file.as_uri()],
        capture_output=True, text=True, check=True,
    )
    return json.loads(done.stdout)


def _page(title: str, marker: str, lead: str, mermaid: str) -> str:
    return "\n".join([
        "---", "diataxis: reference", f"title: {json.dumps(title)}", "---", "",
        marker, "", lead, "", "```mermaid", mermaid.rstrip("\n"), "```", "",
        "## Document References", "", "- [Game Flow](../game_flow.md)", "",
    ])


def _with_fills(chart: str) -> str:
    used = [c for c in _FILLS if c in set(_CLASS.findall(chart))]
    fills = [f"  classDef {c} fill:{_FILLS[c]},color:#1c1c1c" for c in used]
    return "\n".join([chart.rstrip("\n"), *fills])


def _title(mermaid: str, page: str) -> str:
    found = _TITLE.search(mermaid)
    return found.group(1) if found else page


class _NotWritten(Exception):
    """A ``/statechart`` run failed, so no page is written."""

    def __init__(self, what: str, out: Path, code: int) -> None:
        super().__init__(f"{what} failed, see {out.with_suffix('.log').relative_to(REPO)}")
        self.code = code if code in (1, 2) else 1


def _page_name(scenario: str) -> str:
    return scenario.replace("-", "_")


def _chart_page(java: str, jar: str, chart: dict, marker: str) -> str:
    out = OUT / "diagrams" / chart["page"]
    code = _statechart(java, jar, DIAGRAMS / chart["config"], DIAGRAMS / chart["view"], out)
    if code != 0 or not (out / "chart.mmd").is_file():
        raise _NotWritten(f"the chart {chart['page']}", out, code)
    mermaid = _with_fills((out / "chart.mmd").read_text(encoding="utf-8"))
    drawn = _DRAWN.search((out / "report.md").read_text(encoding="utf-8"))
    scope = _SCOPE.search((DIAGRAMS / chart["config"]).read_text(encoding="utf-8"))
    lead = f"{MODEL_WORDS} This state chart shows one part of it."
    if scope:
        lead += f" Its scope: {scope.group(1)}"
    if drawn:
        lead += f" The chart merges the {int(drawn.group(1)):,} states that TLC reached into {drawn.group(2)} states."
    return _page(_title(mermaid, chart["page"]), marker, lead, mermaid)


def _sequence_pages(java: str, jar: str, sequences: dict, scenarios: dict[str, dict], marker: str) -> dict[str, str]:
    out = OUT / "diagrams" / "sequences"
    # `/statechart` reads only its own keys, so it gets a copy without `says`.
    plain = OUT / "diagrams" / "scenarios.mjs"
    tool_scenarios = {n: {k: v for k, v in sc.items() if k != "says"} for n, sc in scenarios.items()}
    plain.write_text(f"export const scenarios = {json.dumps(tool_scenarios, indent=2)};\n", encoding="utf-8")
    config = DIAGRAMS / sequences["config"]
    code = _statechart(java, jar, config, DIAGRAMS / sequences["view"], out, "--scenarios", str(plain))
    if code != 0:
        raise _NotWritten("a scenario", out, code)
    invariants = _invariants(config)
    pages = {}
    for name, scenario in scenarios.items():
        mermaid = (out / f"{name}.mmd").read_text(encoding="utf-8")
        checked = (
            "TLC checked this rule in each state of the order."
            if scenario["rule"] in invariants
            else "TLC checks this rule in the configs of the model, not along one order."
        )
        lead = (
            f"{MODEL_WORDS} This sequence diagram shows one order of its steps, which TLC took one by one."
            f" Every invariant of the model held in each state. The order shows this rule: {scenario['says']} {checked}"
        )
        pages[_page_name(name)] = _page(scenario["title"], marker, lead, mermaid)
    return pages


def _diagrams(java: str, jar: str, force: bool) -> int:
    """Write the pages of ``diagrams/diagrams.toml``. Write nothing when one run fails."""
    manifest_file = DIAGRAMS / "diagrams.toml"
    manifest = tomllib.loads(manifest_file.read_text(encoding="utf-8"))
    scenarios = _scenarios(DIAGRAMS / manifest["sequences"]["scenarios"])
    inputs = verify_coverage.diagram_inputs_hash(SKILL)
    existing = verify_coverage.diagram_pages(REPO, manifest_file)
    names = [c["page"] for c in manifest["chart"]] + [_page_name(n) for n in scenarios]
    if not force and {p.stem for p in existing} == set(names) and set(existing.values()) == {inputs}:
        print(f"diagrams: current ({len(names)} pages in {manifest['out']})", flush=True)
        return 0
    start = time.monotonic()
    marker = verify_coverage.diagram_marker(SKILL.name, inputs)
    try:
        pages = {c["page"]: _chart_page(java, jar, c, marker) for c in manifest["chart"]}
        pages |= _sequence_pages(java, jar, manifest["sequences"], scenarios, marker)
    except _NotWritten as e:
        print(f"diagrams: not written: {e}", flush=True)
        return e.code
    for page in existing:
        page.unlink()
    target = REPO / manifest["out"]
    target.mkdir(parents=True, exist_ok=True)
    for page, text in pages.items():
        (target / f"{page}.md").write_text(text, encoding="utf-8")
    seconds = time.monotonic() - start
    print(f"diagrams: wrote {len(pages)} pages to {manifest['out']} ({seconds:.0f} s)", flush=True)
    return 0


def main(argv: list[str]) -> int:
    if argv and argv[0] == "--mutants":
        java, jar = _tools()
        return _mutants(java, jar, argv[1:])
    if argv and argv[0] == "--diagrams":
        java, jar = _tools()
        return _diagrams(java, jar, force=True)
    coverage = tomllib.loads((SKILL / "coverage.toml").read_text(encoding="utf-8"))
    known: dict[str, list[str]] = coverage.get("known_breaks", {})
    configs = [MODEL / f"{name}.cfg" for name in argv] if argv else sorted(MODEL.glob("*.cfg"))
    java, jar = _tools()
    worst = 0
    for config in configs:
        start = time.monotonic()
        expected = set(known.get(config.stem, []))
        outcome, broken, count = _check(java, jar, MODEL, config, expected, OUT)
        seconds = time.monotonic() - start
        line = f"{config.stem}: {outcome}"
        if broken:
            line += ": " + " ".join(sorted(broken))
        line += f" ({count} states, {seconds:.0f} s)"
        if outcome == "error":
            worst = 2
            line += f"  <- TLC error, see {OUT}"
        elif broken != expected:
            worst = max(worst, 1)
            new = sorted(broken - expected)
            gone = sorted(expected - broken)
            if new:
                line += f"  <- not known: {' '.join(new)}"
            if gone:
                line += f"  <- known, but holds now: {' '.join(gone)}"
        elif expected:
            line += "  (known)"
        print(line, flush=True)
        if broken - expected and outcome != "error":
            _break_path(java, jar, config)
    if argv:
        return worst
    if worst != 0:
        print("diagrams: not written, because a config did not give its expected result", flush=True)
        return worst
    return _diagrams(java, jar, force=False)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
