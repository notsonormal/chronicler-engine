"""Check the TLA+ model of the gate with TLC: every config in ``model/``, one after the other.

Usage: ``python3 .agents/skills/verify-gate/check.py [CONFIG ...]`` (default: every config), or
``python3 .agents/skills/verify-gate/check.py --mutants [NAME ...]`` (default: every mutant).

TLC and the JDK come from ``mise``, with the pins of ``mise.toml`` in this folder. The first run
installs them and needs the network. Each config prints one line: ``holds``, ``breaks: RULE ...``
or ``error``. TLC stops at the first rule that breaks. ``coverage.toml`` lists the rules that
break today (``known_breaks``). When a config breaks a known rule, the config runs again without
that rule, so that a second break cannot hide behind the first. The exit code is 0 when each
config breaks exactly its known rules, 1 when a rule breaks that is not known or a known break no
longer breaks, and 2 on a TLC error. The TLC logs go to
``tmp/verify-gate/<config>.out`` in the repo.

``--mutants`` runs the mutants of ``model/mutants.toml``: each one is a copy of the model with one
text replaced, so that it mirrors one break of the code. A mutant passes when TLC breaks each rule
that it names (a ``fix`` mutant passes when its config holds). Because TLC stops at the first
break, a mutant names the rules that can break first, and it passes when one of them breaks. Its
``without`` list takes rules out of its config, so that a rule behind another rule can show. The exit code is 1 when a mutant
does not pass. The copies go to ``tmp/verify-gate/mutants/<name>/``.

Do not run it while a cargo build runs: memory is the limit of this host.
"""

from __future__ import annotations

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

_BROKEN = re.compile(r"^Error: Invariant (\w+) is violated", re.M)
_BROKEN_PROPERTY = re.compile(r"^Error: Action property (\w+) is violated", re.M)
_DISTINCT = re.compile(r"([\d,]+) distinct states found")


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


def main(argv: list[str]) -> int:
    if argv and argv[0] == "--mutants":
        java, jar = _tools()
        return _mutants(java, jar, argv[1:])
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
    return worst


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
