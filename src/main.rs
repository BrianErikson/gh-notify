#[macro_use]
extern crate log;
extern crate rusthub;
extern crate notify_rust;
extern crate rustc_serialize;
extern crate env_logger;

mod io;
mod notify;

use rusthub::notifications;
use rusthub::notifications::{NotificationResponse, Notifications};
use std::thread;
use std::time::Duration;

const TIMEOUT: i32 = 120;

fn filter_unseen(notifications: &Notifications) -> Notifications {
    let list = match io::get_saved_notifications() {
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

    Notifications { list: list }
}

fn main() {
    // INIT
    env_logger::init().unwrap();
    let token: String = io::retrieve_token(TIMEOUT).unwrap();
    debug!("Token: {}", &token);
    // TODO: Check if this token is stale before using it in the main loop

    // MAIN LOOP
    loop {
        // Get notifications
        let response: NotificationResponse = notifications::get_notifications_oauth(&token).unwrap();

        let unseen = filter_unseen(&response.notifications);
        debug!("Unseen Notifications: {} out of {}", unseen.list.len(), response.notifications.list.len());

        // Display notifications
        for notification in &unseen.list {
            notify::show_notification(&notification);
            thread::sleep(Duration::new(0, 500000000)); // Sleep for a half-second to give notify api a chance
        }

        // Cache Previous Notifications
        io::write_notifications(&response.notifications)
            .unwrap_or_else(|err| error!("While writing notifications: {}", err));

        debug!("\nPoll Interval: {}\nRate Limit: {}\nRate Limit Remaining: {}",
               response.poll_interval, response.rate_limit, response.rate_limit_remaining);
        // Sleep for requested time by GitHub
        let sleep_time: u64 = response.poll_interval as u64;
        thread::sleep(Duration::new(sleep_time, 0));
    }
}