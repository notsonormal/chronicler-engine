//! Detection tests for the four design-rule walkers: one table-driven test per kind, each case running its walker over synthetic files.

use crate::walkers::{
    check_banned_name_rows, check_call_count_rows, check_call_site_rows, check_type_check_rows,
    BannedName, BannedNameRow, CallCountRow, CallSiteRow, TypeCheckPosition, TypeCheckRow,
};
use crate::SourceFile;

/// The rule number of a fixture row. A fixture proves the walker's reach, never a
/// design rule, so it carries a number outside the design rule set; a case that
/// mirrors a landed row keeps that row's number instead.
const FIXTURE_RULE: u8 = 99;

pub struct WalkerCase<Row> {
    pub row: Row,
    pub files: &'static [(&'static str, &'static str)],
    /// The item each expected violation must name. A case that expects more
    /// than one proves which items the walker flagged, not only how many.
    pub names: &'static [&'static str],
}

fn files_of(fixtures: &[(&str, &str)]) -> Vec<SourceFile> {
    fixtures
        .iter()
        .map(|(path, content)| SourceFile {
            path: (*path).to_string(),
            content: (*content).to_string(),
        })
        .collect()
}

fn run_cases<Row>(
    label: &str,
    cases: &[WalkerCase<Row>],
    walker: fn(&[SourceFile], &[Row]) -> Vec<crate::Violation>,
) {
    for case in cases {
        let violations = walker(&files_of(case.files), std::slice::from_ref(&case.row));
        assert_eq!(
            violations.len(),
            case.names.len(),
            "{label} fixture over {} files: {violations:?}",
            case.files.len()
        );
        for name in case.names {
            let flagged: Vec<&crate::Violation> = violations
                .iter()
                .filter(|violation| violation.message.contains(name))
                .collect();
            assert_eq!(
                flagged.len(),
                1,
                "{label} fixture must flag `{name}` once: {violations:?}"
            );
        }
    }
}

pub const BANNED_NAME_CASES: &[WalkerCase<BannedNameRow>] = &[
    WalkerCase {
        row: BannedNameRow {
            rule: 1,
            scope: "adapters/driven/storage/",
            banned: &[BannedName::StaticItem, BannedName::NamePrefix("Atomic")],
        },
        files: &[
            (
                "adapters/driven/storage/core.rs",
                "static GAME_ID: u64 = 0;\n",
            ),
            (
                "adapters/driven/storage/core.rs",
                "pub struct Storage {\n    id: AtomicU64,\n}\n",
            ),
        ],
        names: &["static item `GAME_ID`", "type `AtomicU64`"],
    },
    WalkerCase {
        row: BannedNameRow {
            rule: FIXTURE_RULE,
            scope: "adapters/driven/storage/",
            banned: &[BannedName::StaticItem],
        },
        files: &[(
            "adapters/driven/storage/core.rs",
            "static GAME_ID: u64 = 0;\n",
        )],
        names: &["static item `GAME_ID`"],
    },
    WalkerCase {
        row: BannedNameRow {
            rule: FIXTURE_RULE,
            scope: "adapters/driven/storage/",
            banned: &[BannedName::NamePrefix("Atomic")],
        },
        files: &[(
            "adapters/driven/storage/core.rs",
            "pub struct Holder {\n    id: AtomicU64,\n}\n",
        )],
        names: &["type `AtomicU64`"],
    },
    WalkerCase {
        row: BannedNameRow {
            rule: FIXTURE_RULE,
            scope: "adapters/driven/storage/",
            banned: &[BannedName::NamePrefix("Atomic")],
        },
        files: &[(
            "adapters/driven/storage/core.rs",
            "use std::sync::atomic::AtomicBool;\n",
        )],
        names: &["import `AtomicBool`"],
    },
    WalkerCase {
        row: BannedNameRow {
            rule: FIXTURE_RULE,
            scope: "adapters/driven/storage/",
            banned: &[BannedName::NamePrefix("Atomic")],
        },
        files: &[(
            "adapters/driven/storage/core.rs",
            "fn make() -> bool {\n    AtomicBool::new(false).load(SeqCst)\n}\n",
        )],
        names: &["path `AtomicBool`"],
    },
    WalkerCase {
        row: BannedNameRow {
            rule: FIXTURE_RULE,
            scope: "adapters/driven/storage/",
            banned: &[BannedName::StaticItem],
        },
        files: &[(
            "adapters/driven/storage/core.rs",
            "pub fn get(game: GameId) -> GameId {\n    game\n}\n",
        )],
        names: &[],
    },
    WalkerCase {
        row: BannedNameRow {
            rule: FIXTURE_RULE,
            scope: "adapters/driven/storage/",
            banned: &[BannedName::NamePrefix("Atomic")],
        },
        files: &[(
            "adapters/driven/storage/core.rs",
            "pub struct Holder {\n    id: GameId,\n}\n",
        )],
        names: &[],
    },
    // The case mirrors the rule-6 row: the writer file names no narrator and no
    // Agent, not even behind a trait object.
    WalkerCase {
        row: BannedNameRow {
            rule: 6,
            scope: "application/generation/gate.rs",
            banned: &[
                BannedName::NamePrefix("Narrator"),
                BannedName::NameContains("Agent"),
            ],
        },
        files: &[
            (
                "application/generation/gate.rs",
                "fn run(narrator: &Narrator) {\n    narrator.narrate(&request);\n}\n",
            ),
            (
                "application/generation/gate.rs",
                "use crate::application::agents::runner::AgentRunner;\n",
            ),
            (
                "application/generation/gate.rs",
                "fn analyse(hook: Box<dyn AnalysisAgent>) {\n    hook.run();\n}\n",
            ),
            (
                "application/generation/gate.rs",
                "fn offer(hook: &OptionsAgent) {\n    hook.run();\n}\n",
            ),
            (
                "application/generation/gate.rs",
                "fn write(&self, text: String) {\n    let kind = MessageKind::Narration;\n}\n",
            ),
        ],
        names: &[
            "type `Narrator`",
            "import `AgentRunner`",
            "trait bound `AnalysisAgent`",
            "type `OptionsAgent`",
        ],
    },
    // The `NameContains` kind catches a family of names that share one word, and
    // leaves other names alone.
    WalkerCase {
        row: BannedNameRow {
            rule: FIXTURE_RULE,
            scope: "application/generation/",
            banned: &[BannedName::NameContains("Agent")],
        },
        files: &[(
            "application/generation/gate.rs",
            "fn draft(&self) -> Draft {\n    let draft = Draft::new();\n    draft\n}\n",
        )],
        names: &[],
    },
    // The trait-bound walk reaches a bound's own path, which is not a type path.
    WalkerCase {
        row: BannedNameRow {
            rule: FIXTURE_RULE,
            scope: "application/generation/",
            banned: &[BannedName::NamePrefix("AnalysisAgent")],
        },
        files: &[(
            "application/generation/gate.rs",
            "fn analyse(hook: Box<dyn AnalysisAgent>) {\n    hook.run();\n}\n",
        )],
        names: &["trait bound `AnalysisAgent`"],
    },
];

