"""Unit tests for scripts/trial.py: the setup apply, the Cycles count, the poll of a trial,
the restart rule and the reading pack.

The DB is a real in-memory SQLite with the engine's own table shapes, replayed
from ``src/adapters/driven/storage/utils/plumbing.rs``, so the script's SQL runs
against the columns the migrations make. Only the HTTP function is a fake.
"""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import re
import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import trial  # noqa: E402

PLUMBING = SCRIPTS.parent / "src/adapters/driven/storage/utils/plumbing.rs"

READ_TABLES = (
    "settings",
    "games",
    "prompt_presets",
    "messages",
    "message_swipes",
    "llm_messages",
)

EXAMPLE_SETUP = {
    "world": "redmist_estate",
    "persona": "julian",
    "mode": "novel",
    "perspective": "third",
    "tense": "past",
    "presets": {
        "system": "system_default",
        "impersonate": "impersonate_default",
        "quantifier": "quantifier_default",
        "options": "options_default",
    },
    "connections": {"narration": "conn-go", "quantifier": "conn-go"},
    "cycles": 100,
    "token_limit": 6000000,
    "restart_limit": 5,
    "repeats": 1,
    "out": "tmp/pf18/redmist_julian",
}

CONNECTIONS = [
    {
        "id": "conn-go",
        "provider": "OpenCodeGo",
        "model": "deepseek-v4.1-flash",
        "reasoning_effort": "none",
        "max_tokens_ceiling": 4096,
    },
    {"id": "conn-other", "provider": "OpenRouter", "model": "deepseek/deepseek-v4-flash"},
]


def engine_tables(source=None):
    """The column definitions of the engine's tables, replayed from its migrations.

    The replay keeps the last ``CREATE TABLE`` of a table, then each ``ADD COLUMN``,
    ``DROP COLUMN`` and ``RENAME COLUMN`` after it, so the fixture is the schema the
    engine migrates a DB to rather than a hand copy of it. A statement the replay cannot
    read raises, so a migration it would skip shows as a failure.
    """
    if source is None:
        source = PLUMBING.read_text(encoding="utf-8")
    literals = re.finditer(
        r'"((?:[^"\\]|\\.)*)"', re.sub(r"\\\n\s*", " ", source)
    )
    statements = [
        literal.group(1).replace("\\n", "\n").replace('\\"', '"') for literal in literals
    ]
    create = re.compile(r"^CREATE TABLE (?:IF NOT EXISTS )?(\w+) \((.*)\)$", re.S)
    tables = {}
    for statement in statements:
        statement = statement.strip()
        created = create.match(statement)
        if created:
            tables[created.group(1)] = [
                line.strip().rstrip(",")
                for line in created.group(2).splitlines()
                if line.strip() and not line.strip().startswith("--")
            ]
            continue
        if statement.startswith("DROP TABLE"):
            name = re.search(r"DROP TABLE (?:IF EXISTS )?(\w+)", statement).group(1)
            tables.pop(name, None)
            continue
        added = re.match(r"ALTER TABLE (\w+) ADD COLUMN (\w+) (.+)$", statement)
        if added and added.group(1) in tables:
            known = {column.split()[0] for column in tables[added.group(1)]}
            if added.group(2) not in known:
                tables[added.group(1)].append(f"{added.group(2)} {added.group(3)}")
            continue
        dropped = re.match(r"ALTER TABLE (\w+) DROP COLUMN (\w+)$", statement)
        if dropped and dropped.group(1) in tables:
            tables[dropped.group(1)] = [
                column
                for column in tables[dropped.group(1)]
                if column.split()[0] != dropped.group(2)
            ]
            continue
        renamed = re.match(r"ALTER TABLE (\w+) RENAME COLUMN (\w+) TO (\w+)$", statement)
        if renamed and renamed.group(1) in tables:
            old, new = renamed.group(2), renamed.group(3)
            tables[renamed.group(1)] = [
                (new + column[len(old) :]) if column.split()[0] == old else column
                for column in tables[renamed.group(1)]
            ]
            continue
        if statement.startswith("ALTER TABLE"):
            raise ValueError(f"the replay cannot read the statement: {statement}")
    return tables


def make_db():
    """An in-memory DB with the engine's tables and the rows a trial reads."""
    tables = engine_tables()
    conn = sqlite3.connect(":memory:")
    for table in READ_TABLES:
        definitions = ",\n            ".join(tables[table])
        conn.execute(f"CREATE TABLE {table} (\n            {definitions}\n        )")
    conn.execute(
        """insert into settings (id, connections, narration_connection_id,
                                 quantifier_connection_id, agents,
                                 created_at, updated_at)
           values (1, ?, 'conn-go', 'conn-go', '[]',
                   '2026-10-31T00:00:00Z', '2026-10-31T00:00:00Z')""",
        (json.dumps(CONNECTIONS),),
    )
    conn.execute(
        """insert into games (world_name, world_key, name, created_at, updated_at)
           values ('Redmist', 'redmist_estate', 'Trial 1',
                   '2026-10-31T00:00:00Z', '2026-10-31T00:00:00Z')"""
    )
    for preset_id, preset_type in (
        ("system_default", "system"),
        ("impersonate_default", "impersonate"),
        ("quantifier_default", "quantifier"),
        ("options_default", "options"),
    ):
        conn.execute(
            """insert into prompt_presets (id, name, preset_type, is_default,
                                           created_at, updated_at)
               values (?, 'Default', ?, 1, '2026-10-31T00:00:00Z',
                       '2026-10-31T00:00:00Z')""",
            (preset_id, preset_type),
        )
    return conn


