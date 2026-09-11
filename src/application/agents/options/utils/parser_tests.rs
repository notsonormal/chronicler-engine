//! Tests for options output parsing (both seeded prompt shapes).

use crate::application::agents::options::utils::parser::parse_options;

#[test]
fn parses_suggestion_tags() {
    let response = "<suggestion>Search the desk</suggestion>\n\
                    <suggestion>Question the guard</suggestion>\n\
                    <suggestion>Leave through the gate</suggestion>";
    assert_eq!(
        parse_options(response, 3),
        vec![
            "Search the desk".to_string(),
            "Question the guard".to_string(),
            "Leave through the gate".to_string(),
        ]
    );
}

#[test]
fn parses_numbered_list() {
    let response = "1. Attempt to communicate with the forest creatures.\n\
                    2. Bribe the corrupt city guard.\n\
                    3. Stage a fake ambush.";
    assert_eq!(
        parse_options(response, 3),
        vec![
            "Attempt to communicate with the forest creatures.".to_string(),
            "Bribe the corrupt city guard.".to_string(),
            "Stage a fake ambush.".to_string(),
        ]
    );
}

#[test]
fn parses_numbered_list_with_parenthesis_delimiter() {
    let response = "1) Look behind the tapestry\n2) Ask about the heir";
    assert_eq!(
        parse_options(response, 2),
        vec![
            "Look behind the tapestry".to_string(),
            "Ask about the heir".to_string(),
        ]
    );
}

#[test]
fn parses_suggestion_prefix_fallback() {
    let response = "Suggestion 1: Check the cellar\nSuggestion 2: Follow the cat";
    assert_eq!(
        parse_options(response, 2),
        vec!["Check the cellar".to_string(), "Follow the cat".to_string(),]
    );
}

#[test]
fn parses_suggestion_tags_case_insensitive_and_multiline() {
    let response =
        "<SUGGESTION>First\ncontinued line</SUGGESTION>\n<Suggestion>Second</Suggestion>";
    assert_eq!(
        parse_options(response, 2),
        vec!["First\ncontinued line".to_string(), "Second".to_string()]
    );
}

#[test]
fn first_matching_strategy_wins_no_shape_mixing() {
    // Tags present: numbered lines inside or outside the tags are ignored.
    let response = "<suggestion>Tag option</suggestion>\n1. Numbered option";
    assert_eq!(parse_options(response, 3), vec!["Tag option".to_string()]);
}

#[test]
fn trims_to_requested_count() {
    let response = "<suggestion>One</suggestion><suggestion>Two</suggestion>\
                    <suggestion>Three</suggestion><suggestion>Four</suggestion>";
    assert_eq!(parse_options(response, 3).len(), 3);
    assert_eq!(
        parse_options(response, 3),
        vec!["One".to_string(), "Two".to_string(), "Three".to_string()]
    );
}

#[test]
fn accepts_fewer_than_requested() {
    let response = "1. Only one option";
    assert_eq!(
        parse_options(response, 3),
        vec!["Only one option".to_string()]
    );
}

#[test]
fn unparseable_response_yields_empty() {
    assert!(parse_options("The model rambled about the weather.", 3).is_empty());
    assert!(parse_options("", 3).is_empty());
}

#[test]
fn empty_tag_bodies_are_skipped() {
    let response = "<suggestion></suggestion><suggestion>  </suggestion>\
                    <suggestion>Real option</suggestion>";
    assert_eq!(parse_options(response, 3), vec!["Real option".to_string()]);
}

#[test]
fn zero_count_still_returns_one_option() {
    // Defensive: a zero count would silently drop every option.
    let response = "<suggestion>Only</suggestion>";
    assert_eq!(parse_options(response, 0), vec!["Only".to_string()]);
}
