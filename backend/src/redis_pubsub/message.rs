use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub payload: String,
}

impl Message {
    pub fn new(payload: String) -> Message {
        Message {
            id: Message::generate_id(),
            payload,
        }
    }

    fn generate_id() -> String {
        Uuid::new_v4().simple().to_string()
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: String::from(""),
            payload: String::from(""),
        }
    }
}
