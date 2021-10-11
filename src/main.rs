mod config;
mod request;
mod message;
mod events;
mod app;

use config::Config;
use request::{
    client::GClient,
};
use message::MessageClient;
use events::events::{Event, Events};
use app::EventState;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Tabs,
    },
    Terminal,
};
use crossterm::{
    event::{KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use thiserror::Error;

use std::{
    time::{Duration, Instant},
    sync::mpsc,
    io,
    fs,
    thread,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

const MARK_LIST_PATH: &str = "./data/mark_list.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("mark as read ... start");

    // APIクライアント初期化
    let mut config = Config::new();
    config.init();
    let client = GClient::new(&config.valid_token.unwrap_or_default());
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
    // FIXME: 引数はもう使わないのでmoveすべきかも、いや借用をちゃんと利用すべきかも
    let filled_message_list = message_client.fill_messages_metadata(&unread_message_list).await?;

    // 表示用にアドレスと数値のリストを生成
    let address_count_list = message::get_address_count_list(
        &filled_message_list
    ).unwrap();
    // FIXME: 関数内で生まれた値は一度外だししないと、参照にできないのが不便
    let (address_list, count_list) = message::split_address_count(&address_count_list);

    // rowモード
    enable_raw_mode().expect("raw mode");

    let events = Events::new(200);

    // 画面初期化
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Add", "Delete", "Execute", "Quit"];

    // From選択構造体
    let mut from_list_state = ListState::default();
    from_list_state.select(Some(0));

    // Count選択構造体
    let mut count_list_state = ListState::default();
    count_list_state.select(Some(0));

    terminal.clear()?;

    loop {
        // widget生成
        terminal.draw(|f| {
            // 縦方向分割
            let vertical_chunk = Layout::default()
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
            f.render_widget(tabs, vertical_chunk[0]);

            // 横方向分割
            let horizon_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(47),
                        Constraint::Percentage(6),
                        Constraint::Percentage(47),
                    ].as_ref(),
                )
                .split(vertical_chunk[1]);

            // 左部Fromリスト
            let left = render_froms(
                "From",
                address_list.clone(), // ループするクロージャごとでmoveされるため
            );
            f.render_stateful_widget(left, horizon_chunk[0], &mut from_list_state);

            let mid = render_froms(
                "Count",
                count_list.clone(), // ループするクロージャごとでmoveされるため
            );
            f.render_stateful_widget(mid, horizon_chunk[1], &mut count_list_state);

            // 右部Targetリスト
            let mark_list = read_db().unwrap();
            let right = render_froms(
                "Target",
                mark_list.iter().map(AsRef::as_ref).collect()
            );
            f.render_widget(right, horizon_chunk[2]);
        })?;

        match events.next()? {
            Event::Input(event) => {
                match app::event(event.code) {
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

fn render_froms<'a>(block_name: &'a str, from_list: Vec<&'a str>) -> List<'a> {
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
    let content = fs::read_to_string(MARK_LIST_PATH)?;

    // if db_content == "" {
    //     let empty_vec: Vec<String> = Vec::new();
    //     Ok((empty_vec))
    // }

    let parsed: Vec<String> = serde_json::from_str(&content)?;
    Ok(parsed)
}
