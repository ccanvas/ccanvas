use std::{collections::BTreeMap, path::PathBuf};

use serde::Deserialize;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use crate::structs::{Discriminator, Response, Subscription};

use super::RenderRequest;

/// variations of requests
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(tag = "type")]
pub enum RequestContent {
    #[serde(rename = "confirm recieve")]
    /// confirm that an event has been recieved
    ConfirmRecieve {
        /// event id
        id: u32,
        /// true = does not capture event
        pass: bool,
    },

    #[serde(rename = "subscribe")]
    /// add subscription to a channel with priority
    Subscribe {
        channel: Subscription,
        priority: Option<u32>,
        component: Option<Discriminator>,
    },

    #[serde(rename = "Unsubscribe")]
    /// remove subscription from a channel
    Unsubscribe {
        channel: Subscription,
        component: Option<Discriminator>,
    },
    #[serde(rename = "set socket")]
    /// sent responses to this socket
    SetSocket {
        path: PathBuf,
    },

    #[serde(rename = "drop")]
    /// remove a single component
    Drop {
        discrim: Option<Discriminator>,
    },

    #[serde(rename = "render")]
    /// render something to the terminal
    Render {
        content: RenderRequest,
        flush: bool,
    },

    #[serde(rename = "spawn")]
    /// spawn a new process
    Spawn {
        command: String,
        args: Vec<String>,
        label: String,
        env: BTreeMap<String, String>,
    },

    #[serde(rename = "message")]
    /// send a message to another component
    /// if target specifies a space,
    /// all components under that space will recieve the message
    Message {
        content: Value,
        sender: Discriminator,
        target: Discriminator,
        tag: String,
    },

    /// create a new space at a space
    #[serde(rename = "new space")]
    NewSpace {
        label: String,
    },

    /// focus a specific space
    #[serde(rename = "focus at")]
    FocusAt,

    /// get a state value
    #[serde(rename = "get state")]
    GetState {
        label: StateValue,
    },

    /// get value of an entry
    #[serde(rename = "get entry")]
    GetEntry {
        label: String,
    },

    /// remove an entry
    #[serde(rename = "remove entry")]
    RemoveEntry {
        label: String,
    },

    /// get value of an entry
    #[serde(rename = "set entry")]
    SetEntry {
        label: String,
        value: Value,
    },

    /// watch a certain value in pool
    #[serde(rename = "watch")]
    Watch {
        label: String,
    },

    /// watch a certain value in pool
    #[serde(rename = "unwatch")]
    Unwatch {
        label: String,
        watcher: Discriminator,
    },

    /// suppress a channel
    #[serde(rename = "suppress")]
    Suppress {
        channel: Subscription,
        priority: u32,
    },

    /// unsuppress a channel
    #[serde(rename = "unsuppress")]
    Unsuppress {
        channel: Subscription,
        id: u32,
    },

    WatchInternal(WatchInternal),
}

/// variations of requests
#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum StateValue {
    #[serde(rename = "focused")]
    Focused,
    #[serde(rename = "is focused")]
    IsFocused,
    #[serde(rename = "term size")]
    TermSize,
    #[serde(rename = "working dir")]
    WorkingDir,
}

#[derive(Debug, Clone)]
pub struct WatchInternal {
    pub label: String,
    pub sender: UnboundedSender<Response>,
    pub watcher: Discriminator,
}

impl PartialEq for WatchInternal {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Eq for WatchInternal {}

// this will never get called, so dw about it
impl<'de> Deserialize<'de> for WatchInternal {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        panic!("no")
    }

    fn deserialize_in_place<D>(_: D, _: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        panic!("no")
    }
}