def insert_entry(conn, game_id, message_type, text="text"):
    """One story entry of the Game, with the one Swipe it shows."""
    kind = "ImpersonateInput" if message_type == "Input" else "Narration"
    cursor = conn.execute(
        """insert into messages (game_id, message_type, kind, timestamp,
                                 active_swipe_index)
           values (?, ?, ?, '2026-10-31T00:00:00Z', 0)""",
        (game_id, json.dumps(message_type), json.dumps(kind)),
    )
    conn.execute(
        """insert into message_swipes (message_id, swipe_index, text, state, maker)
           values (?, 0, ?, '{}', '{}')""",
        (cursor.lastrowid, text),
    )


class RecordingServer:
    """The fake HTTP function: it records each request, and `POST /games` makes a Game."""

    def __init__(self, conn, refuse=None):
        self.conn = conn
        self.refuse = refuse or {}
        self.requests = []

    def __call__(self, base, method, path, form=None, headers=None):
        self.requests.append((method, path, form, headers))
        if path in self.refuse:
            return self.refuse[path]
        if path == "/games":
            self.conn.execute(
                """insert into games (world_name, world_key, name, created_at,
                                      updated_at)
                   values ('Redmist', 'redmist_estate', 'Trial 2',
                           '2026-10-31T00:00:01Z', '2026-10-31T00:00:01Z')"""
            )
        return 200, ""


def probe_client(server):
    """The client of the probe server, bound to no Game."""
    return trial.ProbeClient("http://probe", server)


def write_setup(directory, data=None, **changes):
    """A setup file in `directory`, with the example values and `changes` over them."""
    payload = dict(EXAMPLE_SETUP if data is None else data)
    payload.update(changes)
    path = Path(directory) / "setup.json"
    path.write_text(json.dumps(payload), encoding="utf-8")
    return path


def without(*names):
    """The example setup without the named fields."""
    return {name: value for name, value in EXAMPLE_SETUP.items() if name not in names}


class SchemaReplayTests(unittest.TestCase):
    """The fixture replays the migrations, so a statement the replay cannot read raises."""

    def test_a_migration_statement_the_replay_cannot_read_raises(self):
        source = (
            'exec("CREATE TABLE games (id INTEGER)")?;'
            'exec("ALTER TABLE games ALTER COLUMN id SET NOT NULL")?;'
        )
        with self.assertRaisesRegex(ValueError, "ALTER TABLE games ALTER COLUMN"):
            engine_tables(source)


class SetupFileTests(unittest.TestCase):
    """The setup file holds the test; the flags hold the machine."""

    def test_a_file_of_the_required_fields_takes_the_decided_defaults(self):
        with tempfile.TemporaryDirectory() as directory:
            path = write_setup(
                directory,
                data=without("cycles", "token_limit", "restart_limit", "repeats"),
            )
            setup = trial.load_setup(path)
        self.assertEqual(setup["cycles"], trial.BUDGET_DEFAULT)
        self.assertEqual(setup["token_limit"], 6_000_000)
        self.assertEqual(setup["restart_limit"], 5)
        self.assertEqual(setup["repeats"], 1)

    def test_the_shipped_example_setup_passes_the_load(self):
        try:
            trial.load_setup(SCRIPTS / "trial_setups" / "redmist_julian.json")
        except trial.TrialError as refusal:
            self.fail(f"the shipped setup is refused: {refusal}")

    def test_the_cycles_flag_overrides_the_file(self):
        with tempfile.TemporaryDirectory() as directory:
            setup = trial.load_setup(write_setup(directory), cycles_override=5)
        self.assertEqual(setup["cycles"], 5)

    def test_a_field_the_script_does_not_read_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            path = write_setup(directory, temperature=0.7)
            with self.assertRaisesRegex(trial.TrialError, "temperature"):
                trial.load_setup(path)

    def test_a_budget_the_route_would_refuse_is_refused_here(self):
        for cycles in (0, 101):
            with tempfile.TemporaryDirectory() as directory:
                path = write_setup(directory, cycles=cycles)
                with self.assertRaisesRegex(trial.TrialError, "1 to 100 cycles"):
                    trial.load_setup(path)


class ApplySetupTests(unittest.TestCase):
    """The setup goes through the routes, in the order the routes need."""

    def test_the_setup_goes_through_the_routes_in_order_with_the_game_header(self):
        conn = make_db()
        server = RecordingServer(conn)
        with tempfile.TemporaryDirectory() as directory:
            setup = trial.load_setup(write_setup(directory))

            client = trial.apply_setup(probe_client(server), setup, conn)

        self.assertEqual(client.game_id, 2)
        header = {"X-Chronicler-Game": "2"}
        self.assertEqual(
            server.requests,
            [
                ("POST", "/connections/set-narrator", {"connection_id": "conn-go"}, {}),
                ("POST", "/connections/set-quantifier", {"connection_id": "conn-go"}, {}),
                (
                    "POST",
                    "/games",
                    {"world_key": "redmist_estate", "persona_key": "julian"},
                    {},
                ),
                ("POST", "/games/2/mode", {"narrator_mode": "novel"}, header),
                (
                    "POST",
                    "/games/2/posture",
                    {"narrative_perspective": "third", "narrative_tense": "past"},
                    header,
                ),
                (
                    "POST",
                    "/games/2/presets",
                    {
                        "system_preset_id": "system_default",
                        "quantifier_preset_id": "quantifier_default",
                        "impersonate_preset_id": "impersonate_default",
                        "options_preset_id": "options_default",
                    },
                    header,
                ),
            ],
        )

    def test_a_refused_route_stops_the_setup_and_names_the_answer(self):
        conn = make_db()
        server = RecordingServer(
            conn, refuse={"/games/2/presets": (400, "<p>the mode forbids it</p>")}
        )
        with tempfile.TemporaryDirectory() as directory:
            setup = trial.load_setup(write_setup(directory))
            with self.assertRaisesRegex(
                trial.TrialError, r"POST /games/2/presets answered 400: <p>the mode"
            ):
                trial.apply_setup(probe_client(server), setup, conn)

    def test_a_game_that_does_not_rise_stops_the_setup(self):
        conn = make_db()
        server = RecordingServer(conn, refuse={"/games": (200, "")})
        with tempfile.TemporaryDirectory() as directory:
            setup = trial.load_setup(write_setup(directory))
            with self.assertRaisesRegex(trial.TrialError, "did not rise"):
                trial.apply_setup(probe_client(server), setup, conn)


