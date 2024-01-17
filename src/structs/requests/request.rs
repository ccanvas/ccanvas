use std::sync::Mutex;

use crate::structs::{Discriminator, Event, Packet, Response, ResponseContent, Subscription};

use super::RequestContent;
use serde::Deserialize;
use tokio::sync::OnceCell;

/// a signal that comes from a subprocess
#[derive(Deserialize, Debug, Clone)]
pub struct Request {
    /// reciever
    target: Discriminator,
    /// the content of the request
    content: RequestContent,
    /// confirmation identifier
    id: u32,
}

/// for generated requests not coming from a process
static REQ_ALTID: OnceCell<Mutex<u32>> = OnceCell::const_new_with(Mutex::new(u32::MAX));

fn req_id() -> u32 {
    let mut id = REQ_ALTID.get().unwrap().lock().unwrap();
    *id -= 1;
    *id
}

impl Request {
    /// construct new self
    pub fn new(target: Discriminator, content: RequestContent) -> Self {
        Self {
            target,
            content,
            id: req_id(),
        }
    }
    /// returns discrim of target component
    pub fn target(&self) -> &Discriminator {
        &self.target
    }

    /// returns discrim of target component (mutable)
    pub fn target_mut(&mut self) -> &mut Discriminator {
        &mut self.target
    }

    /// returns RequestContent (mutable)
    pub fn content_mut(&mut self) -> &mut RequestContent {
        &mut self.content
    }

    /// returns RequestContent
    pub fn content(&self) -> &RequestContent {
        &self.content
    }

    /// send self to master space, and wait for response
    pub async fn send(self) -> Response {
        let (packet, recv) = Packet::new(self);
        Event::send(Event::from_packet(packet));

        if let Ok(res) = recv.await {
            res
        } else {
            Response::new(ResponseContent::Undelivered)
        }
    }

    /// get request id
    pub fn id(&self) -> &u32 {
        &self.id
    }

    /// returns subscriptions that would want to take in self
    pub fn subscriptions(&self) -> Option<&[Subscription]> {
        match self.content {
            RequestContent::Message { .. } => Some(&[Subscription::AllMessages]),
            _ => None,
        }
    }
}
