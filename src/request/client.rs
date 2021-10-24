use reqwest::{Client, Error, header::*};
use std::collections::HashMap;

// reqwest client wrapper
#[derive(Debug)]
pub struct GClient {
    pub client: Client,
}

pub enum Method {
    GET,
    POST,
}

impl Default for GClient {
    fn default() -> Self {
        Self {
            client: Client::new()
        }
    }
}

impl GClient {
    pub fn new(token: &str) -> Self {
        let client = Client::builder().default_headers(Self::gen_headers(token)).build().unwrap();
        GClient {
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
            Err(_) => {
                unreachable!();
            }
        };

        req_headers
    }

    pub async fn call_api(
        &self,
        url: &str,
        query: &Vec<(&str, &str)>,
        body: &HashMap<&str, Vec<&str>>,
        method: Method,
    ) -> Result<Option<String>, Error>
    {
        match method {
            Method::GET => {
                let res_body = self.client
                    .get(url)
                    .query(query)
                    .send().await?
                    .text().await?;

                Ok(Some(res_body))
            },
            Method::POST => {
                let _ = self.client
                    .post(url)
                    .json(body)
                    .send().await?;

                Ok(None)
            }
        }
    }
}
