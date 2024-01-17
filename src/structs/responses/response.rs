use std::sync::Mutex;

use serde::Serialize;
use tokio::sync::OnceCell;

use super::ResponseContent;

static RESPONSE_ID: OnceCell<Mutex<u32>> = OnceCell::const_new_with(Mutex::new(0));

fn resp_id() -> u32 {
    let mut id = RESPONSE_ID.get().unwrap().lock().unwrap();
    *id += 1;
    *id
}

/// a return signal back to a subprocess
#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Response {
    /// the content of the response
    content: ResponseContent,

    /// send a confirmation to the server using this id
    /// to confirm recieved
    id: u32,

    /// request id for confirmation
    #[serde(skip_serializing_if = "Option::is_none")]
    request: Option<u32>,
}

impl Response {
    /// construct new self
    pub fn new(content: ResponseContent) -> Self {
        Self {
            content,
            id: resp_id(),
            request: None,
        }
    }

    /// construct new self as a response to a request
    pub fn new_with_request(content: ResponseContent, request: u32) -> Self {
        Self {
            content,
            id: resp_id(),
            request: Some(request),
        }
    }

    /// get id of self
    pub fn id(&self) -> u32 {
        self.id
    }

    /// get content of self
    pub fn content(&self) -> &ResponseContent {
        &self.content
    }
}
