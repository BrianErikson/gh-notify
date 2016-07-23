use notify_rust::{Notification, NotificationHint, NotificationUrgency};
use rusthub::notifications;
use std::process::Command;
use std::thread;
use std::path::Path;

const APP_NAME: &'static str = "gh-notify";

pub fn show_notification(notification: &notifications::Notification) {
    let subject = format!("{} - {}", notification.subject.subject_type, notification.repository.name);
    let body = notification.subject.title.clone();
    let url = notification.subject.url.clone();
    let url = url.replace("api.", "").replace("repos/", "").replace("/pulls/", "/pull/");
    thread::spawn(move || {
        notify_action(
            &subject,
            &body,
            "Open in Browser",
            120,
            |action| {
                match action {
                    "default" | "clicked" => {
                        open_link(&url);
                    },
                    "__closed" => (),
                    _ => ()
                }
            }
        ).unwrap_or_else(|err| error!("While showing notification: {}", err));
    });
}

pub fn open_link(url: &str) {
    debug!("Opening browser link: {}", url);
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("xdg-open '{}'", url))
        .output()
        .expect("Failed to open web browser instance.");
}

pub fn notify_action<F>(summary: &str, body: &str, button_text: &str, timeout: i32, action: F) -> Result<(), String> where F: FnOnce(&str) {
    let icon = match Path::new("./icon.png").canonicalize() {
        Ok(path) => path.to_string_lossy().into_owned(),
        Err(_) => "clock".to_string()
    };

    let handle = try!(Notification::new()
                          .appname(APP_NAME)
                          .summary(&summary)
                          .icon(&icon)
                          .body(&body)
                          .action("default", &button_text)    // IDENTIFIER, LABEL
                          .action("clicked", &button_text) // IDENTIFIER, LABEL
                          .hint(NotificationHint::Urgency(NotificationUrgency::Normal))
                          .timeout(timeout)
                          .show().map_err(|err| err.to_string()));

    handle.wait_for_action(action);
    Ok(())
}