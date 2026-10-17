//! The legacy unit-test list and its allowlist guardrail: a unit test lives in a seam contract, a `domain/model/` file, or a listed legacy file.

use crate::{SourceFile, Violation};

// An entry goes when the file's tests move under the one rule of
// `tests/STRATEGY.md`, "The one rule". The list holds files, not test counts.
pub const LEGACY_UNIT_TEST_FILES: &[&str] = &[
    "adapters/driven/llm/providers/deepseek_tests.rs",
    "adapters/driven/llm/providers/mock_tests.rs",
    "adapters/driven/llm/providers/ollama_tests.rs",
    "adapters/driven/llm/providers/openrouter_tests.rs",
    "adapters/driven/llm/transport/utils/request_tests.rs",
    "adapters/driven/llm/transport/utils/response_tests.rs",
    "adapters/driven/storage/characters_tests.rs",
    "adapters/driven/storage/core_tests.rs",
    "adapters/driven/storage/db_tests.rs",
    "adapters/driven/storage/games_tests.rs",
    "adapters/driven/storage/llm_messages_tests.rs",
    "adapters/driven/storage/mappers/llm_message_tests.rs",
    "adapters/driven/storage/mappers/message_tests.rs",
    "adapters/driven/storage/mappers/state_snapshot_tests.rs",
    "adapters/driven/storage/messages_tests.rs",
    "adapters/driven/storage/models/message_tests.rs",
    "adapters/driven/storage/personas_tests.rs",
    "adapters/driven/storage/presets_tests.rs",
    "adapters/driven/storage/settings_tests.rs",
    "adapters/driven/storage/snapshots_tests.rs",
    "adapters/driven/storage/swipes_tests.rs",
    "adapters/driven/storage/utils/plumbing_tests.rs",
    "adapters/driven/storage/worlds_tests.rs",
    "adapters/driven/text_check/harper_text_checker_tests.rs",
    "adapters/driven/utils/harper_tests.rs",
    "adapters/driving/http/action/handlers/actions_tests.rs",
    "adapters/driving/http/app_state_tests.rs",
    "adapters/driving/http/builders/presets_tests.rs",
    "adapters/driving/http/chat_window/handlers/chat_window_tests.rs",
    "adapters/driving/http/debug/handlers/debug_tests.rs",
    "adapters/driving/http/error_tests.rs",
    "adapters/driving/http/games/handlers/games_tests.rs",
    "adapters/driving/http/history/handlers/history_tests.rs",
    "adapters/driving/http/layout/handlers/endpoints_tests.rs",
    "adapters/driving/http/mod_tests.rs",
    "adapters/driving/http/prompt_presets/handlers/prompt_presets_tests.rs",
    "adapters/driving/http/prompt_presets/templates/prompt_presets_tests.rs",
    "adapters/driving/http/settings/handlers/settings_tests.rs",
    "adapters/driving/http/settings/templates/settings_tests.rs",
    "adapters/driving/http/templates_tests.rs",
    "adapters/driving/http/utils/handler_helpers_tests.rs",
    "adapters/driving/http/utils/response_tests.rs",
    "adapters/driving/http/utils/template_helpers_tests.rs",
    "adapters/driving/http/utils/view_mappers_tests.rs",
    "adapters/driving/http/utils/view_models_tests.rs",
    "adapters/driving/http/view_models_tests.rs",
    "adapters/driving/http/worlds/handlers/worlds_tests.rs",
    "application/agents/options/prompt_tests.rs",
    "application/agents/options/utils/parser_tests.rs",
    "application/agents/quantifier/prompt_tests.rs",
    "application/agents/quantifier/utils/orchestration_tests.rs",
    "application/agents/quantifier/utils/parser_tests.rs",
    "application/agents/runner_tests.rs",
    "application/connection_test_service_tests.rs",
    "application/games/catalogue_tests.rs",
    "application/games/view_query_tests.rs",
    "application/generation/gate_tests.rs",
    "application/generation/guard_tests.rs",
    "application/generation/slot_tests.rs",
    "application/llm_recorder_tests.rs",
    "application/message_service_tests.rs",
    "application/persona_catalogue_tests.rs",
    "application/pipeline/action_pipeline/action_tests.rs",
    "application/pipeline/action_pipeline/core_tests.rs",
    "application/pipeline/action_pipeline/options_tests.rs",
    "application/pipeline/action_pipeline/retrigger_tests.rs",
    "application/pipeline/action_pipeline/retry_tests.rs",
    "application/pipeline/narration_generation_tests.rs",
    "application/ports/llm_provider_tests.rs",
    "application/ports/text_checker_tests.rs",
    "application/prompt_preset_service_tests.rs",
    "application/prompting/token_budget_tests.rs",
    "application/prompting/utils/context_tests.rs",
    "application/settings_service_tests.rs",
    "application/text_check_service_tests.rs",
    "application/world_catalogue_tests.rs",
    "bootstrap/load_tests.rs",
    "bootstrap/run_tests.rs",
    "bootstrap/wiring_tests.rs",
    "domain/model/action_tests.rs",
    "domain/model/agent_tests.rs",
    "domain/model/character_tests.rs",
    "domain/model/game_tests.rs",
    "domain/model/llm_backend_tests.rs",
    "domain/model/map_tests.rs",
    "domain/model/message_history_tests.rs",
    "domain/model/message_tests.rs",
    "domain/model/prompt_preset_tests.rs",
    "domain/model/scenario_tests.rs",
    "domain/model/settings_tests.rs",
    "domain/model/state/game_state_snapshot_tests.rs",
    "domain/model/state/game_state_tests.rs",
    "domain/model/state/generation_status_tests.rs",
    "domain/model/template_tests.rs",
    "domain/model/trigger_tests.rs",
    "domain/model/utils/xml_tests.rs",
    "domain/model/world_tests.rs",
    "error_tests.rs",
    "utils/cli_tests.rs",
    "utils/settings_tests.rs",
];

