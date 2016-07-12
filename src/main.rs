#[macro_use] extern crate log;
extern crate rusthub;
extern crate notify_rust;
extern crate rustc_serialize;
extern crate env_logger;
mod configuration;
mod notify;
//use notify_rust::{Notification, NotificationHint, NotificationUrgency};
use rusthub::notifications;
use rusthub::notifications::{NotificationResponse, Notifications};

fn main() {
    const TIMEOUT: i32 = 120;
    env_logger::init().unwrap();
    let token: String = configuration::retrieve_token(TIMEOUT);
    debug!("Token: {}", token);
    //
    let response: NotificationResponse = notifications::get_notifications_oauth(token).unwrap();
    let notifications: Notifications = match response.notifications {
        Some(result) => {
            match result {
                Ok(notifications) => notifications,
                Err(err) => {
                    error!("Error retrieving notifications: {}", err);
                    Notifications {list: vec!()}
                }
            }
        },
        None => Notifications {list: vec!()}
    };
    debug!("Notification: {:#?}", notifications);
    //
    for notification in &notifications.list {
        // TODO Check file to see if notification has already been displayed
        // If so, skip the notification

        notify::show_notification(&notification);
    }
}