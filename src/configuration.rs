use std::io::{Read, Write};
use std::env;
use std::fs::{DirBuilder, OpenOptions, File};
use std::path::PathBuf;
use std::path::Path;
use rustc_serialize::{Decoder, Decodable, Encoder, Encodable};
use rustc_serialize::json;
use rusthub::oauth_web;

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

fn build_new_token() -> String {
    info!("Building new token...");
    let mut secret = String::new();
    debug!("Opening secret...");
    File::open(&Path::new("secret")).unwrap().read_to_string(&mut secret).unwrap();
    debug!("Requesting token...");
    let token = oauth_web::create_authorization(
        "f912851b98b2884f77de".to_string(),
        secret,
        vec!("notifications".to_string()),
        true,
        120
    ).expect("ERROR: Something went wrong when requesting token.");

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

pub fn retrieve_token() -> String {
    info!("Retrieving token...");
    let mut config = open_config();
    let mut json_str = String::new();
    match config.read_to_string(&mut json_str) {
        Ok(_) => {
            debug!("Config file read sucessfully. Parsing...");
            let token: String = match json::decode::<GhNotifyConfig>(&json_str) {
                Ok(config_struct) => config_struct.token,
                Err(_) => build_new_token()
            };
            token
        },
        Err(_) => build_new_token()
    }
}