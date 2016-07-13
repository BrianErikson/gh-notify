use std::io::{Read, Write};
use std::error::Error;
use std::env;
use std::fs::{DirBuilder, OpenOptions, File};
use std::path::PathBuf;
use std::path::Path;
use rustc_serialize::{Decoder, Decodable, Encoder, Encodable};
use rustc_serialize::json;
use rusthub::oauth_web;
use rusthub::notifications::{Notification, Notifications};
use notify;

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

fn get_notifications_path() -> PathBuf {
    let mut path = get_notify_path();
    path.push("saved_notifications.json");
    path
}

fn get_config_path() -> PathBuf {
    let mut path = get_notify_path();
    path.push("config");
    path
}

fn get_notify_path() -> PathBuf {
    let mut path = env::home_dir().expect("Can't find home directory!");
    path.push(".gh-notify");
    path
}

fn open_file(path: &PathBuf) -> File {
    info!("Opening {}...", path.as_path().to_string_lossy());
    match path.is_file() {
        false => {
            info!("File doesn't exist. Creating directories...");
            let mut dir = env::home_dir().expect("Impossible to get your home dir!");
            dir.push(".gh-notify");
            DirBuilder::new()
                .recursive(true)
                .create(&dir).unwrap();

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

    let url = oauth_web::create_authentication_link(client_id.clone(), scope, true);
    notify::notify_action(
        "Authorize gh-notify for GitHub Access",
        "gh-notify needs authorization in order to receive notifications. Click to open an authorization window.",
        "Open Browser",
        timeout,
        |action| {
            match action {
                "default" | "clicked" => {
                    notify::open_link(&url);
                },
                "__closed" => error!("the notification was closed before authentication could occur"),
                _ => ()
            }
        }
    );

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
        .open(&get_config_path())
        .unwrap();

    file.write(json::encode(config).unwrap().as_bytes()).unwrap();
    file.flush().unwrap();
    debug!("Configuration saved.");
}

pub fn retrieve_token(timeout: i32) -> String {
    info!("Retrieving token...");
    let mut config = open_file(&get_config_path());
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

pub fn get_saved_notifications() -> Result<Notifications, String> {
    info!("Getting saved notifications...");
    let file: File = open_file(&get_notifications_path());
    let mut json_str = String::new();
    match file.read_to_string(&mut json_str) {
        Ok(_) => {
            debug!("Notifications file read sucessfully. Parsing...");
            match json::decode::<Notifications>(&json_str) {
                Ok(notifications) => Ok(notifications),
                Err(err) => Err(err.description().to_string())
            }
        },
        Err(err) => Err(err.description().to_string())
    }
}

pub fn write_notifications(w_notifications: &Notifications) -> Result<(), String> {
    let file: File = open_file(&get_notifications_path());
    let w_list: Vec<Notification> = match get_saved_notifications() {
        Ok(r_notifications) => {
            let mut list: Vec<Notification> = w_notifications.list.clone()
                .into_iter()
                .filter(|w_notif| r_notifications.list.iter().any(|r_notif| w_notif.id != r_notif.id))
                .collect();
            list.append(&mut r_notifications.list);
            list
        },
        Err(string) => {
            debug!("Error reading saved notifications. Writing to file without comparison...");
            w_notifications.list
        }
    };

    file.set_len(0); // Truncate to 0
    file.write(json::encode(&Notifications {list: w_list}).as_bytes());
    file.flush();
    info!("Notifications written to file.");
    Ok()
}