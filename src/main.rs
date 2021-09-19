mod request;
mod util;

use request::{
    g_auth,
    g_client::GClient,
};
use util::json_parse;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Row, Table, Tabs,
    },
    Terminal,
};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use serde::{Deserialize, Serialize};
use rand::{distributions::Alphanumeric, prelude::*};
use chrono::prelude::*;
use thiserror::Error;

use std::{
    time::{Duration, Instant},
    sync::mpsc,
    io,
    fs,
    path::Path,
    thread,
    env,
    process::exit,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

const SECRET_PATH: &str = "./data/secret";
const MARK_LIST_PATH: &str = "./data/mark_list.json";
const OAUTH_TOKEN: &str = "OAUTH2_TOKEN";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("mark as read ... start");

    if !Path::new(SECRET_PATH).exists() {
        match fs::File::create(SECRET_PATH) {
            Ok(_) => {},
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }

    if !Path::new(MARK_LIST_PATH).exists() {
        match fs::File::create(MARK_LIST_PATH) {
            Ok(_) => {},
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }

    let mut oauth2_token = fs::read_to_string(SECRET_PATH)?;

    if !&oauth2_token.is_empty() {
        println!("oauth2 token is already set");
    } else {
        oauth2_token = match g_auth::get_oauth2_token() {
            Ok(token) => {
                println!("get oauth2 token ... ok");
                fs::write(SECRET_PATH, &token);
                token
            },
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }

    let client = GClient::new(&oauth2_token);

    let unread_messages = match client.get_unread_messages().await {
        // 練習がてら敢えて構造体にマップせず、ゆるふわパース
        Ok(res) => match json_parse::deserialize(&res) {
            Ok(deserialize) => deserialize,
            Err(err) => {
                panic!("{:?}", err);
            }
        },
        Err(err) => {
            panic!("{:?}", err);
        }
    };

    let unread_count = unread_messages["resultSizeEstimate"].as_u64().unwrap();

    if let 0 = unread_count {
        println!("no unread ... end");
        std::process::exit(0);
    } else {
        println!("unread count is {}", unread_count);

    }

    // From list
    // FIXME: 並行処理しないと遅い！
    // FIXME: Vec<&str> にしたい...けどscopeをうまく操作できない
    let mut from_list: Vec<String> = Vec::new();
    for message in unread_messages["messages"].as_array().unwrap() {
        let metadata = match client.get_metadata_from_only(message["id"].as_str().unwrap()).await {
            Ok(res) => match json_parse::deserialize(&res) {
                Ok(deserialize) => deserialize,
                Err(e) => {
                    panic!("{:?}", e);
                }
            },
            Err(err) => {
                panic!("{:?}", err);
            }
        };

        for header_from_only in metadata["payload"]["headers"].as_array().unwrap() {
            from_list.push(header_from_only["value"].as_str().unwrap().to_string());
        };
    }

    // 重複するFromは除外
    from_list.sort();
    from_list.dedup();

    // rowモード
    enable_raw_mode().expect("raw mode");

    // チャネル送受信機生成
    let (tx, rx) = mpsc::channel();

    // 200ミリ秒間隔でキー受付
    let tick_rate = Duration::from_millis(200);
    // スレッド生成
    // 所有権をスレッド内にmove
    thread::spawn(move || {
        // 現在時間を経過時間を管理するために生成
        let mut last_tick = Instant::now();
        loop {
            // 経過時間の差を取得
            // Durationが0になることを意図して経過時間を記録し続ける
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // Durationが0以外ならpoll
            if event::poll(timeout).expect("poll works") {
                // キー入力をrxにsend
                if let CEvent::Key(key) = event::read().expect("read events") {
                    tx.send(Event::Input(key)).expect("send events");
                }
            }

            // 経過秒が200ミリ秒を超えたらtickを送信して経過秒をリセット
            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    // 画面初期化
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Add", "Delete", "Execute", "Quit"];
    let mut from_list_state = ListState::default();
    from_list_state.select(Some(0));

    loop {
        // widget生成
        terminal.draw(|f| {
            // 縦方向分割
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                    ]
                        .as_ref(),
                )
                .split(f.size());

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            // 上部メニュー
            let tabs = Tabs::new(menu)
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .divider(Span::raw("|"));
            f.render_widget(tabs, chunks[0]);

            // 横方向分割
            let from_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                )
                .split(chunks[1]);

            // 左部Fromリスト
            let left = render_froms("From", from_list.clone());
            f.render_stateful_widget(left, from_chunks[0], &mut from_list_state);

            // 右部Targetリスト
            let right = render_froms("Target",read_db().unwrap());
            f.render_widget(right, from_chunks[1]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('a') => {
                }
                KeyCode::Char('d') => {
                }
                KeyCode::Char('e') => {
                    // post unread
                    // let mut ids: Vec<&str> = Vec::new();
                    // for v in deserialize["messages"].as_array().unwrap() {
                    //     ids.push(v["id"].as_str().unwrap());
                    // }
                    //
                    // match client.post_remove_unread(ids).await {
                    //     Ok(_) => {},
                    //     Err(e) => {
                    //         panic!("{:?}", e);
                    //     }
                    // };

                    println!("mark as read ... complete");
                }
                KeyCode::Down => {
                    if let Some(selected) = from_list_state.selected() {
                        if selected >= from_list.len() - 1 {
                            from_list_state.select(Some(0));
                        } else {
                            from_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = from_list_state.selected() {
                        if selected > 0 {
                            from_list_state.select(Some(selected - 1));
                        } else {
                            from_list_state.select(Some(from_list.len() - 1));
                        }
                    }
                }
                KeyCode::Enter => {
                    if let Some(selected) = from_list_state.selected() {
                        let mark_list = fs::read_to_string(MARK_LIST_PATH)?;
                        let mut add_list: Vec<String> = Vec::new();

                        // リストが空じゃなかったらパースして突っ込む
                        if !&mark_list.is_empty() {
                            let parsed: Vec::<String> = serde_json::from_str(&mark_list)?;
                            add_list.append(&mut parsed.clone());
                        }

                        add_list.push(from_list[selected].clone());

                        // 重複排除
                        add_list.sort();
                        add_list.dedup();

                        fs::write(MARK_LIST_PATH, &serde_json::to_vec(&add_list)?)?;
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn render_froms(block_name: &str, from_list: Vec<String>) -> List {
    let from_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title(block_name)
        .border_type(BorderType::Plain);

    let items: Vec<_> = from_list
        .iter()
        .map(|from| {
            ListItem::new(Spans::from(vec![Span::styled(
                from.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let list = List::new(items).block(from_block).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    list
}

fn read_db() -> Result<Vec<String>, Error> {
    let db_content = fs::read_to_string(MARK_LIST_PATH)?;

    // if db_content == "" {
    //     let empty_vec: Vec<String> = Vec::new();
    //     Ok((empty_vec))
    // }

    let mut parsed: Vec<String> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}