class ConnectionCheckTests(unittest.TestCase):
    """The setup names a Connection; it never creates one."""

    def test_a_connection_the_db_holds_is_accepted(self):
        with tempfile.TemporaryDirectory() as directory:
            setup = trial.load_setup(write_setup(directory))
        known = trial.check_connections(make_db(), setup)
        self.assertEqual(sorted(known), ["conn-go", "conn-other"])

    def test_a_connection_the_db_lacks_is_refused_by_its_name(self):
        with tempfile.TemporaryDirectory() as directory:
            setup = trial.load_setup(
                write_setup(
                    directory,
                    connections={"narration": "conn-missing", "quantifier": "conn-go"},
                )
            )
            with self.assertRaisesRegex(trial.TrialError, "conn-missing"):
                trial.check_connections(make_db(), setup)


    def test_a_slot_the_setup_names_must_match_the_slot_the_db_holds(self):
        conn = make_db()
        conn.execute(
            "update settings set impersonate_connection_id = 'conn-other'"
        )
        with tempfile.TemporaryDirectory() as directory:
            matching = trial.load_setup(
                write_setup(
                    directory,
                    connections={
                        "narration": "conn-go",
                        "quantifier": "conn-go",
                        "impersonate": "conn-other",
                    },
                )
            )
            trial.check_connections(conn, matching)
            for slot, wanted, stored in (
                ("impersonate", "conn-go", "conn-other"),
                ("options", "conn-other", "empty"),
            ):
                setup = trial.load_setup(
                    write_setup(
                        directory,
                        connections={
                            "narration": "conn-go",
                            "quantifier": "conn-go",
                            slot: wanted,
                        },
                    )
                )
                with self.assertRaisesRegex(
                    trial.TrialError,
                    rf"{slot} slot `{stored}`.*asks for `{wanted}`.*no route sets it",
                ):
                    trial.check_connections(conn, setup)

    def test_a_setup_that_names_no_slot_accepts_the_stored_one(self):
        conn = make_db()
        conn.execute("update settings set options_connection_id = 'conn-other'")
        with tempfile.TemporaryDirectory() as directory:
            setup = trial.load_setup(write_setup(directory))
        trial.check_connections(conn, setup)
        snapshot = trial.setup_snapshot(conn, setup, 1)
        self.assertEqual(snapshot["options_connection"]["id"], "conn-other")
        self.assertEqual(snapshot["impersonate_connection"]["id"], "conn-go")
        self.assertIsNone(snapshot["impersonate_slot"])
        self.assertNotIn("response_length", snapshot)


class RunRefusalTests(unittest.TestCase):
    """`run` refuses before it touches the probe server."""

    def run_arguments(self, directory, db_path):
        return argparse.Namespace(
            setup=str(write_setup(directory, out=str(Path(directory) / "pack"))),
            db=str(db_path),
            cycles=None,
            base="http://probe",
            timeout=60,
        )

    def test_a_missing_db_path_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            arguments = self.run_arguments(directory, Path(directory) / "missing.db")
            server = RecordingServer(make_db())
            with self.assertRaisesRegex(trial.TrialError, "does not exist"):
                trial.cmd_run(arguments, transport=server, connect=lambda path: make_db())
            self.assertEqual(server.requests, [])

    def test_a_missing_connection_is_refused_before_any_request(self):
        with tempfile.TemporaryDirectory() as directory:
            db_path = Path(directory) / "probe.db"
            db_path.write_bytes(b"")
            conn = make_db()
            arguments = self.run_arguments(directory, db_path)
            arguments.setup = str(
                write_setup(
                    directory,
                    connections={
                        "narration": "conn-missing",
                        "quantifier": "conn-go",
                    },
                    out=str(Path(directory) / "pack"),
                )
            )
            server = RecordingServer(conn)
            with self.assertRaisesRegex(trial.TrialError, "conn-missing"):
                trial.cmd_run(arguments, transport=server, connect=lambda path: conn)
            self.assertEqual(server.requests, [])
            self.assertFalse((Path(directory) / "pack").exists())


class CyclesPlayedTests(unittest.TestCase):
    """A Cycle ends only on an answered Input."""

    def cycles_for(self, message_types):
        conn = make_db()
        for message_type in message_types:
            insert_entry(conn, 1, message_type)
        return trial.cycles_played(trial.story_rows(conn, 1))

    def test_an_answered_input_counts_one_cycle(self):
        self.assertEqual(self.cycles_for(["Input", "Narration"]), 1)

    def test_each_answered_input_counts_its_own_cycle(self):
        self.assertEqual(
            self.cycles_for(["Input", "Narration", "Input", "Narration"]), 2
        )

    def test_an_input_the_story_ends_on_does_not_count(self):
        self.assertEqual(
            self.cycles_for(["Input", "Narration", "Input", "Narration", "Input"]), 2
        )

    def test_a_story_of_narrations_alone_has_no_cycle(self):
        self.assertEqual(self.cycles_for(["Narration"]), 0)


