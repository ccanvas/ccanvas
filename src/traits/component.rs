use async_trait::async_trait;

use crate::structs::{Discriminator, Event, Storage, Unevaluated};

#[async_trait]
/// a unit of "something"
pub trait Component {
    /// unique identifier of what it is
    fn label(&self) -> &str;

    /// unique identifier which one it is
    fn discrim(&self) -> &Discriminator;

    /// folder for shared storage
    fn storage(&self) -> &Storage;

    /// pass an event into a component
    /// returns true to pass event to next component, false otherwise
    async fn pass(&self, event: &mut Event, suppress_level: Option<u32>) -> Unevaluated<bool>;
}
