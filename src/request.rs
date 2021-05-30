use reqwest::{Client, Error, header::*};
use std::collections::HashMap;

// reqwest client wrapper
pub struct Request {
    client: Client,
}

impl Request {
    pub fn new(token: &str) -> Self {
        let client = Client::builder().default_headers(Self::gen_headers(token)).build().unwrap();
        Request {
            client
        }
    }

    pub fn gen_headers(token: &str) -> HeaderMap {
        let mut req_headers = HeaderMap::new();
        req_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        req_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        match HeaderValue::from_str(&("Bearer ".to_string() + token)) {
            Ok(auth) => {
                req_headers.insert(AUTHORIZATION, auth);
            },
            Err(e) => {}
        };

        req_headers
    }

    pub async fn get_unread_messages(&self) -> Result<String, Error> {
        let res_body = self.client
            .get("https://gmail.googleapis.com/gmail/v1/users/me/messages")
            .query(&[("q", "is:unread")])
            .send().await?
            .text().await?;

        Ok(res_body)
    }

    pub async fn post_remove_unread(&self, ids: Vec<&str>) -> Result<(), Error> {
        let mut req_body = HashMap::new();
        req_body.insert("ids", ids);
        req_body.insert("removeLabelIds", vec!["UNREAD"]);

        let _ = self.client
            .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/batchModify")
            .json(&req_body)
            .send().await?;

        Ok(())
    }
}
