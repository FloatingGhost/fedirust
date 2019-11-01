extern crate dialoguer;
extern crate reqwest;
extern crate url;

use std::fs;
use std::str;
use std::path::Path;
use std::collections::HashMap;
use dialoguer::{theme::CustomPromptCharacterTheme, Input};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    url: String,
    token: String,
    username: String,
}

fn write_config(c: &Config) -> std::io::Result<()> {
    let j = serde_json::to_string(c).unwrap();
    fs::write(".config", j)
}

fn read_config() -> Config {
    let data = fs::read(".config").expect("Could not read file");
    let s = str::from_utf8(&data).unwrap();
    let c : Config = serde_json::from_str(s).unwrap();
    c
}

fn configure() -> Config {
    let config : Config;
    if Path::new(".config").exists() {
        config = read_config();
    } else {
        let theme = CustomPromptCharacterTheme::new('>');
        let url: String = Input::with_theme(&theme)
            .with_prompt("Instance URL:")
            .interact()
            .unwrap();
        let username : String = Input::with_theme(&theme)
            .with_prompt("Username")
            .interact()
            .unwrap();

        config = Config { url: url, token: "".to_owned(), username: username };
    }

    config
}

fn login(c : &mut Config) {
    let url = Url::parse(&c.url).unwrap();
    let client_url = url.join("/api/v1/apps").unwrap();
    let resp = reqwest::Client::new()
        .post(client_url.as_str())
        .form(&[("client_name", "fedirust"), ("redirect_uris", "urn:ietf:wg:oauth:2.0:oob"), ("scopes", "read write follow")])
        .send();
    println!("{:?}", resp);
   c.token = "test".to_owned();
}

fn main() {
    let mut c = configure();
    login(&mut c);
    write_config(&c).expect("Could not write config");
    println!("{:?}", c);
}
