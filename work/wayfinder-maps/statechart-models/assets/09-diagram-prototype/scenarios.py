"""Prototype: draw a gate scenario as a Mermaid sequence diagram from a checked TLC trace.

A scenario is a list of steps of VerifyGate.tla. The script writes a module that lets TLC take
only those steps, in that order, and checks the rules in each state on the way. TLC's trace gives
the state after each step, so each arrow shows whether a write landed or the id check refused it.
A step that the model does not allow stops the script: the diagram is a real path of the model.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent  # run from a scratch folder that holds a copy of VerifyGate.tla
JAVA = "/home/node/.local/share/mise/installs/java/temurin-25/bin/java"
JAR = "/home/node/.local/share/mise/installs/http-tla2tools/1.7.4/tla2tools.jar"
RULES = [
    "OneJob", "BusyRefusesStory", "StoppedJobWritesNothing", "StopNoFailureLine",
    "StopClearsOptionsFailure", "NextJobKeepsHolder", "RunHoldsItsGame", "StoppedRunStartsNoJob",
    "RunRefusesConfig", "RunEndReasons", "BudgetBound", "RunFailureLineStays", "RunEndsIdle",
    "DeleteNotActive",
]

BASE = dict(Games0="{1}", NewGames="{}", Active0="1", MaxJobs="2", MaxRuns="0", Budget="1",
            PlainKinds='{"action"}')

SCENARIOS = {
    "stop-during-the-narrator-call": dict(
        title="A stop while the narrator call runs",
        env='{"plain", "stop"}', consts=dict(MaxJobs="1"),
        steps=["YouStartAJob(1, \"action\")", "AJobStartsAGeneration(1)", "YouPressStop(1)",
               "TheNarratorAnswers(1)", "TheTextIsSaved(1)"]),
    "late-end-of-a-stopped-job": dict(
        title="The late end of a stopped Job does not free the next Job",
        env='{"plain", "stop"}', consts={},
        steps=["YouStartAJob(1, \"action\")", "AJobStartsAGeneration(1)", "TheNarratorFails(1)",
               "TheJobSkipsTheOptionsRun(1)", "TheFailureLineIsSaved(1)", "YouPressStop(1)",
               "YouStartAJob(1, \"action\")", "TheJobEnds(1)", "AJobStartsAGeneration(2)"]),
    "late-end-of-a-stopped-run": dict(
        title="The end of a stopped run does not free the run after it",
        env='{"run", "stop"}', consts=dict(MaxRuns="2"),
        steps=["YouStartAutoplay(1)", "TheRunAccepts(1, TRUE)", "AJobStartsAGeneration(1)",
               "YouPressStop(1)", "YouStartAutoplay(1)", "TheNarratorAnswers(1)",
               "TheTextIsSaved(1)", "TheRunSeesItsJobEnd(1)", "TheRunEnds(1)",
               "TheRunAccepts(2, TRUE)"]),
    "delete-races-a-switch": dict(
        title="Known bug: a delete races a switch of the active Game",
        env='{"delete", "switch"}', consts=dict(Games0="{1, 2}", MaxJobs="0"),
        steps=["YouDeleteAGame(2)", "YouSwitchTheGame(2)", "TheDeleteRemovesTheGame(2)"]),
}


def module(steps: list[str]) -> str:
    arms = "\n".join(f"  \\/ (i = {k} /\\ {s} /\\ i' = {k + 1})" for k, s in enumerate(steps))
    return f"""---- MODULE Scenario ----
