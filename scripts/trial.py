"""Run a test setup through Autoplay on a probe server and write its reading pack.

A trial fixes a World, a Persona, a mode, a posture, four prompt presets and
two Connections in one JSON file under ``scripts/trial_setups/``. This script
applies that setup through the HTTP routes, plays Autoplay until the Budget or
the token limit, restarts a run that a filtered answer ended, and writes one
reading pack for each repeat.

    python scripts/trial.py run scripts/trial_setups/redmist_julian.json
    python scripts/trial.py export tmp/pf18/redmist_julian/trial-1
    python scripts/trial.py compare tmp/pf18/redmist_julian/trial-*

``run`` needs a probe server (see ``/chronicler-ui-investigator``) and reads its
DB directly. ``llm_messages`` keeps only the newest 50 rows, so ``run`` copies
each new row out while the trial plays.
"""

from __future__ import annotations

import argparse
import contextlib
import json
import re
import sqlite3
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Callable, NamedTuple

DEFAULT_BASE = "http://127.0.0.1:3001"
DEFAULT_DB = "target/ui-investigator/debug/chronicler_3001.db"
DEFAULT_TOKEN_LIMIT = 6_000_000
DEFAULT_RESTART_LIMIT = 5
DEFAULT_TIMEOUT = 3600

# Mirrors `Budget::MAX` of `application/autoplay/budget.rs`: `POST /autoplay`
# refuses any other budget.
BUDGET_MAX = 100
# Mirrors `Budget::DEFAULT` of the same file: the Budget of a run that names none.
BUDGET_DEFAULT = 10

POLL_SECONDS = 2
IDLE_POLLS = 2
REQUEST_TIMEOUT = 30
GAME_HEADER = "X-Chronicler-Game"

SETUP_FIELDS = {
    "world",
    "persona",
    "mode",
    "perspective",
    "tense",
    "presets",
    "connections",
    "cycles",
    "token_limit",
    "restart_limit",
    "repeats",
    "out",
}


class CallKind(NamedTuple):
    """One kind of model call: the agent name it records, and whether its answer becomes
    a story entry."""

    agent: str
    story: bool


# The kinds of model call, one row for each. The Narrator records the agent name
# `narrator` for both of its kinds, so `call_kind` reads the answer to tell them apart.
CALL_KINDS = {
    "narration": CallKind("narrator", True),
    "impersonate": CallKind("narrator", True),
    "event-narration": CallKind("trigger", True),
    "quantifier": CallKind("quantifier", False),
    "options": CallKind("options", False),
}
STORY_AGENTS = frozenset(kind.agent for kind in CALL_KINDS.values() if kind.story)
STORY_KINDS = frozenset(name for name, kind in CALL_KINDS.items() if kind.story)
NON_STORY_KINDS = frozenset(name for name, kind in CALL_KINDS.items() if not kind.story)
NON_STORY_AGENTS = frozenset(
    kind.agent for kind in CALL_KINDS.values() if not kind.story
)


class Stop(Enum):
    """Why a trial stopped. Each member carries the words of the pack's Stop line."""

    def __new__(cls, key, words):
        member = object.__new__(cls)
        member._value_ = key
        member.words = words
        return member

    BUDGET = ("budget", "the run played its whole Budget")
    FILTERED = (
        "filtered",
        "a filtered story answer ended the run, and the script started it again",
    )
    TOKEN_LIMIT = (
        "token_limit",
        "the token limit of the trial was spent, so the script stopped the run",
    )
    RESTART_LIMIT = (
        "restart_limit",
        "each restart met the filter again, and the restart limit was reached",
    )
    FAILED = ("failed", "the run ended early, and no restart rule applied")
    TIMEOUT = (
        "timeout",
        "the run passed the `--timeout` of the script, so the script stopped it "
        "and sent `reset-busy`",
    )
    GAME_CHANGED = (
        "game_changed",
        "the active Game changed, so the probe server refused the script's request",
    )
    PROBE_REFUSED = (
        "probe_refused",
        "the probe server refused a request of the run, so the script stopped",
    )


class TrialError(Exception):
    """A condition that stops the trial before it can play."""


class RouteRefused(TrialError):
    """A route that did not answer 200."""

    def __init__(self, request, status, body):
        super().__init__(f"{request.method} {request.path} answered {status}: {body[:500]}")
        self.status = status


class RunInterrupted(TrialError):
    """A request during a run that the probe server refused. The pack is still written."""

    def __init__(self, stop, message):
        super().__init__(message)
        self.stop = stop


class TrialRequest(NamedTuple):
    """One setup request: the method, the path and the form it sends."""

    method: str
    path: str
    form: dict[str, str]


class StopDecision(NamedTuple):
    """What a run that ended asks for: another run, and the reason the trial stopped."""

    restart: bool
    reason: Stop


class Clock(NamedTuple):
    """The two calls that make the poll wait, so a test drives the poll without waiting."""

    sleep: Callable[[float], None]
    now: Callable[[], float]


@dataclass(frozen=True)
class ProbeClient:
    """The probe server and the Game a request is bound to."""

    base: str
    transport: Callable
    game_id: int | None = None

    def for_game(self, game_id):
        return ProbeClient(self.base, self.transport, game_id)

    def send(self, request, quiet=False):
        """One request; a route that does not answer 200 stops the trial."""
        headers = game_header(self.game_id) if self.game_id is not None else {}
        status, body = self.transport(
            self.base, request.method, request.path, request.form, headers
        )
        if status != 200:
            raise RouteRefused(request, status, body)
        if not quiet:
            print(f"  {request.method} {request.path} -> 200")
        return body

    def send_during_run(self, request, quiet=False):
        """A request of a run that has a pack to write: a refusal ends the run, not the script."""
        try:
            return self.send(request, quiet)
        except RouteRefused as refusal:
            stop = Stop.GAME_CHANGED if refusal.status == 409 else Stop.PROBE_REFUSED
            raise RunInterrupted(stop, str(refusal)) from refusal

    def is_busy(self):
        """The busy state of the Game: `/debug/is_busy` answers `true` or `false`."""
        body = self.send_during_run(TrialRequest("GET", "/debug/is_busy", None), quiet=True)
        return body.strip() == "true"

    def start_autoplay(self, cycles):
        """Start one Autoplay run on the trial's Game."""
        self.send(TrialRequest("POST", "/autoplay", {"cycles": str(cycles)}), quiet=True)
        print(f"Autoplay {cycles} cycles started on Game {self.game_id}")

    def reset_busy(self):
        """Stop the run, and clear the failure line of the Game."""
        self.send_during_run(TrialRequest("POST", "/status/reset-busy", {}))


