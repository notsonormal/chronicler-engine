//! The room headers of a story window.

use super::story_window::StoryWindow;

fn names(names: &[Option<&str>]) -> Vec<Option<String>> {
    names.iter().map(|name| name.map(str::to_string)).collect()
}

#[test]
fn room_marker_by_name_shows_a_changed_name_and_hides_a_repeat() {
    let headers = StoryWindow::room_headers(&names(&[
        Some("Hall"),
        Some("Hall"),
        Some("Cellar"),
        Some("Cellar"),
        Some("Hall"),
    ]));

    assert_eq!(
        headers,
        names(&[Some("Hall"), None, Some("Cellar"), None, Some("Hall")])
    );
}

#[test]
fn room_marker_by_name_always_shows_the_oldest_entry() {
    let headers = StoryWindow::room_headers(&names(&[Some("Hall"), Some("Hall")]));

    assert_eq!(headers[0].as_deref(), Some("Hall"));
}

#[test]
fn room_marker_by_name_shows_nothing_for_an_entry_with_no_name_and_resumes_after_it() {
    let headers = StoryWindow::room_headers(&names(&[Some("Hall"), None, Some("Hall")]));

    assert_eq!(headers, names(&[Some("Hall"), None, Some("Hall")]));
}

#[test]
fn room_marker_by_name_of_an_empty_window_is_empty() {
    assert!(StoryWindow::room_headers(&[]).is_empty());
}
