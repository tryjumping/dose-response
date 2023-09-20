use std::time::Duration;

use serde::{Deserialize, Serialize};

/// An enum of windows in the game.
///
/// WARNING: We're cloning the `Window` stack in game (to avoid
/// mutable borrow issues). That means though we can't really store
/// any persistent window info (such as the selected button) here. It
/// needs to go into the `State`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Window {
    MainMenu,
    Game,
    Help,
    Settings,
    Endgame,
    Message {
        title: String,
        message: String,
        ttl: Option<Duration>,
    },
}

pub fn message_box<S: Into<String>>(title: S, message: S) -> Window {
    Window::Message {
        title: title.into(),
        message: message.into(),
        ttl: None,
    }
}

pub fn timed_message_box<S: Into<String>>(title: S, message: S, ttl: Duration) -> Window {
    Window::Message {
        title: title.into(),
        message: message.into(),
        ttl: Some(ttl),
    }
}