def http_request(base, method, path, form=None, headers=None):
    """One request to the probe server: an answer, or the status and body of a refusal."""
    data = urllib.parse.urlencode(form).encode() if form is not None else None
    request = urllib.request.Request(
        base + path, data=data, method=method, headers=headers or {}
    )
    try:
        with urllib.request.urlopen(request, timeout=REQUEST_TIMEOUT) as answer:
            return answer.status, answer.read().decode()
    except urllib.error.HTTPError as refusal:
        return refusal.code, refusal.read().decode()


def rows(conn, sql, args=()):
    """The rows of one read, as dicts keyed by column name."""
    conn.row_factory = sqlite3.Row
    return [dict(row) for row in conn.execute(sql, args)]


def newest_game_id(conn):
    """The newest Game of the DB, which a `POST /games` must raise."""
    return rows(conn, "select coalesce(max(id), 0) as id from games")[0]["id"]


def newest_call_id(conn):
    """The newest model call of the debug record, which the trial copies from."""
    return rows(conn, "select coalesce(max(id), 0) as id from llm_messages")[0]["id"]


def stored_failure(conn, game_id):
    """The failure line of the Game, read before a stop, because `reset-busy` clears it."""
    found = rows(conn, "select failure from games where id = ?", (game_id,))
    return found[0]["failure"] if found else None


def story_rows(conn, game_id):
    """The story of the Game, one row for each entry and its active Swipe."""
    return rows(
        conn,
        """select m.id, m.message_type, m.kind, m.timestamp, s.swipe_index, s.text,
                  s.event_header, s.impersonated, s.state, s.maker
           from messages m join message_swipes s on s.message_id = m.id
                and s.swipe_index = m.active_swipe_index
           where m.game_id = ? order by m.id""",
        (game_id,),
    )


def message_type_of(raw):
    """The `message_type` column holds JSON, so `"Input"` reads as Input."""
    try:
        value = json.loads(raw)
    except (TypeError, json.JSONDecodeError):
        return raw
    return value if isinstance(value, str) else raw


def cycles_played(story):
    """The Cycles the Game played: one for each answered Input. A Cycle ends only on an
    answered Input, so an Input that the story ends on does not count."""
    inputs = [
        entry for entry in story if message_type_of(entry["message_type"]) == "Input"
    ]
    if story and message_type_of(story[-1]["message_type"]) == "Input":
        return max(len(inputs) - 1, 0)
    return len(inputs)


def usage_of(call):
    """The tokens, the host and the finish reason of a model call, read from its raw answer."""
    try:
        answer = json.loads(call["raw_response_json"].strip() or "{}")
    except json.JSONDecodeError:
        return {}
    usage = answer.get("usage") or {}
    details = usage.get("prompt_tokens_details") or {}
    return {
        "prompt": usage.get("prompt_tokens", 0) or 0,
        "completion": usage.get("completion_tokens", 0) or 0,
        "cached": details.get("cached_tokens", 0) or 0,
        "cost": usage.get("cost", 0) or 0,
        "allowance": (usage.get("opencode") or {}).get("allowance"),
        "host": answer.get("provider", ""),
        "finish": ((answer.get("choices") or [{}])[0] or {}).get("finish_reason"),
    }


def text_field(given, name, label=None):
    """A field that the setup file must state as a non-empty string."""
    value = given.get(name)
    if not isinstance(value, str) or not value.strip():
        raise TrialError(f"the setup file needs `{label or name}` as a non-empty string")
    return value


def text_fields(given, name, keys):
    """A group of named strings that the setup file must state."""
    group = given.get(name)
    if not isinstance(group, dict):
        raise TrialError(f"the setup file needs `{name}` as an object")
    return {key: text_field(group, key, f"{name}.{key}") for key in keys}


# The Connection slots that no route sets. The DB holds them, and a setup file may
# name one only to state the value it expects.
STORED_SLOTS = ("impersonate", "options")


def connection_fields(given):
    """The Connections a setup names: the two slots that a route sets, and each stored slot
    whose value the setup states."""
    group = text_fields(given, "connections", ("narration", "quantifier"))
    for slot in STORED_SLOTS:
        if slot in given["connections"]:
            group[slot] = text_field(given["connections"], slot, f"connections.{slot}")
    return group


def whole_number(given, name, default):
    """A count of zero or more, with its default."""
    value = given.get(name, default)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise TrialError(f"the setup file needs `{name}` as a whole number of zero or more")
    return value


def load_setup(path, cycles_override=None):
    """The setup file with its defaults and the flags applied; a field that cannot be
    applied is refused before any request goes out."""
    try:
        given = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise TrialError(f"cannot read the setup file {path}: {error}")
    if not isinstance(given, dict):
        raise TrialError(f"the setup file {path} holds no object")
    unknown = sorted(set(given) - SETUP_FIELDS)
    if unknown:
        raise TrialError(
            f"{path} names fields the script does not read: {', '.join(unknown)}"
        )
    setup = {
        "world": text_field(given, "world"),
        "persona": text_field(given, "persona"),
        "mode": text_field(given, "mode"),
        "perspective": text_field(given, "perspective"),
        "tense": text_field(given, "tense"),
        "presets": text_fields(
            given, "presets", ("system", "impersonate", "quantifier", "options")
        ),
        "connections": connection_fields(given),
        "cycles": (
            cycles_override
            if cycles_override is not None
            else whole_number(given, "cycles", BUDGET_DEFAULT)
        ),
        "token_limit": whole_number(given, "token_limit", DEFAULT_TOKEN_LIMIT),
        "restart_limit": whole_number(given, "restart_limit", DEFAULT_RESTART_LIMIT),
        "repeats": whole_number(given, "repeats", 1),
        "out": text_field(given, "out"),
    }
    if not 1 <= setup["cycles"] <= BUDGET_MAX:
        raise TrialError(
            f"an Autoplay run takes 1 to {BUDGET_MAX} cycles, and the setup asks for "
            f"{setup['cycles']}"
        )
    if setup["repeats"] < 1:
        raise TrialError("`repeats` is 1 or more: each repeat is one trial")
    return setup


