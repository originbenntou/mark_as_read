use crate::request::client::{
    GClient,
    Method
};
use reqwest::Error;

extern crate serde;
extern crate serde_json;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: Option<String>,
    pub thread_id: Option<String>,
    pub payload: Option<Payload>,
}

pub struct Payload {
    headers: Option<Vec<Header>>,
}

pub struct Header {
    pub name: Option<String>,
    pub value: Option<String>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: None,
            thread_id: None,
            payload: None,
        }
    }
}

impl Message {
    pub fn new() -> Self { Self::default() }

    // Fromに絞ったメタデータ取得
    pub async fn get_metadata_from_only(&self, id: &str) -> Result<String, Error> {
        let url = "https://gmail.googleapis.com/gmail/v1/users/me/messages/".to_string() + id;
        let res_body = self.client
            .get(&url)
            .query(&[
                ("format", "metadata"),
                ("metadataHeaders", "From"),
            ])
            .send().await?
            .text().await?;

        Ok(res_body)
    }
}
