use crate::model::message::Message;
use crate::model::message_history::MessageHistory;
use crate::model::state::LogType;

fn make_message(id: u64, text: &str, log_type: LogType) -> Message {
    Message::new(id, Some("Player".to_string()), text, log_type, None, None)
}

#[test]
fn test_new_is_empty() {
    let history = MessageHistory::new();
    assert!(history.is_empty());
    assert_eq!(history.len(), 0);
}

#[test]
fn test_from_messages() {
    let msgs = vec![make_message(1, "hi", LogType::Input)];
    let history = MessageHistory::from_messages(msgs);
    assert_eq!(history.len(), 1);
}

#[test]
fn test_append_adds_message() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "hello", LogType::Input));
    assert_eq!(history.len(), 1);
    assert_eq!(history.last().unwrap().text, "hello");
}

#[test]
fn test_append_caps_capacity() {
    let mut history = MessageHistory::new();
    for i in 0..1005 {
        history.append(make_message(
            i as u64,
            &format!("msg{i}"),
            LogType::Narration,
        ));
    }
    assert_eq!(history.len(), 1000);
    // Oldest messages should have been evicted
    assert!(history.get(0).is_none());
    assert!(history.get(1004).is_some());
}

#[test]
fn test_edit_success() {
    let mut history = MessageHistory::new();
    history.append(make_message(42, "old", LogType::Narration));
    history.edit(42, "new".to_string()).unwrap();
    assert_eq!(history.get(42).unwrap().text, "new");
}

#[test]
fn test_edit_failure() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "hi", LogType::Input));
    assert!(history.edit(999, "new".to_string()).is_err());
}

#[test]
fn test_delete_last_success() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "a", LogType::Narration));
    history.append(make_message(2, "b", LogType::Narration));
    history.delete_last().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history.last().unwrap().text, "a");
}

#[test]
fn test_delete_last_empty_fails() {
    let mut history = MessageHistory::new();
    assert!(history.delete_last().is_err());
}

#[test]
fn test_get_and_last() {
    let mut history = MessageHistory::new();
    history.append(make_message(7, "seven", LogType::Input));
    assert_eq!(history.get(7).unwrap().text, "seven");
    assert_eq!(history.last().unwrap().text, "seven");
    assert!(history.get(99).is_none());
}

#[test]
fn test_last_mut() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "x", LogType::Narration));
    history.last_mut().unwrap().text = "y".to_string();
    assert_eq!(history.last().unwrap().text, "y");
}

#[test]
fn test_iter_and_iter_mut() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "a", LogType::Narration));
    history.append(make_message(2, "b", LogType::Input));

    let texts: Vec<_> = history.iter().map(|m| m.text.clone()).collect();
    assert_eq!(texts, vec!["a", "b"]);

    for m in history.iter_mut() {
        m.text.push('!');
    }
    assert_eq!(history.last().unwrap().text, "b!");
}

#[test]
fn test_as_slice() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "a", LogType::Narration));
    assert_eq!(history.as_slice()[0].text, "a");
}

#[test]
fn test_replace() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "old", LogType::Narration));
    history.replace(vec![make_message(2, "new", LogType::Input)]);
    assert_eq!(history.len(), 1);
    assert_eq!(history.last().unwrap().text, "new");
}

#[test]
fn test_retain() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "a", LogType::Narration));
    history.append(make_message(2, "b", LogType::Input));
    history.retain(|m| m.log_type == LogType::Input);
    assert_eq!(history.len(), 1);
    assert_eq!(history.last().unwrap().id, 2);
}

#[test]
fn test_clear() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "a", LogType::Narration));
    history.clear();
    assert!(history.is_empty());
}

#[test]
fn test_last_ai_response_index() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "input", LogType::Input));
    history.append(make_message(2, "narration", LogType::Narration));
    history.append(make_message(3, "dialogue", LogType::Dialogue));
    assert_eq!(history.last_ai_response_index(), Some(2));
}

#[test]
fn test_last_input_index() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "narration", LogType::Narration));
    history.append(make_message(2, "input", LogType::Input));
    assert_eq!(history.last_input_index(), Some(1));
}

#[test]
fn test_last_input_text() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "narration", LogType::Narration));
    assert!(history.last_input_text().is_none());
    history.append(make_message(2, "go north", LogType::Input));
    assert_eq!(
        history.last_input_text(),
        Some(("Player".to_string(), "go north".to_string()))
    );
}

#[test]
fn test_is_last_ai_response_event_continuation() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "input", LogType::Input));
    history.append(make_message(2, "narration", LogType::Narration));
    assert!(!history.is_last_ai_response_event_continuation());

    let mut msg = make_message(3, "event", LogType::Narration);
    msg.event_header = Some("Event".to_string());
    history.append(msg);
    assert!(history.is_last_ai_response_event_continuation());
}

#[test]
fn test_to_log_entries() {
    let mut history = MessageHistory::new();
    history.append(make_message(1, "text", LogType::Narration));
    let entries = history.to_log_entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].text, "text");
    assert_eq!(entries[0].log_type, LogType::Narration);
}