class StopDecisionTests(unittest.TestCase):
    """A filtered answer restarts the run; any other end stops the trial."""

    def decide(self, **changes):
        facts = {
            "budget": 100,
            "cycles": 10,
            "tokens": 1_000,
            "token_limit": 6_000_000,
            "restarts": 0,
            "restart_limit": 5,
            "finish_reason": "content_filter",
        }
        facts.update(changes)
        return trial.stop_decision(**facts)

    def test_a_filtered_answer_restarts_while_a_restart_remains(self):
        self.assertEqual(self.decide(), trial.StopDecision(True, trial.Stop.FILTERED))

    def test_a_filtered_answer_at_the_restart_limit_stops(self):
        self.assertEqual(
            self.decide(restarts=5), trial.StopDecision(False, trial.Stop.RESTART_LIMIT)
        )

    def test_a_spent_token_limit_stops_even_a_filtered_answer(self):
        self.assertEqual(
            self.decide(tokens=6_000_001), trial.StopDecision(False, trial.Stop.TOKEN_LIMIT)
        )

    def test_another_reason_stops_the_trial(self):
        self.assertEqual(
            self.decide(finish_reason="stop"), trial.StopDecision(False, trial.Stop.FAILED)
        )
        self.assertEqual(
            self.decide(finish_reason=None), trial.StopDecision(False, trial.Stop.FAILED)
        )

    def test_a_played_budget_stops_the_trial(self):
        self.assertEqual(
            self.decide(cycles=100), trial.StopDecision(False, trial.Stop.BUDGET)
        )


class PackTests(unittest.TestCase):
    """The pack reads the tokens and the finish reason out of the stored raw answer."""

    def test_usage_reads_the_tokens_and_the_finish_reason_of_a_canned_answer(self):
        call = {
            "raw_response_json": json.dumps(
                {
                    "provider": "OpenCodeGo",
                    "usage": {
                        "prompt_tokens": 1200,
                        "completion_tokens": 0,
                        "prompt_tokens_details": {"cached_tokens": 300},
                        "opencode": {"allowance": 42},
                    },
                    "choices": [{"finish_reason": "content_filter"}],
                }
            )
        }
        self.assertEqual(
            trial.usage_of(call),
            {
                "prompt": 1200,
                "completion": 0,
                "cached": 300,
                "cost": 0,
                "allowance": 42,
                "host": "OpenCodeGo",
                "finish": "content_filter",
            },
        )

    def test_the_pack_line_of_a_failed_call_names_its_finish_reason(self):
        call = {
            "kind": "impersonate",
            "id": 507,
            "file": "calls/00507-impersonate.md",
            "error_message": "LLM returned an empty response",
            "usage": {"finish": "content_filter"},
        }
        self.assertEqual(
            trial.failed_call_line(call),
            "[impersonate 507](calls/00507-impersonate.md): **FAILED** LLM returned an "
            "empty response (finish_reason `content_filter`)",
        )


CONTENT_FILTER_TEXT = "LLM returned an empty response (finish_reason: content_filter)"
GAME_HEADER = {"X-Chronicler-Game": "2"}


class FakeClock:
    """A clock that a sleep moves forward, so the poll runs with no wait."""

    def __init__(self):
        self.time = 1000.0

    def now(self):
        return self.time

    def sleep(self, seconds):
        self.time += seconds

    def clock(self):
        return trial.Clock(self.sleep, self.now)


def add_call(conn, agent_name, finish="stop", error=None, prompt=100, completion=50):
    """One model call of the debug record, with the raw answer that the script reads."""
    raw = json.dumps(
        {
            "provider": "OpenCodeGo",
            "usage": {"prompt_tokens": prompt, "completion_tokens": completion},
            "choices": [{"finish_reason": finish}],
        }
    )
    conn.execute(
        """insert into llm_messages (agent_name, backend_name, model_name,
                                     system_prompt, user_prompt, raw_request_json,
                                     raw_response_json, parsed_response, error_message,
                                     created_at)
           values (?, 'OpenCodeGo', 'model', '', '', '{}', ?, '', ?,
                   '2026-10-31T00:00:00Z')""",
        (agent_name, raw, error),
    )


def add_filtered_story_call(conn):
    add_call(conn, "narrator", "content_filter", CONTENT_FILTER_TEXT, completion=0)


def add_filtered_quantifier_call(conn):
    add_call(conn, "quantifier", "content_filter", CONTENT_FILTER_TEXT, completion=0)


def play_cycles(conn, count, game_id=2):
    """`count` answered Cycles: an Input and its Narration, with the calls that made them."""
    for _ in range(count):
        insert_entry(conn, game_id, "Input")
        insert_entry(conn, game_id, "Narration")
        add_call(conn, "narrator")
        add_call(conn, "quantifier")


def set_failure(conn, text):
    conn.execute("update games set failure = ? where id = 2", (text,))


class ProbeServer(RecordingServer):
    """The fake probe server. `POST /autoplay` plays the next scripted run against the DB,
    and the Game stays busy for `busy_polls` reads of `GET /debug/is_busy`, or until
    `reset-busy`. A reset clears the failure line, as the engine does."""

    def __init__(self, conn, runs, busy_polls=1, busy_answer=None):
        super().__init__(conn)
        self.runs = list(runs)
        self.busy_polls = busy_polls
        self.busy_left = 0
        self.busy_answer = busy_answer

    def __call__(self, base, method, path, form=None, headers=None):
        if path == "/autoplay":
            self.requests.append((method, path, form, headers))
            self.runs.pop(0)(self.conn)
            self.busy_left = self.busy_polls
            return 200, ""
        if path == "/debug/is_busy":
            self.requests.append((method, path, form, headers))
            if self.busy_answer:
                return self.busy_answer
            busy = self.busy_left > 0
            self.busy_left -= 1
            return 200, "true" if busy else "false"
        if path == "/status/reset-busy":
            self.requests.append((method, path, form, headers))
            self.busy_left = 0
            set_failure(self.conn, None)
            return 200, "reset"
        return super().__call__(base, method, path, form, headers)

    def autoplay_cycles(self):
        return [form["cycles"] for _, path, form, _ in self.requests if path == "/autoplay"]

    def paths(self):
        return [path for _, path, _, _ in self.requests]


