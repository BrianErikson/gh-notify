#[macro_use]
extern crate log;
extern crate rusthub;
extern crate notify_rust;
extern crate rustc_serialize;
extern crate env_logger;
extern crate gtk;

mod io;
mod notify;

use std::thread;
use std::time::Duration;
use std::fs::{File};
use std::path::Path;
use rusthub::notifications;
use rusthub::notifications::{NotificationResponse, Notifications};
use gtk::{StatusIcon, Builder, Menu};

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

    // Initialize popup menu
    let builder = Builder::new_from_file(&Path::new("popup.glade"));
    let menu: Menu = builder.get_object::<Menu>("menu").unwrap();
    //
    // Publish status icon for system tray
    gtk::init().unwrap();
    let status_icon: StatusIcon;
    match Path::new("./icon.png").canonicalize() {
        Ok(path) => {
            let icon_path = path.to_string_lossy().into_owned();
            debug!("Using real icon in system tray: {}", icon_path);
            status_icon = StatusIcon::new_from_file(icon_path);
        },
        Err(_) => {
            debug!("Using fallback icon in system tray");
            status_icon = StatusIcon::new_from_icon_name("clock");
        }
    };
    status_icon.set_name("gh-notify");
    status_icon.set_title("gh-notify");
    status_icon.set_tooltip_text("gh-notify");
    status_icon.connect_popup_menu(|_, _, _| {
        //menu.position_menu(0, 0, &status_icon);
        menu.popup(None, None, false, 0, 0);
    });
    //

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