use chrono::Utc;

use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::prompt_part::PromptPart;

fn row(system_prompt: &str, user_prompt: &str, raw_request_json: &str) -> LlmMessage {
    LlmMessage {
        id: 0,
        agent_name: "narrator".to_string(),
        backend_name: "Mock".to_string(),
        model_name: "mock".to_string(),
        system_prompt: system_prompt.to_string(),
        user_prompt: user_prompt.to_string(),
        raw_request_json: raw_request_json.to_string(),
        raw_response_json: String::new(),
        parsed_response: String::new(),
        error_message: None,
        created_at: Utc::now(),
    }
}

#[test]
fn test_a_new_row_shows_the_messages_of_its_payload_in_order() {
    let message = row(
        "",
        "",
        r#"{"model":"m","messages":[
            {"role":"system","content":"rules"},
            {"role":"user","content":"blocks"},
            {"role":"assistant","content":"narration"},
            {"role":"user","content":"input"}]}"#,
    );

    assert_eq!(
        message.sent_parts(),
        vec![
            PromptPart::system("rules"),
            PromptPart::user("blocks"),
            PromptPart::assistant("narration"),
            PromptPart::user("input"),
        ]
    );
}

#[test]
fn test_an_old_row_falls_back_to_its_two_columns() {
    let message = row("system text", "user text", "");

    assert_eq!(
        message.sent_parts(),
        PromptPart::system_and_user("system text", "user text")
    );
}

#[test]
fn test_a_payload_with_no_messages_falls_back_to_the_columns() {
    let message = row("system text", "user text", r#"{"request":"marker"}"#);

    assert_eq!(
        message.sent_parts(),
        PromptPart::system_and_user("system text", "user text")
    );
}

#[test]
fn test_a_failure_before_any_send_shows_no_message() {
    assert!(row("", "", "").sent_parts().is_empty());
}

#[test]
fn test_a_message_of_an_unknown_role_is_left_out() {
    let message = row(
        "",
        "",
        r#"{"messages":[{"role":"tool","content":"x"},{"role":"user","content":"input"}]}"#,
    );

    assert_eq!(message.sent_parts(), vec![PromptPart::user("input")]);
}
