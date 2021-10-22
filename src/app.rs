use crossterm::event::KeyCode;
use crate::events::EventState;
use crate::config::Config;
use std::fs;

use tui::widgets::ListState;

pub struct App<'a> {
    config: Config<'a>,
    list_state: ListStates<'a>,
    address_list: Vec<&'a str>,
    count_list: Vec<&'a str>,
}

impl App<'_> {
    pub fn new(
        config: Config<'static>,
        list_state: ListStates<'static>,
        address_list: Vec<&'static str>,
        count_list: Vec<&'static str>
    ) -> Self {
        Self {
            config,
            list_state,
            address_list,
            count_list
        }
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

pub struct ListStates<'a> {
    from: &'a mut ListState,
    count: &'a mut ListState,
}

impl<'a> ListStates<'a> {
    pub fn new(from: &'a mut ListState, count: &'a mut ListState) -> Self {
        Self {
            from,
            count,
        }
    }
}
