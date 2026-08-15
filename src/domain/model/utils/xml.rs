//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! XML string formatting utilities.

pub(crate) fn wrap_xml(content: &str, tag: &str) -> String {
    let indented = content
        .lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("    {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("<{tag}>\n{indented}\n</{tag}>")
}