// The seam-contract files. A box adds its contract file here; the checker below
// fails a file that declares cases without being listed.
pub const SEAM_CONTRACT_FILES: &[&str] = &[
    // Each Agent is a box behind the LLM port, so its test file declares its cases.
    "application/agents/options/agent_tests.rs",
    "application/agents/quantifier/agent_tests.rs",
    "application/narration/narrator_tests.rs",
];

const TEST_MARKERS: &[&str] = &["#[test]", "#[tokio::test", "#[rstest]", "proptest!"];

const PURE_DOMAIN_PREFIX: &str = "domain/model/";

fn first_test_line(content: &str) -> Option<usize> {
    content
        .lines()
        .position(|line| {
            TEST_MARKERS
                .iter()
                .any(|marker| line.trim_start().starts_with(marker))
        })
        .map(|index| index + 1)
}

fn is_case_id(id: &str) -> bool {
    !id.is_empty()
        && id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

/// The form `scripts/validate_seam_cases.py` parses: `//! CASE <id> [<label>]: <description>`,
/// the label optional.
pub fn is_case_declaration(line: &str) -> bool {
    let Some(rest) = line.trim_start().strip_prefix("//!") else {
        return false;
    };
    let Some(rest) = rest.trim_start().strip_prefix("CASE") else {
        return false;
    };
    let rest = rest.trim_start();
    let id_end = rest
        .find(|character: char| {
            !(character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-')
        })
        .unwrap_or(rest.len());
    let (id, rest) = rest.split_at(id_end);
    if !is_case_id(id) {
        return false;
    }
    let rest = rest.trim_start();
    let rest = match rest.strip_prefix('[') {
        Some(after_open) => match after_open.split_once(']') {
            Some((_, after_close)) => after_close.trim_start(),
            None => return false,
        },
        None => rest,
    };
    match rest.strip_prefix(':') {
        Some(description) => !description.trim().is_empty(),
        None => false,
    }
}

fn declares_cases(content: &str) -> bool {
    content.lines().any(is_case_declaration)
}

/// Fails a unit test outside the one rule, and a list entry that names no file.
///
/// The one rule is `tests/STRATEGY.md` "The one rule": a unit test lives in a
/// listed seam contract, in a `domain/model/` file, or on the legacy list.
pub fn check_unit_test_allowlist(
    files: &[SourceFile],
    legacy: &[&str],
    seam_contracts: &[&str],
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for listed in legacy {
        if !files.iter().any(|file| file.path == *listed) {
            violations.push(Violation::error(
                listed,
                1,
                "this legacy unit-test entry names no file: drop the entry, so the \
                 list keeps saying which files still live outside the one rule",
            ));
        }
    }

    for listed in seam_contracts {
        if !files.iter().any(|file| file.path == *listed) {
            violations.push(Violation::error(
                listed,
                1,
                "this seam-contract entry names no file: drop the entry, so the table \
                 keeps saying which boxes have a contract",
            ));
        }
    }

    for file in files {
        if !file.path.ends_with("_tests.rs") {
            continue;
        }
        let Some(line) = first_test_line(&file.content) else {
            continue;
        };
        if declares_cases(&file.content) && !seam_contracts.contains(&file.path.as_str()) {
            violations.push(Violation::error(
                &file.path,
                line,
                "this file declares `//! CASE` clauses but is not on \
                 `SEAM_CONTRACT_FILES`: add the file to the table, or move the \
                 clauses out of the file",
            ));
            continue;
        }
        if legacy.contains(&file.path.as_str())
            || seam_contracts.contains(&file.path.as_str())
            || file.path.starts_with(PURE_DOMAIN_PREFIX)
        {
            continue;
        }
        violations.push(Violation::error(
            &file.path,
            line,
            format!(
                "unit test outside the one rule: a unit test lives in a seam contract \
                 (a listed `_tests.rs` file that declares `//! CASE` clauses), in a \
                 `{PURE_DOMAIN_PREFIX}` file, or on `LEGACY_UNIT_TEST_FILES` — move the \
                 player-visible behaviour to a `docs/specs/` scenario, or put the clause \
                 in its box's seam contract"
            ),
        ));
    }

    violations
}