EXTENDS VerifyGate
VARIABLE i
ScnInit == Init /\\ i = 0
ScnNext ==
{arms}
ScnSpec == ScnInit /\\ [][ScnNext]_<<vars, i>>
NotDone == i < {len(steps)}
====
"""


def config(sc: dict) -> str:
    consts = {**BASE, **sc["consts"], "Env": sc["env"]}
    lines = ["SPECIFICATION ScnSpec", "CONSTANTS"] + [f"  {k} = {v}" for k, v in consts.items()]
    return "\n".join(lines + ["INVARIANTS"] + [f"  {r}" for r in RULES] + ["  NotDone", ""])


def run_tlc(name: str, sc: dict) -> str:
    work = HERE / "scn" / name
    work.mkdir(parents=True, exist_ok=True)
    (work / "VerifyGate.tla").write_text((HERE / "VerifyGate.tla").read_text())
    (work / "Scenario.tla").write_text(module(sc["steps"]))
    (work / "Scenario.cfg").write_text(config(sc))
    done = subprocess.run(
        [JAVA, "-XX:+UseParallelGC", "-cp", JAR, "tlc2.TLC", "-deadlock", "-workers", "1",
         "-metadir", str(work / "states"), "-config", "Scenario.cfg", "Scenario.tla"],
        cwd=work, capture_output=True, text=True, check=False)
    (work / "tlc.out").write_text(done.stdout + done.stderr)
    return done.stdout


def states(out: str) -> tuple[str, list[dict[str, str]]]:
    broken = re.search(r"Invariant (\w+) is violated", out)
    if not broken:
        sys.exit("the model does not allow this scenario: TLC found no path through its steps")
    trace: list[dict[str, str]] = []
    for block in re.split(r"\nState \d+: ", out)[1:]:
        values: dict[str, str] = {}
        current = None
        for line in block.splitlines()[1:]:
            m = re.match(r"/\\ (\w+) = (.*)", line)
            if m:
                current = m.group(1)
                values[current] = m.group(2)
            elif current and line.startswith(" "):
                values[current] += " " + line.strip()
            elif not line.strip():
                break
        trace.append(values)
    return broken.group(1), trace


def field(value: str, name: str) -> list[str]:
    return re.findall(name + r' \|-> "?(-?\w+)"?', value)


def draw(sc: dict, rule: str, trace: list[dict[str, str]]) -> str:
    out: list[str] = []
    lanes: set[str] = set()

    def say(line: str, *used: str) -> None:
        lanes.update(used)
        out.append("  " + line)

    for step, (a, b) in zip(sc["steps"], zip(trace, trace[1:])):
        act, args = re.match(r"(\w+)\((.*)\)", step).groups()
        n = args.split(",")[0].strip()
        jpc_a, jpc_b = field(a.get("job", ""), "pc"), field(b.get("job", ""), "pc")
        jfail_b = field(b.get("job", ""), "fail")
        hjob_b, hkind_b = field(b["holder"], "job"), field(b["holder"], "kind")
        moved = lambda j: jpc_b[j - 1] == "done" and jfail_b[j - 1] == "moved" and jpc_a[j - 1] != "done"
        J = f"J{n}"
        if act == "YouStartAJob":
            j = a["nextJob"]
            say(f"You->>G: send an Action", "You", "G")
            say(f"Note over G: Job {j} holds the Game", f"J{j}")
        elif act == "YouStartAutoplay":
            r = a["nextRun"]
            say(f"You->>G: start Autoplay", "You", "G")
            say(f"Note over G: Run {r} holds the Game", f"R{r}")
        elif act == "YouPressStop":
            say("You->>G: Stop", "You", "G")
            say("Note over G: the holder goes, and its work is cut off")
        elif act == "YouDeleteAGame":
            say(f"You->>P: delete Game {n}: is it the active Game?", "You", "P")
            say(f"P-->>You: no, Game {a['active']} is active")
        elif act == "YouSwitchTheGame":
            say(f"You->>P: switch to Game {n}", "You", "P")
            say(f"Note over P: the active Game is Game {n}")
        elif act == "TheDeleteRemovesTheGame":
            say(f"You->>G: delete the rows of Game {n}", "You", "G")
            say(f"Note over G,P: Game {n} is gone, and it is the active Game")
        elif act == "AJobStartsAGeneration":
            if moved(int(n)):
                say(f"{J}-xG: set the step: refused, the holder is not Job {n}", J, "G")
                say(f"Note over {J}: Job {n} ends and writes nothing more")
            elif field(a["job"], "kind")[int(n) - 1] == "action" and field(a["job"], "gen")[int(n) - 1] == "first":
                say(f"{J}->>G: save the Input, set the step", J, "G")
                say(f"{J}->>L: narrator call", "L")
            else:
                say(f"{J}->>G: set the step", J, "G")
                say(f"{J}->>L: narrator call", "L")
        elif act == "TheNarratorAnswers":
            say(f"L-->>{J}: the Narration", "L", J)
        elif act == "TheNarratorFails":
            say(f"L--x{J}: the call fails", "L", J)
        elif act == "TheTextIsSaved":
            if moved(int(n)):
                say(f"{J}-xG: save the text: refused, the holder is not Job {n}", J, "G")
                say(f"Note over {J}: Job {n} ends and writes nothing more")
            else:
                say(f"{J}->>G: save the text", J, "G")
        elif act == "TheJobSkipsTheOptionsRun":
            say(f"Note over {J}: no Options run", J)
        elif act == "TheFailureLineIsSaved":
            say(f"{J}->>G: save the failure line", J, "G")
        elif act == "TheJobEnds":
            say(f"{J}->>G: let go of the Game", J, "G")
            g_holder = hjob_b[0]
            if hkind_b[0] != "none" and g_holder != n:
                say(f"Note over G: the holder is Job {g_holder}, so it stays")
            else:
                say("Note over G: the holder goes")
        elif act == "TheRunAccepts":
            R = f"R{n}"
            say(f"{R}->>G: accept my next Job", R, "G")
            if jpc_b != jpc_a and any(p == "start" for p in jpc_b):
                j = int(b["nextJob"]) - 1
                say(f"G-->>{R}: accepted: Job {j} starts", f"J{j}")
            else:
                say(f"G-->>{R}: ended")
        elif act == "TheRunSeesItsJobEnd":
            R = f"R{n}"
            say(f"Note over {R}: its Job ended, so the run ends", R)
        elif act == "TheRunEnds":
            R = f"R{n}"
            say(f"{R}->>G: end the run", R, "G")
            if hkind_b[0] == "run":
                say("Note over G: the holder is the next run, so it stays")
            else:
                say("Note over G: the holder goes")
        else:
            say(f"Note over G: {act}({args})", "G")
    if rule != "NotDone":
        say(f"Note over G,P: rule {rule} breaks here")
    names = [("You", "You"), ("P", "Active Game pointer"), ("G", "Busy registry"),
             ("R1", "Run 1"), ("R2", "Run 2"), ("J1", "Job 1"), ("J2", "Job 2"), ("L", "LLM")]
    head = [f"  participant {k} as {v}" for k, v in names if k in lanes]
    return "\n".join(["---", f'title: "{sc["title"]}"', "---", "sequenceDiagram", *head, *out])


def main() -> None:
    picked = sys.argv[1:] or list(SCENARIOS)
    for name in picked:
        sc = SCENARIOS[name]
        rule, trace = states(run_tlc(name, sc))
        text = draw(sc, rule, trace)
        (HERE / "scn" / f"{name}.mmd").write_text(text + "\n")
        print(f"{name}: {'reached' if rule == 'NotDone' else 'breaks ' + rule}, {len(trace) - 1} steps")


if __name__ == "__main__":
    main()
