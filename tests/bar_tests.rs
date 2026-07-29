use statusline::bar::{build_bar, build_quota_bar, quota_color, usage_color};
use statusline::format::{format_pct_display, format_reset_time, human_format};

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            while let Some(d) = chars.next() {
                if d == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn build_bar_classic_full() {
    let bar = build_bar(100.0, "\x1b[92m", true);
    assert_eq!(bar, "██████████");
}

#[test]
fn build_bar_classic_empty() {
    let bar = build_bar(0.0, "\x1b[92m", true);
    assert_eq!(bar, "··········");
}

#[test]
fn build_bar_classic_partial() {
    // 12.5% → 10 grades → 1 full + 2/8 partial
    let bar = build_bar(12.5, "\x1b[92m", true);
    assert_eq!(bar.chars().count(), 10);
    assert!(bar.starts_with('█'));
}

#[test]
fn build_bar_nerd_has_ansi() {
    let bar = build_bar(50.0, "\x1b[92m", false);
    assert!(bar.contains('\x1b'));
    assert_eq!(strip_ansi(&bar).chars().count(), 10);
}

#[test]
fn quota_bar_na() {
    let out = build_quota_bar(-1.0, "5H", -1, true, "⌛");
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("N/A"));
    assert!(stripped.contains("5H"));
}

#[test]
fn quota_bar_with_reset() {
    let out = build_quota_bar(79.0, "5H", 3600, true, "⌛");
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("5H"));
    assert!(stripped.contains("79%"));
    assert!(stripped.contains("1h"));
}

#[test]
fn colors() {
    assert_eq!(quota_color(10.0), "\x1b[91m");
    assert_eq!(quota_color(30.0), "\x1b[93m");
    assert_eq!(quota_color(80.0), "\x1b[92m");
    assert_eq!(usage_color(10.0), "\x1b[92m");
    assert_eq!(usage_color(60.0), "\x1b[93m");
    assert_eq!(usage_color(90.0), "\x1b[91m");
}

#[test]
fn human_format_units() {
    assert_eq!(human_format(500), "500");
    assert_eq!(human_format(1500), "1.5K");
    assert_eq!(human_format(1_500_000), "1.5M");
}

#[test]
fn reset_time_formats() {
    assert_eq!(format_reset_time(0), "");
    assert_eq!(format_reset_time(30), "<1m");
    assert_eq!(format_reset_time(90), "1m");
    assert_eq!(format_reset_time(3600), "1h");
    assert_eq!(format_reset_time(3660), "1h 1m");
    assert_eq!(format_reset_time(86400), "1d");
}

#[test]
fn pct_display() {
    assert_eq!(format_pct_display(45.0), "45.0");
    assert_eq!(format_pct_display(45.5), "45.5");
}
