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
pub struct Metadata {
    pub messages: Option<Vec<MassageIdSet>>,
    pub result_size_estimate: Option<usize>,
}

impl Default for MassageList {
    fn default() -> Self {
        Self {
            messages: None,
            result_size_estimate: None,
        }
    }
}

impl MassageList {
    pub fn new() -> Self { Self::default() }

    // 未読メッセージ取得
    // TODO: self取ってるけど使ってない...
    pub async fn get_unread_messages(&self, g_client: &GClient) -> Result<MassageList, Error> {
        let res_body = g_client.call_api(
            &"https://gmail.googleapis.com/gmail/v1/users/me/messages",
            &vec![("q", "is:unread")],
            &vec![],
            Method::GET,
        ).await?;

        Ok(serde_json::from_str(&res_body).unwrap())
    }
}
