use serde::{Deserialize, Serialize};

use crate::structs::{KeyCode, KeyEvent, KeyModifier, MouseType};

use super::Discriminator;

/// a single subscription item, such as a key press event
#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Subscription {
    /// Every, single, event
    #[serde(rename = "everything")]
    Everything,
    /// subscribes to all key press events
    #[serde(rename = "all key presses")]
    AllKeyPresses,
    /// all mouse click and drag events
    #[serde(rename = "all mouse events")]
    AllMouseEvents,
    /// subscribe to all messages from other components
    #[serde(rename = "all messages")]
    AllMessages,
    /// a specific key event
    #[serde(rename = "specific key press")]
    SpecificKeyPress { key: KeyEvent },
    /// all key events with that key modifier
    #[serde(rename = "specific key modifier")]
    SpecificKeyModifier { modifier: KeyModifier },
    /// all key events with that key modifier
    #[serde(rename = "specific key code")]
    SpecificKeyCode { code: KeyCode },
    /// a specific mouse event
    #[serde(rename = "specific mouse event")]
    SpecificMouseEvent { mouse: MouseType },
    /// a specific message from someone
    #[serde(rename = "specific message")]
    SpecificMessage { source: Discriminator },
    /// screen resize events
    #[serde(rename = "screen resize")]
    ScreenResize,
    #[serde(rename = "focused")]
    /// current space focused
    Focused,
    #[serde(rename = "unfocused")]
    /// current space unfocused
    Unfocused,

    #[serde(rename = "multiple")]
    /// subscribe to multiple channels at once
    Multiple {
        subs: Vec<(Subscription, Option<u32>)>,
    },
}
