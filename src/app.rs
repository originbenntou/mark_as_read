use crossterm::event::KeyCode;
use crate::events::EventState;
use crate::config::Config;
use crate::message::{Message, Header};

use tui::{
    Frame,
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Tabs,
    },
};

use std::fs;

pub struct App<'a> {
    config: &'a Config<'a>,
    message_list: &'a Vec<Message>,
    list_state: ListStates,
    address_list: &'a Vec<&'a str>,
    count_list: &'a Vec<&'a str>,
}

impl<'a> App<'a> {
    pub fn new(
        config: &'a Config<'a>,
        message_list: &'a Vec<Message>,
        address_list: &'a Vec<&'a str>,
        count_list: &'a Vec<&'a str>
    ) -> Self {
        // From選択構造体
        let mut from_list_state = ListState::default();
        from_list_state.select(Some(0));

        // Count選択構造体
        let mut count_list_state = ListState::default();
        count_list_state.select(Some(0));

        let list_state = ListStates::new(
            from_list_state,
            count_list_state
        );

        Self {
            config,
            message_list,
            list_state,
            address_list,
            count_list
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<'_, B>) {
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

        let menu = vec!["Add", "Delete", "Execute", "Quit"]
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
            self.address_list.clone(), // ループするクロージャごとでmoveされるため
        );
        f.render_stateful_widget(left, horizon_chunk[0], &mut self.list_state.from);

        let mid = render_froms(
            "Count",
            self.count_list.clone(), // ループするクロージャごとでmoveされるため
        );
        f.render_stateful_widget(mid, horizon_chunk[1], &mut self.list_state.count);

        let content = fs::read_to_string(self.config.mark_list_path).unwrap();

        // if db_content == "" {
        //     let empty_vec: Vec<String> = Vec::new();
        //     Ok((empty_vec))
        // }

        let mark_list: Vec<String> = serde_json::from_str(&content).unwrap();

        // 右部Targetリスト
        let right = render_froms(
            "Target",
            mark_list.iter().map(AsRef::as_ref).collect()
        );
        f.render_widget(right, horizon_chunk[2]);
    }

    pub fn event(&mut self, key: KeyCode) -> Result<EventState, ()> {
        match key {
            KeyCode::Char('q') => {
                return Ok(EventState::NotConsumed);
            },
            KeyCode::Char('a') => {
                if let Some(selected) = self.list_state.from.selected() {
                    let mark_list = fs::read_to_string(self.config.mark_list_path).unwrap();
                    let mut add_list: Vec<String> = Vec::new();

                    // リストが空じゃなかったらパースして突っ込む
                    if !&mark_list.is_empty() {
                        let parsed: Vec::<String> = serde_json::from_str(&mark_list).unwrap();
                        add_list.append(&mut parsed.clone());
                    }

                    add_list.push(self.address_list[selected].to_string());

                    // 重複排除
                    add_list.sort();
                    add_list.dedup();

                    fs::write(self.config.mark_list_path, &serde_json::to_vec(&add_list).unwrap()).unwrap();
                }
                return Ok(EventState::Consumed);
            },
            KeyCode::Char('d') => {
                // 消せるようにしたいなー
                return Ok(EventState::Consumed);
            },
            KeyCode::Char('e') => {
                let test = filled_message_list.into_iter().filter(
                    |m| m.payload.as_ref().unwrap().headers.as_ref().unwrap().contains(
                        &Header {
                            name: Some("From".to_string()),
                            value: Some("Coincheck <support@coincheck.com>".to_string()),
                        }
                    )
                ).collect::<Vec<Message>>();

                println!("mark as read ... complete");
                return Ok(EventState::Consumed);
            },
            KeyCode::Down => {
                if let Some(selected) = self.list_state.from.selected() {
                    if selected >= self.address_list.len() - 1 {
                        self.list_state.from.select(Some(0));
                    } else {
                        self.list_state.from.select(Some(selected + 1));
                    }
                }
                if let Some(selected) = self.list_state.count.selected() {
                    if selected >= self.count_list.len() - 1 {
                        self.list_state.count.select(Some(0));
                    } else {
                        self.list_state.count.select(Some(selected + 1));
                    }
                }
                return Ok(EventState::Consumed);
            },
            KeyCode::Up => {
                if let Some(selected) = self.list_state.from.selected() {
                    if selected > 0 {
                        self.list_state.from.select(Some(selected - 1));
                    } else {
                        self.list_state.from.select(Some(self.address_list.len() - 1));
                    }
                }
                if let Some(selected) = self.list_state.count.selected() {
                    if selected > 0 {
                        self.list_state.count.select(Some(selected - 1));
                    } else {
                        self.list_state.count.select(Some(self.count_list.len() - 1));
                    }
                }
                return Ok(EventState::Consumed);
            },
            _ => {
                return Ok(EventState::Consumed);
            }
        }
    }
}

pub struct ListStates {
    from: ListState,
    count: ListState,
}

impl ListStates {
    pub fn new(from: ListState, count: ListState) -> Self {
        Self {
            from,
            count,
        }
    }
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