class PlayTrialTests(unittest.TestCase):
    """The poll of a trial, driven with a fake clock, a fake probe server and a real DB."""

    def play(self, runs, timeout=3600, busy_polls=1, busy_answer=None, **changes):
        """Play one trial, and answer the server, the run record and the pack folder."""
        conn = make_db()
        self.addCleanup(conn.close)
        server = ProbeServer(conn, runs, busy_polls, busy_answer)
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        setup = trial.load_setup(write_setup(directory.name, cycles=5, **changes))
        pack = Path(directory.name) / "pack"
        with contextlib.redirect_stdout(io.StringIO()):
            run = trial.play_trial(
                probe_client(server), conn, setup, pack, timeout, FakeClock().clock()
            )
        return server, run, pack

    def test_a_story_call_filtered_at_the_end_of_a_run_restarts_with_the_cycles_that_remain(
        self,
    ):
        def filtered_after_two_cycles(conn):
            play_cycles(conn, 2)
            add_filtered_story_call(conn)

        server, run, _ = self.play([filtered_after_two_cycles, lambda c: play_cycles(c, 3)])

        self.assertEqual(server.autoplay_cycles(), ["5", "3"])
        self.assertEqual(
            run["restarts"], [{"cycle": 2, "finish_reason": "content_filter"}]
        )
        self.assertEqual(run["stop"]["reason"], "budget")

    def test_a_filtered_quantifier_call_at_the_end_of_a_run_does_not_restart(self):
        def quantifier_filtered_last(conn):
            play_cycles(conn, 2)
            add_filtered_quantifier_call(conn)

        server, run, _ = self.play([quantifier_filtered_last])

        self.assertEqual(server.autoplay_cycles(), ["5"])
        self.assertEqual(run["restarts"], [])
        self.assertEqual(run["stop"]["reason"], "failed")

    def test_a_filtered_quantifier_call_before_a_good_call_does_not_restart(self):
        def quantifier_filtered_earlier(conn):
            play_cycles(conn, 2)
            add_filtered_quantifier_call(conn)
            add_call(conn, "narrator")

        server, run, _ = self.play([quantifier_filtered_earlier])

        self.assertEqual(server.autoplay_cycles(), ["5"])
        self.assertEqual(run["stop"]["reason"], "failed")

    def test_a_story_call_filtered_at_the_restart_limit_stops_the_trial(self):
        def filtered_after_two_cycles(conn):
            play_cycles(conn, 2)
            add_filtered_story_call(conn)

        server, run, _ = self.play(
            [filtered_after_two_cycles, add_filtered_story_call], restart_limit=1
        )

        self.assertEqual(server.autoplay_cycles(), ["5", "3"])
        self.assertEqual(run["stop"]["reason"], "restart_limit")

    def test_every_busy_read_carries_the_game_header(self):
        server, _, _ = self.play([lambda c: play_cycles(c, 5)])

        reads = [headers for _, path, _, headers in server.requests if path == "/debug/is_busy"]
        self.assertTrue(reads)
        self.assertEqual(reads, [GAME_HEADER] * len(reads))

    def test_a_busy_read_that_names_a_changed_game_stops_the_trial_and_writes_the_pack(self):
        server, run, pack = self.play(
            [lambda c: play_cycles(c, 1)],
            busy_answer=(409, "<p>The active Game changed</p>"),
        )

        self.assertEqual(run["stop"]["reason"], "game_changed")
        self.assertNotIn("/status/reset-busy", server.paths())
        self.assertIn(
            "the active Game changed",
            (pack / "pack.md").read_text(encoding="utf-8"),
        )

    def test_a_busy_read_that_another_status_refuses_stops_the_trial_and_writes_the_pack(self):
        _, run, pack = self.play(
            [lambda c: play_cycles(c, 1)], busy_answer=(500, "<p>engine fault</p>")
        )

        self.assertEqual(run["stop"]["reason"], "probe_refused")
        self.assertIn("answered 500", run["stop"]["detail"])
        self.assertTrue((pack / "pack.md").is_file())

    def spend_the_token_limit(self):
        def two_cycles_that_spend_the_limit(conn):
            play_cycles(conn, 2)
            set_failure(conn, "the model failed")

        return self.play(
            [two_cycles_that_spend_the_limit], busy_polls=float("inf"), token_limit=100
        )

    def test_the_token_limit_stops_the_run_after_the_failure_line_is_read(self):
        server, run, _ = self.spend_the_token_limit()

        self.assertEqual(run["stop"]["reason"], "token_limit")
        self.assertEqual(server.paths().count("/status/reset-busy"), 1)
        self.assertEqual(run["stop"]["game_failure"], "the model failed")

    def test_the_token_limit_marks_the_last_cycle_as_cut(self):
        _, run, pack = self.spend_the_token_limit()

        self.assertEqual(run["cut_entries"], [3, 4])
        text = (pack / "pack.md").read_text(encoding="utf-8")
        marked = re.findall(r"^### #(\d+) .*cut by the token limit", text, re.M)
        self.assertEqual(marked, ["3", "4"])
        self.assertIn("The token limit cut the last Cycle", text)

    def test_a_trial_that_stays_under_the_token_limit_marks_no_cycle(self):
        _, run, pack = self.play([lambda c: play_cycles(c, 5)])

        self.assertEqual(run["cut_entries"], [])
        self.assertNotIn("cut by the token limit", (pack / "pack.md").read_text("utf-8"))

    def stop_by_timeout(self):
        def run_that_never_ends(conn):
            play_cycles(conn, 1)
            set_failure(conn, "the model failed")

        return self.play([run_that_never_ends], timeout=10, busy_polls=float("inf"))

    def test_the_timeout_stops_the_run_with_reset_busy_after_the_failure_line_is_read(self):
        server, run, _ = self.stop_by_timeout()

        self.assertEqual(run["stop"]["reason"], "timeout")
        self.assertEqual(server.paths().count("/status/reset-busy"), 1)
        self.assertEqual(run["stop"]["game_failure"], "the model failed")

    def test_the_pack_says_that_the_timeout_stopped_the_run_with_reset_busy(self):
        _, _, pack = self.stop_by_timeout()

        stop_line = next(
            line
            for line in (pack / "pack.md").read_text(encoding="utf-8").splitlines()
            if line.startswith("- Stop:")
        )
        self.assertIn("`--timeout`", stop_line)
        self.assertIn("`reset-busy`", stop_line)

    def test_the_pack_and_the_summary_count_the_same_cycles_when_an_input_is_unanswered(self):
        def two_cycles_and_an_unanswered_input(conn):
            play_cycles(conn, 2)
            insert_entry(conn, 2, "Input")

        _, _, pack = self.play([two_cycles_and_an_unanswered_input])

        self.assertIn(
            "Cycles played: 2 of 5", (pack / "pack.md").read_text(encoding="utf-8")
        )
        self.assertEqual(
            json.loads((pack / "summary.json").read_text(encoding="utf-8"))["cycles"], 2
        )
        compare = pack.parent / "compare.md"
        trial.cmd_compare(argparse.Namespace(packs=[str(pack)], out=str(compare)))
        self.assertIn("| Cycles played | 2 |", compare.read_text(encoding="utf-8"))

    def test_the_pack_says_that_the_token_count_covers_every_call_of_the_server(self):
        _, _, pack = self.play([lambda c: play_cycles(c, 5)])

        self.assertIn(
            "covers every model call that the server made during the trial",
            (pack / "pack.md").read_text(encoding="utf-8"),
        )


