"""Extract per-session facts from the pi session logs of the retro window."""
import collections
import glob
import json
import os
import re
import sys

ROOT = os.path.expanduser("~/.pi/agent/sessions/--workspace-chronicler-engine--/")
OUT = "/workspace/chronicler-engine/.scratch/weekly-retro-2026-11-04/assets/local-only/data"
SINCE = sys.argv[1] if len(sys.argv) > 1 else "2026-10-28"
os.makedirs(OUT, exist_ok=True)


def text_of(content):
    if isinstance(content, str):
        return content
    parts = []
    for c in content or []:
        if c.get("type") == "text":
            parts.append(c.get("text", ""))
    return "\n".join(parts)


def classify_error(tool, txt):
    t = txt[:400]
    if "Denied by policy" in t or "pi-permission-system" in t:
        return "permission-denied"
    if tool == "edit" and ("Could not find" in t or "must match" in t or "not unique" in t or "oldText" in t):
        return "edit-mismatch"
    if "build.py" in t or "error[E" in t or "FAILED" in t or "test result" in t:
        return "build-or-test-fail"
    if "timed out" in t.lower() or "timeout" in t.lower():
        return "timeout"
    if "No such file" in t or "ENOENT" in t or "not found" in t.lower():
        return "not-found"
    if "exited with code" in t:
        return "nonzero-exit"
    return "other"


rows = []
user_msgs = []
errors = []
delegates = []
files = sorted(f for f in glob.glob(ROOT + "*.jsonl") if os.path.basename(f)[:10] >= SINCE)
for f in files:
    sid = os.path.basename(f)[:-6]
    r = {
        "file": os.path.basename(f), "name": None, "models": collections.Counter(), "first_user": None,
        "skill": None, "n_user": 0, "n_assistant": 0, "tools": collections.Counter(),
        "errors": collections.Counter(), "cost": 0.0, "out_tokens": 0, "in_tokens": 0,
        "start": None, "end": None, "build_runs": 0, "build_cmds": collections.Counter(),
        "delegate_calls": 0, "bytes": os.path.getsize(f), "tool_result_chars": 0,
        "biggest_results": [],
    }
    calls = {}
    for line in open(f, encoding="utf-8"):
        try:
            o = json.loads(line)
        except json.JSONDecodeError:
            continue
        ts = o.get("timestamp")
        if isinstance(ts, str):
            r["start"] = r["start"] or ts
            r["end"] = ts
        t = o.get("type")
        if t == "session_info" and o.get("name"):
            r["name"] = o["name"]
        if t == "model_change":
            r["models"][o.get("modelId")] += 1
        if t != "message":
            continue
        m = o["message"]
        role = m.get("role")
        if role == "user":
            txt = text_of(m.get("content"))
            r["n_user"] += 1
            sk = re.match(r'<skill name="([^"]+)"', txt.strip())
            if r["first_user"] is None:
                r["first_user"] = txt[:400]
                if sk:
                    r["skill"] = sk.group(1)
            ask = txt.split("</skill>")[-1].strip()
            if r.get("ask") is None:
                r["ask"] = ask[:600]
            user_msgs.append({"session": sid, "name": r["name"], "ts": ts, "skill": sk.group(1) if sk else None,
                              "text": ask[:3000], "len": len(txt)})
        elif role == "assistant":
            r["n_assistant"] += 1
            u = m.get("usage") or {}
            r["cost"] += (u.get("cost") or {}).get("total", 0) or 0
            r["out_tokens"] += u.get("output", 0) or 0
            r["in_tokens"] += (u.get("input", 0) or 0) + (u.get("cacheRead", 0) or 0) + (u.get("cacheWrite", 0) or 0)
            if m.get("model"):
                r["models"][m["model"]] += 0
            for c in m.get("content") or []:
                if c.get("type") == "toolCall":
                    name = c.get("name")
                    r["tools"][name] += 1
                    args = c.get("arguments") or {}
                    calls[c.get("id")] = (name, args)
                    if name == "bash":
                        cmd = args.get("command", "")
                        for mm in re.finditer(r"build\.py\s+([\w-]+)?", cmd):
                            r["build_runs"] += 1
                            r["build_cmds"][mm.group(1) or "(full)"] += 1
                    if name == "delegate":
                        r["delegate_calls"] += 1
                        delegates.append({"session": sid, "role": args.get("role"), "label": args.get("label"),
                                          "task_len": len(args.get("task", "")), "resume": bool(args.get("resume"))})
        elif role == "toolResult":
            txt = text_of(m.get("content"))
            r["tool_result_chars"] += len(txt)
            name, args = calls.get(m.get("toolCallId"), (m.get("toolName"), {}))
            r["biggest_results"].append((len(txt), name, json.dumps(args)[:160]))
            if m.get("isError"):
                cls = classify_error(name, txt)
                r["errors"][cls] += 1
                errors.append({"session": sid, "tool": name, "class": cls, "args": json.dumps(args)[:300],
                               "text": txt[:500]})
    r["biggest_results"] = sorted(r["biggest_results"], reverse=True)[:3]
    for k in ("models", "tools", "errors", "build_cmds"):
        r[k] = dict(r[k])
    rows.append(r)

with open(f"{OUT}/sessions.json", "w") as fh:
    json.dump(rows, fh, indent=1)
with open(f"{OUT}/user_messages.jsonl", "w") as fh:
    for u in user_msgs:
        fh.write(json.dumps(u) + "\n")
with open(f"{OUT}/tool_errors.jsonl", "w") as fh:
    for e in errors:
        fh.write(json.dumps(e) + "\n")
with open(f"{OUT}/delegates.jsonl", "w") as fh:
    for d in delegates:
        fh.write(json.dumps(d) + "\n")

# Summary
tot = collections.Counter()
err = collections.Counter()
for r in rows:
    tot.update(r["tools"])
    err.update(r["errors"])
print("sessions", len(rows), "cost", round(sum(r["cost"] for r in rows), 2),
      "out_tokens", sum(r["out_tokens"] for r in rows))
print("tools", tot.most_common(25))
print("errors", err.most_common())
print("build runs", sum(r["build_runs"] for r in rows))
bc = collections.Counter()
for r in rows:
    bc.update(r["build_cmds"])
print("build cmds", bc.most_common())
kinds = collections.Counter()
for r in rows:
    n = r["name"] or ""
    if n.startswith("delegate"):
        kinds["delegate:" + n.split("·")[1].strip()] += 1
    else:
        kinds["top:" + (r["skill"] or "none")] += 1
print("kinds", kinds.most_common())
by_cost = sorted(rows, key=lambda r: -r["cost"])[:15]
for r in by_cost:
    print(round(r["cost"], 2), r["file"][:19], (r["name"] or r["skill"] or (r["first_user"] or "")[:60]).replace("\n", " "),
          "user", r["n_user"], "asst", r["n_assistant"])
