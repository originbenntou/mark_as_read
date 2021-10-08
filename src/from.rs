use reqwest::Error;
use crate::request::{
    client::{GClient, Method},
    message::Message,
    message_list::MessageList,
};

extern crate serde;
extern crate serde_json;
use serde::{Serialize, Deserialize};

use futures::future::join_all;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct From {
    pub id: Option<String>,
    pub address: Option<String>,
}

impl Default for From {
    fn default() -> Self {
        Self {
            id: None,
            address: None,
        }
    }
}

impl From {
    pub fn new() -> Self { Self::default() }

    pub async fn get_sorted_list(&self, request: &GClient, message_list: &MessageList) -> Result<Vec<From>, Error> {
        let meta_list = join_all(
            message_list.messages.as_ref().unwrap().iter().map(
                |m| {
                    m.get_metadata_from_only(request)
                }
            )
        ).await;

        let list = meta_list.into_iter().map(
            |meta| {
                let m: Message = serde_json::from_str(&meta.unwrap()).unwrap();
                m
            }
        ).inspect(|m| println!("{:?}", m)).collect::<Vec<Message>>();

        Ok(vec![self::From::new()])
    }
}
