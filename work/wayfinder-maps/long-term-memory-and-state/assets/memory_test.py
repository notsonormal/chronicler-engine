"""The memory test: score a memory candidate on a stored trial story, with no engine change.

A question is asked at one story entry. The tool takes the stored Narration request of that
entry, which holds the history the engine sent. A control or a candidate changes only the
memory part of that request, and the tool adds the question as the last user turn:

- ``none``: the stored request as it is. The facts of the question left the history.
- ``whole``: the stored request with every dropped entry put back into the history.
- ``<candidate>``: the stored request with the memory that the candidate wrote from the
  dropped entries.

A candidate is a JSON file: a writer prompt, a schedule (``once``: one call over all the
dropped entries; ``each_cut``: one call at each history cut, which gets the previous memory
and the entries of that cut) and a position (``before_history`` or ``after_history``).

    python .scratch/long-term-memory-and-state/assets/memory_test.py run none --runs 3
    python .scratch/long-term-memory-and-state/assets/memory_test.py run \\
        .scratch/long-term-memory-and-state/assets/candidates/one_summary.json --runs 3
    python .scratch/long-term-memory-and-state/assets/memory_test.py report

The key: the tool reads only ``OPENCODE_GO_API_KEY``, from the environment or else from
``.env`` at the repo root. It never prints or stores the key.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import os
import re
import statistics
import sys
import threading
import time
import urllib.error
import urllib.request
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "scripts"))
import trial  # noqa: E402  (the pack readers of the trial tool)

HERE = Path(__file__).resolve().parent
QUESTIONS = HERE / "memory_test_questions.json"
DEFAULT_PACK = ROOT / "tmp/lt15/new/trial-1"
DEFAULT_OUT = ROOT / "tmp/ltm04"
ENDPOINT = "https://opencode.ai/zen/go/v1/chat/completions"
KEY_NAME = "OPENCODE_GO_API_KEY"
HISTORY_OPEN = "<ConversationHistory>\n"
CONTROLS = ("none", "whole")
SHAPES = ("full", "short")

ASK = (
    "[OUT OF CHARACTER. This is not a story turn. Do not continue the story.]\n"
    "Answer one question about the story so far, as the narrator who knows everything "
    "that happened in it. Use only what the story so far says. If the story so far does "
    "not say, answer \"I don't know\".\n"
)
ASK_PLAIN = "Answer in one short sentence."
ASK_YESNO = "Begin the answer with Yes or No, then give one short reason."

SHORT_SYSTEM = (
    "You answer questions about an interactive story from the memory notes you are given."
)

GRADER = """You grade one answer to a question about a story.

Question: {question}
Right answer: {answer}
Given answer: {given}

