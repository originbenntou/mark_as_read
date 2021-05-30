mod auth;
mod request;
mod json_parse;

use request::Request;

#[tokio::main]
async fn main() {
    // let oauth2_token = auth::get_oauth2_token();
    // println!("token is ... {}", oauth2_token);

    // tokenは有効期限が切れる
    let oauth2_token = "";

    let client = Request::new(oauth2_token);

    let res = match client.get_unread_messages().await {
        Ok(res) => res,
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    println!("response body is ... {}", res);

    let deserialize = match json_parse::deserialize(&res) {
        Ok(deserialize) => deserialize,
        Err(e) => {
            panic!("{:?}", e);
        }
    };

    let mut ids: Vec<String> = Vec::new();
    for v in deserialize["messages"].as_array().unwrap() {
        ids.push(v["id"].as_str().unwrap().to_string());
    }
    println!("ids is ... {:?}", ids);

    client.post_remove_unread(ids).await;
}
