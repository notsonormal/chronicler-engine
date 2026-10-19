//! Layer-boundary guardrail tests: server vs. application vs. storage separation, handler return-type enforcement, urlencoded form safety, and the HTTP adapter's storage leak.

use crate::Violation;

const WIREDAPP_SCOPE_ALLOWLIST_PREFIXES: &[&str] =
    &["bootstrap/", "adapters/driving/http/", "test_support/"];

/// Restricts `WiredApp` imports to composition-root, HTTP, test-support, and test scopes.
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

/// The scope of the two path-limited guards below, relative to `src/`. Their tests
/// assert it matches a file, so a stale scope cannot pass by checking nothing.
pub const HTTP_ADAPTER_SCOPE: &str = "adapters/driving/http/";

/// The whole-state types the HTTP layer must not name: `SwipeState` is the design's
/// state.
const WHOLE_STATE_TYPES: [&str; 1] = ["SwipeState"];

/// Requires server handlers to return `Response<Body>` instead of `(StatusCode, String)`.
pub fn check_handler_return_type(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    let normalized_path = file_path.replace('\\', "/");

    // Paths are relative to `src/`, so the HTTP adapter's own folder is the scope.
    if !normalized_path.starts_with(HTTP_ADAPTER_SCOPE)
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

/// Flags collection-typed fields on HTTP form structs.
///
/// axum's `Form` (serde_urlencoded) cannot decode `Vec`/`HashMap` from
/// urlencoded bodies — a single checked checkbox posts one scalar value and
/// the extractor rejects the request with 422. Use per-value boolean fields
/// with `#[serde(default)]`.
pub fn check_form_fields_urlencoded_safe(file_path: &str, content: &str) -> Vec<Violation> {
    const UNSAFE_COLLECTIONS: [&str; 6] = [
        "Vec<",
        "HashMap<",
        "BTreeMap<",
        "HashSet<",
        "LinkedList<",
        "VecDeque<",
    ];

    let mut violations = Vec::new();

    let normalized_path = file_path.replace('\\', "/");
    if !normalized_path.starts_with("adapters/driving/http/")
        || !normalized_path.contains("/handlers/")
        || normalized_path.ends_with("_tests.rs")
    {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }
        if !trimmed.starts_with("pub ") {
            continue;
        }

        // A `pub fn` parameter matches the `pub <name>: <type>` shape but is
        // not a deserialized form field.
        if trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub async fn ")
            || trimmed.starts_with("pub const fn ")
            || trimmed.starts_with("pub unsafe fn ")
        {
            continue;
        }

        let Some(colon) = trimmed.find(':') else {
            continue;
        };
        let field_type = trimmed[colon + 1..].trim_start();

        if UNSAFE_COLLECTIONS
            .iter()
            .any(|collection| field_type.starts_with(collection))
        {
            violations.push(Violation::error(
                file_path,
                line_num,
                "form field is a collection: serde_urlencoded cannot decode \
                 `Vec`/`HashMap` from urlencoded bodies (a single checked checkbox \
                 posts one scalar value, so the `Form` extractor rejects with 422) — \
                 use one boolean field per value with `#[serde(default)]`; if this \
                 struct is not a `Form` payload, move it out of the handlers module",
            ));
        }
    }

    violations
}

/// Prevents HTTP-layer files from referencing the whole-state types directly.
pub fn check_server_layer_boundaries(file_path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if file_path.ends_with("mod.rs") || file_path.ends_with("debug.rs") {
        return violations;
    }

    if !file_path.starts_with(HTTP_ADAPTER_SCOPE) {
        return violations;
    }

    for (line_no, line) in content.lines().enumerate() {
        let line_num = line_no + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("*") {
            continue;
        }

        for state_type in WHOLE_STATE_TYPES {
            if trimmed.contains(state_type) {
                violations.push(Violation::error(
                    file_path,
                    line_num,
                    format!(
                        "HTTP layer file references `{state_type}` — read the state through \
                         `GameViewQuery`"
                    ),
                ));
            }
        }
    }

    violations
}

/// Prevents HTTP layer files from directly referencing the driven `Storage` namespace.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_handler_return_type_catches_violation() {
        let violations = check_handler_return_type(
            "adapters/driving/http/test_handler.rs",
            "pub async fn bad_handler() -> (StatusCode, String) { }",
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Response<Body>"));
    }

    #[test]
    fn test_check_handler_return_type_allows_correct() {
        let violations = check_handler_return_type(
            "adapters/driving/http/test_handler.rs",
            "pub async fn good_handler() -> Response<Body> { }",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_handler_return_type_skips_comments() {
        let violations = check_handler_return_type(
            "adapters/driving/http/test_handler.rs",
            "// pub async fn bad_handler() -> (StatusCode, String) { }",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_handler_return_type_skips_debug_rs() {
        let violations = check_handler_return_type(
            "adapters/driving/http/debug.rs",
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
            "app.settings_service.update_settings(|_| Ok(())).unwrap();\n",
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

    #[test]
    fn test_check_form_fields_urlencoded_safe_catches_violation() {
        let violations = check_form_fields_urlencoded_safe(
            "adapters/driving/http/prompt_presets/handlers/prompt_presets.rs",
            "pub allowed_modes: Vec<String>,\n",
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("boolean field"));
    }

    #[test]
    fn test_check_form_fields_urlencoded_safe_catches_hashmap() {
        let violations = check_form_fields_urlencoded_safe(
            "adapters/driving/http/games/handlers/games.rs",
            "pub flags: HashMap<String, bool>,\n",
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("serde_urlencoded"));
    }

    #[test]
    fn test_check_form_fields_urlencoded_safe_allows_correct() {
        let violations = check_form_fields_urlencoded_safe(
            "adapters/driving/http/prompt_presets/handlers/prompt_presets.rs",
            "pub allowed_mode_novel: bool,\npub name: String,\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_form_fields_urlencoded_safe_skips_tests() {
        let violations = check_form_fields_urlencoded_safe(
            "adapters/driving/http/prompt_presets/handlers/prompt_presets_tests.rs",
            "pub allowed_modes: Vec<String>,\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_form_fields_urlencoded_safe_skips_non_handler_path() {
        let violations = check_form_fields_urlencoded_safe(
            "adapters/driving/http/prompt_presets/templates/prompt_presets.rs",
            "pub allowed_modes: Vec<String>,\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_form_fields_urlencoded_safe_skips_comments() {
        let violations = check_form_fields_urlencoded_safe(
            "adapters/driving/http/prompt_presets/handlers/prompt_presets.rs",
            "// pub allowed_modes: Vec<String>,\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_form_fields_urlencoded_safe_skips_non_pub() {
        let violations = check_form_fields_urlencoded_safe(
            "adapters/driving/http/prompt_presets/handlers/prompt_presets.rs",
            "    allowed_modes: Vec<String>,\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_form_fields_urlencoded_safe_allows_pub_fn_with_vec_param() {
        let violations = check_form_fields_urlencoded_safe(
            "adapters/driving/http/games/handlers/games.rs",
            "pub async fn handler(items: Vec<String>) {}\n",
        );
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_check_server_layer_boundaries_catches_swipe_state() {
        let violations = check_server_layer_boundaries(
            "adapters/driving/http/renderers.rs",
            "fn render(state: &SwipeState) {}\n",
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("SwipeState"));
    }

    #[test]
    fn test_check_server_layer_boundaries_allows_a_plain_field() {
        let violations = check_server_layer_boundaries(
            "adapters/driving/http/renderers.rs",
            "fn render(room: &str) {}\n",
        );
        assert_eq!(violations.len(), 0);
    }
}
