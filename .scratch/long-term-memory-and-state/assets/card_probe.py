"""The card probe: replay stored calls with four card layouts, and count card leaks and entry faults.

Ticket 11 of the long-term memory map. A scene is one stored Narration, event Narration or
Impersonate call of a trial pack. An arm changes only the card text of that request:

1. ``today``: the stored request as it is (`<KnownCharacters>` short entries before the
   history, full cards of the Characters present after it).
2. ``minimal``: every card in one `<CharacterCards>` block in place of `<KnownCharacters>`;
   `secrets` cut out of `scenario`, `summary` gone, `example_dialogue` in. After the history,
   `<CharactersPresent>` names the Characters present only.
3. ``minimal_rule``: arm 2 with the rule text at the head of the block.
4. ``split``: the full split of the card structure mapping, a label on each part, and the
   rule text.

A judge reads each answer with the scene's facts and the full cards, and quotes its evidence.
The tool keeps a finding only when its quote is in the answer.

    T=.scratch/long-term-memory-and-state/assets/card_probe.py
    python3 $T check                      # the facts of each scene against the story so far
    python3 $T show I1 minimal_rule       # print the request of one scene and arm
    python3 $T run today --runs 3         # one arm, 3 runs, then the judge
    python3 $T report

The key: the tool reads only ``OPENCODE_GO_API_KEY`` (through memory_test.api_key). It never
prints or stores the key.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import re
import statistics
import sys
import threading
import uuid
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import memory_test  # noqa: E402  (the model client, and through it the trial pack readers)
from memory_test import ROOT, Model, trial  # noqa: E402

SCENES = HERE / "card_probe_scenes.json"
CARDS = HERE / "card_probe_cards.json"
DATA = ROOT / "data/characters/redmist_estate"
DEFAULT_PACK = ROOT / "tmp/lt15/new/trial-1"
DEFAULT_OUT = ROOT / "tmp/ltm11"
ARMS = ("today", "minimal", "minimal_rule", "split")
ORDER = ("carla", "gabriella", "jezebel", "lisette", "louise")
USER = "Julian"

KNOWN_RE = re.compile(r"<KnownCharacters>.*?</KnownCharacters>", re.S)
PRESENT_RE = re.compile(r"<CharactersPresent>(.*?)</CharactersPresent>", re.S)
LAST_RE = re.compile(r"<LastMessage>\n.*?\n</LastMessage>", re.S)


# ---------------------------------------------------------------- the cards


def fill(text):
    return text.replace("{{user}}", USER).replace("{{char}}", "she")


def load_cards():
    """The five Redmist cards. Some card files hold CRLF line ends; the cuts use LF."""
    text = {cid: (DATA / f"{cid}.json").read_text(encoding="utf-8") for cid in ORDER}
    return {cid: json.loads(raw.replace("\\r\\n", "\\n")) for cid, raw in text.items()}


def must_find(part, text, where):
    if part not in text:
        raise SystemExit(f"card_probe_cards.json: {where}: cut not found: {part[:60]!r}")


def relationship_lines(card, cards):
    return [f"- {cards[r['with']]['name']}: {fill(r['static'])}" for r in card.get("relationships", [])]


def minimal_card(cid, card, spec, cards):
    scenario = card["scenario"]
    secrets = []
    for secret in spec["secrets"]:
        must_find(secret["cut"], scenario, cid)
        scenario = scenario.replace(secret["cut"], "")
        secrets.append(secret.get("text", secret["cut"].strip()))
    for drop in spec["drop"]:
        must_find(drop, scenario, cid)
        scenario = scenario.replace(drop, "")
    lines = [f"--- {card['name']} ---", f"Description: {fill(card['description'])}"]
    lines.append(f"Personality: {fill(card['personality'])}")
    lines.append(f"Scenario: {fill(scenario.strip())}")
    if secrets:
        lines.append("Secrets: " + fill(" ".join(secrets)))
    rel = relationship_lines(card, cards)
    if rel:
        lines.append("Relationships:\n" + "\n".join(rel))
    lines.append(
        "Example dialogue (an example of her voice, not an event of the story):\n"
        + fill(card["example_dialogue"].strip())
    )
    return "\n".join(lines)


def sentence(part):
    """A cut that ends inside a sentence gets a full stop, so the cuts read as sentences."""
    part = part.strip()
    return part if part[-1] in ".!?" else part + "."


def split_card(cid, card, spec, cards):
    source = card["description"] + "\n" + card["scenario"]
    for part in spec["facts"] + spec["internals"] + spec["guide"]:
        must_find(part, source, cid)
    appearance = card["description"].split("Appearance:", 1)[1].strip()
    name = card["name"]
    lines = [f"--- {name} ---", f"Seen by anyone who meets her: {fill(appearance)}"]
    lines.append(
        f"Known to the cast ({USER} learns these in play): "
        + " ".join(sentence(fill(p)) for p in spec["facts"])
    )
    internals = [sentence(fill(p)) for p in spec["internals"]]
    rel = relationship_lines(card, cards)
    if internals or rel:
        text = f"Known only to {name}:" + (" " + " ".join(internals) if internals else "")
        if rel:
            text += f"\n{name}'s view of the others:\n" + "\n".join(rel)
        lines.append(text.rstrip())
    guide = " ".join([f"Personality: {card['personality']}"] + [sentence(p) for p in spec["guide"]])
    lines.append(f"Writing guide (how to write her; not facts of the story): {fill(guide)}")
    lines.append(
        "Example dialogue (an example of her voice, not an event of the story):\n"
        + fill(card["example_dialogue"].strip())
    )
    return "\n".join(lines)


def card_block(arm):
    spec = json.loads(CARDS.read_text(encoding="utf-8"))
    cards = load_cards()
    if arm in ("minimal", "minimal_rule"):
        parts = [minimal_card(c, cards[c], spec["minimal"][c], cards) for c in ORDER]
    else:
        parts = [split_card(c, cards[c], spec["split"][c], cards) for c in ORDER]
    head = fill(spec["rule_text"]) + "\n\n" if arm in ("minimal_rule", "split") else ""
    return "<CharacterCards>\n" + head + "\n\n".join(parts) + "\n</CharacterCards>"


def judge_card(cid, cards):
    """The full card as the author wrote it, for the judge."""
    card = cards[cid]
    rel = relationship_lines(card, cards)
    return "\n".join(
        [f"--- {card['name']} ---", fill(card["description"]), f"Personality: {card['personality']}",
         f"Scenario: {fill(card['scenario'])}"]
        + (["Relationships:"] + rel if rel else [])
    )


# ---------------------------------------------------------------- the pack and the requests


class Pack:
    def __init__(self, folder):
        self.folder = Path(folder)
        self.story = trial.read_story(self.folder)
        self.calls = {}
        for line in (self.folder / "calls.jsonl").read_text(encoding="utf-8").splitlines():
            call = json.loads(line)
            self.calls[call["id"]] = call

    def call(self, call_id):
        call = self.calls[call_id]
        trial.read_prompts(call)
        return call

    def owner(self, call_id):
        return trial.owner_entry(self.call(call_id), self.story)

    def before(self, call_id):
        """The story entries before the entry this call wrote."""
        owner = self.owner(call_id)
        return [e for e in self.story if e["id"] < owner]


def request(pack, scene, arm):
    """The messages of one scene and arm."""
    call = pack.call(scene["call"])
    body = json.loads(call["raw_request_json"])
    messages = [dict(m) for m in body["messages"]]
    last = messages[-1]
    if scene.get("input") is not None:
        if not LAST_RE.search(last["content"]):
            raise SystemExit(f"{scene['id']}: the call has no <LastMessage>")
        new = "<LastMessage>\n" + scene["input"] + "\n</LastMessage>"
        last["content"] = LAST_RE.sub(lambda _m: new, last["content"], count=1)
    if scene.get("direction") is not None:
        if LAST_RE.search(last["content"]):
            raise SystemExit(f"{scene['id']}: the Impersonate call already has a <LastMessage>")
        last["content"] = "<LastMessage>\n" + scene["direction"] + "\n</LastMessage>\n\n" + last["content"]
    if arm != "today":
        holders = [m for m in messages if KNOWN_RE.search(m["content"])]
        if len(holders) != 1:
            raise SystemExit(f"{scene['id']}: expected one <KnownCharacters>, found {len(holders)}")
        block = card_block(arm)
        holders[0]["content"] = KNOWN_RE.sub(lambda _m: block, holders[0]["content"], count=1)
        match = PRESENT_RE.search(last["content"])
        if match:
            names = re.findall(r"^--- (.+?) ---$", match.group(1), re.M)
            present = "<CharactersPresent>" + (", ".join(names) if names else "nobody") + "</CharactersPresent>"
            last["content"] = PRESENT_RE.sub(lambda _m: present, last["content"], count=1)
    return messages, body.get("max_tokens", 2048)


# ---------------------------------------------------------------- the judge

JUDGE = """You check one passage of an interactive story for three kinds of fault. You do not
grade style. Quote the passage word for word for every finding.

