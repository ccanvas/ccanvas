use crate::structs::{Discriminator, Event, KeyEvent, MouseEvent};

use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum EventSerde {
    /// keyboard event
    #[serde(rename = "key")]
    Key(KeyEvent),
    /// mouse event
    #[serde(rename = "mouse")]
    Mouse(MouseEvent),
    /// screen resize event (should trigger a rerender)
    #[serde(rename = "resize")]
    Resize { width: u32, height: u32 },
    /// message passed from another process
    #[serde(rename = "message")]
    Message {
        sender: Discriminator,
        target: Discriminator,
        content: String,
    },
    #[serde(rename = "focused")]
    Focused,
    #[serde(rename = "unfocused")]
    Unfocused,
    #[serde(rename = "value updated")]
    ValueUpdated {
        label: String,
        new: Value,
        discrim: Discriminator,
    },
    #[serde(rename = "value removed")]
    ValueRemoved {
        label: String,
        discrim: Discriminator,
    },
}

impl EventSerde {
    pub fn from_event(value: &Event) -> Self {
        match value {
            Event::KeyPress(key) => Self::Key(*key),
            Event::ScreenResize(width, height) => Self::Resize {
                width: *width,
                height: *height,
            },
            Event::MouseEvent(mouse) => Self::Mouse(*mouse),
            Event::Message {
                sender,
                target,
                content,
            } => Self::Message {
                sender: sender.clone(),
                target: target.clone(),
                content: content.clone(),
            },
            Event::Focus { .. } => Self::Focused,
            Event::Unfocus => Self::Unfocused,
            Event::RequestPacket(_) => unreachable!("should not happend"),
        }
    }
}