def check_connections(conn, setup):
    """The Connection rows of the probe DB; refuses an id that the DB does not hold,
    because the setup file names a Connection and never creates one."""
    found = rows(conn, "select * from settings")
    if not found:
        raise TrialError("the database holds no settings row")
    known = {
        connection.get("id"): connection
        for connection in json.loads(found[0]["connections"] or "[]")
    }
    missing = [
        f'{name} `{setup["connections"][name]}`'
        for name in ("narration", "quantifier")
        if setup["connections"][name] not in known
    ]
    if missing:
        raise TrialError(f"the database holds no Connection {', '.join(missing)}")
    for slot in STORED_SLOTS:
        wanted = setup["connections"].get(slot)
        stored = stored_slot(found[0], slot)
        if wanted is not None and wanted != stored:
            raise TrialError(
                f"the database holds the {slot} slot `{stored or 'empty'}`, and the "
                f"setup asks for `{wanted}`: no route sets it"
            )
    return known


def stored_slot(settings_row, slot):
    """`None` when the slot is empty, so the call uses the Narration Connection."""
    return settings_row[f"{slot}_connection_id"] or None


def slot_connection(settings, setup, slot):
    """The Connection row of a stored slot. An empty slot uses the Narration Connection,
    as `AppSettings::connection_for` does."""
    connections = {
        connection["id"]: connection
        for connection in json.loads(settings["connections"] or "[]")
    }
    return connections.get(
        stored_slot(settings, slot) or setup["connections"]["narration"]
    )


def connection_requests(setup):
    """The two Connection routes. They set a global setting, so they carry no Game header."""
    return [
        TrialRequest(
            "POST",
            "/connections/set-narrator",
            {"connection_id": setup["connections"]["narration"]},
        ),
        TrialRequest(
            "POST",
            "/connections/set-quantifier",
            {"connection_id": setup["connections"]["quantifier"]},
        ),
    ]


def game_setup_requests(setup, game_id):
    """The Game routes of a setup, in order: the mode (which rewrites the preset
    bundle), then the posture, then the four presets."""
    return [
        TrialRequest(
            "POST", f"/games/{game_id}/mode", {"narrator_mode": setup["mode"]}
        ),
        TrialRequest(
            "POST",
            f"/games/{game_id}/posture",
            {
                "narrative_perspective": setup["perspective"],
                "narrative_tense": setup["tense"],
            },
        ),
        TrialRequest(
            "POST",
            f"/games/{game_id}/presets",
            {
                "system_preset_id": setup["presets"]["system"],
                "quantifier_preset_id": setup["presets"]["quantifier"],
                "impersonate_preset_id": setup["presets"]["impersonate"],
                "options_preset_id": setup["presets"]["options"],
            },
        ),
    ]


def game_header(game_id):
    """The header that ties a request to the trial's Game, so another active Game
    cannot take the trial's writes."""
    return {GAME_HEADER: str(game_id)}


def apply_setup(client, setup, conn):
    """Apply the setup through the routes. Answers the client bound to the new Game."""
    for request in connection_requests(setup):
        client.send(request)
    before = newest_game_id(conn)
    client.send(
        TrialRequest(
            "POST",
            "/games",
            {"world_key": setup["world"], "persona_key": setup["persona"]},
        )
    )
    game_id = newest_game_id(conn)
    if game_id <= before:
        raise TrialError(
            "POST /games answered 200, but the newest Game id did not rise"
        )
    bound = client.for_game(game_id)
    for request in game_setup_requests(setup, game_id):
        bound.send(request)
    return bound


def setup_snapshot(conn, setup, game_id):
    """The values the trial applied, with the names of the rows they point at."""
    settings = rows(conn, "select * from settings")[0]
    game = rows(conn, "select * from games where id = ?", (game_id,))[0]
    connections = {
        connection["id"]: connection
        for connection in json.loads(settings["connections"])
    }
    presets = {}
    for role, preset_id in setup["presets"].items():
        found = rows(conn, "select * from prompt_presets where id = ?", (preset_id,))
        presets[role] = found[0] if found else {"id": preset_id, "missing": True}
    return {
        "game_id": game_id,
        "world": setup["world"],
        "persona": setup["persona"],
        "narrator_mode": setup["mode"],
        "posture": f'{setup["perspective"]} person, {setup["tense"]} tense',
        "options_always_on": bool(game["options_always_on"]),
        "cycles": setup["cycles"],
        "token_limit": setup["token_limit"],
        "restart_limit": setup["restart_limit"],
        "narration_connection": connections.get(setup["connections"]["narration"]),
        "quantifier_connection": connections.get(setup["connections"]["quantifier"]),
        "impersonate_slot": stored_slot(settings, "impersonate"),
        "impersonate_connection": slot_connection(settings, setup, "impersonate"),
        "options_slot": stored_slot(settings, "options"),
        "options_connection": slot_connection(settings, setup, "options"),
        "agents": json.loads(settings["agents"]),
        "presets": presets,
    }