## The passage

It was written by {writer}. The scene so far ends with this:

<scene_end>
{scene_end}
</scene_end>

<passage>
{passage}
</passage>

## 1. Facts nobody in the scene has learned

At this point of the story, these facts are written only on the author's cards. No Character
has learned them in the story yet, except the owner and the ones listed as "may know".
Julian is the player character; he knows only the World rules, his own past and what he saw.
The World rules, which every Character knows:

{world}

{facts}

For each fact, say whether the passage uses it:
- "by": who uses it: "Julian", a Character's name, or "narrator" (the narrating voice, outside
  any Character's speech or thought).
- "how": "states" (says or thinks it as a known fact, or acts on it as known), "guesses"
  (a hedged guess that the Character could make from what she saw or what the World rules
  say), or "asks" (asks about it without knowing the answer).
A fact is used only when the passage gives its specific content. A claim that follows from the
World rules or from what the Character saw in the story is not a use of the fact (for
example: that Gabriella is manipulative, wants something, or was left out of the will). A
Character who speaks of her own fact is not a finding. Leave a fact out when the passage does
not touch it.

## 2. Contradictions of a card

The author's cards of the Characters below. A finding is a portrayal, a description or a
mention of one of these Characters that contradicts her card: her appearance, her age, her
past, her role, her surname, her relationships. A detail the card does not cover is not a
contradiction.

