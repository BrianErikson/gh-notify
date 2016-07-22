#[macro_use] extern crate log;
extern crate rusthub;
extern crate notify_rust;
extern crate rustc_serialize;
extern crate env_logger;
mod io;
mod notify;
//use notify_rust::{Notification, NotificationHint, NotificationUrgency};
use rusthub::notifications;
use rusthub::notifications::{NotificationResponse, Notifications, Notification};
use std::thread;
use std::time::Duration;

fn main() {
    // INIT
    const TIMEOUT: i32 = 120;
    const DEFAULT_POLL_INTERVAL: u64 = 60; // In seconds
    env_logger::init().unwrap();
    let token: String = io::retrieve_token(TIMEOUT).unwrap();
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
                        error!("While retrieving notifications: {}", err);
                        Notifications {list: vec!()}
                    }
                }
            },
            None => Notifications {list: vec!()}
        };
        //
        // Filter unread notifications that haven't been updated since last check
        let display_notifications: Vec<Notification> = match io::get_saved_notifications() {
            Ok(saves) => notifications.list
                .clone()
                .into_iter()
                .filter(|n| !saves.list.iter().any(|sn| n.updated_at == sn.updated_at && n.subject.title == sn.subject.title))
                .collect(),
            Err(err) => {
                error!("{}", err);
                notifications.list.clone()
            }
        };
        debug!("\nUnread Notifications: {} out of {}", display_notifications.len(), notifications.list.len());
        //
        // Display notifications
        for notification in display_notifications {
            notify::show_notification(&notification);
            thread::sleep(Duration::new(0, 500000000)); // Sleep for a half-second to give notify api a chance
        }
        //
        // Cache Previous Notifications
        io::write_notifications(&notifications)
            .unwrap_or_else(|err| error!("While writing notifications: {}", err));
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