pub const CALL_SITE_CASES: &[WalkerCase<CallSiteRow>] = &[
    WalkerCase {
        row: CallSiteRow {
            rule: 2,
            scope: "",
            callees: &["ActiveGame::get", "ActiveGame::set"],
            allowed: &["application/games/catalogue.rs"],
        },
        files: &[
            (
                "application/games/catalogue.rs",
                "fn move_it(&self, game: GameId) {\n    self.active_game.set(game);\n}\n",
            ),
            (
                "application/pipeline/job.rs",
                "fn read(&self) -> Option<GameId> {\n    self.active_game.get()\n}\n",
            ),
            (
                "application/pipeline/job.rs",
                "fn move_it(&self, game: GameId) {\n    self.active_game.set(game);\n}\n",
            ),
        ],
        names: &["call to `ActiveGame::get`", "call to `ActiveGame::set`"],
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callees: &["ActiveGame::get"],
            allowed: &["application/games/catalogue.rs"],
        },
        files: &[(
            "application/pipeline/job.rs",
            "fn run() {\n    let state = ActiveGame::get(&app);\n}\n",
        )],
        names: &["call to `ActiveGame::get`"],
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callees: &["ActiveGame::get"],
            allowed: &["application/games/catalogue.rs"],
        },
        files: &[(
            "application/games/catalogue.rs",
            "fn run() {\n    let state = ActiveGame::get(&app);\n}\n",
        )],
        names: &[],
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callees: &["ActiveGame::get"],
            allowed: &["application/games/"],
        },
        files: &[(
            "application/games/view.rs",
            "fn run() {\n    let state = ActiveGame::get(&app);\n}\n",
        )],
        names: &[],
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callees: &["ActiveGame::get"],
            allowed: &[],
        },
        files: &[(
            "application/pipeline/job.rs",
            "fn run(active_game: &ActiveGame) {\n    let id = active_game.get();\n}\n",
        )],
        names: &["call to `ActiveGame::get`"],
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callees: &["Narrator::narrate"],
            allowed: &[],
        },
        files: &[(
            "application/pipeline/job.rs",
            "fn run(state: &State) {\n    state.narrator.narrate(&request);\n}\n",
        )],
        names: &["call to `Narrator::narrate`"],
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callees: &["ActiveGame::get"],
            allowed: &[],
        },
        files: &[(
            "application/pipeline/job.rs",
            "fn run(job: &Job) {\n    let id = job.get();\n}\n",
        )],
        names: &[],
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callees: &["hold"],
            allowed: &[],
        },
        files: &[(
            "application/generation/gate.rs",
            "fn run() {\n    self.entries.hold(game);\n}\n",
        )],
        names: &["call to `hold`"],
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callees: &["hold"],
            allowed: &[],
        },
        files: &[("adapters/driving/http/action.rs", "pub fn run() {}\n")],
        names: &["matches no file under `application/`"],
    },
    WalkerCase {
        row: CallSiteRow {
            rule: 3,
            scope: "",
            callees: &["TxToken::new"],
            allowed: &["application/generation/gate.rs", "test_support/"],
        },
        files: &[
            (
                "application/generation/gate.rs",
                "fn save(&self) {\n    let token = TxToken::new();\n}\n",
            ),
            (
                "src/adapters/driven/storage/transaction.rs",
                "fn open(&self) {\n    let token = TxToken::new();\n}\n",
            ),
            (
                "test_support/fixtures.rs",
                "pub fn tx_token() -> TxToken {\n    TxToken::new()\n}\n",
            ),
        ],
        names: &["call to `TxToken::new`"],
    },
    // The case mirrors the rule-13 row: only `job.rs` calls the narrator and the
    // hooks of the Agent runner, by path or through a receiver named for the type.
    WalkerCase {
        row: CallSiteRow {
            rule: 13,
            scope: "",
            callees: &[
                "Narrator::narrate",
                "AgentRunner::analysis",
                "AgentRunner::options",
            ],
            allowed: &[
                "application/pipeline/job.rs",
                "application/narration/narrator_tests.rs",
            ],
        },
        files: &[
            (
                "application/pipeline/job.rs",
                "fn narrate_step(&self) {\n    self.narrator.narrate(role);\n    self.agent_runner.analysis(&input);\n    self.agent_runner.options(&input);\n}\n",
            ),
            (
                "application/generation/gate.rs",
                "fn rogue(&self) {\n    self.narrator.narrate(role);\n}\n",
            ),
            (
                "application/games/catalogue.rs",
                "fn rogue(&self) {\n    self.agent_runner.options(&input);\n    AgentRunner::analysis(&runner, &input);\n}\n",
            ),
        ],
        names: &[
            "call to `Narrator::narrate`",
            "call to `AgentRunner::options`",
            "call to `AgentRunner::analysis`",
        ],
    },
];