{cards}

## 3. A repeated first meeting

Julian has already met: {met}. A finding is one of these Characters who introduces herself
to Julian as at a first meeting, or greets him as a stranger.

## The answer

Reply with JSON only, in this form:
{{"facts": [{{"fact": "<id>", "by": "...", "how": "states|guesses|asks", "quote": "..."}}],
 "contradictions": [{{"character": "<name>", "quote": "...", "card": "<the card text it contradicts>"}}],
 "repeated_meeting": [{{"character": "<name>", "quote": "..."}}]}}
"""


def norm(text):
    text = text.replace("\u2019", "'").replace("\u2018", "'").replace("\u201c", '"').replace("\u201d", '"')
    text = text.replace("\u2014", "-").replace("\u2013", "-").replace("…", "...")
    return re.sub(r"\s+", " ", text).strip().lower()


def quoted(quote, passage):
    q = norm(quote).strip(" .\"'")
    return bool(q) and q in norm(passage)


def scene_end(pack, scene, messages):
    if scene.get("input"):
        return f"Julian: {scene['input']}"
    if scene.get("direction"):
        text = pack.before(scene["call"])[-1]["text"]
        return f"{text.strip()}\n\n(The player asked: {scene['direction']})"
    return pack.before(scene["call"])[-1]["text"].strip()


def judge_prompt(pack, scene, passage, facts, cards):
    writer = (
        "the player, as Julian (an Impersonate turn: every word is Julian's)"
        if scene["kind"] == "impersonate"
        else "the narrator, who writes the world and every Character except Julian"
    )
    fact_lines = []
    for fid in scene["facts"]:
        fact = facts[fid]
        may = [cards[fact["owner"]]["name"]] + [cards[c]["name"] for c in fact.get("known_to", [])]
        fact_lines.append(f"- {fid}: {fact['text']} (owner: {may[0]}; may know: {', '.join(may)})")
    focus = sorted(set(scene["focus"]) | {facts[f]["owner"] for f in scene["facts"]}, key=ORDER.index)
    system = json.loads(pack.call(scene["call"])["raw_request_json"])["messages"][0]["content"]
    world = re.search(r"<global_rules>(.*?)</global_rules>", system, re.S)
    return JUDGE.format(
        writer=writer,
        world=world.group(1).strip() if world else "(none)",
        scene_end=scene_end(pack, scene, None)[-1500:],
        passage=passage,
        facts="\n".join(fact_lines),
        cards="\n\n".join(judge_card(c, cards) for c in focus),
        met=", ".join(cards[c]["name"] for c in scene["met"]),
    )


def parse_json(text):
    """The last JSON answer of the judge: it sometimes corrects a first answer in the same reply."""
    text = text.split("</think>")[-1] if "</think>" in text and '{"facts"' in text.split("</think>")[-1] else text
    decoder = json.JSONDecoder()
    found = None
    for start in [m.start() for m in re.finditer(r'\{\s*"facts"', text)]:
        try:
            found, _ = decoder.raw_decode(text[start:])
        except json.JSONDecodeError:
            continue
    return found


def score(scene, passage, verdict, facts, cards):
    """The counted findings of one answer: leaks, stated secrets, contradictions, meetings."""
    out = {"leaks": [], "stated": [], "guesses": [], "contradictions": [], "meetings": [],
           "rule_leaks": [], "unquoted": 0, "judge_failed": verdict is None}
    verdict = verdict or {}
    names = {cards[c]["name"].lower(): c for c in ORDER}
    for item in verdict.get("facts", []) or []:
        fid = item.get("fact")
        if fid not in scene["facts"]:
            continue
        if not quoted(item.get("quote", ""), passage):
            out["unquoted"] += 1
            continue
        fact = facts[fid]
        by = (item.get("by") or "").strip().lower()
        how = (item.get("how") or "").strip().lower()
        allowed = {fact["owner"], *fact.get("known_to", [])}
        finding = {"fact": fid, "by": item.get("by"), "quote": item.get("quote")}
        if by == "narrator" or (scene["kind"] != "impersonate" and by not in names and by != "julian"):
            if fact["secret"] and how == "states":
                out["stated"].append(finding)
            continue
        if scene["kind"] == "impersonate":
            by = "julian"
        if by == "julian" or names.get(by) not in allowed:
            if how == "states":
                out["leaks"].append(finding)
            elif how == "guesses":
                out["guesses"].append(finding)
    for item in verdict.get("contradictions", []) or []:
        if quoted(item.get("quote", ""), passage):
            out["contradictions"].append(item)
        else:
            out["unquoted"] += 1
    met = {cards[c]["name"].lower() for c in scene["met"]}
    for item in verdict.get("repeated_meeting", []) or []:
        if (item.get("character") or "").strip().lower() not in met:
            continue
        if quoted(item.get("quote", ""), passage):
            out["meetings"].append(item)
        else:
            out["unquoted"] += 1
    if scene["kind"] == "impersonate":
        for fid in scene["facts"]:
            pattern = facts[fid].get("pattern")
            if pattern and re.search(pattern, passage, re.I):
                out["rule_leaks"].append(fid)
    return out


# ---------------------------------------------------------------- the runs


def load_scenes():
    data = json.loads(SCENES.read_text(encoding="utf-8"))
    return data["facts"], data["scenes"]


def run_arm(pack, arm, run, args):
    facts, scenes = load_scenes()
    if args.only:
        scenes = [s for s in scenes if s["id"] in args.only.split(",")]
    cards = load_cards()
    out = Path(args.out) / "runs" / f"{arm}-r{run}.jsonl"
    out.parent.mkdir(parents=True, exist_ok=True)
    done = set()
    if out.exists():
        done = {json.loads(line)["scene"] for line in out.read_text(encoding="utf-8").splitlines()}
    todo = [s for s in scenes if s["id"] not in done]
    tag = uuid.uuid4().hex[:8]
    model = Model(args.model, args.temperature, f"ltm11-{arm}-r{run}-{tag}")
    judge = Model(args.judge_model, 0.0, f"ltm11-judge-{arm}-r{run}-{tag}")
    lock = threading.Lock()

    def one(scene):
        messages, max_tokens = request(pack, scene, arm)
        passage = model.call(messages, max_tokens)
        prompt = judge_prompt(pack, scene, passage, facts, cards)
        raw = judge.call([{"role": "user", "content": prompt}], 1500)
        verdict = parse_json(raw)
        row = {"scene": scene["id"], "kind": scene["kind"], "arm": arm, "run": run,
               "passage": passage, "verdict": verdict, "judge_raw": None if verdict else raw,
               "score": score(scene, passage, verdict, facts, cards)}
        with lock, out.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(row) + "\n")

    with concurrent.futures.ThreadPoolExecutor(args.workers) as pool:
        for future in concurrent.futures.as_completed([pool.submit(one, s) for s in todo]):
            try:
                future.result()
            except Exception as error:  # noqa: BLE001  one failed scene must not stop the run
                print(f"  failed: {error}", file=sys.stderr)
    cost_file = out.with_suffix(".cost.json")
    old = json.loads(cost_file.read_text()) if cost_file.exists() else {}
    for name, m in (("model", model), ("judge", judge)):
        old[name] = {k: old.get(name, {}).get(k, 0) + v for k, v in m.cost.items()}
    cost_file.write_text(json.dumps(old), encoding="utf-8")
    print(f"{out.name}: {len(todo)} scenes, model {model.cost}, judge {judge.cost}")


def rescore(args):
    """Score the stored verdicts again after a change of the score rules."""
    facts, scenes = load_scenes()
    by_id = {s["id"]: s for s in scenes}
    cards = load_cards()
    for path in sorted((Path(args.out) / "runs").glob("*.jsonl")):
        rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
        for row in rows:
            if row["verdict"] is None and row.get("judge_raw"):
                row["verdict"] = parse_json(row["judge_raw"])
            row["score"] = score(by_id[row["scene"]], row["passage"], row["verdict"], facts, cards)
        path.write_text("".join(json.dumps(r) + "\n" for r in rows), encoding="utf-8")
    print("rescored")


# ---------------------------------------------------------------- the report


def read_rows(out):
    rows, costs = {}, {}
    for path in sorted((Path(out) / "runs").glob("*.jsonl")):
        match = re.match(r"(.+)-r(\d+)$", path.stem)
        if not match:
            continue
        arm, run = match.group(1), int(match.group(2))
        rows.setdefault(arm, {})[run] = [json.loads(l) for l in path.read_text(encoding="utf-8").splitlines()]
        cost = path.with_suffix(".cost.json")
        if cost.exists():
            costs.setdefault(arm, {})[run] = json.loads(cost.read_text())
    return rows, costs


def spread(values):
    if not values:
        return "-"
    return f"{statistics.mean(values):.0f} % ({min(values):.0f}-{max(values):.0f})"


def report(args):
    rows, costs = read_rows(args.out)
    arms = [a for a in ARMS if a in rows]
    measures = (
        ("leaks", "answers with a card leak"),
        ("stated", "answers where the narrator states a secret"),
        ("contradictions", "answers with a card contradiction"),
        ("meetings", "answers with a repeated first meeting"),
        ("guesses", "answers with a guess at a card fact"),
    )
    for kinds, label in ((("impersonate",), "Impersonate"), (("narration", "event"), "Narration and event")):
        print(f"## {label} scenes\n")
        print("Share of answers with at least one finding, mean over the runs (min-max).\n")
        print("| Arm | Runs | " + " | ".join(m[1] for m in measures) + " | findings (leak / contradiction) | rule hits (pattern) | unquoted | judge failed |")
        print("|---|---|" + "---|" * (len(measures) + 4))
        for arm in arms:
            cells = []
            per_run = rows[arm]
            for key, _ in measures:
                values = []
                for run_rows in per_run.values():
                    sel = [r for r in run_rows if r["kind"] in kinds]
                    if sel:
                        values.append(100 * sum(bool(r["score"][key]) for r in sel) / len(sel))
                cells.append(spread(values))
            sel_all = [r for run_rows in per_run.values() for r in run_rows if r["kind"] in kinds]
            leaks = sum(len(r["score"]["leaks"]) for r in sel_all)
            contra = sum(len(r["score"]["contradictions"]) for r in sel_all)
            rule = sum(len(r["score"]["rule_leaks"]) for r in sel_all)
            unq = sum(r["score"]["unquoted"] for r in sel_all)
            failed = sum(r["score"]["judge_failed"] for r in sel_all)
            n = len(sel_all)
            print(f"| {arm} | {len(per_run)} ({n} answers) | " + " | ".join(cells) + f" | {leaks} / {contra} | {rule} | {unq} | {failed} |")
        print()
    print("## Cost per run\n")
    print("| Arm | Answer calls | Prompt tokens (cached %) | Completion | Judge prompt tokens |")
    print("|---|---|---|---|---|")
    for arm in arms:
        runs = list(costs.get(arm, {}).values())
        if not runs:
            continue
        m = [r["model"] for r in runs]
        j = [r["judge"] for r in runs]
        prompt = statistics.mean(c["prompt"] for c in m)
        cached = statistics.mean(c["cached"] for c in m)
        print(f"| {arm} | {statistics.mean(c['calls'] for c in m):.0f} | {prompt:,.0f} ({100 * cached / prompt if prompt else 0:.0f} %) | "
              f"{statistics.mean(c['completion'] for c in m):,.0f} | {statistics.mean(c['prompt'] for c in j):,.0f} |")
    print()
    if args.detail:
        print("## Findings per scene\n")
        print("Each cell: the runs of the arm; L = leak, S = stated secret, C = contradiction, M = repeated meeting, . = none.\n")
        _, scenes = load_scenes()
        print("| Scene | " + " | ".join(arms) + " |")
        print("|---|" + "---|" * len(arms))
        for scene in scenes:
            cells = []
            for arm in arms:
                marks = []
                for run in sorted(rows[arm]):
                    row = next((r for r in rows[arm][run] if r["scene"] == scene["id"]), None)
                    if row is None:
                        marks.append("·")
                        continue
                    s = row["score"]
                    mark = "".join(k for k, key in (("L", "leaks"), ("S", "stated"), ("C", "contradictions"), ("M", "meetings")) if s[key])
                    marks.append(mark or ".")
                cells.append(" ".join(marks))
            print(f"| {scene['id']} | " + " | ".join(cells) + " |")
        print()
    if args.findings:
        for arm in arms:
            for run in sorted(rows[arm]):
                for row in rows[arm][run]:
                    s = row["score"]
                    for key in ("leaks", "stated", "contradictions", "meetings"):
                        for f in s[key]:
                            print(f"- {arm} r{run} {row['scene']} {key}: {json.dumps(f, ensure_ascii=False)}")


# ---------------------------------------------------------------- the commands


def check(pack):
    """Warn when a fact of a scene is already in the story before its call, or in its input."""
    facts, scenes = load_scenes()
    for scene in scenes:
        before = pack.before(scene["call"])
        text = "\n".join(e["text"] for e in before) + "\n" + (scene.get("input") or "")
        owner = pack.owner(scene["call"])
        warn = []
        for fid in scene["facts"]:
            pattern = facts[fid].get("learned_if")
            hits = [e["id"] for e in before if pattern and re.search(pattern, e["text"], re.I)]
            if pattern and re.search(pattern, scene.get("input") or "", re.I):
                hits.append("input")
            if hits:
                warn.append(f"{fid} at {hits}")
        for arm in ARMS:
            request(pack, scene, arm)
        print(f"{scene['id']:4} call {scene['call']} writes #{owner} ({scene['kind']})" + (f"  ** learned? {'; '.join(warn)}" if warn else ""))
        del text


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    run = sub.add_parser("run", help="run one arm and judge its answers")
    run.add_argument("arm", choices=ARMS)
    run.add_argument("--runs", type=int, default=1)
    run.add_argument("--first-run", type=int, default=1)
    run.add_argument("--only", help="comma-separated scene ids")
    run.add_argument("--pack", default=str(DEFAULT_PACK))
    run.add_argument("--out", default=str(DEFAULT_OUT))
    run.add_argument("--model", default="deepseek-v4.1-flash")
    run.add_argument("--judge-model", default="glm-5.3-flash")
    run.add_argument("--temperature", type=float, default=0.0)
    run.add_argument("--workers", type=int, default=6)
    rep = sub.add_parser("report", help="the rates and the cost of every arm run so far")
    rep.add_argument("--out", default=str(DEFAULT_OUT))
    rep.add_argument("--detail", action="store_true")
    rep.add_argument("--findings", action="store_true")
    res = sub.add_parser("rescore", help="score the stored verdicts again")
    res.add_argument("--out", default=str(DEFAULT_OUT))
    chk = sub.add_parser("check", help="the facts of each scene against the story so far")
    chk.add_argument("--pack", default=str(DEFAULT_PACK))
    show = sub.add_parser("show", help="print the request of one scene and arm")
    show.add_argument("scene")
    show.add_argument("arm", choices=ARMS)
    show.add_argument("--pack", default=str(DEFAULT_PACK))
    show.add_argument("--judge", action="store_true", help="print the judge prompt for a dummy passage")
    args = parser.parse_args()

    if args.command == "report":
        report(args)
        return
    if args.command == "rescore":
        rescore(args)
        return
    pack = Pack(args.pack)
    if args.command == "check":
        check(pack)
        return
    if args.command == "show":
        facts, scenes = load_scenes()
        scene = next(s for s in scenes if s["id"] == args.scene)
        if args.judge:
            print(judge_prompt(pack, scene, "(passage)", facts, load_cards()))
            return
        messages, _ = request(pack, scene, args.arm)
        for m in messages:
            if "<ConversationHistory>" in m["content"] and "</ConversationHistory>" not in m["content"]:
                print(f"== {m['role']} (history start) {m['content'][:200]!r}")
                continue
            if m["role"] == "assistant" or (m["role"] == "user" and not m["content"].lstrip().startswith("<")):
                continue
            print(f"== {m['role']}\n{m['content']}\n")
        return
    for number in range(args.first_run, args.first_run + args.runs):
        run_arm(pack, args.arm, number, args)


if __name__ == "__main__":
    main()
