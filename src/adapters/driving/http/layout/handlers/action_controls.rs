//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! The action control parts: Send, and Autoplay with Stop, each rendered from the busy state of the Game

use crate::adapters::driving::http::AppState;
use crate::application::games::view_query::Status;
use crate::application::generation::gate::Busy;
use crate::domain::model::game::GameId;

const SEND_LABEL: &str =
    r##"<svg class="icon" aria-hidden="true"><use href="#i-send"></use></svg> Send"##;
const GENERATING_LABEL: &str = r##"<svg class="icon icon-spin" aria-hidden="true"><use href="#i-loader-circle"></use></svg> Generating…"##;
const AUTOPLAY_LABEL: &str =
    r##"<svg class="icon" aria-hidden="true"><use href="#i-play"></use></svg> Autoplay"##;
const STOP_BUTTON: &str = r##"<button type="button" id="stop-btn" hx-post="/status/reset-busy" hx-swap="none"><svg class="icon" aria-hidden="true"><use href="#i-x"></use></svg> Stop</button>"##;

/// `None` is a Game that accepts a send. Send submits `#command-form`, which is in
/// the shell, through `form`.
pub fn send_control_answer(busy: Option<&Busy>) -> String {
    let (disabled, label) = if busy.is_some() {
        (" disabled", GENERATING_LABEL)
    } else {
        ("", SEND_LABEL)
    };
    format!(
        r#"<button type="submit" id="submit-btn" form="command-form"{disabled}>{label}</button>"#
    )
}

/// `None` is a Game that accepts a send. Autoplay submits `#autoplay-form`, which is
/// in the shell, through `form`. Stop exists only while a run holds the Game.
pub fn autoplay_controls_answer(busy: Option<&Busy>) -> String {
    let disabled = if busy.is_some() { " disabled" } else { "" };
    let stop = if matches!(busy, Some(Busy::Autoplay { .. })) {
        STOP_BUTTON
    } else {
        ""
    };
    format!(
        r#"<button type="submit" id="autoplay-btn" form="autoplay-form" title="Let the engine play the Persona for this many cycles"{disabled}>{AUTOPLAY_LABEL}</button>{stop}"#
    )
}

/// The busy state that the controls render. A failure line is not busy. A Game
/// that cannot be read, or no Game, is idle: the engine refuses a request that it
/// cannot take, and the status display reports the unreadable Game.
fn busy_of_game(state: &AppState, game: Option<GameId>) -> Option<Busy> {
    game.and_then(|game_id| match state.game_view_query.status(game_id) {
        Ok(Status::Busy(busy)) => Some(busy),
        Ok(Status::Failed(_) | Status::Idle) | Err(_) => None,
    })
}

pub fn render_send_control(state: &AppState, game: Option<GameId>) -> String {
    send_control_answer(busy_of_game(state, game).as_ref())
}

pub fn render_autoplay_controls(state: &AppState, game: Option<GameId>) -> String {
    autoplay_controls_answer(busy_of_game(state, game).as_ref())
}