def stored_history_sizes(conn):
    stored = json.loads(conn.execute("select connections from settings").fetchone()[0])
    return {connection["id"]: connection.get("max_history_tokens") for connection in stored}


class HistorySizeTests(unittest.TestCase):
    def load(self, **changes):
        with tempfile.TemporaryDirectory() as directory:
            return trial.load_setup(write_setup(directory, **changes))

    def test_a_setup_without_the_field_removes_the_size_an_earlier_trial_left(self):
        conn = make_db()
        self.addCleanup(conn.close)
        conn.execute("update settings set impersonate_connection_id = 'conn-other'")
        trial.set_history_size(conn, self.load(history_tokens=16000))
        self.assertEqual(
            stored_history_sizes(conn), {"conn-go": 16000, "conn-other": 16000}
        )
        setup = self.load()

        trial.set_history_size(conn, setup)

        self.assertIsNone(setup["history_tokens"])
        self.assertEqual(
            stored_history_sizes(conn), {"conn-go": None, "conn-other": None}
        )
        kept = json.loads(conn.execute("select connections from settings").fetchone()[0])
        self.assertEqual(kept[0]["max_tokens_ceiling"], 4096)

    def test_the_field_is_read_from_the_file(self):
        self.assertEqual(self.load(history_tokens=16000)["history_tokens"], 16000)

    def test_a_size_that_is_not_a_whole_number_of_one_or_more_is_refused(self):
        for size in (0, -1, 1.5, "16000", True):
            with self.subTest(size=size):
                with self.assertRaisesRegex(trial.TrialError, "history_tokens"):
                    self.load(history_tokens=size)

    def test_the_size_goes_on_the_narration_connection_and_no_other(self):
        conn = make_db()
        self.addCleanup(conn.close)
        setup = self.load(
            history_tokens=16000,
            connections={"narration": "conn-go", "quantifier": "conn-other"},
        )

        trial.set_history_size(conn, setup)

        self.assertEqual(
            stored_history_sizes(conn), {"conn-go": 16000, "conn-other": None}
        )
        kept = json.loads(conn.execute("select connections from settings").fetchone()[0])
        self.assertEqual(kept[0]["max_tokens_ceiling"], 4096)

    def test_the_size_goes_on_the_impersonate_connection_when_it_has_its_own_slot(self):
        conn = make_db()
        self.addCleanup(conn.close)
        conn.execute("update settings set impersonate_connection_id = 'conn-other'")
        setup = self.load(history_tokens=16000)

        trial.set_history_size(conn, setup)

        self.assertEqual(
            stored_history_sizes(conn), {"conn-go": 16000, "conn-other": 16000}
        )

    def test_run_writes_the_size_before_it_sends_any_request(self):
        conn = make_db()
        self.addCleanup(conn.close)
        seen = []

        class Refusing(RecordingServer):
            def __call__(self, base, method, path, form=None, headers=None):
                seen.append(stored_history_sizes(conn))
                return 500, "refused"

        with tempfile.TemporaryDirectory() as directory:
            db_path = Path(directory) / "probe.db"
            db_path.write_bytes(b"")
            arguments = argparse.Namespace(
                setup=str(
                    write_setup(
                        directory, history_tokens=16000, out=str(Path(directory) / "pack")
                    )
                ),
                db=str(db_path),
                cycles=None,
                base="http://probe",
                timeout=60,
            )
            with contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaises(trial.RouteRefused):
                    trial.cmd_run(
                        arguments, transport=Refusing(conn), connect=lambda path: conn
                    )

        self.assertEqual(seen[0]["conn-go"], 16000)

    def test_the_pack_shows_the_size_the_connection_held_for_the_run(self):
        conn = make_db()
        self.addCleanup(conn.close)
        setup = self.load(history_tokens=16000, cycles=5)
        trial.set_history_size(conn, setup)
        server = ProbeServer(conn, [lambda c: play_cycles(c, 5)])
        with tempfile.TemporaryDirectory() as directory:
            pack = Path(directory) / "pack"
            with contextlib.redirect_stdout(io.StringIO()):
                trial.play_trial(
                    probe_client(server), conn, setup, pack, 3600, FakeClock().clock()
                )
            text = (pack / "pack.md").read_text(encoding="utf-8")

        self.assertIn("| History size | 16,000 tokens |", text)

    def test_the_pack_of_a_connection_with_no_size_shows_the_default(self):
        conn = make_db()
        self.addCleanup(conn.close)
        setup = self.load(cycles=5)
        server = ProbeServer(conn, [lambda c: play_cycles(c, 5)])
        with tempfile.TemporaryDirectory() as directory:
            pack = Path(directory) / "pack"
            with contextlib.redirect_stdout(io.StringIO()):
                trial.play_trial(
                    probe_client(server), conn, setup, pack, 3600, FakeClock().clock()
                )
            text = (pack / "pack.md").read_text(encoding="utf-8")

        self.assertIn("| History size | 65,536 tokens (the default) |", text)


