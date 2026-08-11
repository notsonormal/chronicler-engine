//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings template rendering helpers (provider options HTML).

fn provider_option_html(value: &str, label: &str, selected: bool) -> String {
    let sel = if selected { " selected" } else { "" };
    format!(r#"<option value="{value}"{sel}>{label}</option>"#)
}

pub(crate) fn provider_options_html(selected: &str) -> String {
    [
        ("openrouter", "OpenRouter"),
        ("deepseek", "DeepSeek"),
        ("ollama", "Ollama"),
    ]
    .iter()
    .map(|(v, l)| provider_option_html(v, l, *v == selected))
    .collect::<Vec<_>>()
    .join("\n")
}