class TrialState:
    """What the poll of one trial keeps: the copied calls, their tokens and the last
    failure line read before a stop."""

    def __init__(self, calls_path, last_call_id, token_limit):
        self.calls_path = calls_path
        self.last_call_id = last_call_id
        self.token_limit = token_limit
        self.tokens = 0
        self.failure_line = None
        self.run_newest_call = None
        self.run_failed_call = None
        self.stopped_by_limit = False
        self.timed_out = False

    def begin_run(self):
        """A run answers for its own calls: an older run's failure must not restart it."""
        self.run_newest_call = None
        self.run_failed_call = None

    def copy_new_calls(self, conn):
        """Copy each new row of the debug record out, before the record drops it."""
        new_calls = rows(
            conn,
            "select * from llm_messages where id > ? order by id",
            (self.last_call_id,),
        )
        with self.calls_path.open("a", encoding="utf-8") as handle:
            for call in new_calls:
                handle.write(json.dumps(call) + "\n")
                self.last_call_id = call["id"]
                usage = usage_of(call)
                self.tokens += usage.get("prompt", 0) + usage.get("completion", 0)
                self.run_newest_call = call
                if call["error_message"]:
                    self.run_failed_call = call
                flag = (
                    f' FAILED: {call["error_message"][:80]}'
                    if call["error_message"]
                    else ""
                )
                print(
                    f'  call {call["id"]} {call["agent_name"]} '
                    f'{usage.get("prompt", 0)}+{usage.get("completion", 0)} tok, '
                    f'cached {usage.get("cached", 0)}{flag}'
                )


def poll_run(client, conn, state, deadline, clock):
    """Copy the new model calls until the Game is idle twice in a row, or the deadline
    passes. A run that passes the token limit is stopped, with its failure line read first."""
    idle_polls = 0
    while True:
        clock.sleep(POLL_SECONDS)
        state.copy_new_calls(conn)
        if state.tokens > state.token_limit and not state.stopped_by_limit:
            print(
                f"Token limit {state.token_limit:,} passed ({state.tokens:,}); "
                "stopping the run"
            )
            state.failure_line = stored_failure(conn, client.game_id)
            client.reset_busy()
            state.stopped_by_limit = True
        if client.is_busy():
            idle_polls = 0
        else:
            idle_polls += 1
            if idle_polls >= IDLE_POLLS:
                return
        if clock.now() > deadline:
            print("Timeout; stopping the run")
            state.failure_line = stored_failure(conn, client.game_id)
            client.reset_busy()
            state.timed_out = True
            return


def is_story_call(call):
    """A call whose answer becomes a story entry: the Narrator (Narration or Impersonate)
    and the `trigger` agent (Event Narration) write one. The Quantifier and the Options
    agent write none."""
    return call["agent_name"] not in NON_STORY_AGENTS


def restart_reason(call):
    """The `finish_reason` of the call that ended a run, when that call is a failed story
    call. A failure of another agent does not end a run, so it never asks for a restart."""
    if call is None or not call["error_message"] or not is_story_call(call):
        return None
    return usage_of(call).get("finish")


def stop_decision(
    budget, cycles, tokens, token_limit, restarts, restart_limit, finish_reason
):
    """The rule of a run that ended: a filtered story answer starts another run while
    restarts remain and the token limit is not spent. Any other end stops the trial."""
    if cycles >= budget:
        return StopDecision(False, Stop.BUDGET)
    if tokens > token_limit:
        return StopDecision(False, Stop.TOKEN_LIMIT)
    if finish_reason == "content_filter":
        if restarts < restart_limit:
            return StopDecision(True, Stop.FILTERED)
        return StopDecision(False, Stop.RESTART_LIMIT)
    return StopDecision(False, Stop.FAILED)


def last_cycle_entries(story):
    """The ids of the entries of the newest Cycle: the newest Input and what follows it."""
    for index in range(len(story) - 1, -1, -1):
        if message_type_of(story[index]["message_type"]) == "Input":
            return [entry["id"] for entry in story[index:]]
    return []


def play_trial(client, conn, setup, pack, timeout, clock):
    """One trial: apply the setup, play its runs, and write its reading pack."""
    pack.mkdir(parents=True, exist_ok=True)
    client = apply_setup(client, setup, conn)
    game_id = client.game_id
    snapshot = setup_snapshot(conn, setup, game_id)
    (pack / "setup.json").write_text(json.dumps(snapshot, indent=2), encoding="utf-8")

    calls_path = pack / "calls.jsonl"
    calls_path.write_text("", encoding="utf-8")
    state = TrialState(calls_path, newest_call_id(conn), setup["token_limit"])

    budget = setup["cycles"]
    started = clock.now()
    restarts = []
    stop = None
    interruption = None
    while stop is None:
        played = cycles_played(story_rows(conn, game_id))
        remaining = budget - played
        if remaining < 1:
            stop = StopDecision(False, Stop.BUDGET)
            break
        state.begin_run()
        try:
            client.start_autoplay(remaining)
            poll_run(client, conn, state, clock.now() + timeout, clock)
        except RunInterrupted as interrupted:
            print(f"Run interrupted: {interrupted}")
            stop = StopDecision(False, interrupted.stop)
            interruption = str(interrupted)
            break
        if state.timed_out:
            stop = StopDecision(False, Stop.TIMEOUT)
            break
        played = cycles_played(story_rows(conn, game_id))
        finish_reason = restart_reason(state.run_newest_call)
        decision = stop_decision(
            budget=budget,
            cycles=played,
            tokens=state.tokens,
            token_limit=setup["token_limit"],
            restarts=len(restarts),
            restart_limit=setup["restart_limit"],
            finish_reason=finish_reason,
        )
        if decision.restart:
            restarts.append({"cycle": played, "finish_reason": finish_reason})
            print(
                f'Restart {len(restarts)} of {setup["restart_limit"]}: the run ended at '
                f"cycle {played} on finish_reason `{finish_reason}`"
            )
            continue
        stop = decision

    story = story_rows(conn, game_id)
    failed = state.run_failed_call
    run = {
        "game_id": game_id,
        "started": started,
        "seconds": round(clock.now() - started),
        "budget": budget,
        "cycles_played": cycles_played(story),
        "token_limit": setup["token_limit"],
        "restart_limit": setup["restart_limit"],
        "tokens_total": state.tokens,
        "stopped_by_limit": state.stopped_by_limit,
        "cut_entries": last_cycle_entries(story) if state.stopped_by_limit else [],
        "restarts": restarts,
        "stop": {
            "reason": stop.reason.value,
            "detail": interruption,
            "failed_call": failed["id"] if failed else None,
            "finish_reason": usage_of(failed).get("finish") if failed else None,
            "game_failure": state.failure_line or stored_failure(conn, game_id),
        },
    }
    (pack / "story.json").write_text(json.dumps(story, indent=2), encoding="utf-8")
    (pack / "run.json").write_text(json.dumps(run, indent=2), encoding="utf-8")
    export_pack(pack)
    return run


