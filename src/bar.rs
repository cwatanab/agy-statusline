// ─── Bar & Badge Rendering ──────────────────────────────────────────────────

use std::fmt::Write;
use crate::format::write_reset_time;
use crate::icons::{to_ansi_color, BOLD, RESET};

pub fn usage_color(used_pct: f64) -> &'static str {
    let pct_int = used_pct as u32;
    if pct_int >= 80 {
        "\x1b[91m"
    } else if pct_int >= 50 {
        "\x1b[93m"
    } else {
        "\x1b[92m"
    }
}

/// Render a progress bar (Context or Quota bar) directly into a String buffer.
pub fn append_bar(buf: &mut String, filled_pct: f64, bar_len: usize, bar_color_num: &str, classic: bool) {
    let val_int = (filled_pct.max(0.0) as usize).min(100);
    let filled = val_int * bar_len / 100;
    let remainder = (val_int * bar_len) % 100;

    for i in 0..bar_len {
        if i < filled {
            if classic {
                buf.push('█');
            } else {
                let _ = write!(buf, "\x1b[38;5;{bar_color_num}m█{RESET}");
            }
        } else if i == filled {
            if classic {
                if remainder >= 75 {
                    buf.push('▓');
                } else if remainder >= 50 {
                    buf.push('▒');
                } else if remainder >= 25 {
                    buf.push('░');
                } else {
                    buf.push('·');
                }
            } else if remainder >= 75 {
                let _ = write!(buf, "\x1b[38;5;{bar_color_num}m▓{RESET}\x1b[90m");
            } else if remainder >= 50 {
                let _ = write!(buf, "\x1b[38;5;{bar_color_num}m▒{RESET}\x1b[90m");
            } else if remainder >= 25 {
                let _ = write!(buf, "\x1b[38;5;{bar_color_num}m░{RESET}\x1b[90m");
            } else {
                buf.push_str("\x1b[38;5;236m░\x1b[0m");
            }
        } else if classic {
            buf.push('·');
        } else {
            buf.push_str("\x1b[38;5;236m░\x1b[0m");
        }
    }
}

pub fn build_bar(filled_pct: f64, bar_len: usize, bar_color_num: &str, classic: bool) -> String {
    let mut bar = String::with_capacity(64);
    append_bar(&mut bar, filled_pct, bar_len, bar_color_num, classic);
    bar
}

/// Render a rounded pill badge into a buffer.
pub fn append_badge(buf: &mut String, icon: &str, val: &str, icon_color: &str, classic: bool) {
    if classic {
        let ansi_c = to_ansi_color(icon_color);
        let _ = write!(buf, "{ansi_c}{icon} \x1b[97m{BOLD}{val}{RESET}");
    } else {
        let bg_color = "236";
        let _ = write!(
            buf,
            "\x1b[38;5;{bg_color}m\x1b[48;5;{bg_color}m\x1b[38;5;{icon_color}m{icon} \x1b[38;5;255m\x1b[1m{val}\x1b[0m\x1b[38;5;{bg_color}m\x1b[0m"
        );
    }
}

pub fn make_badge(icon: &str, val: &str, icon_color: &str, classic: bool) -> String {
    let mut buf = String::with_capacity(64);
    append_badge(&mut buf, icon, val, icon_color, classic);
    buf
}

/// Render quota bar (5H or 7D) into a buffer.
pub fn append_quota_bar(
    buf: &mut String,
    val: f64,
    label: &str,
    quota_bar_len: usize,
    bar_color_num: &str,
    reset_sec: i64,
    classic: bool,
    reset_icon: &str,
) {
    let separator = if classic {
        "\x1b[90m · \x1b[0m"
    } else {
        " "
    };

    if val < 0.0 {
        buf.push_str(separator);
        let _ = write!(buf, "\x1b[97m{BOLD}{label}{RESET} \x1b[90m");
        for _ in 0..quota_bar_len {
            if classic { buf.push('·'); } else { buf.push('░'); }
        }
        buf.push_str(" N/A\x1b[0m");
        return;
    }

    let val_int = val as usize;
    let text_color = if val_int < 20 {
        "197"
    } else if val_int < 50 {
        "214"
    } else {
        "76"
    };

    if classic {
        let text_ansi = to_ansi_color(text_color);
        let bar_ansi = to_ansi_color(bar_color_num);
        buf.push_str(separator);
        let _ = write!(buf, "\x1b[97m{BOLD}{label}{RESET} {bar_ansi}");
        append_bar(buf, val, quota_bar_len, bar_color_num, classic);
        let _ = write!(buf, "{RESET} {text_ansi}{val_int}%{RESET}");
        if reset_sec > 0 {
            let start_len = buf.len();
            let _ = write!(buf, " {reset_icon}");
            let icon_len = buf.len();
            write_reset_time(buf, reset_sec);
            if buf.len() == icon_len {
                buf.truncate(start_len);
            }
        }
    } else {
        let label_bg = "236";
        let bar_bg = "235";
        buf.push_str(separator);
        let _ = write!(
            buf,
            "\x1b[38;5;{label_bg}m\x1b[48;5;{label_bg}m\x1b[38;5;{text_color}m{label}\x1b[48;5;{bar_bg}m \x1b[0m"
        );
        append_bar(buf, val, quota_bar_len, bar_color_num, classic);
        let _ = write!(
            buf,
            "\x1b[48;5;{label_bg}m \x1b[38;5;{text_color}m\x1b[1m{val_int}%\x1b[0m\x1b[38;5;{label_bg}m\x1b[0m"
        );
        if reset_sec > 0 {
            let start_len = buf.len();
            let _ = write!(buf, " {reset_icon}");
            let icon_len = buf.len();
            write_reset_time(buf, reset_sec);
            if buf.len() == icon_len {
                buf.truncate(start_len);
            }
        }
    }
}

pub fn make_quota_bar(
    val: f64,
    label: &str,
    quota_bar_len: usize,
    bar_color_num: &str,
    reset_sec: i64,
    classic: bool,
    reset_icon: &str,
) -> String {
    let mut buf = String::with_capacity(128);
    append_quota_bar(
        &mut buf,
        val,
        label,
        quota_bar_len,
        bar_color_num,
        reset_sec,
        classic,
        reset_icon,
    );
    buf
}
