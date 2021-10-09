use crate::request::client::{
    GClient,
    Method
};
use reqwest::Error;
use futures::future::join_all;

extern crate serde;
extern crate serde_json;
use serde::{Serialize, Deserialize};
use serde_json::Value;

pub struct MessageClient<'a> {
    pub client: &'a GClient,
    pub message: Option<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: Option<String>,
    pub thread_id: Option<String>,
    pub payload: Option<Payload>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub headers: Option<Vec<Header>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub name: Option<String>,
    pub value: Option<String>,
}

impl<'a> MessageClient<'a> {
    pub fn new(client: &'a GClient) -> Self {
        Self {
            client,
            message: None,
        }
    }

    // 未読メッセージのリストを取得
    pub async fn get_unread_list(&self) -> Result<Vec<Message>, Error> {
        let res_body = self.client.call_api(
            &"https://gmail.googleapis.com/gmail/v1/users/me/messages",
            &vec![("q", "is:unread")],
            &vec![],
            Method::GET,
        ).await?;

        let v: Value = serde_json::from_str(&res_body).unwrap();
        let v_m = v["messages"].as_array().unwrap();

        let messages = v_m.to_vec().into_iter().map(|m| {
            serde_json::from_value(m).unwrap()
        }).collect::<Vec<Message>>();

        Ok(messages)
    }

    // メッセージのメタデータを埋める
    pub async fn fill_messages_metadata(&self, message_list: &Vec<Message>) -> Result<Vec<Message>, Error> {
        let res_list = join_all(
            message_list.iter().map(|m| {
                self.get_metadata_from_only(m)
            })
        ).await;

        let filled = res_list.into_iter().map(
            |meta| {
                let m: Message = serde_json::from_str(&meta.unwrap()).unwrap();
                m
            }
        )
            // .inspect(|m| println!("{:?}", m))
            .collect::<Vec<Message>>();

        Ok(filled)
    }

    // metadataHeaders: From に絞ったメタデータを取得
    async fn get_metadata_from_only(&self, message: &Message) -> Result<String, Error> {
        let id = message.id.as_ref().unwrap();
        let url =
            "https://gmail.googleapis.com/gmail/v1/users/me/messages/".to_string() + id;

        let res_body = self.client.call_api(
            &url,
            &vec![
                ("format", "metadata"),
                ("metadataHeaders", "From"),
            ],
            &vec![],
            Method::GET,
        ).await?;

        Ok(res_body)
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: None,
            thread_id: None,
            payload: None
        }
    }
}

impl Message {
    pub fn new() -> Self { Self::default() }
}