pub const TYPE_CHECK_CASES: &[WalkerCase<TypeCheckRow>] = &[
    WalkerCase {
        row: TypeCheckRow {
            rule: FIXTURE_RULE,
            scope: "application/pipeline/",
            banned_types: &["Tail", "SwipeState"],
            position: TypeCheckPosition::Fields,
        },
        files: &[(
            "application/pipeline/job.rs",
            "pub struct Job {\n    state: SwipeState,\n}\n",
        )],
        names: &["field type `SwipeState`"],
    },
    WalkerCase {
        row: TypeCheckRow {
            rule: FIXTURE_RULE,
            scope: "application/pipeline/",
            banned_types: &["Tail"],
            position: TypeCheckPosition::Parameters,
        },
        files: &[(
            "application/pipeline/job.rs",
            "pub fn run(state: &Tail) {\n    let _ = state;\n}\n",
        )],
        names: &["parameter type `Tail`"],
    },
    WalkerCase {
        row: TypeCheckRow {
            rule: FIXTURE_RULE,
            scope: "application/pipeline/",
            banned_types: &["Tail"],
            position: TypeCheckPosition::Parameters,
        },
        files: &[(
            "application/pipeline/job.rs",
            "pub struct Job {\n    state: Tail,\n}\n",
        )],
        names: &[],
    },
    WalkerCase {
        row: TypeCheckRow {
            rule: FIXTURE_RULE,
            scope: "application/pipeline/",
            banned_types: &["Tail"],
            position: TypeCheckPosition::Fields,
        },
        files: &[("application/games/catalogue.rs", "pub struct Job {}\n")],
        names: &["matches no file under `application/pipeline/`"],
    },
    // The case mirrors the rule-7 row: a Job field of a record type is caught, a
    // field of an id or a text is not.
    WalkerCase {
        row: TypeCheckRow {
            rule: 7,
            scope: "application/pipeline/job.rs",
            banned_types: &[
                "Game",
                "Message",
                "SwipeState",
                "Tail",
                "DockRecord",
                "WorldBundle",
            ],
            position: TypeCheckPosition::Fields,
        },
        files: &[
            (
                "application/pipeline/job.rs",
                "pub struct Job {\n    writer: JobWriter,\n    state: SwipeState,\n}\n",
            ),
            (
                "application/pipeline/job.rs",
                "pub struct Job {\n    writer: JobWriter,\n    game: GameId,\n    tail: Tail,\n}\n",
            ),
        ],
        names: &["field type `SwipeState`", "field type `Tail`"],
    },
];

