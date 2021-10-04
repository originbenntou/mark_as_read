use crate::request::client::{
    GClient,
    Method
};
use reqwest::Error;

extern crate serde;
extern crate serde_json;
use serde::{Serialize, Deserialize};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct From {
    id: String,
    address: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MassageList {
    pub messages: Vec<MassageIdSet>,
    pub result_size_estimate: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MassageIdSet {
    id: String,
    thread_id: String
}

impl MassageList {
    // 未読メッセージ取得
    pub async fn get_unread_messages(g_client: &GClient) -> Result<MassageList, Error> {
        let res_body = g_client.call_api(
            &"https://gmail.googleapis.com/gmail/v1/users/me/messages",
            &vec![("q", "is:unread")],
            &vec![],
            Method::GET,
        ).await?;

        Ok(serde_json::from_str(&res_body).unwrap())
    }
}
