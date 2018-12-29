use std::time::Duration;

use serde::{Deserialize, Serialize};

/// An enum of windows in the game.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Window {
    MainMenu,
    Game,
    Help,
    Settings,
    Endgame,
    Message {
        message: String,
        ttl: Option<Duration>,
    },
}

pub fn message_box<S: Into<String>>(message: S) -> Window {
    Window::Message {
        message: message.into(),
        ttl: None,
    }
}

pub fn timed_message_box<S: Into<String>>(message: S, ttl: Duration) -> Window {
    Window::Message {
        message: message.into(),
        ttl: Some(ttl),
    }
}
