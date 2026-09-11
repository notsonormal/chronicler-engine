//! Worlds HTTP handler tests.

use axum::{body::to_bytes, extract::Path};

use crate::adapters::driving::http::worlds::handlers::{
    update_world_handler, update_world_posture_handler, WorldForm, WorldPostureForm,
};
use crate::adapters::driving::http::AppState;
use crate::domain::model::map::MapDef;
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use crate::domain::model::world::WorldCard;
use crate::test_support::TestAppBuilder;

async fn body_string(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn seed_world(state: &AppState, key: &str) {
    let world = WorldCard {
        key: key.to_string(),
        name: "Posture World".to_string(),
        ..Default::default()
    };
    state
        .world_catalogue
        .create_world(world, MapDef::default())
        .expect("world seed should succeed");
}

async fn post_posture(state: &AppState, key: &str, form: WorldPostureForm) -> String {
    let response = update_world_posture_handler(
        axum::extract::State(state.clone()),
        Path(key.to_string()),
        axum::extract::Form(form),
    )
    .await;
    body_string(response).await
}

#[tokio::test]
async fn test_world_posture_invalid_value_renders_error_and_mutates_nothing() {
    let state = TestAppBuilder::default_test().build_service();
    seed_world(&state, "posture_world");

    // Move the world off its defaults so "unchanged" is observable.
    let saved = post_posture(
        &state,
        "posture_world",
        WorldPostureForm {
            narrator_mode: Some("interactive_fiction".to_string()),
            narrative_perspective: Some("second".to_string()),
            narrative_tense: Some("past".to_string()),
        },
    )
    .await;
    assert!(
        saved.contains(r#"posture-status">Saved</span>"#),
        "valid save should report Saved: {saved}"
    );

    let body = post_posture(
        &state,
        "posture_world",
        WorldPostureForm {
            narrator_mode: Some("warp_drive".to_string()),
            narrative_perspective: None,
            narrative_tense: None,
        },
    )
    .await;
    assert!(
        body.contains(r#"class="error-message""#),
        "invalid value should render the error span: {body}"
    );
    assert!(
        body.contains("Unknown narrator mode"),
        "error should name the parse failure: {body}"
    );

    let (_, world, _) = state
        .world_catalogue
        .get_world("posture_world")
        .unwrap()
        .expect("world should exist");
    assert_eq!(world.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(world.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(world.narrative_tense, NarrativeTense::Past);
}

#[tokio::test]
async fn test_world_posture_omitted_field_keeps_stored_value() {
    let state = TestAppBuilder::default_test().build_service();
    seed_world(&state, "posture_world");

    let saved = post_posture(
        &state,
        "posture_world",
        WorldPostureForm {
            narrator_mode: Some("interactive_fiction".to_string()),
            narrative_perspective: Some("second".to_string()),
            narrative_tense: Some("past".to_string()),
        },
    )
    .await;
    assert!(
        saved.contains(r#"posture-status">Saved</span>"#),
        "valid save should report Saved: {saved}"
    );

    // Only the tense is posted; mode and perspective must keep their
    // stored values (the posture endpoint patches per-field).
    let saved = post_posture(
        &state,
        "posture_world",
        WorldPostureForm {
            narrator_mode: None,
            narrative_perspective: None,
            narrative_tense: Some("present".to_string()),
        },
    )
    .await;
    assert!(
        saved.contains(r#"posture-status">Saved</span>"#),
        "partial save should report Saved: {saved}"
    );

    let (_, world, _) = state
        .world_catalogue
        .get_world("posture_world")
        .unwrap()
        .expect("world should exist");
    assert_eq!(
        world.narrator_mode,
        NarratorMode::InteractiveFiction,
        "omitted mode must keep the stored value"
    );
    assert_eq!(
        world.narrative_perspective,
        NarrativePerspective::Second,
        "omitted perspective must keep the stored value"
    );
    assert_eq!(world.narrative_tense, NarrativeTense::Present);
}

#[test]
fn test_world_form_options_always_on_checkbox_grammar() {
    // Checked checkbox: the body carries an explicit value="true".
    let checked = "key=w&name=N&description=D&global_rules=&map_json={}&scenarios_json=[]&options_always_on=true";
    let form: WorldForm = serde_urlencoded::from_str(checked).unwrap();
    assert!(form.options_always_on);

    // Unchecked checkbox posts nothing for the field; serde default fills
    // false (same grammar as the PresetForm allowed-mode flags).
    let unchecked = "key=w&name=N&description=D&global_rules=&map_json={}&scenarios_json=[]";
    let form: WorldForm = serde_urlencoded::from_str(unchecked).unwrap();
    assert!(!form.options_always_on);
}

#[tokio::test]
async fn test_update_world_handler_flips_options_always_on_from_form() {
    let state = TestAppBuilder::default_test().build_service();
    seed_world(&state, "toggle_world");

    let map_json = serde_json::to_string(&MapDef::default()).unwrap();
    let post = |always_on: Option<&str>| {
        let body = match always_on {
            Some(v) => format!(
                "key=toggle_world&name=Posture World&description=D&global_rules=&default_room_image=&map_json={map_json}&scenarios_json=[]&options_always_on={v}"
            ),
            None => format!(
                "key=toggle_world&name=Posture World&description=D&global_rules=&default_room_image=&map_json={map_json}&scenarios_json=[]"
            ),
        };
        let form: WorldForm = serde_urlencoded::from_str(&body).unwrap();
        form
    };

    // Absent checkbox (unchecked save) resets to false — the absent→reset
    // contract shared with the posture fields.
    update_world_handler(
        axum::extract::State(state.clone()),
        axum::extract::Path("toggle_world".to_string()),
        axum::extract::Form(post(None)),
    )
    .await;
    let (_, world, _) = state
        .world_catalogue
        .get_world("toggle_world")
        .unwrap()
        .expect("world should exist");
    assert!(!world.options_always_on);

    // Checked save flips it on.
    update_world_handler(
        axum::extract::State(state.clone()),
        axum::extract::Path("toggle_world".to_string()),
        axum::extract::Form(post(Some("true"))),
    )
    .await;
    let (_, world, _) = state
        .world_catalogue
        .get_world("toggle_world")
        .unwrap()
        .expect("world should exist");
    assert!(world.options_always_on);
}