pub const CALL_COUNT_CASES: &[WalkerCase<CallCountRow>] = &[
    WalkerCase {
        row: CallCountRow {
            rule: 18,
            scope: "application/generation/gate.rs",
            file: None,
            callee: "hold",
            max_calls: 1,
        },
        files: &[
            (
                "application/generation/gate.rs",
                "fn save(&self) {\n    self.hold(|entries| write(entries));\n}\n",
            ),
            (
                "application/generation/gate.rs",
                "fn save(&self) {\n    self.hold(|entries| check(entries));\n    self.hold(|entries| write(entries));\n}\n",
            ),
        ],
        names: &["calls `hold` 2 times (max 1)"],
    },
    WalkerCase {
        row: CallCountRow {
            rule: FIXTURE_RULE,
            scope: "application/generation/",
            file: None,
            callee: "hold",
            max_calls: 1,
        },
        files: &[(
            "application/generation/gate.rs",
            "fn save() {\n    hold(game);\n    hold(game);\n}\n",
        )],
        names: &["calls `hold` 2 times (max 1)"],
    },
    WalkerCase {
        row: CallCountRow {
            rule: FIXTURE_RULE,
            scope: "application/generation/",
            file: None,
            callee: "hold",
            max_calls: 1,
        },
        files: &[(
            "application/generation/gate.rs",
            "fn save() {\n    hold(game);\n}\n\nfn other() {}\n",
        )],
        names: &[],
    },
    WalkerCase {
        row: CallCountRow {
            rule: FIXTURE_RULE,
            scope: "application/generation/",
            file: None,
            callee: "hold",
            max_calls: 1,
        },
        files: &[(
            "application/generation/gate.rs",
            "fn save() {\n    with(|entries| {\n        hold(entries);\n        hold(entries);\n    });\n}\n",
        )],
        names: &["calls `hold` 2 times (max 1)"],
    },
    WalkerCase {
        row: CallCountRow {
            rule: FIXTURE_RULE,
            scope: "application/generation/",
            file: Some("application/generation/gate.rs"),
            callee: "hold",
            max_calls: 1,
        },
        files: &[("application/games/catalogue.rs", "fn save() {}\n")],
        names: &["matches `application/generation/gate.rs`"],
    },
];

#[test]
fn banned_name_walker_detects_every_fixture() {
    run_cases("banned-name", BANNED_NAME_CASES, |files, rows| {
        check_banned_name_rows(files, rows)
    });
}

#[test]
fn call_site_walker_detects_every_fixture() {
    run_cases("call-site", CALL_SITE_CASES, |files, rows| {
        check_call_site_rows(files, rows)
    });
}

#[test]
fn type_check_walker_detects_every_fixture() {
    run_cases("type-check", TYPE_CHECK_CASES, |files, rows| {
        check_type_check_rows(files, rows)
    });
}

#[test]
fn call_count_walker_detects_every_fixture() {
    run_cases("call-count", CALL_COUNT_CASES, |files, rows| {
        check_call_count_rows(files, rows)
    });
}

#[test]
fn a_dead_path_names_the_rule_and_the_scope() {
    let row = BannedNameRow {
        rule: FIXTURE_RULE,
        scope: "nowhere/",
        banned: &[BannedName::StaticItem],
    };
    let violations = check_banned_name_rows(&files_of(&[("application/x.rs", "")]), &[row]);
    assert_eq!(violations.len(), 1);
    assert!(
        violations[0].message.contains("rule 99") && violations[0].message.contains("nowhere/"),
        "{:?}",
        violations[0].message
    );
}

#[test]
fn a_violation_names_the_rule_its_item_and_the_row_table() {
    let row = BannedNameRow {
        rule: FIXTURE_RULE,
        scope: "adapters/driven/storage/",
        banned: &[BannedName::StaticItem],
    };
    let files = files_of(&[(
        "adapters/driven/storage/core.rs",
        "static GAME_ID: u64 = 0;\n",
    )]);
    let violations = check_banned_name_rows(&files, &[row]);
    assert_eq!(violations.len(), 1);
    let message = &violations[0].message;
    assert!(message.contains("design rule 99"), "{message}");
    assert!(message.contains("static item `GAME_ID`"), "{message}");
    assert!(message.contains("walker_rows.rs"), "{message}");
    assert!(
        message.contains("adapters/driven/storage/core.rs"),
        "{message}"
    );
}
