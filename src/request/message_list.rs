use crate::request::{
    client::{GClient, Method},
    message::Message,
};
use reqwest::Error;

extern crate serde;
extern crate serde_json;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageList {
    pub messages: Option<Vec<Message>>,
    pub result_size_estimate: Option<usize>,
}

impl Default for MessageList {
    fn default() -> Self {
        Self {
            messages: None,
            result_size_estimate: None,
        }
    }
}

impl MessageList {
    pub fn new() -> Self { Self::default() }

    // 未読メッセージ取得
    pub async fn get_unread_messages(&self, request: &GClient) -> Result<MessageList, Error> {
        let res_body = request.call_api(
            &"https://gmail.googleapis.com/gmail/v1/users/me/messages",
            &vec![("q", "is:unread")],
            &vec![],
            Method::GET,
        ).await?;

        Ok(serde_json::from_str(&res_body).unwrap())
    }
}
