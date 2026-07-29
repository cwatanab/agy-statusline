// ─── Bar Drawing ──────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const ANSI_GRAY: &str = "\x1b[90m";
const ANSI_BRIGHT_RED: &str = "\x1b[91m";
const ANSI_BRIGHT_YELLOW: &str = "\x1b[93m";
const ANSI_BRIGHT_GREEN: &str = "\x1b[92m";

const BLOCK_CHARS: [&str; 8] = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];

/// Build a 10-segment bar with 1/8-step partial blocks.
/// `filled_pct` is 0.0..=100.0 (how much of the bar to fill).
pub fn build_bar(filled_pct: f64, color: &str, classic: bool) -> String {
    let filled_grades = ((filled_pct * 80.0) / 100.0).round() as i32;
    let filled_grades = filled_grades.clamp(0, 80) as u32;
    let filled_chars = filled_grades / 8;
    let rem_grades = filled_grades % 8;

    let block_full = format!("{color}█{RESET}");
    let block_empty = format!("{ANSI_GRAY}·{RESET}");

    let mut bar = String::with_capacity(if classic { 20 } else { 80 });
    for i in 0..10u32 {
        if i < filled_chars {
            if classic {
                bar.push('█');
            } else {
                bar.push_str(&block_full);
            }
        } else if i == filled_chars {
            let block_char = BLOCK_CHARS[rem_grades as usize];
            if classic {
                if block_char.is_empty() {
                    bar.push('·');
                } else {
                    bar.push_str(block_char);
                }
            } else if block_char.is_empty() {
                bar.push_str(&block_empty);
            } else {
                bar.push_str(&format!("{color}{block_char}{RESET}"));
            }
        } else if classic {
            bar.push('·');
        } else {
            bar.push_str(&block_empty);
        }
    }
    bar
}

pub fn quota_color(remaining_pct: f64) -> &'static str {
    match remaining_pct as u32 {
        0..=19 => ANSI_BRIGHT_RED,
        20..=49 => ANSI_BRIGHT_YELLOW,
        _ => ANSI_BRIGHT_GREEN,
    }
}

pub fn usage_color(used_pct: f64) -> &'static str {
    let pct_int = used_pct as u32;
    if pct_int >= 80 {
        ANSI_BRIGHT_RED
    } else if pct_int >= 50 {
        ANSI_BRIGHT_YELLOW
    } else {
        ANSI_BRIGHT_GREEN
    }
}

pub fn build_quota_bar(
    remaining_pct: f64,
    label: &str,
    reset_sec: i64,
    classic: bool,
    reset_icon: &str,
) -> String {
    if remaining_pct < -0.5 {
        let bar_empty: String = (0..10)
            .map(|_| if classic { "·" } else { "░" })
            .collect();
        return format!("{ANSI_GRAY}{BOLD}{label}{RESET} {ANSI_GRAY}{bar_empty} N/A{RESET}");
    }

    let text_color = quota_color(remaining_pct);
    let bar = build_bar(remaining_pct, text_color, classic);

    let reset_label = if reset_sec > 0 {
        format!(
            " {reset_icon} {}",
            crate::format::format_reset_time(reset_sec)
        )
    } else {
        String::new()
    };

    let pct_str = if remaining_pct % 1.0 == 0.0 {
        format!("{:.0}", remaining_pct)
    } else {
        format!("{:.1}", remaining_pct)
    };

    if classic {
        format!(
            "{text_color}{BOLD}{label}{RESET} {text_color}{bar}{RESET} {text_color}{pct_str}%{RESET}{reset_label}"
        )
    } else {
        format!(
            "{text_color}{BOLD}{label}{RESET} {bar} {text_color}{pct_str}%{RESET}{reset_label}"
        )
    }
}
