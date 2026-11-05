"""Aggregate the extracted session data: cost by effort, error classes, friction signals."""
import collections
import json
import re

D = "/workspace/chronicler-engine/.scratch/weekly-retro-2026-11-04/assets/local-only/data"
rows = json.load(open(f"{D}/sessions.json"))
users = [json.loads(l) for l in open(f"{D}/user_messages.jsonl")]
errors = [json.loads(l) for l in open(f"{D}/tool_errors.jsonl")]
dels = [json.loads(l) for l in open(f"{D}/delegates.jsonl")]


def effort(r):
    n = (r["name"] or "")
    lab = n.split("·")[-1].strip() if n.startswith("delegate") else ""
    fu = (r["first_user"] or "")
    blob = (lab + " " + fu).lower()
    keys = [
        ("art", r"\bart[-_]|character-art|art-style|facegen|chibi|pixel|grid48|grid64|svg64"),
        ("long-term-memory", r"ltm|long-term-memory"),
        ("layout-trials", r"\blt\d|layout-trials"),
        ("prompt-flexibility", r"\bpf|prompt-flexibility|review-fixes"),
        ("action-answers/dashboard", r"\baa-|action-answers|dashboard|part.routes|ddr|dash"),
        ("statechart", r"statechart|tla|\bsc-"),
        ("hex-comparison", r"hex"),
        ("build/tooling", r"build\.py|build-|nextest|sccache|bt-"),
        ("plannotator/pi", r"plannotator|pi-emote|extension|chief of staff|rate-limit"),
        ("chat-completions/opencode", r"opencode|chat-completions|ocg"),
        ("ste/docs", r"\bste\b|vale|prose"),
        ("code-concepts/integrity", r"conceptual|code-concepts|metacognitive"),
    ]
    for k, rx in keys:
        if re.search(rx, blob):
            return k
    return "other"


by = collections.defaultdict(lambda: [0, 0.0, 0])
for r in rows:
    e = effort(r)
    r["effort"] = e
    by[e][0] += 1
    by[e][1] += r["cost"]
    by[e][2] += sum(r["tools"].values())
print("== cost by effort (sessions, $, tool calls)")
for k, v in sorted(by.items(), key=lambda kv: -kv[1][1]):
    print(f"{k:28s} {v[0]:4d} {v[1]:8.2f} {v[2]:6d}")

role_cost = collections.defaultdict(lambda: [0, 0.0])
for r in rows:
    n = r["name"] or ""
    role = n.split("·")[1].strip() if n.startswith("delegate") else "lead"
    role_cost[role][0] += 1
    role_cost[role][1] += r["cost"]
print("== cost by role")
for k, v in sorted(role_cost.items(), key=lambda kv: -kv[1][1]):
    print(f"{k:22s} {v[0]:4d} {v[1]:8.2f}")

mc = collections.Counter()
for r in rows:
    for m in r["models"]:
        mc[m] += 1
print("== models", mc.most_common())

print("== permission denials by rule")
pr = collections.Counter()
for e in errors:
    if e["class"] == "permission-denied":
        m = re.search(r"\(rule '([^']+)'\)", e["text"])
        pr[m.group(1) if m else e["text"][:80]] += 1
for k, v in pr.most_common(30):
    print(v, k)

print("== 'other' error samples")
oc = collections.Counter()
for e in errors:
    if e["class"] in ("other", "nonzero-exit", "not-found", "timeout"):
        oc[(e["tool"], e["text"][:90].replace("\n", " "))] += 1
for k, v in oc.most_common(40):
    print(v, k)

print("== delegates by role", collections.Counter(d["role"] for d in dels).most_common())
print("== resumes", sum(d["resume"] for d in dels))
print("== brief length median", sorted(d["task_len"] for d in dels)[len(dels) // 2], "max", max(d["task_len"] for d in dels))

# Real user turns: skip skill expansions and delegate briefs (the first message of a delegate session)
first_ids = {}
for u in users:
    first_ids.setdefault(u["session"], u)
real = [u for u in users if not (u["name"] or "").startswith("delegate") and first_ids[u["session"]] is not u]
print("== follow-up user turns in lead sessions", len(real))
json.dump(real, open(f"{D}/followup_user_turns.json", "w"), indent=1)
corr = re.compile(r"\b(no[,. ]|not what|wrong|why did|why are|you didn't|you did not|stop|instead|don't|do not|actually|again|still|missed|forgot|broken|doesn't work|isn't|incorrect|mistake|undo|revert)", re.I)
cs = [u for u in real if corr.search(u["text"][:600])]
print("== follow-ups with correction words", len(cs))
json.dump(cs, open(f"{D}/correction_candidates.json", "w"), indent=1)

# Biggest tool results
big = []
for r in rows:
    for b in r["biggest_results"]:
        big.append((b[0], r["file"][:19], b[1], b[2][:120]))
print("== biggest tool results")
for b in sorted(big, reverse=True)[:15]:
    print(b)

tot_chars = sum(r["tool_result_chars"] for r in rows)
print("== total tool result chars", tot_chars)
json.dump(rows, open(f"{D}/sessions_with_effort.json", "w"), indent=1)
