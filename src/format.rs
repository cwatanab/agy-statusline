// ─── Formatting Helpers ───────────────────────────────────────────────────

use std::env;
use std::fmt::Write;

pub fn write_human_format(buf: &mut String, num: u64) {
    if num >= 1_000_000 {
        let _ = write!(buf, "{}.{}M", num / 1_000_000, (num % 1_000_000) / 100_000);
    } else if num >= 1000 {
        let _ = write!(buf, "{}.{}K", num / 1000, (num % 1000) / 100);
    } else {
        let _ = write!(buf, "{num}");
    }
}

pub fn human_format(num: u64) -> String {
    let mut buf = String::with_capacity(16);
    write_human_format(&mut buf, num);
    buf
}

pub fn write_reset_time(buf: &mut String, sec: i64) {
    if sec <= 0 {
        return;
    }
    let days = sec / 86400;
    let rem = sec % 86400;
    let hours = rem / 3600;
    let rem = rem % 3600;
    let minutes = rem / 60;

    if days > 0 {
        if hours > 0 {
            let _ = write!(buf, "{days}d {hours}h");
        } else {
            let _ = write!(buf, "{days}d");
        }
    } else if hours > 0 {
        if minutes > 0 {
            let _ = write!(buf, "{hours}h {minutes}m");
        } else {
            let _ = write!(buf, "{hours}h");
        }
    } else if minutes > 0 {
        let _ = write!(buf, "{minutes}m");
    } else {
        buf.push_str("<1m");
    }
}

pub fn format_reset_time(sec: i64) -> String {
    let mut buf = String::with_capacity(16);
    write_reset_time(&mut buf, sec);
    buf
}

pub fn write_pct_display(buf: &mut String, pct: f64) {
    let pct_x10 = (pct * 10.0).round() as u32;
    let _ = write!(buf, "{}.{}", pct_x10 / 10, pct_x10 % 10);
}

pub fn format_pct_display(pct: f64) -> String {
    let mut buf = String::with_capacity(16);
    write_pct_display(&mut buf, pct);
    buf
}

pub fn shorten_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_default();
    
    let path_str = if !home.is_empty() && path.starts_with(&home) {
        format!("~{}", &path[home.len()..])
    } else {
        path.to_string()
    };

    if path_str.chars().count() > 25 {
        let p = std::path::Path::new(path);
        let base = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path_str);
        format!("...{base}")
    } else {
        path_str
    }
}

/// Calculate visible length of an ANSI string (stripping escape codes).
#[inline]
pub fn visible_len(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut len = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\x1b' {
            i += 1;
            while i < bytes.len() && bytes[i] != b'm' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
        } else {
            // Count UTF-8 lead bytes / ASCII bytes
            if (bytes[i] & 0xC0) != 0x80 {
                len += 1;
            }
            i += 1;
        }
    }
    len
}
