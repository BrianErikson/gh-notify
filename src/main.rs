#[macro_use] extern crate log;
extern crate rusthub;
extern crate notify_rust;
extern crate rustc_serialize;
extern crate env_logger;
mod configuration;

fn main() {
    env_logger::init().unwrap();
    let token: String = configuration::retrieve_token();
    debug!("Token: {}", token);
}