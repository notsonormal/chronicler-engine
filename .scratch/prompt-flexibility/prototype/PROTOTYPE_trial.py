"""PROTOTYPE — throwaway. Run one trial through Autoplay and export its reading pack.

Answers the question of "Shape the test setup and the trial" (issues/08-test-setup-and-trial.md):
what a trial fixes, saves and shows. Delete it once the build plan absorbs the answer.

    python .scratch/prompt-flexibility/prototype/PROTOTYPE_trial.py run --cycles 6 --out tmp/pf08/trial-a
    python .scratch/prompt-flexibility/prototype/PROTOTYPE_trial.py export tmp/pf08/trial-a
    python .scratch/prompt-flexibility/prototype/PROTOTYPE_trial.py compare tmp/pf08/trial-a tmp/pf08/trial-b

`run` needs a probe server (see /chronicler-ui-investigator) and reads its DB directly.
`llm_messages` keeps only 50 rows, so `run` copies each new row out while the trial plays.
"""

import argparse
import json
import re
import sqlite3
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

DEFAULT_DB = "target/ui-investigator/debug/chronicler_3001.db"


def http(base, method, path, form=None):
    data = urllib.parse.urlencode(form).encode() if form is not None else None
    req = urllib.request.Request(base + path, data=data, method=method)
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return resp.status, resp.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()


def rows(conn, sql, args=()):
    conn.row_factory = sqlite3.Row
    return [dict(r) for r in conn.execute(sql, args)]


def usage_of(call):
    try:
        resp = json.loads(call["raw_response_json"].strip() or "{}")
    except json.JSONDecodeError:
        return {}
    u = resp.get("usage") or {}
    details = u.get("prompt_tokens_details") or {}
    return {
        "prompt": u.get("prompt_tokens", 0) or 0,
        "completion": u.get("completion_tokens", 0) or 0,
        "cached": details.get("cached_tokens", 0) or 0,
        "cost": u.get("cost", 0) or 0,
        "allowance": (u.get("opencode") or {}).get("allowance"),
        "host": resp.get("provider", ""),
        "finish": ((resp.get("choices") or [{}])[0] or {}).get("finish_reason"),
    }


def snapshot_setup(conn, game_id, cycles, token_limit):
    settings = rows(conn, "select * from settings")[0]
    game = rows(conn, "select * from games where id=?", (game_id,))[0]
    conns = {c["id"]: c for c in json.loads(settings["connections"])}
    preset_ids = {
        "system": game["active_system_prompt_preset_id"],
        "impersonate": game["active_impersonate_prompt_preset_id"],
        "quantifier": game["active_quantifier_prompt_preset_id"],
        "options": game["active_options_prompt_preset_id"],
    }
    presets = {}
    for role, pid in preset_ids.items():
        found = rows(conn, "select * from prompt_presets where id=?", (pid,))
        presets[role] = found[0] if found else {"id": pid, "missing": True}
    return {
        "game_id": game_id,
        "world": game["world_key"],
        "persona": game["persona_key"],
        "narrator_mode": game["narrator_mode"],
        "posture": f'{game["narrative_perspective"]} person, {game["narrative_tense"]} tense',
        "options_always_on": bool(game["options_always_on"]),
        "cycles": cycles,
        "token_limit": token_limit,
        "response_length": settings["response_length"],
        "narration_connection": conns.get(settings["narration_connection_id"]),
        "quantifier_connection": conns.get(settings["quantifier_connection_id"]),
        "agents": json.loads(settings["agents"]),
        "presets": presets,
    }


