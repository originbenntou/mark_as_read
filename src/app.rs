use crossterm::event::KeyCode;
extern crate events::EventState;

pub fn event(key: KeyCode) -> Result<EventState, ()> {
    match key {
        KeyCode::Char('q') => {
            return Ok(EventState::NotConsumed);
        },
        KeyCode::Char('a') => {
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
            return Ok(EventState::Consumed);
        },
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
            return Ok(EventState::Consumed);
        },
        _ => {
            return Ok(EventState::Consumed);
        }
    }
}
