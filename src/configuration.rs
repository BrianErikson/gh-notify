use std::io::{Read, Write};
use std::env;
use std::fs::{DirBuilder, OpenOptions, File};
use std::path::PathBuf;
use std::path::Path;
use std::process::Command;
use rustc_serialize::{Decoder, Decodable, Encoder, Encodable};
use rustc_serialize::json;
use rusthub::oauth_web;
use notify_rust::{Notification, NotificationHint, NotificationUrgency};

#[derive(Debug)]
struct GhNotifyConfig {
    token: String
}

impl Decodable for GhNotifyConfig {
    fn decode<D: Decoder>(d: &mut D) -> Result<GhNotifyConfig, D::Error> {
        d.read_struct("root", 1, |d| {
            Ok(GhNotifyConfig{
                token: try!(d.read_struct_field("token", 0, |d| Decodable::decode(d)))
            })
        })
    }
}

impl Encodable for GhNotifyConfig {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("root",  1, |s| {
            try!(s.emit_struct_field("token", 0, |s| {
                s.emit_str(&self.token)
            }));
            Ok(())
        })
    }
}

fn retrieve_config_path() -> PathBuf {
    debug!("Retrieving configuration path...");
    let mut path = env::home_dir().expect("Can't find home directory!");
    path.push(".gh-notify");
    path.push("config");
    path
}

fn open_config() -> File {
    let path = retrieve_config_path();
    info!("Opening configuration file...");
    match path.is_file() {
        false => {
            info!("Configuration file doesn't exist. Creating directories...");
            let mut path = env::home_dir().expect("Impossible to get your home dir!");
            path.push(".gh-notify");
            DirBuilder::new()
                .recursive(true)
                .create(&path).unwrap();

            path.push("config");

            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&path)
                .unwrap()
        }
        true => OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .unwrap()
    }
}

fn request_browser_open(url: String, timeout: i32) {
    Notification::new()
        .appname("gh-notify")
        .summary("Authorize gh-notify for GitHub Access")
        .body("gh-notify needs authorization in order to receive notifications. Click to open an authorization window.")
        .action("default", "Open Browser")    // IDENTIFIER, LABEL
        .action("clicked", "Open Browser") // IDENTIFIER, LABEL
        .hint(NotificationHint::Urgency(NotificationUrgency::Normal))
        .timeout(timeout)
        .show()
        .unwrap()
        .wait_for_action({|action|
            match action {
                "default" | "clicked" => {
                    debug!("Opening browser to authentication link.");
                    let _ = Command::new("sh")
                        .arg("-c")
                        .arg(format!("xdg-open '{}'", url))
                        .output()
                        .expect("Failed to open web browser instance.");
                },
                "__closed" => error!("the notification was closed before authentication could occur"),
                _ => ()
            }
        });
}

fn build_new_token(timeout: i32) -> String {
    info!("Building new token...");
    let client_id = "f912851b98b2884f77de".to_string();
    let scope = vec!("notifications".to_string());
    let mut secret = String::new();

    debug!("Opening secret...");
    File::open(&Path::new("secret"))
        .expect("Could not open secret")
        .read_to_string(&mut secret)
        .unwrap();

    request_browser_open(oauth_web::create_authentication_link(client_id.clone(), scope, true), timeout);

    debug!("Capturing authorization from GitHub redirect. Blocking...");
    let token = oauth_web::capture_authorization(client_id, secret, timeout as u64)
        .expect("ERROR: Something went wrong when requesting token.");

    write_config(&GhNotifyConfig {token: token.clone()});
    token
}

fn write_config(config: &GhNotifyConfig) {
    info!("Writing configuration to disk...");
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&retrieve_config_path())
        .unwrap();

    file.write(json::encode(config).unwrap().as_bytes()).unwrap();
    file.flush().unwrap();
    debug!("Configuration saved.");
}

pub fn retrieve_token(timeout: i32) -> String {
    info!("Retrieving token...");
    let mut config = open_config();
    let mut json_str = String::new();
    match config.read_to_string(&mut json_str) {
        Ok(_) => {
            debug!("Config file read sucessfully. Parsing...");
            let token: String = match json::decode::<GhNotifyConfig>(&json_str) {
                Ok(config_struct) => config_struct.token,
                Err(_) => build_new_token(timeout)
            };
            token
        },
        Err(_) => build_new_token(timeout)
    }
}