def payload_of(*messages):
    return json.dumps(
        {"model": "m", "messages": [{"role": role, "content": text} for role, text in messages]}
    )


def stored_call(call_id, agent, system="", user="", payload="", answer="", created="2026-11-02T10:00:10Z"):
    """An old row fills the two prompt columns; a new row fills `raw_request_json`."""
    return {
        "id": call_id,
        "agent_name": agent,
        "backend_name": "OpenCodeGo",
        "model_name": "model",
        "system_prompt": system,
        "user_prompt": user,
        "raw_request_json": payload,
        "raw_response_json": "{}",
        "parsed_response": answer,
        "error_message": None,
        "created_at": created,
    }


def story_entry(entry_id, text, message_type, impersonated=False):
    return {
        "id": entry_id,
        "text": text,
        "message_type": json.dumps(message_type),
        "impersonated": impersonated,
        "timestamp": f"2026-11-02T10:00:0{entry_id}Z",
        "state": "{}",
    }


OPENING = "The gate stands open and the lamps are lit."
FIRST_INPUT = "Julian asks the porter about the gate."
REPLY = "The porter shrugs and points at the steps."

STORY = [
    story_entry(1, OPENING, "Narration"),
    story_entry(2, FIRST_INPUT, "Input", impersonated=True),
    story_entry(3, REPLY, "Narration"),
]


def write_pack_folder(directory, calls, story=STORY):
    folder = Path(directory) / "pack"
    folder.mkdir()
    (folder / "setup.json").write_text("{}", encoding="utf-8")
    (folder / "run.json").write_text("{}", encoding="utf-8")
    (folder / "story.json").write_text(json.dumps(story), encoding="utf-8")
    (folder / "calls.jsonl").write_text(
        "".join(json.dumps(call) + "\n" for call in calls), encoding="utf-8"
    )
    return folder


class SentMessagesTests(unittest.TestCase):
    def test_an_old_row_has_its_two_columns_as_messages(self):
        call = stored_call(1, "narrator", system="rules", user="blocks and history")

        self.assertEqual(
            trial.sent_messages(call),
            [
                {"role": "system", "text": "rules"},
                {"role": "user", "text": "blocks and history"},
            ],
        )

    def test_a_new_row_has_the_messages_of_its_payload_in_order(self):
        call = stored_call(
            1,
            "narrator",
            payload=payload_of(
                ("system", "rules"),
                ("user", "blocks"),
                ("assistant", OPENING),
                ("user", FIRST_INPUT),
            ),
        )

        self.assertEqual(
            [(message["role"], message["text"]) for message in trial.sent_messages(call)],
            [
                ("system", "rules"),
                ("user", "blocks"),
                ("assistant", OPENING),
                ("user", FIRST_INPUT),
            ],
        )

    def test_a_row_of_a_call_that_failed_before_any_send_has_no_message(self):
        self.assertEqual(trial.sent_messages(stored_call(1, "narrator")), [])
        self.assertEqual(
            trial.sent_messages(stored_call(1, "narrator", payload="{}")), []
        )


