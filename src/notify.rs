use notify_rust::{Notification, NotificationHint, NotificationUrgency};
use rusthub::notifications;
use std::process::Command;

const APP_NAME: &'static str = "gh-notify";

pub fn show_notification(notification: &notifications::Notification) {
    notify_action(
        &notification.subject.title,
        &notification.reason,
        "Open in Browser",
        120,
        {|action|
            match action {
                "default" | "clicked" => {
                    open_link(&notification.url);
                },
                "__closed" => (),
                _ => ()
            }
        }
    );
}

pub fn open_link(url: &str) {
    debug!("Opening browser link: {}", url);
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("xdg-open '{}'", url))
        .output()
        .expect("Failed to open web browser instance.");
}

pub fn notify_action<F>(summary: &str, body: &str, button_text: &str, timeout: i32, action: F) where F:FnOnce(&str) {
    Notification::new()
        .appname(APP_NAME)
        .summary(summary)
        .body(body)
        .action("default", button_text)    // IDENTIFIER, LABEL
        .action("clicked", button_text) // IDENTIFIER, LABEL
        .hint(NotificationHint::Urgency(NotificationUrgency::Normal))
        .timeout(timeout)
        .show()
        .unwrap()
        .wait_for_action(action);
}