def cmd_run(
    args,
    transport=http_request,
    connect=sqlite3.connect,
    clock=Clock(time.sleep, time.time),
):
    """Run one setup file: each repeat is one trial, and `compare.md` sorts them."""
    setup = load_setup(args.setup, cycles_override=args.cycles)
    db_path = Path(args.db)
    if not db_path.is_file():
        raise TrialError(
            f"the database {db_path} does not exist; --db names the probe server's DB"
        )
    with contextlib.closing(connect(str(db_path))) as conn:
        check_connections(conn, setup)
        out = Path(setup["out"])
        out.mkdir(parents=True, exist_ok=True)
        client = ProbeClient(args.base, transport)
        packs = []
        trials = []
        for repeat in range(1, setup["repeats"] + 1):
            pack = out / f"trial-{repeat}"
            print(f"Trial {repeat} of {setup['repeats']}: {pack}")
            trials.append(play_trial(client, conn, setup, pack, args.timeout, clock))
            packs.append(pack)
        compare_packs(packs, out / "compare.md")
        for number, trial in enumerate(trials, start=1):
            print(f'trial-{number}: {trial["tokens_total"]:,} tokens')
        print(f'total: {sum(trial["tokens_total"] for trial in trials):,} tokens')
        return trials


def words(text):
    """The number of words of a text."""
    return len(text.split())


def sent_messages(call):
    """An old row holds the two prompt columns and no more: they are the messages.

    A new row leaves the columns empty, and the `messages` of its `raw_request_json`
    are the record. A call that failed before any send has no message.
    """
    system, user = call.get("system_prompt") or "", call.get("user_prompt") or ""
    if system or user:
        return [
            {"role": role, "text": text}
            for role, text in (("system", system), ("user", user))
            if text
        ]
    try:
        payload = json.loads(call.get("raw_request_json") or "")
    except json.JSONDecodeError:
        return []
    messages = payload.get("messages") if isinstance(payload, dict) else None
    return [
        {"role": message.get("role"), "text": message.get("content") or ""}
        for message in messages or []
        if isinstance(message, dict)
    ]


def read_prompts(call):
    """The other text joins every message after the system messages, so the readers that
    look for a block (`<LatestMessage>`) find it in an old row and in a new one."""
    call["old_row"] = bool(call.get("system_prompt") or call.get("user_prompt"))
    call["messages"] = sent_messages(call)
    call["system_prompt"] = "\n\n".join(
        message["text"] for message in call["messages"] if message["role"] == "system"
    )
    call["user_prompt"] = "\n\n".join(
        message["text"] for message in call["messages"] if message["role"] != "system"
    )
    refuse_a_merged_message(call)


def refuse_a_merged_message(call):
    """A `single_user_message` Connection sends one user message that holds the whole call.

    The history is then one text with `[SYSTEM]` and `[ASSISTANT]` headers, and the pack
    cannot tell its entries apart. The text is not split, so the pack would be wrong with
    no error.
    """
    if call["old_row"] or call["agent_name"] not in STORY_AGENTS:
        return
    messages = call["messages"]
    if len(messages) != 1 or messages[0]["role"] == "system":
        return
    text = messages[0]["text"]
    if text.startswith("[SYSTEM]\n") or "\n[ASSISTANT]\n" in "\n" + text:
        raise TrialError(
            f"call {call['id']} sent its whole prompt as one merged user message: "
            "a `single_user_message` pack is not supported"
        )


def call_kind(call, story):
    """Name a call by the story entry its answer became.

    Only the Narrator records two kinds under one agent name, so the kind of every other
    agent comes straight from the table.
    """
    names = [
        name for name, kind in CALL_KINDS.items() if kind.agent == call["agent_name"]
    ]
    if len(names) == 1:
        return names[0]
    if not names:
        return call["agent_name"]
    answer = (call["parsed_response"] or "").strip()[:120]
    for entry in story:
        if answer and answer in entry["text"]:
            return "impersonate" if entry["impersonated"] else "narration"
    # The agent name cannot name the kind here: an Impersonate call records `narrator` too,
    # because `GameViewQuery::role_health` counts the rows of a Role by agent name, and a
    # name of its own would move the Impersonate failures out of the Narrator health. So an
    # answer that reached no entry is named by the words of its prompt.
    if "player character" in call["system_prompt"]:
        return "impersonate"
    return "narration"


def owner_entry(call, story):
    """The story entry a call made or judged, found by its text; else the last entry before it."""
    if call["agent_name"] == "quantifier":
        match = re.search(
            r"<LatestMessage>\s*(.*?)</LatestMessage>", call["user_prompt"], re.S
        )
        probe = match.group(1).strip()[:100] if match else ""
    else:
        probe = (call["parsed_response"] or "").strip()[:100]
    if probe:
        for entry in story:
            if probe in entry["text"]:
                return entry["id"]
    before = [entry for entry in story if entry["timestamp"] <= call["created_at"]]
    return before[-1]["id"] if before else None


def history_block(call):
    """An old row holds the history in `<ConversationHistory>` of its one user text.

    A new row sends each history entry as its own message. The tags find them, not a
    position: the system message is skipped when it is empty, so a position moves. The
    history runs from its first tag to the last closing tag. The closing tag is the end
    of the last entry, or the end of the placeholder text of an empty history. The blocks
    after the history hold no entry text and fall outside.
    """
    if call["old_row"]:
        match = re.search(
            r"<ConversationHistory>(.*?)</ConversationHistory>",
            call["user_prompt"],
            re.S,
        )
        return match.group(1) if match else None
    text = call["user_prompt"]
    opens = [
        index
        for index in (text.find("<ConversationHistory>"), text.find("<LastMessage>"))
        if index >= 0
    ]
    if not opens:
        return None
    for closing in ("</LastMessage>", "</ConversationHistory>"):
        end = text.rfind(closing)
        if end >= 0:
            return text[min(opens) : end + len(closing)]
    return None