def cmd_run(a):
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(a.db)
    before = conn.execute("select coalesce(max(id),0) from games").fetchone()[0]
    status, body = http(a.base, "POST", "/games", {"world_key": a.world, "persona_key": a.persona})
    if status != 200:
        sys.exit(f"POST /games -> {status} {body[:200]}")
    game_id = conn.execute("select max(id) from games").fetchone()[0]
    assert game_id > before, "no new Game"
    http(a.base, "POST", f"/games/{game_id}/switch", {})
    setup = snapshot_setup(conn, game_id, a.cycles, a.token_limit)
    (out / "setup.json").write_text(json.dumps(setup, indent=2))

    last_call = conn.execute("select coalesce(max(id),0) from llm_messages").fetchone()[0]
    calls_path = out / "calls.jsonl"
    calls_path.write_text("")
    started = time.time()
    status, body = http(a.base, "POST", "/autoplay", {"cycles": str(a.cycles)})
    if status != 200:
        sys.exit(f"POST /autoplay -> {status} {body[:200]}")
    print(f"Game {game_id}: Autoplay {a.cycles} cycles started")

    tokens = 0
    stopped_by_limit = False
    idle_polls = 0
    while True:
        time.sleep(2)
        new = rows(conn, "select * from llm_messages where id>? order by id", (last_call,))
        with calls_path.open("a") as f:
            for call in new:
                f.write(json.dumps(call) + "\n")
                last_call = call["id"]
                u = usage_of(call)
                tokens += u.get("prompt", 0) + u.get("completion", 0)
                flag = f' FAILED: {call["error_message"][:80]}' if call["error_message"] else ""
                print(f'  call {call["id"]} {call["agent_name"]} {u.get("prompt",0)}+{u.get("completion",0)} tok, cached {u.get("cached",0)}{flag}')
        if tokens > a.token_limit and not stopped_by_limit:
            print(f"Token limit {a.token_limit} passed ({tokens}); stopping the run")
            http(a.base, "POST", "/status/reset-busy", {})
            stopped_by_limit = True
        _, busy = http(a.base, "GET", "/debug/is_busy")
        idle_polls = idle_polls + 1 if busy.strip() == "false" else 0
        if idle_polls >= 2:
            break
        if time.time() - started > a.timeout:
            print("Timeout; stopping the run")
            http(a.base, "POST", "/status/reset-busy", {})
            break

    story = rows(
        conn,
        """select m.id, m.message_type, m.kind, m.timestamp, s.swipe_index, s.text, s.event_header,
                  s.impersonated, s.state, s.maker
           from messages m join message_swipes s on s.message_id=m.id and s.swipe_index=m.active_swipe_index
           where m.game_id=? order by m.id""",
        (game_id,),
    )
    game = rows(conn, "select failure from games where id=?", (game_id,))[0]
    run = {
        "started": started,
        "seconds": round(time.time() - started),
        "stopped_by_limit": stopped_by_limit,
        "game_failure": game["failure"],
    }
    (out / "story.json").write_text(json.dumps(story, indent=2))
    (out / "run.json").write_text(json.dumps(run, indent=2))
    cmd_export(argparse.Namespace(pack=str(out)))


# ---------- export ----------

def words(text):
    return len(text.split())


def call_kind(call, story):
    """Name a call by the story entry its answer became; fall back to the agent name."""
    if call["agent_name"] != "narrator":
        return {"trigger": "event-narration"}.get(call["agent_name"], call["agent_name"])
    answer = (call["parsed_response"] or "").strip()[:120]
    for entry in story:
        if answer and answer in entry["text"]:
            return "impersonate" if entry["impersonated"] else "narration"
    if "player character" in call["system_prompt"]:
        return "impersonate"
    return "narration"


def owner_entry(call, story):
    """The story entry a call made or judged, found by its text; else the last entry before it."""
    if call["agent_name"] == "quantifier":
        m = re.search(r"<LatestNarration>\s*[^:\n]*:\s*(.*?)</LatestNarration>", call["user_prompt"], re.S)
        probe = m.group(1).strip()[:100] if m else ""
    else:
        probe = (call["parsed_response"] or "").strip()[:100]
    if probe:
        for e in story:
            if probe in e["text"]:
                return e["id"]
    before = [e for e in story if e["timestamp"] <= call["created_at"]]
    return before[-1]["id"] if before else None


def history_block(user_prompt):
    m = re.search(r"<ConversationHistory>(.*?)</ConversationHistory>", user_prompt, re.S)
    return m.group(1) if m else None


