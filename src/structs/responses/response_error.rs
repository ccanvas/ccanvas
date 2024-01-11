use serde::Serialize;

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum ResponseError {
    /// target component is not found
    #[serde(rename = "component not found")]
    ComponentNotFound,
    /// spawning process failed
    #[serde(rename = "spawn failed")]
    SpawnFailed,
    /// pool entry not found
    #[serde(rename = "entry not found")]
    EntryNotFound,
}
