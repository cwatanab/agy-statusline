// ─── Formatting Helpers ───────────────────────────────────────────────────

pub fn human_format(num: u64) -> String {
    if num >= 1_000_000 {
        format!("{}.{}M", num / 1_000_000, (num % 1_000_000) / 100_000)
    } else if num >= 1000 {
        format!("{}.{}K", num / 1000, (num % 1000) / 100)
    } else {
        num.to_string()
    }
}

pub fn format_reset_time(sec: i64) -> String {
    if sec <= 0 {
        return String::new();
    }
    let days = sec / 86400;
    let rem = sec % 86400;
    let hours = rem / 3600;
    let rem = rem % 3600;
    let minutes = rem / 60;

    if days > 0 {
        if hours > 0 {
            format!("{days}d {hours}h")
        } else {
            format!("{days}d")
        }
    } else if hours > 0 {
        if minutes > 0 {
            format!("{hours}h {minutes}m")
        } else {
            format!("{hours}h")
        }
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        "<1m".to_string()
    }
}

pub fn format_pct_display(pct: f64) -> String {
    let pct_x10 = (pct * 10.0).round() as u32;
    format!("{}.{}", pct_x10 / 10, pct_x10 % 10)
}