def history_cover(call, story_before):
    """Which earlier story entries the call's <ConversationHistory> holds."""
    block = history_block(call["user_prompt"])
    if block is None:
        return None
    held = [e["id"] for e in story_before if e["text"].strip()[:100] in block]
    dropped = [e["id"] for e in story_before if e["id"] not in held]
    return {"held": held, "dropped": dropped, "chars": len(block)}


def state_line(entry):
    try:
        st = json.loads(entry["state"])
    except (TypeError, json.JSONDecodeError):
        return "?", []
    room = (st.get("movement") or {}).get("current_room_id", "?")
    present = [n.get("id") for n in (st.get("scene") or {}).get("npcs_in_area", [])]
    return room, present


def cmd_export(a):
    pack = Path(a.pack)
    setup = json.loads((pack / "setup.json").read_text())
    story = json.loads((pack / "story.json").read_text())
    for e in story:
        e["message_type"] = json.loads(e["message_type"]) if e["message_type"].startswith('"') else e["message_type"]
    run = json.loads((pack / "run.json").read_text())
    calls = [json.loads(l) for l in (pack / "calls.jsonl").read_text().splitlines() if l.strip()]
    calls_dir = pack / "calls"
    calls_dir.mkdir(exist_ok=True)

    # Attach each call to the first story entry stored after it (the Job that made it).
    by_entry = {e["id"]: [] for e in story}
    trailing = []
    for call in calls:
        call["kind"] = call_kind(call, story)
        call["usage"] = usage_of(call)
        before = [e for e in story if e["timestamp"] <= call["created_at"]]
        call["cover"] = history_cover(call, before) if call["kind"] in ("narration", "impersonate", "event-narration") else None
        owner = owner_entry(call, story)
        (by_entry[owner] if owner is not None else trailing).append(call)
        name = f'{call["id"]:05d}-{call["kind"]}.md'
        call["file"] = f"calls/{name}"
        u = call["usage"]
        (calls_dir / name).write_text(
            f'# Call {call["id"]} — {call["kind"]} ({call["model_name"]} via {u.get("host") or call["backend_name"]})\n\n'
            f'{call["created_at"]}. Tokens: {u.get("prompt")} in, {u.get("completion")} out, {u.get("cached")} cached. '
            f'Finish: {u.get("finish")}. Allowance: {u.get("allowance")}. Cost: {u.get("cost")}.\n\n'
            + (f'**FAILED:** {call["error_message"]}\n\n' if call["error_message"] else "")
            + f'## System\n\n```text\n{call["system_prompt"]}\n```\n\n## User\n\n```text\n{call["user_prompt"]}\n```\n\n'
            f'## Answer\n\n```text\n{call["parsed_response"]}\n```\n'
        )

    kinds = {}
    for c in calls:
        k = kinds.setdefault(c["kind"], {"n": 0, "failed": 0, "in": 0, "out": 0, "cached": 0})
        k["n"] += 1
        k["failed"] += 1 if c["error_message"] else 0
        k["in"] += c["usage"].get("prompt", 0)
        k["out"] += c["usage"].get("completion", 0)
        k["cached"] += c["usage"].get("cached", 0)
    tot_in = sum(k["in"] for k in kinds.values())
    tot_out = sum(k["out"] for k in kinds.values())
    tot_cached = sum(k["cached"] for k in kinds.values())
    hosts = sorted({c["usage"].get("host") for c in calls if c["usage"].get("host")})
    inputs = [e for e in story if e["message_type"] == "Input"]
    allowances = [c["usage"]["allowance"] for c in calls if c["usage"].get("allowance") is not None]

    nconn = setup["narration_connection"] or {}
    qconn = setup["quantifier_connection"] or {}
    L = []
    L.append(f'# Reading pack — {pack.name}\n')
    L.append("## Test setup\n")
    L.append("| Part | Value |\n|---|---|")
    L.append(f'| World / Persona | `{setup["world"]}` / `{setup["persona"]}` |')
    L.append(f'| Mode, posture | {setup["narrator_mode"]}, {setup["posture"]}; Options always on: {setup["options_always_on"]} |')
    L.append(f'| Narration connection | {nconn.get("provider")} `{nconn.get("model")}` reasoning={nconn.get("reasoning_effort")} max_tokens={nconn.get("max_tokens")} |')
    L.append(f'| Quantifier connection | {qconn.get("provider")} `{qconn.get("model")}` reasoning={qconn.get("reasoning_effort")} |')
    for role, p in setup["presets"].items():
        L.append(f'| {role} preset | `{p.get("id")}` ({p.get("name", "?")}) |')
    L.append(f'| response_length | {setup["response_length"][:90]}… |')
    L.append(f'| Budget | {setup["cycles"]} cycles, token limit {setup["token_limit"]:,} |')
    L.append("")
    L.append("## Summary\n")
    L.append(f'- Cycles played: {len(inputs)} of {setup["cycles"]}. Story entries: {len(story)}. Wall time: {run["seconds"]} s.')
    L.append(f'- Stopped by the token limit: {run["stopped_by_limit"]}. Game failure at the end: {run["game_failure"] or "none"}.')
    L.append(f'- Tokens: {tot_in:,} in, {tot_out:,} out, {tot_cached:,} cached ({(100*tot_cached/tot_in if tot_in else 0):.0f} % of input). Hosts: {", ".join(hosts) or "?"}.')
    if allowances:
        L.append(f'- OpenCode Go allowance, first and last call: {allowances[0]} → {allowances[-1]}.')
    L.append("")
    L.append("| Call kind | Calls | Failed | Tokens in | Tokens out | Cached |\n|---|---|---|---|---|---|")
    for name, k in sorted(kinds.items()):
        L.append(f'| {name} | {k["n"]} | {k["failed"]} | {k["in"]:,} | {k["out"]:,} | {k["cached"]:,} |')
    L.append("")
    failed = [c for c in calls if c["error_message"]]
    if failed:
        L.append("### Failed calls\n")
        for c in failed:
            L.append(f'- [{c["kind"]} {c["id"]}]({c["file"]}): {c["error_message"][:200]} (finish_reason `{c["usage"].get("finish")}`)')
        L.append("")
    first_cut = next((c for c in calls if c["cover"] and c["cover"]["dropped"]), None)
    L.append("### History cut\n")
    if first_cut:
        L.append(f'- First cut: [call {first_cut["id"]}]({first_cut["file"]}) drops {len(first_cut["cover"]["dropped"])} entries.')
    else:
        last = next((c for c in reversed(calls) if c["cover"]), None)
        size = f'; the last history block held {last["cover"]["chars"]:,} characters' if last else ""
        L.append(f"- No story entry fell out of `<ConversationHistory>` in this trial{size}.")
    L.append("")
    L.append("## Story\n")
    L.append("Each entry shows the Room and the Characters present of its Swipe state (Δ marks a change), then the call that wrote it and the Quantifier call that judged it. A failed call with no entry is listed under the last entry before it.\n")
    prev_room, prev_present = None, None
    for e in story:
        label = "Impersonate Input" if e["impersonated"] else e["message_type"]
        if e["event_header"]:
            label += f' — {e["event_header"]}'
        L.append(f'### #{e["id"]} {label} ({words(e["text"])} words)\n')
        L.append(e["text"].strip() + "\n")
        room, present = state_line(e)
        mark = " Δ" if (room, present) != (prev_room, prev_present) and prev_room is not None else ""
        L.append(f'> State: room `{room}`, present {present or "nobody"}{mark}')
        prev_room, prev_present = room, present
        for c in by_entry.get(e["id"], []):
            u = c["usage"]
            cover = ""
            if c["cover"]:
                cover = f', history holds {len(c["cover"]["held"])} entries' + (f' (dropped {len(c["cover"]["dropped"])})' if c["cover"]["dropped"] else "")
            fail = f' — **FAILED** {c["error_message"][:120]} (finish_reason `{u.get("finish")}`)' if c["error_message"] else ""
            answer = ""
            if c["kind"] in ("quantifier", "options") and not c["error_message"]:
                answer = f' → `{(c["parsed_response"] or "").strip()[:160]}`'
            L.append(f'> - [{c["kind"]} {c["id"]}]({c["file"]}): {u.get("prompt")}+{u.get("completion")} tok, cached {u.get("cached")}{cover}{fail}{answer}')
        L.append("")
    if trailing:
        L.append("### Calls after the last entry\n")
        for c in trailing:
            fail = f' — **FAILED** {c["error_message"][:120]} (finish_reason `{c["usage"].get("finish")}`)' if c["error_message"] else ""
            L.append(f'- [{c["kind"]} {c["id"]}]({c["file"]}){fail}')
    (pack / "pack.md").write_text("\n".join(L) + "\n")
    summary = {
        "cycles": len(inputs), "entries": len(story), "seconds": run["seconds"],
        "tokens_in": tot_in, "tokens_out": tot_out, "cached": tot_cached,
        "kinds": kinds, "failed": len(failed),
        "first_cut_call": first_cut["id"] if first_cut else None,
        "words_narration": [words(e["text"]) for e in story if e["message_type"] == "Narration"],
        "words_input": [words(e["text"]) for e in inputs],
    }
    (pack / "summary.json").write_text(json.dumps(summary, indent=2))
    print(f"Wrote {pack / 'pack.md'}")


