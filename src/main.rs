mod auth;
mod request;
mod json_parse;

use request::Request;
use std::env;
use std::process::exit;

#[tokio::main]
async fn main() {
    println!("mark as read ... start");

    // let oauth2_token = auth::get_oauth2_token();
    // println!("token is ... {}", oauth2_token);
    // exit(0);

    // tokenは有効期限が切れる
    let oauth2_token = env::var("OAUTH2_TOKEN").unwrap();

    let client = Request::new(&oauth2_token);

    let res_unread = match client.get_unread_messages().await {
        Ok(res) => res,
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    // 練習がてら敢えて構造体にマップせず、ゆるふわでやってみる
    let deserialize = match json_parse::deserialize(&res_unread) {
        Ok(deserialize) => deserialize,
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    if deserialize["resultSizeEstimate"].as_u64().unwrap() == 0 {
        println!("no unread ... end");
        std::process::exit(0);
    }

    let mut ids: Vec<&str> = Vec::new();
    for v in deserialize["messages"].as_array().unwrap() {
        ids.push(v["id"].as_str().unwrap());
    }

    match client.post_remove_unread(ids).await {
        Ok(_) => {},
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    println!("mark as read ... complete");
}