def history_cover(call, story_before):
    block = history_block(call)
    if block is None:
        return None
    held = [entry["id"] for entry in story_before if entry["text"].strip()[:100] in block]
    dropped = [entry["id"] for entry in story_before if entry["id"] not in held]
    return {"held": held, "dropped": dropped, "chars": len(block)}


def state_line(entry):
    try:
        state = json.loads(entry["state"])
    except (TypeError, json.JSONDecodeError):
        return "?", []
    room = (state.get("movement") or {}).get("current_room_id", "?")
    present = [npc.get("id") for npc in (state.get("scene") or {}).get("npcs_in_area", [])]
    return room, present


def failed_call_line(call, summary=""):
    """The pack line of a failed call, naming the finish reason of its raw answer."""
    lead = f"{summary} — " if summary else ""
    return (
        f'[{call["kind"]} {call["id"]}]({call["file"]}): {lead}'
        f'**FAILED** {call["error_message"][:200]} '
        f'(finish_reason `{call["usage"].get("finish")}`)'
    )


def entry_label(entry):
    """The heading word of a story entry."""
    return "Impersonate Input" if entry["impersonated"] else entry["message_type"]


def median_of(values):
    """The median of a list of numbers, or `-` for an empty list."""
    return sorted(values)[len(values) // 2] if values else "-"


def read_story(pack):
    """The story of a pack, with each `message_type` read out of its JSON."""
    story = json.loads((pack / "story.json").read_text(encoding="utf-8"))
    for entry in story:
        entry["message_type"] = message_type_of(entry["message_type"])
    return story


@dataclass
class PackData:
    """What the pack files hold, and what is derived from the calls and the story."""

    folder: Path
    setup: dict
    story: list
    run: dict
    calls: list
    by_entry: dict
    trailing: list
    kinds: dict

    @property
    def tokens_in(self):
        return sum(kind["in"] for kind in self.kinds.values())

    @property
    def tokens_out(self):
        return sum(kind["out"] for kind in self.kinds.values())

    @property
    def tokens_cached(self):
        return sum(kind["cached"] for kind in self.kinds.values())

    @property
    def failed_calls(self):
        return [call for call in self.calls if call["error_message"]]

    @property
    def first_cut_call(self):
        return next(
            (call for call in self.calls if call["cover"] and call["cover"]["dropped"]),
            None,
        )


def kind_totals(calls):
    """The calls, tokens and failures of each call kind."""
    kinds = {}
    for call in calls:
        kind = kinds.setdefault(
            call["kind"], {"n": 0, "failed": 0, "in": 0, "out": 0, "cached": 0}
        )
        kind["n"] += 1
        kind["failed"] += 1 if call["error_message"] else 0
        kind["in"] += call["usage"].get("prompt", 0)
        kind["out"] += call["usage"].get("completion", 0)
        kind["cached"] += call["usage"].get("cached", 0)
    return kinds


def load_pack(folder):
    """Read the pack files, and attach each call to the story entry it made or judged
    (the Job that made it)."""
    setup = json.loads((folder / "setup.json").read_text(encoding="utf-8"))
    run = json.loads((folder / "run.json").read_text(encoding="utf-8"))
    story = read_story(folder)
    calls = [
        json.loads(line)
        for line in (folder / "calls.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    by_entry = {entry["id"]: [] for entry in story}
    trailing = []
    for call in calls:
        read_prompts(call)
        call["kind"] = call_kind(call, story)
        call["usage"] = usage_of(call)
        before = [entry for entry in story if entry["timestamp"] <= call["created_at"]]
        call["cover"] = (
            history_cover(call, before) if call["kind"] in STORY_KINDS else None
        )
        owner = owner_entry(call, story)
        (by_entry[owner] if owner is not None else trailing).append(call)
        call["file"] = f'calls/{call["id"]:05d}-{call["kind"]}.md'
    return PackData(
        folder, setup, story, run, calls, by_entry, trailing, kind_totals(calls)
    )


def message_sections(call):
    numbered = len(call["messages"]) > 2
    sections = []
    for number, message in enumerate(call["messages"], start=1):
        role = (message["role"] or "?").capitalize()
        heading = f"{number}. {role}" if numbered else role
        sections.append(f'## {heading}\n\n```text\n{message["text"]}\n```\n\n')
    return "".join(sections)


def write_call_files(pack):
    """One markdown file for each call, with its prompts and its answer."""
    (pack.folder / "calls").mkdir(exist_ok=True)
    for call in pack.calls:
        usage = call["usage"]
        (pack.folder / call["file"]).write_text(
            f'# Call {call["id"]} — {call["kind"]} ({call["model_name"]} via '
            f'{usage.get("host") or call["backend_name"]})\n\n'
            f'{call["created_at"]}. Tokens: {usage.get("prompt")} in, '
            f'{usage.get("completion")} out, {usage.get("cached")} cached. '
            f'Finish: {usage.get("finish")}. Allowance: {usage.get("allowance")}. '
            f'Cost: {usage.get("cost")}.\n\n'
            + (
                f'**FAILED:** {call["error_message"]}\n\n'
                if call["error_message"]
                else ""
            )
            + message_sections(call)
            + f'## Answer\n\n```text\n{call["parsed_response"]}\n```\n',
            encoding="utf-8",
        )


def setup_lines(pack):
    """The test setup table of the pack."""
    setup = pack.setup
    narration_connection = setup["narration_connection"] or {}
    quantifier_connection = setup["quantifier_connection"] or {}
    lines = ["## Test setup\n", "| Part | Value |\n|---|---|"]
    lines.append(f'| World / Persona | `{setup["world"]}` / `{setup["persona"]}` |')
    lines.append(
        f'| Mode, posture | {setup["narrator_mode"]}, {setup["posture"]}; '
        f'Options always on: {setup["options_always_on"]} |'
    )
    lines.append(
        f'| Narration connection | {narration_connection.get("provider")} '
        f'`{narration_connection.get("model")}` '
        f'reasoning={narration_connection.get("reasoning_effort")} '
        f'max_tokens_ceiling={narration_connection.get("max_tokens_ceiling")} |'
    )
    lines.append(
        f'| Quantifier connection | {quantifier_connection.get("provider")} '
        f'`{quantifier_connection.get("model")}` '
        f'reasoning={quantifier_connection.get("reasoning_effort")} |'
    )
    for role, preset in setup["presets"].items():
        lines.append(f'| {role} preset | `{preset.get("id")}` ({preset.get("name", "?")}) |')
    for slot in STORED_SLOTS:
        slot_row = setup[f"{slot}_connection"] or {}
        where = "own slot" if setup[f"{slot}_slot"] else "the Narration Connection"
        lines.append(
            f'| {slot.capitalize()} connection | {slot_row.get("provider")} '
            f'`{slot_row.get("model")}` ({where}) |'
        )
    lines.append(
        f'| Budget | {setup["cycles"]} cycles, token limit {setup["token_limit"]:,}, '
        f'restart limit {setup["restart_limit"]} |'
    )
    lines.append("")
    return lines


def summary_lines(pack):
    """The summary of the pack: the Cycles, the stop, the restarts, the tokens and the
    table of call kinds."""
    run = pack.run
    stop = Stop(run["stop"]["reason"])
    hosts = sorted(
        {call["usage"].get("host") for call in pack.calls if call["usage"].get("host")}
    )
    allowances = [
        call["usage"]["allowance"]
        for call in pack.calls
        if call["usage"].get("allowance") is not None
    ]
    tokens_in = pack.tokens_in
    lines = ["## Summary\n"]
    lines.append(
        f'- Cycles played: {run["cycles_played"]} of {pack.setup["cycles"]}. '
        f'Story entries: {len(pack.story)}. Wall time: {run["seconds"]} s.'
    )
    lines.append(
        f'- Stopped by the token limit: {run["stopped_by_limit"]}. '
        f'Game failure at the end: {run["stop"]["game_failure"] or "none"}.'
    )
    cut = run.get("cut_entries", [])
    if cut:
        lines.append(
            f"- The token limit cut the last Cycle: entries #{cut[0]} to #{cut[-1]} may be "
            "incomplete, and the story shows them marked."
        )
    lines.append(
        f"- Stop: {stop.words}"
        + (f' ({run["stop"]["detail"]})' if run["stop"].get("detail") else "")
        + f'. Newest failed call: {run["stop"]["failed_call"] or "none"} '
        f'(finish_reason `{run["stop"]["finish_reason"] or "none"}`).'
    )
    lines.append(
        f'- Restarts: {len(run["restarts"])} of {run["restart_limit"]}.'
        + "".join(
            f'\n    - after cycle {gap["cycle"]}: finish_reason `{gap["finish_reason"]}`'
            for gap in run["restarts"]
        )
    )
    lines.append(
        f'- Tokens: {tokens_in:,} in, {pack.tokens_out:,} out, '
        f'{pack.tokens_cached:,} cached '
        f'({(100 * pack.tokens_cached / tokens_in if tokens_in else 0):.0f} % of input). '
        f'Hosts: {", ".join(hosts) or "?"}.'
    )
    lines.append(
        "- The token count covers every model call that the server made during the "
        "trial, because `llm_messages` has no Game column. The trial expects a probe "
        "server with no other client."
    )
    if allowances:
        lines.append(
            f'- OpenCode Go allowance, first and last call: '
            f"{allowances[0]} → {allowances[-1]}."
        )
    lines.append("")
    lines.append(
        "| Call kind | Calls | Failed | Tokens in | Tokens out | Cached |\n"
        "|---|---|---|---|---|---|"
    )
    for name, kind in sorted(pack.kinds.items()):
        lines.append(
            f'| {name} | {kind["n"]} | {kind["failed"]} | {kind["in"]:,} | '
            f'{kind["out"]:,} | {kind["cached"]:,} |'
        )
    lines.append("")
    return lines


def failure_and_cut_lines(pack):
    lines = []
    if pack.failed_calls:
        lines.append("### Failed calls\n")
        lines.extend(f"- {failed_call_line(call)}" for call in pack.failed_calls)
        lines.append("")
    first_cut = pack.first_cut_call
    lines.append("### History cut\n")
    if first_cut:
        lines.append(
            f'- First cut: [call {first_cut["id"]}]({first_cut["file"]}) drops '
            f'{len(first_cut["cover"]["dropped"])} entries.'
        )
    else:
        last = next((call for call in reversed(pack.calls) if call["cover"]), None)
        size = (
            f'; the last history block held {last["cover"]["chars"]:,} characters'
            if last
            else ""
        )
        lines.append(
            f"- No story entry fell out of the history in this trial{size}."
        )
    lines.append("")
    return lines


def call_line(call):
    """The line of a call under its story entry."""
    usage = call["usage"]
    cover = ""
    if call["cover"]:
        cover = f', history holds {len(call["cover"]["held"])} entries' + (
            f' (dropped {len(call["cover"]["dropped"])})'
            if call["cover"]["dropped"]
            else ""
        )
    tokens = (
        f'{usage.get("prompt")}+{usage.get("completion")} tok, '
        f'cached {usage.get("cached")}{cover}'
    )
    if call["error_message"]:
        return f"> - {failed_call_line(call, tokens)}"
    answer = ""
    if call["kind"] in NON_STORY_KINDS:
        answer = f' → `{(call["parsed_response"] or "").strip()[:160]}`'
    return f'> - [{call["kind"]} {call["id"]}]({call["file"]}): {tokens}{answer}'


def story_lines(pack):
    """The story, each entry with the calls that made and judged it."""
    lines = ["## Story\n"]
    lines.append(
        "Each entry shows the Room and the Characters present of its Swipe state "
        "(Δ marks a change), then the call that wrote it and the Quantifier call that "
        "judged it. A failed call with no entry is listed under the last entry before it.\n"
    )
    cut = set(pack.run.get("cut_entries", []))
    prev_room, prev_present = None, None
    for entry in pack.story:
        label = entry_label(entry)
        if entry["event_header"]:
            label += f' — {entry["event_header"]}'
        if entry["id"] in cut:
            label += " — cut by the token limit"
        lines.append(f'### #{entry["id"]} {label} ({words(entry["text"])} words)\n')
        lines.append(entry["text"].strip() + "\n")
        room, present = state_line(entry)
        mark = (
            " Δ"
            if (room, present) != (prev_room, prev_present) and prev_room is not None
            else ""
        )
        lines.append(f'> State: room `{room}`, present {present or "nobody"}{mark}')
        prev_room, prev_present = room, present
        lines.extend(call_line(call) for call in pack.by_entry.get(entry["id"], []))
        lines.append("")
    if pack.trailing:
        lines.append("### Calls after the last entry\n")
        for call in pack.trailing:
            lines.append(
                f"- {failed_call_line(call)}"
                if call["error_message"]
                else f'- [{call["kind"]} {call["id"]}]({call["file"]})'
            )
    return lines


def write_pack_markdown(pack):
    """`pack.md`: the setup, the summary, the failures and the story."""
    lines = [f"# Reading pack — {pack.folder.name}\n"]
    lines += setup_lines(pack)
    lines += summary_lines(pack)
    lines += failure_and_cut_lines(pack)
    lines += story_lines(pack)
    (pack.folder / "pack.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_summary(pack):
    """`summary.json`: the numbers that `compare` puts side by side."""
    first_cut = pack.first_cut_call
    summary = {
        "cycles": pack.run["cycles_played"],
        "entries": len(pack.story),
        "seconds": pack.run["seconds"],
        "tokens_in": pack.tokens_in,
        "tokens_out": pack.tokens_out,
        "cached": pack.tokens_cached,
        "kinds": pack.kinds,
        "failed": len(pack.failed_calls),
        "first_cut_call": first_cut["id"] if first_cut else None,
        "words_narration": [
            words(entry["text"])
            for entry in pack.story
            if entry["message_type"] == "Narration"
        ],
        "words_input": [
            words(entry["text"])
            for entry in pack.story
            if entry["message_type"] == "Input"
        ],
    }
    (pack.folder / "summary.json").write_text(
        json.dumps(summary, indent=2), encoding="utf-8"
    )


def export_pack(folder):
    """Write `pack.md`, `summary.json` and one file for each call; answer the pack."""
    pack = load_pack(Path(folder))
    write_call_files(pack)
    write_pack_markdown(pack)
    write_summary(pack)
    return pack


def cmd_export(args):
    """Write the pack of one trial folder."""
    pack = export_pack(args.pack)
    print(f"Wrote {pack.folder / 'pack.md'}")


def compare_packs(packs, out):
    """Write the summary of two or more trial packs side by side, then their stories;
    answer the path of the file it wrote."""
    paths = [Path(pack) for pack in packs]
    sums = [
        json.loads((path / "summary.json").read_text(encoding="utf-8"))
        for path in paths
    ]
    stories = [read_story(path) for path in paths]
    lines = [
        "# Trials side by side\n",
        "| | " + " | ".join(path.name for path in paths) + " |",
        "|---|" + "---|" * len(paths),
    ]

    def row(label, read):
        lines.append(f"| {label} | " + " | ".join(str(read(s)) for s in sums) + " |")

    row("Cycles played", lambda s: s["cycles"])
    row("Story entries", lambda s: s["entries"])
    row("Wall time (s)", lambda s: s["seconds"])
    row("Tokens in / out", lambda s: f'{s["tokens_in"]:,} / {s["tokens_out"]:,}')
    row(
        "Cached input",
        lambda s: f'{(100 * s["cached"] / s["tokens_in"] if s["tokens_in"] else 0):.0f} %',
    )
    row("Failed calls", lambda s: s["failed"])
    row("First history cut (call)", lambda s: s["first_cut_call"] or "none")
    row("Narration words (median)", lambda s: median_of(s["words_narration"]))
    row("Input words (median)", lambda s: median_of(s["words_input"]))
    lines.append("\n## Story, entry by entry\n")
    longest = max(len(story) for story in stories)
    for index in range(longest):
        lines.append(f"### Entry {index + 1}\n")
        for pack, story in zip(paths, stories):
            if index < len(story):
                entry = story[index]
                lines.append(
                    f"**{pack.name}** — {entry_label(entry)} "
                    f"({words(entry['text'])} words)\n"
                )
                lines.append("> " + entry["text"].strip().replace("\n", "\n> ") + "\n")
    path = Path(out) if out else paths[0].parent / "compare.md"
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


def cmd_compare(args):
    """Write the summary of two or more packs side by side."""
    print(f"Wrote {compare_packs(args.packs, args.out)}")


def main():
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    commands = parser.add_subparsers(dest="cmd", required=True)
    run_parser = commands.add_parser("run", help="apply a setup file and play its trials")
    run_parser.add_argument(
        "setup", help="the setup file, such as scripts/trial_setups/redmist_julian.json"
    )
    run_parser.add_argument(
        "--base", default=DEFAULT_BASE, help=f"the probe server ({DEFAULT_BASE})"
    )
    run_parser.add_argument(
        "--db", default=DEFAULT_DB, help=f"the probe server's DB ({DEFAULT_DB})"
    )
    run_parser.add_argument(
        "--cycles",
        type=int,
        default=None,
        help="the Budget of each trial, over the value of the setup file",
    )
    run_parser.add_argument(
        "--timeout",
        type=int,
        default=DEFAULT_TIMEOUT,
        help="seconds to wait for one run before the script stops it",
    )
    export_parser = commands.add_parser("export", help="write a pack from a trial folder")
    export_parser.add_argument("pack")
    compare_parser = commands.add_parser(
        "compare", help="write the summary of two or more packs side by side"
    )
    compare_parser.add_argument("packs", nargs="+")
    compare_parser.add_argument(
        "--out", default=None, help="the summary path (compare.md beside the first pack)"
    )
    args = parser.parse_args()
    try:
        {"run": cmd_run, "export": cmd_export, "compare": cmd_compare}[args.cmd](args)
    except TrialError as refusal:
        sys.exit(f"trial: {refusal}")


if __name__ == "__main__":
    main()
