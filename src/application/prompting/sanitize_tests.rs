use crate::application::prompting::sanitize::sanitize_llm_output;

#[test]
fn test_sanitize_leading_channel_close() {
    let input = "<channel|>The heavy iron gates...";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "The heavy iron gates...");
}

#[test]
fn test_sanitize_thought_block() {
    let input = "<thought>The user wants to continue...</thought>The gates creaked.";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "The gates creaked.");
}

#[test]
fn test_sanitize_channel_thought_block() {
    let input = "<|channel>thought\nSome reasoning here\n<channel|>Narrative text.";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "Narrative text.");
}

#[test]
fn test_sanitize_orphan_turn_markers() {
    let input = "<|turn>modelStart of text<turn|>more text<|turn>end.";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "Start of textmore textend.");
}

#[test]
fn test_sanitize_combined_artifacts() {
    let input = "<channel|><thought>reasoning</thought><|turn>modelThe real content.";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "The real content.");
}

#[test]
fn test_sanitize_clean_text_unchanged() {
    let input = "The heavy iron gates offered no resistance.";
    let result = sanitize_llm_output(input);
    assert_eq!(result, input);
}

#[test]
fn test_sanitize_empty_string() {
    assert_eq!(sanitize_llm_output(""), "");
}

#[test]
fn test_sanitize_whitespace_only() {
    assert_eq!(sanitize_llm_output("   "), "");
}

#[test]
fn test_sanitize_multiple_thought_blocks() {
    let input = "<thought>first</thought>A<thought>second</thought>B";
    let result = sanitize_llm_output(input);
    assert_eq!(result, "AB");
}

#[test]
fn test_sanitize_paragraph_indentation() {
    let input = "  First paragraph.\n\n        Second paragraph.\n\n        Third paragraph.";
    let result = sanitize_llm_output(input);
    assert_eq!(
        result,
        "First paragraph.\n\nSecond paragraph.\n\nThird paragraph."
    );
}
