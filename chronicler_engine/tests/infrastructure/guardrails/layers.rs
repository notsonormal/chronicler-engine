//! Layer-boundary guardrail tests: server vs. application vs. storage separation, handler return-type enforcement, and tests-vs-messages/swipes separation.

use crate::Violation;

// TODO: We shouldn't be blank allowing the `adapters/driving/http/`
//  just so that we can use it the create the AppState. We
//  can just move app_state.rs into `chronicler_engine/src/adapters/driving/http/bootstrap`
//  instead, since that counts counts as bootstrap folder too
const WIREDAPP_SCOPE_ALLOWLIST_PREFIXES: &[&str] =
    &["bootstrap/", "adapters/driving/http/", "test_support/"];

pub fn check_wiredapp_scope(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if file_path.starts_with("tests/") {
        return violations;
    }
    if file_path.ends_with("_tests.rs") {
        return violations;
    }
    if WIREDAPP_SCOPE_ALLOWLIST_PREFIXES
        .iter()
        .any(|prefix| file_path.starts_with(prefix))
    {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        if (trimmed.starts_with("use ") || trimmed.starts_with("pub use "))
            && trimmed.contains("WiredApp")
        {
            violations.push(Violation::error(
                file_path,
                line_num,
                "Composition-root `WiredApp` is consumed only by `bootstrap/`, \
                 `adapters/driving/http/`, `test_support/`, and `tests/` — \
                 take collaborators as `Arc<...>` parameters instead",
            ));
        }
    }

    violations
}

// Guardrail: `messages.rs` must not reference the `message_swipes` table.
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

    // TODO: Are these exceptions really valid? Or this a new with
    //  more exceptions to the rule then actual rules? Or can we
    //  get all of these to follow the rule?
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

// Guardrail: HTTP layer files must not directly reference the driven `Storage` namespace.
// This catches fully-qualified paths such as `crate::adapters::driven::storage::Storage`
// that arch-lint's import-based deny cannot see.
pub fn check_http_storage_leak(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if !file_path.starts_with("adapters/driving/http/") {
        return violations;
    }
    if file_path.ends_with("_tests.rs") || file_path.ends_with("mod.rs") {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        if trimmed.contains("adapters::driven::storage::Storage")
            || trimmed.contains("driven::storage::Storage")
            || trimmed.contains("storage::Storage")
        {
            violations.push(Violation::error(
                file_path,
                line_num,
                "HTTP layer file references driven `Storage` directly — use an application-layer service instead",
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
    fn test_check_wiredapp_scope_catches_violation() {
        let violations = check_wiredapp_scope(
            "application/pipeline/mod.rs",
            "use crate::bootstrap::wiring::WiredApp;\n",
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("WiredApp"));
    }

    #[test]
    fn test_check_wiredapp_scope_catches_pub_use() {
        let violations = check_wiredapp_scope(
            "application/mod.rs",
            "pub use crate::bootstrap::wiring::WiredApp;\n",
        );
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_check_wiredapp_scope_allows_scoped_consumers() {
        for path in [
            "bootstrap/wiring.rs",
            "bootstrap/run.rs",
            "adapters/driving/http/app_state.rs",
            "adapters/driving/http/bootstrap/server.rs",
            "test_support/context.rs",
        ] {
            let violations =
                check_wiredapp_scope(path, "use crate::bootstrap::wiring::WiredApp;\n");
            assert_eq!(violations.len(), 0, "expected {path} to be allowed");
        }
    }

    #[test]
    fn test_check_wiredapp_scope_skips_tests() {
        let violations = check_wiredapp_scope(
            "tests/http/server_impl_wiring.rs",
            "use chronicler::bootstrap::wiring::WiredApp;\n",
        );
        assert_eq!(violations.len(), 0);
        let violations = check_wiredapp_scope(
            "bootstrap/wiring_tests.rs",
            "use crate::bootstrap::wiring::WiredApp;\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_http_storage_leak_catches_fully_qualified_path() {
        let violations = check_http_storage_leak(
            "adapters/driving/http/settings.rs",
            "let s: crate::adapters::driven::storage::Storage = app.storage.clone();\n",
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Storage"));
    }

    #[test]
    fn test_check_http_storage_leak_catches_relative_path() {
        let violations = check_http_storage_leak(
            "adapters/driving/http/prompt_presets.rs",
            "fn f(s: storage::Storage) {}\n",
        );
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_check_http_storage_leak_allows_application_service() {
        let violations = check_http_storage_leak(
            "adapters/driving/http/settings.rs",
            "app.settings_service.save_settings(&settings).unwrap();\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_http_storage_leak_skips_tests() {
        let violations = check_http_storage_leak(
            "adapters/driving/http/app_state_tests.rs",
            "let s: crate::adapters::driven::storage::Storage;\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_http_storage_leak_skips_other_layers() {
        let violations = check_http_storage_leak(
            "bootstrap/wiring.rs",
            "let s: crate::adapters::driven::storage::Storage;\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_http_storage_leak_skips_comments() {
        let violations = check_http_storage_leak(
            "adapters/driving/http/settings.rs",
            "// crate::adapters::driven::storage::Storage\n",
        );
        assert_eq!(violations.len(), 0);
    }
}
