use std::env;
use std::fs;

fn main() {
    let home_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir: String;
    match env::var("CARGO_FEATURE_RELEASE") {
        Ok(_) => out_dir = format!("{}/target/release", home_dir),
        Err(_) => out_dir = format!("{}/target/debug", home_dir)
    };

    let in_icon_path = format!("{}/icon.png", home_dir);
    let out_icon_path = format!("{}/icon.png", out_dir);
    fs::copy(&in_icon_path, &out_icon_path).unwrap();
}