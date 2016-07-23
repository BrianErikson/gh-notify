use std::env;
use std::fs;

fn main() {
    let home_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir: String;
    let target = env::var("OUT_DIR").unwrap();
    match target {
        _ if target.contains("release") => out_dir = format!("{}/target/release", home_dir),
        _ if target.contains("debug") => out_dir = format!("{}/target/debug", home_dir),
        _ => panic!("Could not find target directory.")
    };

    let in_secret_path = format!("{}/secret", home_dir);
    let out_secret_path = format!("{}/secret", out_dir);

    let in_icon_path = format!("{}/icon.png", home_dir);
    let out_icon_path = format!("{}/icon.png", out_dir);

    fs::copy(&in_secret_path, &out_secret_path).unwrap();
    fs::copy(&in_icon_path, &out_icon_path).unwrap();
}