def cmd_compare(a):
    packs = [Path(p) for p in a.packs]
    sums = [json.loads((p / "summary.json").read_text()) for p in packs]
    stories = [json.loads((p / "story.json").read_text()) for p in packs]
    for st in stories:
        for e in st:
            e["message_type"] = json.loads(e["message_type"]) if e["message_type"].startswith('"') else e["message_type"]
    L = ["# Trials side by side\n", "| | " + " | ".join(p.name for p in packs) + " |", "|---|" + "---|" * len(packs)]

    def row(label, f):
        L.append(f"| {label} | " + " | ".join(str(f(s)) for s in sums) + " |")

    row("Cycles played", lambda s: s["cycles"])
    row("Story entries", lambda s: s["entries"])
    row("Wall time (s)", lambda s: s["seconds"])
    row("Tokens in / out", lambda s: f'{s["tokens_in"]:,} / {s["tokens_out"]:,}')
    row("Cached input", lambda s: f'{(100*s["cached"]/s["tokens_in"] if s["tokens_in"] else 0):.0f} %')
    row("Failed calls", lambda s: s["failed"])
    row("First history cut (call)", lambda s: s["first_cut_call"] or "none")
    row("Narration words (median)", lambda s: sorted(s["words_narration"])[len(s["words_narration"]) // 2] if s["words_narration"] else "-")
    row("Input words (median)", lambda s: sorted(s["words_input"])[len(s["words_input"]) // 2] if s["words_input"] else "-")
    L.append("\n## Story, entry by entry\n")
    n = max(len(s) for s in stories)
    for i in range(n):
        L.append(f"### Entry {i + 1}\n")
        for p, st in zip(packs, stories):
            if i < len(st):
                e = st[i]
                label = "Impersonate Input" if e["impersonated"] else e["message_type"]
                L.append(f"**{p.name}** — {label} ({words(e['text'])} words)\n")
                L.append("> " + e["text"].strip().replace("\n", "\n> ") + "\n")
    out = Path(a.out)
    out.write_text("\n".join(L) + "\n")
    print(f"Wrote {out}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("--base", default="http://127.0.0.1:3001")
    r.add_argument("--db", default=DEFAULT_DB)
    r.add_argument("--world", default="redmist_estate")
    r.add_argument("--persona", default="julian")
    r.add_argument("--cycles", type=int, default=6)
    r.add_argument("--token-limit", type=int, default=5_000_000)
    r.add_argument("--timeout", type=int, default=3600)
    r.add_argument("--out", required=True)
    e = sub.add_parser("export")
    e.add_argument("pack")
    c = sub.add_parser("compare")
    c.add_argument("packs", nargs="+")
    c.add_argument("--out", default="tmp/pf08/compare.md")
    a = ap.parse_args()
    {"run": cmd_run, "export": cmd_export, "compare": cmd_compare}[a.cmd](a)


if __name__ == "__main__":
    main()
