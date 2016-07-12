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
use std::thread;
use std::time::Duration;

fn main() {
    // INIT
    const TIMEOUT: i32 = 120;
    const DEFAULT_POLL_INTERVAL: u64 = 60; // In seconds
    env_logger::init().unwrap();
    let token: String = configuration::retrieve_token(TIMEOUT);
    debug!("Token: {}", &token);
    // TODO: Check if this token is stale before using it in the main loop
    //
    // MAIN LOOP
    loop {
        // GET NOTIFICATIONS
        let response: NotificationResponse = notifications::get_notifications_oauth(&token).unwrap();
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
        debug!("\nUnread Notifications: {}", notifications.list.len());
        //
        // DISPLAY NOTIFICATIONS
        for notification in &notifications.list {
            // TODO Check file to see if notification has already been displayed
            // If so, skip the notification

            notify::show_notification(&notification);
        }
        //
        // Sleep for requested time by GitHub
        let sleep_time: u64 = match response.poll_interval {
            Some(time) => {
                // Safe to assume that if rate_limit was returned, the other values were as well.
                debug!("\nPoll Interval: {}\nRate Limit: {}\nRate Limit Remaining: {}",
                       time, response.rate_limit.unwrap(), response.rate_limit_remaining.unwrap());
                time as u64
            },
            None => DEFAULT_POLL_INTERVAL
        };
        thread::sleep(Duration::new(sleep_time, 0));
        //
    }
}