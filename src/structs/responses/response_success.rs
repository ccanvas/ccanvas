use serde::Serialize;
use serde_json::Value;

use crate::structs::Discriminator;

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum ResponseSuccess {
    /// subscription added
    #[serde(rename = "subscribe added")]
    SubscribeAdded,

    /// subscription removed
    #[serde(rename = "subscribe removed")]
    SubscribeRemoved,

    /// listener socket set
    #[serde(rename = "listener set")]
    ListenerSet { discrim: Discriminator },

    /// component dropped
    #[serde(rename = "dropped")]
    Dropped,

    /// render task completed
    #[serde(rename = "rendered")]
    Rendered,

    /// process spawned with discrim
    #[serde(rename = "spawned")]
    Spawned { discrim: Discriminator },

    /// message delivered ot target
    #[serde(rename = "message delivered")]
    MessageDelivered,

    /// space created with discrim
    #[serde(rename = "space created")]
    SpaceCreated { discrim: Discriminator },

    /// focus changed successfully
    #[serde(rename = "focus changed")]
    FocusChanged,

    /// got state
    #[serde(rename = "value")]
    Value { value: Value },

    /// set value
    #[serde(rename = "value set")]
    ValueSet,

    /// value removed
    #[serde(rename = "removed value")]
    RemovedValue,

    /// now watching a value
    #[serde(rename = "watching")]
    Watching,

    /// unwatched a value
    #[serde(rename = "unwatched")]
    Unwatched,

    /// suppressing a channel
    #[serde(rename = "suppressed")]
    Suppressed { id: u32 },

    /// unsuppressing a channel
    #[serde(rename = "unsuppressed")]
    Unsuppressed,
}
