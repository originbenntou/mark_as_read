use crate::request::secret;
use std::{
    fs,
    path::Path,
};

#[derive(Debug)]
pub struct Config<'a> {
    pub secret_path: &'a str,
    pub mark_list_path: &'a str,
    pub valid_token: Option<String>,
}

impl Default for Config<'_> {
    fn default() -> Self {
        Self {
            secret_path: "./data/secret",
            mark_list_path: "./data/mark_list.json",
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
            self.secret_path,
            self.mark_list_path,
        ]);
        self.set_token();
    }

    fn set_token(&mut self) {
        let mut token = fs::read_to_string(self.secret_path).unwrap_or_default();

        if !&token.is_empty() {
            println!("oauth2 token is already set");
        } else {
            token = secret::get_oauth2_token().unwrap();
            fs::write(self.secret_path, &token).unwrap();
            println!("get oauth2 token ... ok");
        }

        self.valid_token = Some(token);
    }
}

fn create_essential_files(paths: Vec<&str>) {
    paths.into_iter().map(|p| {
        if !Path::new(p).exists() {
            fs::File::create(p).unwrap();
        }
    });
}

