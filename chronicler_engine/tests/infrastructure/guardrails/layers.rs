//! Layer-boundary guardrail tests: server vs. application vs. storage separation, handler return-type enforcement, and tests-vs-messages/swipes separation.

use crate::Violation;

const APPLICATION_STORAGE_GRANDFATHERED: &[&str] = &[
    "application/context.rs",
    "application/game_service.rs",
    "application/application_service.rs",
    "application/agents/registry.rs",
    "application/agents/quantifier/agent.rs",
];

/// Guardrail: `application/` files must not import `Storage` directly except for the
/// 5 grandfathered persistence-boundary files (see ADR-027). Test files (`*_tests.rs`)
/// are exempt — they may construct `Storage::new_in_memory()` for fixtures.
pub fn check_application_storage_direct(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if !file_path.starts_with("application/") {
        return violations;
    }
    if file_path.ends_with("_tests.rs") {
        return violations;
    }
    if APPLICATION_STORAGE_GRANDFATHERED.contains(&file_path) {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        if trimmed.starts_with("use ") && trimmed.contains("adapters::driven::storage::Storage") {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Application layer imports `Storage` directly — see ADR-027 for the 5 grandfathered files",
            ));
        }
    }

    violations
}

/// Guardrail: `messages.rs` must not reference the `message_swipes` table.
pub fn check_messages_swipes_separation(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if !file_path.ends_with("storage/backend/messages.rs") {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }
        if trimmed.contains("FROM message_swipes")
            || trimmed.contains("INTO message_swipes")
            || trimmed.contains("UPDATE message_swipes")
            || trimmed.contains("JOIN message_swipes")
            || trimmed.contains("DELETE FROM message_swipes")
        {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Storage module `messages.rs` references `message_swipes` table",
            ));
        }
    }

    violations
}

pub fn check_handler_return_type(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    let normalized_path = file_path.replace('\\', "/");

    if !normalized_path.starts_with("src/server/")
        || normalized_path.ends_with("mod.rs")
        || normalized_path.ends_with("debug.rs")
        || normalized_path.ends_with("renderers.rs")
        || normalized_path.ends_with("fragment_renderers.rs")
        || normalized_path.ends_with("response.rs")
    {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        if trimmed.contains(") -> (StatusCode, String)") {
            violations.push(Violation::error(
                file_path,
                line_num,
                "handler returns `(StatusCode, String)` — use `Response<Body>` with `app_err_to_response()` instead",
            ));
        }
    }

    violations
}

pub fn check_server_layer_boundaries(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if file_path.ends_with("mod.rs") || file_path.ends_with("debug.rs") {
        return violations;
    }

    if !file_path.starts_with("server/") {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        if trimmed.contains("GameState") && !trimmed.contains("GameStateSnapshot") {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Server layer file references `GameState`",
            ));
        }
    }

    violations
}

pub fn check_test_layer_boundaries(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if !file_path.starts_with("tests/components/") {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        if trimmed.contains("GameState::new(") {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Component test constructs `GameState` directly",
            ));
        }

        if (trimmed.contains("use") || trimmed.contains("model::state::GameState"))
            && trimmed.contains("GameState")
            && !trimmed.contains("GameStateSnapshot")
        {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Component test imports `GameState`",
            ));
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_handler_return_type_catches_violation() {
        let violations = check_handler_return_type(
            "src/server/test_handler.rs",
            "pub async fn bad_handler() -> (StatusCode, String) { }",
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Response<Body>"));
    }

    #[test]
    fn test_check_handler_return_type_allows_correct() {
        let violations = check_handler_return_type(
            "src/server/test_handler.rs",
            "pub async fn good_handler() -> Response<Body> { }",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_handler_return_type_skips_comments() {
        let violations = check_handler_return_type(
            "server/test_handler.rs",
            "// pub async fn bad_handler() -> (StatusCode, String) { }",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_handler_return_type_skips_debug_rs() {
        let violations = check_handler_return_type(
            "server/debug.rs",
            "pub async fn bad_handler() -> (StatusCode, String) { }",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_handler_return_type_skips_mod_rs() {
        let violations = check_handler_return_type(
            "server/mod.rs",
            "pub async fn bad_handler() -> (StatusCode, String) { }",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_application_storage_direct_catches_violation() {
        let violations = check_application_storage_direct(
            "application/query_handlers.rs",
            "use crate::adapters::driven::storage::Storage;\n",
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("ADR-027"));
    }

    #[test]
    fn test_check_application_storage_direct_allows_grandfathered() {
        let violations = check_application_storage_direct(
            "application/context.rs",
            "use crate::adapters::driven::storage::Storage;\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_application_storage_direct_skips_tests_files() {
        let violations = check_application_storage_direct(
            "application/query_handlers_tests.rs",
            "use crate::adapters::driven::storage::Storage;\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_application_storage_direct_skips_non_application() {
        let violations = check_application_storage_direct(
            "adapters/driven/storage/backend/core.rs",
            "use crate::adapters::driven::storage::Storage;\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_application_storage_direct_skips_comments() {
        let violations = check_application_storage_direct(
            "application/query_handlers.rs",
            "// use crate::adapters::driven::storage::Storage;\n",
        );
        assert_eq!(violations.len(), 0);
    }
}
