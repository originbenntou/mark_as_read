mod config;
mod request;
mod message;
mod events;
mod app;

use config::Config;
use request::{
    client::GClient,
};
use message::{MessageClient};
use events::events::{Event, Events};
use app::App;
use crate::events::EventState;

use tui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode},
};

use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("mark as read ... start");

    // APIクライアント初期化
    let mut config = Config::new();
    config.init();
    let client = GClient::new(&config.valid_token.as_ref().unwrap());
    let message_client = MessageClient::new(&client);

    // 未読リスト取得
    let unread_message_list = message_client.get_unread_list().await?;
    let unread_num = unread_message_list.len();

    if unread_num == 0 {
        println!("no unread ... end");
        std::process::exit(0);
    } else {
        println!("unread count is {}", unread_num);
    }

    // 未読リストの詳細データを埋める
    let filled_message_list = message_client.fill_messages_metadata(&unread_message_list).await?;

    // 表示用にアドレスと数値のリストを生成
    let address_count_list = message::get_address_count_list(
        &filled_message_list
    ).unwrap();
    // FIXME: 関数内で生まれた値は一度外だししないと、参照にできないのが不便？
    let (address_list, count_list) = message::split_address_count(&address_count_list);

    // rowモード
    enable_raw_mode().expect("raw mode");

    let events = Events::new(200);

    // 画面初期化
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(
        &config,
        &filled_message_list,
        &address_list,
        &count_list,
    );

    terminal.clear()?;

    loop {
        // widget生成
        terminal.draw(|f| {
            app.draw(f);
        })?;

        match events.next()? {
            Event::Input(event) => {
                match app.event(event.code) {
                    Ok(state) => {
                        if state == EventState::NotConsumed {
                            break;
                        }
                    },
                    Err(_err) => {
                        break;
                    }
                }
            },
            Event::Tick => {
                // 次のループへ
            }
        }
    }

    disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
}
