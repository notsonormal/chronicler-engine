//! Detection tests for the four design-rule walkers: one table-driven test per kind, each case running its walker over synthetic files.

use crate::walkers::{
    check_banned_name_rows, check_call_count_rows, check_call_site_rows, check_type_check_rows,
    BannedName, BannedNameRow, CallCountRow, CallSiteRow, TypeCheckPosition, TypeCheckRow,
};
use crate::SourceFile;

/// The rule number of a fixture row. A fixture proves the walker's reach, never a
/// design rule, so it carries a number outside the design rule set.
const FIXTURE_RULE: u8 = 99;

pub struct WalkerCase<Row> {
    pub row: Row,
    pub files: &'static [(&'static str, &'static str)],
    pub expected: usize,
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
            case.expected,
            "{label} fixture over {} files: {violations:?}",
            case.files.len()
        );
    }
}

/// The banned-name cases: a `static` item, a prefix in each name position, an
/// unused name, and a dead path.
pub const BANNED_NAME_CASES: &[WalkerCase<BannedNameRow>] = &[
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
        expected: 1,
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
        expected: 1,
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
        expected: 1,
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
        expected: 1,
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
        expected: 0,
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
        expected: 0,
    },
];

/// The call-site cases: an allowed file, an allowed folder, a disallowed file,
/// both method-call shapes, a receiver of another name, a bare name, and a dead path.
pub const CALL_SITE_CASES: &[WalkerCase<CallSiteRow>] = &[
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callee: "ActiveGame::get",
            allowed: &["application/games/catalogue.rs"],
        },
        files: &[(
            "application/pipeline/job.rs",
            "fn run() {\n    let state = ActiveGame::get(&app);\n}\n",
        )],
        expected: 1,
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callee: "ActiveGame::get",
            allowed: &["application/games/catalogue.rs"],
        },
        files: &[(
            "application/games/catalogue.rs",
            "fn run() {\n    let state = ActiveGame::get(&app);\n}\n",
        )],
        expected: 0,
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callee: "ActiveGame::get",
            allowed: &["application/games/"],
        },
        files: &[(
            "application/games/view.rs",
            "fn run() {\n    let state = ActiveGame::get(&app);\n}\n",
        )],
        expected: 0,
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callee: "ActiveGame::get",
            allowed: &[],
        },
        files: &[(
            "application/pipeline/job.rs",
            "fn run(active_game: &ActiveGame) {\n    let id = active_game.get();\n}\n",
        )],
        expected: 1,
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callee: "Narrator::narrate",
            allowed: &[],
        },
        files: &[(
            "application/pipeline/job.rs",
            "fn run(state: &State) {\n    state.narrator.narrate(&request);\n}\n",
        )],
        expected: 1,
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callee: "ActiveGame::get",
            allowed: &[],
        },
        files: &[(
            "application/pipeline/job.rs",
            "fn run(job: &Job) {\n    let id = job.get();\n}\n",
        )],
        expected: 0,
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callee: "hold",
            allowed: &[],
        },
        files: &[(
            "application/generation/gate.rs",
            "fn run() {\n    self.entries.hold(game);\n}\n",
        )],
        expected: 1,
    },
    WalkerCase {
        row: CallSiteRow {
            rule: FIXTURE_RULE,
            scope: "application/",
            callee: "hold",
            allowed: &[],
        },
        files: &[("adapters/driving/http/action.rs", "pub fn run() {}\n")],
        expected: 1,
    },
];

/// The type-check cases: a field, a parameter, a position the row does not cover, and a dead path.
pub const TYPE_CHECK_CASES: &[WalkerCase<TypeCheckRow>] = &[
    WalkerCase {
        row: TypeCheckRow {
            rule: FIXTURE_RULE,
            scope: "application/pipeline/",
            banned_types: &["GameState", "SwipeState"],
            position: TypeCheckPosition::Fields,
        },
        files: &[(
            "application/pipeline/job.rs",
            "pub struct Job {\n    state: SwipeState,\n}\n",
        )],
        expected: 1,
    },
    WalkerCase {
        row: TypeCheckRow {
            rule: FIXTURE_RULE,
            scope: "application/pipeline/",
            banned_types: &["GameState"],
            position: TypeCheckPosition::Parameters,
        },
        files: &[(
            "application/pipeline/job.rs",
            "pub fn run(state: &GameState) {\n    let _ = state;\n}\n",
        )],
        expected: 1,
    },
    WalkerCase {
        row: TypeCheckRow {
            rule: FIXTURE_RULE,
            scope: "application/pipeline/",
            banned_types: &["GameState"],
            position: TypeCheckPosition::Parameters,
        },
        files: &[(
            "application/pipeline/job.rs",
            "pub struct Job {\n    state: GameState,\n}\n",
        )],
        expected: 0,
    },
    WalkerCase {
        row: TypeCheckRow {
            rule: FIXTURE_RULE,
            scope: "application/pipeline/",
            banned_types: &["GameState"],
            position: TypeCheckPosition::Fields,
        },
        files: &[("application/games/catalogue.rs", "pub struct Job {}\n")],
        expected: 1,
    },
];

/// The call-count cases: two calls, one call, a nested closure, and a dead path.
pub const CALL_COUNT_CASES: &[WalkerCase<CallCountRow>] = &[
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
        expected: 1,
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
        expected: 0,
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
        expected: 1,
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
        expected: 1,
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
