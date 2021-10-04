mod config;
mod request;
mod util;

use request::{
    client::GClient,
    message_list::MessageList,
};
use config::Config;
use util::json_parse;
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
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use thiserror::Error;

use std::{
    time::{Duration, Instant},
    sync::mpsc,
    io,
    fs,
    thread,
    collections::HashMap,
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

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct From {
    id: String,
    address: String,
}

const MARK_LIST_PATH: &str = "./data/mark_list.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("mark as read ... start");

    let mut config = Config::new();
    config.init();
    let client = GClient::new(&config.valid_token.unwrap_or_default());

    let unread_message_list = MessageList::new().get_unread_messages(&client).await?;
    let unread_num = unread_message_list.result_size_estimate.unwrap();

    if unread_num == 0 {
        println!("no unread ... end");
        std::process::exit(0);
    } else {
        println!("unread count is {}", unread_num);
    }

    let mut from_list: Vec<From> = Vec::new();

    // FIXME: 並行処理しないと遅い！
    for message in unread_messages["messages"].as_array().unwrap() {
        let id = message["id"].as_str().unwrap();
        /* TODO:
            こういうloop内にあるGETした変数のlifetimeを永続化することはできないか（レスポンスをヒープに収めたらそれ以外はすべて参照にしたい）
            所有権を移すしかないのか...
         */
        let metadata = match client.get_metadata_from_only(id).await {
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

        for header_from_only in metadata["payload"]["headers"].as_array().unwrap() {
            let address = header_from_only["value"].as_str().unwrap();
            from_list.push(From{
                id: id.to_string(),
                address: address.to_string(),
            });
        };
    }

    from_list.sort_by(|a, b| a.address.cmp(&b.address));

    // アドレスをkeyにメールIDをリスト化
    let mut address_ids_hash_map: HashMap<&String, Vec<&String>> = HashMap::new();

    // アドレスだけのリスト
    let mut address_list: Vec<&String> = Vec::new();
    // タイトルごとに集計した数のリスト
    let mut count_list: Vec<String> = Vec::new();

    // TODO: クロージャでも掛けるようになっておきたい
    // from_list.iter().map(|f| hash_map.insert(&f.address, &f.id));
    for f in from_list.iter() {
        if !address_ids_hash_map.contains_key(&f.address) {
            address_ids_hash_map.insert(&f.address, vec![&f.id]);
            address_list.push(&f.address);
        } else {
            address_ids_hash_map.get_mut(&f.address).unwrap().push(&f.id);
        }
    }

    count_list = address_ids_hash_map.iter().map(|x| x.1.len().to_string()).collect::<Vec<String>>();

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

    // From選択構造体
    let mut from_list_state = ListState::default();
    from_list_state.select(Some(0));

    // Count選択構造体
    let mut count_list_state = ListState::default();
    count_list_state.select(Some(0));

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
                address_list.iter().map(|x| x.as_str()).collect::<Vec<&str>>()
            );
            f.render_stateful_widget(left, horizon_chunk[0], &mut from_list_state);

            let mid = render_froms(
                "Count",
                count_list.iter().map(|x| x.as_str()).collect::<Vec<&str>>()
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
                        if selected >= address_list.len() - 1 {
                            from_list_state.select(Some(0));
                        } else {
                            from_list_state.select(Some(selected + 1));
                        }
                    }
                    if let Some(selected) = count_list_state.selected() {
                        if selected >= count_list.len() - 1 {
                            count_list_state.select(Some(0));
                        } else {
                            count_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = from_list_state.selected() {
                        if selected > 0 {
                            from_list_state.select(Some(selected - 1));
                        } else {
                            from_list_state.select(Some(address_list.len() - 1));
                        }
                    }
                    if let Some(selected) = count_list_state.selected() {
                        if selected > 0 {
                            count_list_state.select(Some(selected - 1));
                        } else {
                            count_list_state.select(Some(count_list.len() - 1));
                        }
                    }
                }
                KeyCode::Enter => {
                    if let Some(selected) = from_list_state.selected() {
                        let mark_list = fs::read_to_string(config.mark_list_path)?;
                        let mut add_list: Vec<String> = Vec::new();

                        // リストが空じゃなかったらパースして突っ込む
                        if !&mark_list.is_empty() {
                            let parsed: Vec::<String> = serde_json::from_str(&mark_list)?;
                            add_list.append(&mut parsed.clone());
                        }

                        add_list.push(address_list[selected].to_string());

                        // 重複排除
                        add_list.sort();
                        add_list.dedup();

                        fs::write(config.mark_list_path, &serde_json::to_vec(&add_list)?)?;
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

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
