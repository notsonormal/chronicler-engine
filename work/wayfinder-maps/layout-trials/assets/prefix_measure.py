"""Measure the shared byte prefix of a trial's prompts, today and with a layout candidate applied.

Reads ``calls.jsonl`` of a trial pack. For each call kind (Narration, Impersonate, Quantifier),
it pairs each call with the previous successful call of the same kind and measures the byte
prefix that the two requests share (``system_prompt + "\\n" + user_prompt``). The candidates
rewrite the stored user prompt before the measurement, so the story stays the same and only the
layout changes. A provider caches only a prefix, so this is an upper bound for the cached share,
with no model call and no noise from a second story.

    python .scratch/layout-trials/assets/prefix_measure.py tmp/pf20/redmist_julian/trial-1
"""

from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

MARK = re.compile(r" \((in room|elsewhere)\)\n")
BLOCK = re.compile(r"^<([A-Za-z_]+)>.*?^</\1>\n?", re.M | re.S)
HISTORY = "ConversationHistory"


def kind(call):
    if call["agent_name"] != "narrator":
        return call["agent_name"]
    return "impersonate" if "player character" in call["system_prompt"] else "narration"


def blocks(user):
    """The user prompt as a list of (tag, text), with the text between blocks kept."""
    parts, pos = [], 0
    for match in BLOCK.finditer(user):
        if match.start() > pos:
            parts.append(("", user[pos : match.start()]))
        parts.append((match.group(1), match.group(0)))
        pos = match.end()
    if pos < len(user):
        parts.append(("", user[pos:]))
    return parts


def no_marks(user):
    return MARK.sub("\n", user)


def after_history(names):
    """Move the named blocks to just after the history, before the style and format sections."""

    def apply(user):
        parts = blocks(user)
        moved = [part for part in parts if part[0] in names]
        kept = [part for part in parts if part[0] not in names]
        if not any(tag == HISTORY for tag, _ in kept):
            return user
        out = []
        for part in kept:
            out.append(part)
            if part[0] == HISTORY:
                out.extend(moved)
        return "".join(text for _, text in out)

    return apply


CANDIDATES = {
    "today": lambda user: user,
    "no presence marks": no_marks,
    "GameState after the history": after_history({"GameState"}),
    "no marks + GameState after": lambda user: after_history({"GameState"})(no_marks(user)),
    "no marks + GameState and CharactersPresent after": lambda user: after_history(
        {"GameState", "CharactersPresent"}
    )(no_marks(user)),
}


def measure(calls, rewrite):
    shared = total = early = pairs = 0
    for before, after in zip(calls, calls[1:]):
        a = before["system_prompt"] + "\n" + rewrite(before["user_prompt"])
        b = after["system_prompt"] + "\n" + rewrite(after["user_prompt"])
        n, limit = 0, min(len(a), len(b))
        while n < limit and a[n] == b[n]:
            n += 1
        shared, total, pairs = shared + n, total + len(b), pairs + 1
        history = b.find(f"<{HISTORY}>")
        if history >= 0 and n < history:
            early += 1
    return 100 * shared / total if total else 0.0, early, pairs


def main(pack):
    by_kind = defaultdict(list)
    for line in (Path(pack) / "calls.jsonl").read_text(encoding="utf-8").splitlines():
        call = json.loads(line)
        if not call["error_message"]:
            by_kind[kind(call)].append(call)
    for name in ("narration", "impersonate"):
        print(f"{name}: shared prefix (character-weighted) | pairs that break before the history")
        for label, rewrite in CANDIDATES.items():
            share, early, pairs = measure(by_kind[name], rewrite)
            print(f"  {label:50s} {share:5.1f} %  {early}/{pairs}")
    share, _, pairs = measure(by_kind["quantifier"], CANDIDATES["today"])
    print(f"quantifier: shared prefix today {share:5.1f} % over {pairs} pairs")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "tmp/pf20/redmist_julian/trial-1")
