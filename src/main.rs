extern crate dialoguer;
extern crate reqwest;
extern crate url;

use dialoguer::{theme::CustomPromptCharacterTheme, Input, PasswordInput, Select};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use std::str;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    url: String,
    token: String,
    username: String,
}

#[derive(Deserialize)]
struct ClientResponse {
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct StatusResponse {
    url: String,
}

fn write_config(c: &Config) -> std::io::Result<()> {
    let j = serde_json::to_string(c).unwrap();
    fs::write(".config", j)
}

fn read_config() -> Config {
    let data = fs::read(".config").expect("Could not read file");
    let s = str::from_utf8(&data).unwrap();
    let c: Config = serde_json::from_str(s).unwrap();
    c
}

fn create_client() -> reqwest::Client {
    let mut client = reqwest::Client::builder();
    let proxy = env::var("https_proxy").unwrap();
    if proxy != "" {
        let p = reqwest::Proxy::https(&proxy).unwrap();
        client = client.proxy(p);
    }

    client.build().unwrap()
}

fn configure() -> Config {
    let config: Config;
    if Path::new(".config").exists() {
        config = read_config();
    } else {
        let theme = CustomPromptCharacterTheme::new('>');
        let url: String = Input::with_theme(&theme)
            .with_prompt("Instance URL ")
            .interact()
            .unwrap();
        let username: String = Input::with_theme(&theme)
            .with_prompt("Username ")
            .interact()
            .unwrap();

        config = Config {
            url: url,
            token: "".to_owned(),
            username: username,
        };
    }

    config
}

fn create_login_client(c: &Config) -> ClientResponse {
    let url = Url::parse(&c.url).unwrap();
    let client_url = url.join("/api/v1/apps").unwrap();
    let resp: ClientResponse = create_client()
        .post(client_url.as_str())
        .form(&[
            ("client_name", "fedirust"),
            ("redirect_uris", "urn:ietf:wg:oauth:2.0:oob"),
            ("scopes", "read write follow"),
        ])
        .send()
        .unwrap()
        .json()
        .unwrap();
    resp
}

fn login(c: &mut Config) {
    let url = Url::parse(&c.url).unwrap();
    let client_url = url.join("/oauth/token").unwrap();

    let client = create_login_client(c);
    let theme = CustomPromptCharacterTheme::new('>');
    let password: String = PasswordInput::with_theme(&theme)
        .with_prompt("Password")
        .interact()
        .unwrap();
    let resp: LoginResponse = create_client()
        .post(client_url.as_str())
        .form(&[
            ("client_id", &client.client_id),
            ("client_secret", &client.client_secret),
            ("username", &c.username),
            ("password", &password),
            ("grant_type", &"password".to_owned()),
            ("scope", &"read write follow".to_owned()),
        ])
        .send()
        .unwrap()
        .json()
        .unwrap();
    c.token = resp.access_token.clone();
}

fn post(c: &Config) {
    let token = format!("Bearer {}", c.token);
    let url = Url::parse(&c.url).unwrap();
    let client_url = url.join("/api/v1/statuses").unwrap();
    let theme = CustomPromptCharacterTheme::new('>');

    let status: String = Input::with_theme(&theme)
        .with_prompt("Status")
        .interact()
        .unwrap();

    let visibilities = &[
        "public",
        "unlisted",
        "private"
    ];

    let visibility = Select::with_theme(&theme)
        .with_prompt("visibility")
        .default(2)
        .items(&visibilities[..])
        .interact()
        .unwrap();

    let resp: StatusResponse = create_client()
        .post(client_url.as_str())
        .header("Authorization", token)
        .form(&[
            ("visibility", visibilities[visibility]),
            ("status", &status)
        ])
        .send()
        .unwrap()
        .json()
        .unwrap();
    println!("{}", resp.url)
}

fn main() {
    let mut c = configure();
    if c.token == "" {
        login(&mut c);
    }
    write_config(&c).expect("Could not write config");
    post(&c);
}
