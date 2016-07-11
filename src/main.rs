#[macro_use] extern crate log;
extern crate rusthub;
extern crate notify_rust;
extern crate rustc_serialize;
extern crate env_logger;
mod configuration;
//use notify_rust::{Notification, NotificationHint, NotificationUrgency};
use rusthub::notifications;
use rusthub::notifications::NotificationResponse;

fn main() {
    const TIMEOUT: i32 = 120;
    env_logger::init().unwrap();
    let token: String = configuration::retrieve_token(TIMEOUT);
    debug!("Token: {}", token);
    //
    let response: NotificationResponse = notifications::get_notifications_oauth(token).unwrap();
    debug!("Notification: {:#?}", response);
}