class PackOfNewRowsTests(unittest.TestCase):
    def load(self, calls):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        return trial.load_pack(write_pack_folder(directory.name, calls)), directory

    def test_the_kind_of_a_new_impersonate_row_comes_from_its_system_message(self):
        call = stored_call(
            5,
            "narrator",
            payload=payload_of(
                ("system", "You write as the player character."),
                ("user", "blocks"),
                ("assistant", OPENING),
            ),
            answer="An answer that reached no entry.",
        )

        pack, _ = self.load([call])

        self.assertEqual(pack.calls[0]["kind"], "impersonate")

    def test_a_new_row_names_the_entries_its_history_messages_hold_and_the_ones_it_drops(self):
        call = stored_call(
            5,
            "narrator",
            payload=payload_of(
                ("system", "rules"),
                ("user", "blocks before"),
                ("assistant", f"<ConversationHistory>\n{OPENING}"),
                ("user", f"<LastMessage>\n{FIRST_INPUT}\n</LastMessage>\n\n<GameState>"),
            ),
            answer=REPLY,
            created="2026-11-02T10:00:09Z",
        )
        story = STORY + [story_entry(4, "A later entry.", "Narration")]
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)

        pack = trial.load_pack(write_pack_folder(directory.name, [call], story))

        cover = pack.calls[0]["cover"]
        self.assertEqual(cover["held"], [1, 2])
        self.assertEqual(cover["dropped"], [3, 4])

    def test_a_new_row_with_an_empty_system_part_keeps_its_first_history_entry(self):
        call = stored_call(
            5,
            "narrator",
            payload=payload_of(
                ("user", "blocks before"),
                ("assistant", f"<ConversationHistory>\n{OPENING}"),
                ("user", f"<LastMessage>\n{FIRST_INPUT}\n</LastMessage>\n\n<GameState>"),
            ),
            answer=REPLY,
            created="2026-11-02T10:00:09Z",
        )

        pack, _ = self.load([call])

        self.assertEqual(pack.calls[0]["cover"]["held"], [1, 2])

    def test_a_new_row_with_an_empty_history_holds_no_entry(self):
        call = stored_call(
            5,
            "narrator",
            payload=payload_of(
                ("system", "rules"),
                (
                    "user",
                    "<ConversationHistory>\n(No history yet)\n</ConversationHistory>\n\n<GameState>",
                ),
            ),
            answer=REPLY,
            created="2026-11-02T10:00:09Z",
        )

        pack, _ = self.load([call])

        self.assertEqual(pack.calls[0]["cover"]["held"], [])

    def test_a_new_narrator_row_of_one_merged_message_stops_the_pack_with_its_call_id(self):
        merged = f"[SYSTEM]\nrules\n\nblocks\n\n[ASSISTANT]\n{OPENING}\n\n{FIRST_INPUT}"
        call = stored_call(7, "narrator", payload=payload_of(("user", merged)))
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        folder = write_pack_folder(directory.name, [call])

        for action in (trial.load_pack, trial.export_pack):
            with self.assertRaises(trial.TrialError) as stopped:
                action(folder)
            self.assertIn("7", str(stopped.exception))
            self.assertIn("single_user_message", str(stopped.exception))

    def test_a_merged_message_with_an_empty_system_part_and_an_assistant_header_stops_too(self):
        merged = f"blocks\n\n[ASSISTANT]\n{OPENING}\n\n{FIRST_INPUT}"
        call = stored_call(8, "narrator", payload=payload_of(("user", merged)))

        with self.assertRaises(trial.TrialError):
            self.load([call])

    def test_a_new_row_with_a_cut_history_names_the_dropped_entry(self):
        call = stored_call(
            5,
            "narrator",
            payload=payload_of(
                ("system", "rules"),
                ("user", "blocks before"),
                ("assistant", f"<LastMessage>\n{FIRST_INPUT}\n</LastMessage>"),
            ),
            answer=REPLY,
            created="2026-11-02T10:00:02Z",
        )

        pack, _ = self.load([call])

        self.assertEqual(pack.calls[0]["cover"]["held"], [2])
        self.assertEqual(pack.calls[0]["cover"]["dropped"], [1])
        self.assertIs(pack.first_cut_call, pack.calls[0])

    def test_an_old_row_still_finds_its_history_in_the_conversation_history_tag(self):
        user = f"<ConversationHistory>\nNarrator: {OPENING}\n</ConversationHistory>\n\n"
        call = stored_call(
            5,
            "narrator",
            system="rules",
            user=user,
            answer=REPLY,
            created="2026-11-02T10:00:02Z",
        )

        pack, _ = self.load([call])

        cover = pack.calls[0]["cover"]
        self.assertEqual(cover["held"], [1])
        self.assertEqual(cover["dropped"], [2])

    def test_the_quantifier_owner_comes_from_the_latest_message_of_a_new_row(self):
        call = stored_call(
            6,
            "quantifier",
            payload=payload_of(
                ("system", "You find who is present."),
                ("user", f"<LatestMessage>\n{REPLY}\n</LatestMessage>"),
            ),
            created="2026-11-02T10:00:01Z",
        )

        pack, _ = self.load([call])

        self.assertEqual([c["id"] for c in pack.by_entry[3]], [6])

    def test_the_quantifier_owner_comes_from_the_latest_message_of_an_old_row(self):
        call = stored_call(
            6,
            "quantifier",
            system="You find who is present.",
            user=f"<LatestMessage>\n{REPLY}\n</LatestMessage>",
            created="2026-11-02T10:00:01Z",
        )

        pack, _ = self.load([call])

        self.assertEqual([c["id"] for c in pack.by_entry[3]], [6])

    def test_the_call_file_of_a_new_row_shows_each_message_under_its_role(self):
        call = stored_call(
            5,
            "narrator",
            payload=payload_of(
                ("system", "rules"),
                ("user", "blocks"),
                ("assistant", OPENING),
                ("user", FIRST_INPUT),
            ),
            answer=REPLY,
        )
        pack, _ = self.load([call])

        trial.write_call_files(pack)

        text = (pack.folder / pack.calls[0]["file"]).read_text(encoding="utf-8")
        headings = re.findall(r"^## (.*)$", text, re.M)
        self.assertEqual(
            headings, ["1. System", "2. User", "3. Assistant", "4. User", "Answer"]
        )
        self.assertIn(OPENING, text)

    def test_the_call_file_of_an_old_row_keeps_its_two_headings(self):
        call = stored_call(5, "narrator", system="rules", user="blocks", answer=REPLY)
        pack, _ = self.load([call])

        trial.write_call_files(pack)

        text = (pack.folder / pack.calls[0]["file"]).read_text(encoding="utf-8")
        self.assertEqual(re.findall(r"^## (.*)$", text, re.M), ["System", "User", "Answer"])


if __name__ == "__main__":
    unittest.main()