The given answer is right if it states the right answer, in any wording. Extra true detail is
fine. An answer of "I don't know", a different value, or two values that conflict is wrong.
For a time, an error of one unit (one day, one hour) is wrong unless the right answer says
"about". Reply with one word: RIGHT or WRONG."""


# ---------------------------------------------------------------- the model


def api_key():
    """The key from the environment, else the one line of `.env` that names it."""
    if os.environ.get(KEY_NAME):
        return os.environ[KEY_NAME]
    env = ROOT / ".env"
    if env.exists():
        for line in env.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line.startswith("export "):
                line = line[len("export ") :]
            if line.startswith(KEY_NAME + "="):
                return line.split("=", 1)[1].strip().strip("'\"")
    raise SystemExit(f"{KEY_NAME} is not in the environment or in .env")


class Model:
    """One OpenCode Go model. Counts the calls and the tokens of a run."""

    def __init__(self, model, temperature, session):
        self.key = api_key()
        self.model = model
        self.temperature = temperature
        self.session = session
        self.lock = threading.Lock()
        self.cost = {"calls": 0, "prompt": 0, "cached": 0, "completion": 0, "failed": 0}

    def call(self, messages, max_tokens):
        body = {
            "model": self.model,
            "messages": messages,
            "max_tokens": max_tokens,
            "reasoning_effort": "none",
            "stream": False,
        }
        if self.temperature is not None:
            body["temperature"] = self.temperature
        data = json.dumps(body).encode()
        for attempt in range(4):
            request = urllib.request.Request(
                ENDPOINT,
                data=data,
                headers={
                    "Content-Type": "application/json",
                    "Authorization": f"Bearer {self.key}",
                    "x-opencode-session": self.session,
                    # Cloudflare refuses the default `Python-urllib` agent (error 1010).
                    "User-Agent": "chronicler-memory-test/1.0",
                },
            )
            try:
                with urllib.request.urlopen(request, timeout=300) as response:
                    answer = json.loads(response.read())
                break
            except (urllib.error.URLError, TimeoutError) as error:
                detail = ""
                if isinstance(error, urllib.error.HTTPError):
                    detail = error.read().decode(errors="replace")[:300]
                    if error.code < 500 and error.code != 429:
                        raise RuntimeError(f"HTTP {error.code}: {detail}") from None
                if attempt == 3:
                    with self.lock:
                        self.cost["failed"] += 1
                    raise RuntimeError(f"no answer after 4 tries: {error} {detail}") from None
                time.sleep(5 * (attempt + 1))
        usage = answer.get("usage") or {}
        with self.lock:
            self.cost["calls"] += 1
            self.cost["prompt"] += usage.get("prompt_tokens", 0)
            self.cost["completion"] += usage.get("completion_tokens", 0)
            self.cost["cached"] += (usage.get("prompt_tokens_details") or {}).get(
                "cached_tokens", 0
            )
        return (answer["choices"][0]["message"].get("content") or "").strip()


# ---------------------------------------------------------------- the pack


class Pack:
    """The story and the stored Narration requests of one trial pack."""

    def __init__(self, folder):
        self.folder = Path(folder)
        self.story = trial.read_story(self.folder)
        self.by_id = {entry["id"]: entry for entry in self.story}
        self.narration = {}
        for line in (self.folder / "calls.jsonl").read_text(encoding="utf-8").splitlines():
            call = json.loads(line)
            if call["agent_name"] != "narrator" or call["error_message"]:
                continue
            trial.read_prompts(call)
            if trial.call_kind(call, self.story) != "narration":
                continue
            owner = trial.owner_entry(call, self.story)
            if owner is not None:
                self.narration[owner] = call

    def label(self, entry):
        return "Julian" if entry["impersonated"] else "Narration"

    def ask_call(self, ask_at):
        """The stored request of the first Narration entry at or after the ask entry."""
        for entry_id in sorted(self.narration):
            if entry_id >= ask_at:
                return entry_id, self.narration[entry_id]
        raise SystemExit(f"no Narration call at or after #{ask_at}")

    def messages(self, call):
        return json.loads(call["raw_request_json"])["messages"]

    def history_span(self, messages):
        """The indexes of the first and the last history message."""
        first = next(i for i, m in enumerate(messages) if m["content"].startswith(HISTORY_OPEN))
        last = next(
            i for i, m in enumerate(messages) if m["content"].rstrip().endswith("</ConversationHistory>")
        )
        return first, last

    def held(self, entry_id):
        """The first story entry that the history of this call still held."""
        call = self.narration[entry_id]
        first, _ = self.history_span(self.messages(call))
        text = self.messages(call)[first]["content"][len(HISTORY_OPEN) :].strip()[:100]
        for entry in self.story:
            if entry["id"] < entry_id and entry["text"].strip()[:100] == text[:100]:
                return entry["id"]
        raise SystemExit(f"cannot find the first history entry of #{entry_id}")

    def dropped(self, entry_id):
        oldest = self.held(entry_id)
        return [entry for entry in self.story if entry["id"] < oldest]

    def cuts(self, entry_id):
        """The oldest held entry after each history cut up to this call, in order."""
        seen = []
        for other in sorted(self.narration):
            if other > entry_id:
                break
            oldest = self.held(other)
            if oldest != self.story[0]["id"] and oldest not in seen:
                seen.append(oldest)
        return seen

    def transcript(self, entries):
        return "\n\n".join(f"[#{e['id']} {self.label(e)}]\n{e['text'].strip()}" for e in entries)


# ---------------------------------------------------------------- the prompts


def add_question(messages, question):
    tail = ASK_YESNO if question["kind"] == "who" else ASK_PLAIN
    text = f"{ASK}{tail}\n\nQuestion: {question['question']}"
    return messages + [{"role": "user", "content": text}]


def with_whole_story(pack, messages, dropped):
    """Put the dropped entries back at the front of the history, one message each."""
    first, _ = pack.history_span(messages)
    if not dropped:
        return messages
    back = [
        {"role": "user" if e["impersonated"] else "assistant", "content": e["text"]}
        for e in dropped
    ]
    back[0]["content"] = HISTORY_OPEN + back[0]["content"]
    old_first = dict(messages[first])
    old_first["content"] = old_first["content"][len(HISTORY_OPEN) :]
    return messages[:first] + back + [old_first] + messages[first + 1 :]


def with_memory(pack, messages, memory, position):
    block = f"<StoryMemory>\n{memory.strip()}\n</StoryMemory>"
    first, last = pack.history_span(messages)
    messages = [dict(m) for m in messages]
    if position == "before_history":
        messages[first - 1]["content"] = messages[first - 1]["content"].rstrip() + "\n\n" + block
    else:
        messages.insert(last + 1, {"role": "user", "content": block})
    return messages


def short_prompt(memory, question):
    memory_text = f"<StoryMemory>\n{memory.strip()}\n</StoryMemory>\n\n" if memory else ""
    return [
        {"role": "system", "content": SHORT_SYSTEM},
        {"role": "user", "content": memory_text},
    ]


# ---------------------------------------------------------------- the candidates


def load_candidate(path):
    candidate = json.loads(Path(path).read_text(encoding="utf-8"))
    for field in ("name", "schedule", "position", "writer_system", "writer_user"):
        if field not in candidate:
            raise SystemExit(f"candidate {path} has no `{field}`")
    if candidate["schedule"] not in ("once", "each_cut"):
        raise SystemExit("schedule must be `once` or `each_cut`")
    return candidate


class Memories:
    """The memory a candidate wrote for a dropped range, kept on disk for each run."""

    def __init__(self, pack, candidate, model, folder):
        self.pack, self.candidate, self.model = pack, candidate, model
        self.folder = folder
        self.folder.mkdir(parents=True, exist_ok=True)
        self.lock = threading.Lock()
        self.locks = {}

    def write(self, previous, entries):
        c = self.candidate
        user = c["writer_user"].format(
            previous=previous or "(none)",
            entries=self.pack.transcript(entries),
            first=entries[0]["id"],
            last=entries[-1]["id"],
        )
        messages = [{"role": "system", "content": c["writer_system"]}, {"role": "user", "content": user}]
        return self.model.call(messages, c.get("max_tokens", 2048))

    def stored(self, key, make):
        path = self.folder / f"{key}.md"
        with self.lock:
            lock = self.locks.setdefault(key, threading.Lock())
        with lock:
            if path.exists():
                return path.read_text(encoding="utf-8")
            text = make()
            path.write_text(text, encoding="utf-8")
            return text

    def for_call(self, entry_id):
        dropped = self.pack.dropped(entry_id)
        if not dropped:
            return ""
        if self.candidate["schedule"] == "once":
            return self.stored(f"to-{dropped[-1]['id']}", lambda: self.write(None, dropped))
        memory, start = None, self.pack.story[0]["id"]
        for oldest in self.pack.cuts(entry_id):
            part = [e for e in self.pack.story if start <= e["id"] < oldest]
            memory = self.stored(
                f"cut-{oldest}", lambda prev=memory, part=part: self.write(prev, part)
            )
            start = oldest
        return memory or ""


# ---------------------------------------------------------------- the score


def by_rule(question, given):
    """True or False when a rule decides, None when the grader must."""
    text = given.lower()
    if question["kind"] == "who":
        first = re.match(r"\W*(yes|no)\b", text)
        if not first:
            return False
        return first.group(1) == question["answer"].split(".")[0].strip().lower()
    if "accept" not in question:
        return None
    if any(re.search(p, text) for p in question.get("reject", [])):
        return False
    matched = any(re.search(p, text) for p in question["accept"])
    hedged = re.search(r"\bi don'?t know\b|\bdoes(n'?t| not) say\b|\bnot (stated|mentioned)\b", text)
    if hedged:
        # "I don't know — the drawer was empty" can hold the right fact: the grader decides.
        return None if matched else False
    return matched


def grade(model, question, given):
    verdict = by_rule(question, given)
    if verdict is not None:
        return verdict, "rule"
    prompt = GRADER.format(question=question["question"], answer=question["answer"], given=given)
    word = model.call([{"role": "user", "content": prompt}], 10)
    return word.strip().upper().startswith("RIGHT"), "grader"


# ---------------------------------------------------------------- one run


def run_one(pack, questions, arm, shape, run, args):
    """Run every question once for one arm; append to its result file, skipping done ones."""
    name = arm if arm in CONTROLS else load_candidate(arm)["name"]
    out = Path(args.out) / "runs" / f"{name}-{shape}-r{run}.jsonl"
    out.parent.mkdir(parents=True, exist_ok=True)
    done = set()
    if out.exists():
        done = {json.loads(line)["id"] for line in out.read_text(encoding="utf-8").splitlines()}
    todo = [q for q in questions if q["id"] not in done]
    session = f"ltm04-{name}-{shape}-r{run}-{uuid.uuid4().hex[:8]}"
    model = Model(args.model, args.temperature, session)
    memories = None
    if arm not in CONTROLS:
        candidate = load_candidate(arm)
        memories = Memories(pack, candidate, model, Path(args.out) / "memory" / f"{name}-r{run}")
    write_lock = threading.Lock()

    def ask(question):
        entry_id, call = pack.ask_call(question["ask_at"])
        started = model.cost["calls"]
        memory = memories.for_call(entry_id) if memories else ""
        if shape == "short":
            if arm == "whole":
                memory = pack.transcript([e for e in pack.story if e["id"] < entry_id])
            messages = add_question(short_prompt(memory, question), question)
            messages[1]["content"] += messages.pop()["content"]
        else:
            messages = pack.messages(call)
            if arm == "whole":
                messages = with_whole_story(pack, messages, pack.dropped(entry_id))
            elif memories:
                messages = with_memory(pack, messages, memory, candidate["position"])
            messages = add_question(messages, question)
        given = model.call(messages, 200)
        right, how = grade(model, question, given)
        row = {
            "id": question["id"],
            "kind": question["kind"],
            "question": question["question"],
            "call_at": entry_id,
            "oldest_held": pack.held(entry_id),
            "given": given,
            "right": right,
            "graded_by": how,
            "memory_words": len(memory.split()) if arm not in CONTROLS else 0,
            "calls": model.cost["calls"] - started,
        }
        with write_lock, out.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(row) + "\n")
        return row

    with concurrent.futures.ThreadPoolExecutor(args.workers) as pool:
        for future in concurrent.futures.as_completed([pool.submit(ask, q) for q in todo]):
            try:
                future.result()
            except Exception as error:  # noqa: BLE001  one failed question must not stop the run
                print(f"  failed: {error}", file=sys.stderr)
    cost_file = out.with_suffix(".cost.json")
    old = json.loads(cost_file.read_text()) if cost_file.exists() else {}
    total = {k: old.get(k, 0) + v for k, v in model.cost.items()}
    cost_file.write_text(json.dumps(total), encoding="utf-8")
    print(f"{out.name}: {len(todo)} asked, cost {model.cost}")


def regrade_all(args):
    """Score every stored answer again; drop the rows of questions that left the set."""
    questions = {q["id"]: q for q in json.loads(QUESTIONS.read_text(encoding="utf-8"))}
    model = Model(args.model, 0.0, f"ltm04-regrade-{uuid.uuid4().hex[:8]}")
    changed = 0
    for path in sorted((Path(args.out) / "runs").glob("*.jsonl")):
        rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
        kept = []
        for row in rows:
            question = questions.get(row["id"])
            if question is None or question["question"] != row.get("question", question["question"]):
                continue
            right, how = grade(model, question, row["given"])
            changed += right != row["right"]
            row["right"], row["graded_by"] = right, how
            kept.append(row)
        path.write_text("".join(json.dumps(row) + "\n" for row in kept), encoding="utf-8")
    print(f"regraded: {changed} verdicts changed, grader cost {model.cost}")


# ---------------------------------------------------------------- the report


def read_runs(out):
    """{(arm, shape): [ {id: row} for each run ]} and the cost of each run."""
    runs, costs = {}, {}
    for path in sorted((Path(out) / "runs").glob("*.jsonl")):
        match = re.match(r"(.+)-(full|short)-r(\d+)$", path.stem)
        if not match:
            continue
        arm, shape, _ = match.groups()
        rows = {}
        for line in path.read_text(encoding="utf-8").splitlines():
            row = json.loads(line)
            rows[row["id"]] = row
        runs.setdefault((arm, shape), []).append(rows)
        cost = path.with_suffix(".cost.json")
        if cost.exists():
            costs.setdefault((arm, shape), []).append(json.loads(cost.read_text()))
    return runs, costs


def kept_questions(questions, runs, shape):
    """A question is kept when the no-memory control answers it wrong in every run."""
    none = runs.get(("none", shape), [])
    if not none:
        return [q["id"] for q in questions]
    return [
        q["id"]
        for q in questions
        if all(q["id"] in rows and not rows[q["id"]]["right"] for rows in none)
    ]


def report(args):
    questions = json.loads(QUESTIONS.read_text(encoding="utf-8"))
    runs, costs = read_runs(args.out)
    for shape in SHAPES:
        arms = sorted((a for a, s in runs if s == shape), key=lambda a: (a not in CONTROLS, a))
        if not arms:
            continue
        kept = kept_questions(questions, runs, shape)
        print(f"## Shape `{shape}`\n")
        print(f"Kept questions (no-memory control wrong in every run): {len(kept)} of {len(questions)}\n")
        print("| Arm | Runs | Kept: fact | Kept: time | Kept: who | Kept: all, mean (min-max) | All questions | Calls per run | Prompt tokens per run | Memory words |")
        print("|---|---|---|---|---|---|---|---|---|---|")
        for arm in arms:
            all_runs = runs[(arm, shape)]
            def share(ids, rows):
                ids = [i for i in ids if i in rows]
                return sum(rows[i]["right"] for i in ids), len(ids)
            cells = []
            for kind in ("fact", "time", "who"):
                ids = [q["id"] for q in questions if q["kind"] == kind and q["id"] in kept]
                rights = [share(ids, rows) for rows in all_runs]
                cells.append(" / ".join(f"{r}/{n}" for r, n in rights))
            totals = [share(kept, rows) for rows in all_runs]
            values = [r / n * 100 if n else 0 for r, n in totals]
            spread = f"{statistics.mean(values):.0f} % ({min(values):.0f}-{max(values):.0f})"
            every = [share([q["id"] for q in questions], rows) for rows in all_runs]
            cost = costs.get((arm, shape), [])
            calls = statistics.mean(c["calls"] for c in cost) if cost else 0
            tokens = statistics.mean(c["prompt"] for c in cost) if cost else 0
            words = [row["memory_words"] for rows in all_runs for row in rows.values()]
            print(
                f"| {arm} | {len(all_runs)} | {cells[0]} | {cells[1]} | {cells[2]} | {spread} | "
                f"{' / '.join(f'{r}/{n}' for r, n in every)} | {calls:.0f} | {tokens:,.0f} | "
                f"{statistics.mean(words) if words else 0:.0f} |"
            )
        print()
        if args.detail:
            print("| Question | " + " | ".join(arms) + " |")
            print("|---|" + "---|" * len(arms))
            for q in questions:
                marks = []
                for arm in arms:
                    marks.append("".join(
                        ("✓" if rows[q["id"]]["right"] else "✗") if q["id"] in rows else "·"
                        for rows in runs[(arm, shape)]
                    ))
                print(f"| {q['id']}{'' if q['id'] in kept else ' (dropped)'} | " + " | ".join(marks) + " |")
            print()


# ---------------------------------------------------------------- the commands


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    run = sub.add_parser("run", help="run one arm: `none`, `whole`, or a candidate JSON file")
    run.add_argument("arm")
    run.add_argument("--runs", type=int, default=1)
    run.add_argument("--first-run", type=int, default=1)
    run.add_argument("--shape", choices=SHAPES, default="full")
    run.add_argument("--only", help="comma-separated question ids")
    run.add_argument("--pack", default=str(DEFAULT_PACK))
    run.add_argument("--out", default=str(DEFAULT_OUT))
    run.add_argument("--model", default="deepseek-v4.1-flash")
    run.add_argument("--temperature", type=float, default=0.0)
    run.add_argument("--workers", type=int, default=6)
    rep = sub.add_parser("report", help="the score, spread and cost of every arm run so far")
    rep.add_argument("--out", default=str(DEFAULT_OUT))
    rep.add_argument("--detail", action="store_true")
    regrade = sub.add_parser("regrade", help="score the stored answers again with the current rules")
    regrade.add_argument("--out", default=str(DEFAULT_OUT))
    regrade.add_argument("--model", default="deepseek-v4.1-flash")
    check = sub.add_parser("check", help="print the call, the cut and the dropped range of each question")
    check.add_argument("--pack", default=str(DEFAULT_PACK))
    args = parser.parse_args()

    if args.command == "report":
        report(args)
        return
    if args.command == "regrade":
        regrade_all(args)
        return
    pack = Pack(args.pack)
    questions = json.loads(QUESTIONS.read_text(encoding="utf-8"))
    if args.command == "check":
        for q in questions:
            entry_id, _ = pack.ask_call(q["ask_at"])
            oldest = pack.held(entry_id)
            stated = q.get("stated_at", [])
            ok = "" if not stated or max(stated) < oldest else "  ** a stating entry is still in the history"
            print(f"{q['id']:6} ask #{q['ask_at']} call #{entry_id} oldest held #{oldest}{ok}")
        return
    if args.only:
        wanted = set(args.only.split(","))
        questions = [q for q in questions if q["id"] in wanted]
    for number in range(args.first_run, args.first_run + args.runs):
        run_one(pack, questions, args.arm, args.shape, number, args)


if __name__ == "__main__":
    main()
