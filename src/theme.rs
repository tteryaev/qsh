use std::collections::HashMap;

pub mod color;
use crate::{config::SyntaxTheme, theme::color::Color};

pub struct Theme {
    pub greeting: String,

    pub syntax: SyntaxTheme,
}

pub fn format_greeting(
    template: &str,
    username: &str,
    directory: &str,
    widgets: &HashMap<String, String>,
    colors: &HashMap<String, String>,
) -> String {
    let mut output = String::new();
    let mut remaining = template;
    let time = local_time();

    while let Some(open) = remaining.find('{') {
        output.push_str(&remaining[..open]);
        let after_open = &remaining[open + 1..];
        let Some(close) = after_open.find('}') else {
            output.push_str(&remaining[open..]);
            return output;
        };

        let name = &after_open[..close];
        let value = match name {
            "username" => Some(username),
            "current_directory" => Some(directory),
            "time" => Some(time.as_str()),
            _ => widgets.get(name).map(String::as_str),
        };

        if let Some(value) = value {
            if let Some(hex) = colors.get(name) {
                let color = Color::from_hex(hex).unwrap_or(Color::Default);
                output.push_str(&color.ansi());
                output.push_str(value);
                output.push_str(&Color::Default.ansi());
            } else {
                output.push_str(value);
            }
        } else {
            output.push('{');
            output.push_str(name);
            output.push('}');
        }

        remaining = &after_open[close + 1..];
    }

    output.push_str(remaining);
    output
}

fn local_time() -> String {
    let mut timestamp: libc::time_t = 0;
    let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();

    // libc converts the current Unix timestamp using the system's local
    // timezone, so the widget follows the user's TZ setting.
    unsafe {
        libc::time(&mut timestamp);
        if libc::localtime_r(&timestamp, local.as_mut_ptr()).is_null() {
            return String::new();
        }

        let local = local.assume_init();
        format!(
            "{:02}:{:02}:{:02}",
            local.tm_hour, local.tm_min, local.tm_sec
        )
    }
}
