use crate::request::secret;
use std::{
    fs,
    path::Path,
};

#[derive(Debug)]
pub struct Config<'a> {
    pub token_path: &'a str,
    pub mark_list_path: &'a str,
    pub log_path: &'a str,
    pub valid_token: Option<String>,
}

impl Default for Config<'_> {
    fn default() -> Self {
        Self {
            token_path: "./data/token",
            mark_list_path: "./data/mark_list.json",
            log_path: "./data/log.json",
            valid_token: None,
        }
    }
}

impl Config<'_> {
    pub fn new() -> Self {
        Config::default()
    }

    pub fn init(&mut self) {
        create_essential_files(vec![
            self.token_path,
            self.mark_list_path,
            self.log_path,
        ]);
        self.set_token();
    }

    fn set_token(&mut self) {
        let mut token = fs::read_to_string(self.token_path).unwrap_or_default();

        if !&token.is_empty() {
            println!("oauth2 token is already set");
        } else {
            token = secret::get_oauth2_token().unwrap();
            fs::write(self.token_path, &token).unwrap();
            println!("get oauth2 token ... ok");
        }

        self.valid_token = Some(token);
    }
}

#[allow(unused_must_use)]
fn create_essential_files(paths: Vec<&str>) {
    paths.into_iter().map(|p| {
        if !Path::new(p).exists() {
            fs::File::create(p).unwrap();
        }
    });
}

