//! HTTP E2E tests for the worlds update endpoint: the posture merge contract and the options-toggle checkbox grammar.

use std::sync::Arc;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{TestAppBuilder, TestMap};

use crate::test_helpers::{post_form, world_form_body};

fn posture_world() -> WorldCard {
    WorldCard {
        key: "posture_world".to_string(),
        name: "Posture World".to_string(),
        description: "A world for the update contract tests.".to_string(),
        narrator_mode: NarratorMode::InteractiveFiction,
        narrative_perspective: NarrativePerspective::Second,
        narrative_tense: NarrativeTense::Past,
        ..Default::default()
    }
}

async fn updated_world(state: &chronicler_engine::adapters::driving::http::AppState) -> WorldCard {
    let (_world_id, card, _map) = state
        .world_catalogue
        .get_world("posture_world")
        .expect("get_world should succeed")
        .expect("the world should exist");
    card
}

// [docs/specs/worlds.md] SCENARIO: 25.1
#[tokio::test]
async fn test_world_update_without_posture_fields_preserves_posture_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let resp = post_form(
        &app,
        "/worlds/posture_world",
        &world_form_body(&world, &TestMap::single_room("start")),
    )
    .await;
    assert!(resp.status().is_success(), "the update should succeed");

    let stored = updated_world(&state).await;
    assert_eq!(stored.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(stored.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(stored.narrative_tense, NarrativeTense::Past);
}

// [docs/specs/worlds.md] SCENARIO: 25.2
#[tokio::test]
async fn test_world_update_partial_posture_merges_per_field_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let body = format!(
        "{}&narrative_tense=present",
        world_form_body(&world, &TestMap::single_room("start"))
    );
    let resp = post_form(&app, "/worlds/posture_world", &body).await;
    assert!(resp.status().is_success());

    let stored = updated_world(&state).await;
    assert_eq!(stored.narrative_tense, NarrativeTense::Present);
    assert_eq!(stored.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(stored.narrative_perspective, NarrativePerspective::Second);
}

// [docs/specs/worlds.md] SCENARIO: 25.3
#[tokio::test]
async fn test_world_update_unknown_posture_value_falls_back_to_default_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let body = format!(
        "{}&narrator_mode=warp_drive",
        world_form_body(&world, &TestMap::single_room("start"))
    );
    let resp = post_form(&app, "/worlds/posture_world", &body).await;
    assert!(
        resp.status().is_success(),
        "the update should still succeed"
    );

    let stored = updated_world(&state).await;
    assert_eq!(
        stored.narrator_mode,
        NarratorMode::Novel,
        "an unknown value must fall back to the domain default"
    );
}

// [docs/specs/worlds.md] SCENARIO: 25.4
#[tokio::test]
async fn test_world_update_options_toggle_checkbox_grammar_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let world = posture_world();
    storage
        .seed_world(&world, &TestMap::single_room("start"))
        .expect("seed posture world");

    let checked = format!(
        "{}&options_always_on=true",
        world_form_body(&world, &TestMap::single_room("start"))
    );
    let resp = post_form(&app, "/worlds/posture_world", &checked).await;
    assert!(resp.status().is_success());
    assert!(
        updated_world(&state).await.options_always_on,
        "a checked box must set the toggle"
    );

    let resp = post_form(
        &app,
        "/worlds/posture_world",
        &world_form_body(&world, &TestMap::single_room("start")),
    )
    .await;
    assert!(resp.status().is_success());
    assert!(
        !updated_world(&state).await.options_always_on,
        "an absent field must reset the toggle to false"
    );
}
