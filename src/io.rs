use std::io::{Read, Write};
use std::env;
use std::fs::{DirBuilder, OpenOptions, File};
use std::path::PathBuf;
use std::path::Path;
use rustc_serialize::{Decoder, Decodable, Encoder, Encodable};
use rustc_serialize::json;
use rusthub::oauth_web;
use rusthub::notifications::{Notifications};
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
    path.push("prev_notifications.json");
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

fn open_file(path: &PathBuf) -> Result<File, String> {
    info!("Opening {}...", path.as_path().to_string_lossy());
    match path.is_file() {
        false => {
            info!("File doesn't exist. Creating directories...");
            let mut dir = env::home_dir().expect("Impossible to get your home dir!");
            dir.push(".gh-notify");
            try!(DirBuilder::new()
                .recursive(true)
                .create(&dir).map_err(|err| err.to_string()));

            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&path).map_err(|err| err.to_string())
        }
        true => OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path).map_err(|err| err.to_string())
    }
}

fn build_new_token(timeout: i32) -> Result<String, String> {
    info!("Building new token...");
    let client_id = "f912851b98b2884f77de".to_string();
    let scope = vec!("notifications".to_string());
    let mut secret = String::new();

    debug!("Opening secret...");
    try!(File::open(&Path::new("secret"))
        .expect("Could not open secret")
        .read_to_string(&mut secret).map_err(|err| err.to_string()));

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
    let token = try!(match oauth_web::capture_authorization(client_id, secret, timeout as u64) {
        Some(token) => Ok(token),
        None => Err("Authorization capture either timed out or something went wrong...")
    });

    try!(write_config(&GhNotifyConfig {token: token.clone()}));
    Ok(token)
}

fn write_config(config: &GhNotifyConfig) -> Result<(), String> {
    info!("Writing configuration to disk...");
    let mut file = try!(OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&get_config_path())
        .map_err(|err| err.to_string()));

    try!(file.write(json::encode(config).unwrap().as_bytes()).map_err(|err| err.to_string()));
    try!(file.flush().map_err(|err| err.to_string()));
    debug!("Configuration saved.");
    Ok(())
}

pub fn retrieve_token(timeout: i32) -> Result<String, String> {
    info!("Retrieving token...");
    let mut config = try!(open_file(&get_config_path()));
    let mut json_str = String::new();
    let token: String = match config.read_to_string(&mut json_str) {
        Ok(_) => {
            debug!("Config file read sucessfully. Parsing...");
            match json::decode::<GhNotifyConfig>(&json_str) {
                Ok(config_struct) => config_struct.token,
                Err(_) => try!(build_new_token(timeout))
            }
        },
        Err(_) => try!(build_new_token(timeout))
    };
    Ok(token)
}

pub fn get_saved_notifications() -> Result<Notifications, String> {
    info!("Getting saved notifications...");
    let mut file: File = try!(open_file(&get_notifications_path()));
    let mut json_str = String::new();
    match file.read_to_string(&mut json_str) {
        Ok(_) => {
            info!("Notifications file read sucessfully. Parsing...");
            match json::decode::<Notifications>(&json_str) {
                Ok(notifications) => Ok(notifications),
                Err(err) => Err(err.to_string())
            }
        },
        Err(err) => Err(err.to_string())
    }
}

pub fn write_notifications(w_notifications: &Notifications) -> Result<(), String> {
    match json::encode(&w_notifications) {
        Ok(str) => {
            let mut write_file = try!(open_file(&get_notifications_path()));
            try!(write_file.set_len(0).map_err(|err| err.to_string())); // Truncate to 0
            try!(write_file.write(str.as_bytes()).map_err(|err| err.to_string()));
            try!(write_file.flush().map_err(|err| err.to_string()));
            info!("Notifications written to file.");
            Ok(())
        }
        Err(err) => Err(err.to_string())
    }
}