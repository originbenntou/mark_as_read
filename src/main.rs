mod auth;
mod request;
mod json_parse;

use request::Request;
use std::env;
use std::process::exit;

#[tokio::main]
async fn main() {
    println!("mark as read ... start");

    // let oauth2_token = match auth::get_oauth2_token() {
    //     Ok(token) => token,
    //     Err(e) => {
    //         panic!("{:?}", e);
    //     }
    // };
    // println!("token is ... {}", oauth2_token);

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

    let result_size = deserialize["resultSizeEstimate"].as_u64().unwrap();

    if result_size == 0 {
        println!("no unread ... end");
        std::process::exit(0);
    };

    println!("unread num is {}", result_size);

    // From list
    // FIXME: 並行処理しないと遅い！
    // FIXME: Vec<&str> にしたい...けどscopeをうまく操作できない
    let mut from_list: Vec<String> = Vec::new();
    for v in deserialize["messages"].as_array().unwrap() {
        let res_message_from = match client.get_message_from(v["id"].as_str().unwrap()).await {
            Ok(res) => res,
            Err(e) => {
                panic!("{:?}", e);
            }
        };

        let deserialize = match json_parse::deserialize(&res_message_from) {
            Ok(deserialize) => deserialize,
            Err(e) => {
                panic!("{:?}", e);
            }
        };

        for header in deserialize["payload"]["headers"].as_array().unwrap() {
            from_list.push(header["value"].as_str().unwrap().to_string());
        };
    }
    println!("{:?}", from_list);

    // post unread
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
