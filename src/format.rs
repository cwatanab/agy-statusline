// ─── Formatting Helpers ───────────────────────────────────────────────────

use std::env;
use std::fmt::Write;

pub fn write_human_format(buf: &mut String, num: u64) {
    if num >= 999_500 {
        let rounded = num + 50_000;
        let _ = write!(buf, "{}.{}M", rounded / 1_000_000, (rounded % 1_000_000) / 100_000);
    } else if num >= 1000 {
        let rounded = num + 50;
        let _ = write!(buf, "{}.{}K", rounded / 1000, (rounded % 1000) / 100);
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

/// Format a non-negative decimal/scientific USD string into `buf` without double-rounding.
/// Returns `Some(whole_dollars_after_rounding)` if formatted, or `None` if zero/invalid.
pub fn write_usd(buf: &mut String, raw: &str) -> Option<u64> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }

    // Split mantissa and optional exponent
    let (mant, exp_str) = match s.find(['e', 'E']) {
        Some(idx) => (&s[..idx], Some(&s[idx + 1..])),
        None => (s, None),
    };

    // Split integer and fractional parts of mantissa
    let (int_raw, frac_raw) = match mant.find('.') {
        Some(idx) => (&mant[..idx], &mant[idx + 1..]),
        None => (mant, ""),
    };

    // Validate mantissa: non-empty integer part, all ASCII digits, and if '.' present, non-empty frac
    if int_raw.is_empty()
        || !int_raw.bytes().all(|b| b.is_ascii_digit())
        || (mant.contains('.') && (frac_raw.is_empty() || !frac_raw.bytes().all(|b| b.is_ascii_digit())))
    {
        return None;
    }

    // Must contain at least one non-zero digit
    if !int_raw.bytes().any(|b| b != b'0') && !frac_raw.bytes().any(|b| b != b'0') {
        return None;
    }

    // Parse exponent if present
    let exp: i32 = if let Some(es) = exp_str {
        if es.is_empty() {
            return None;
        }
        let (neg, digits) = if let Some(rest) = es.strip_prefix('-') {
            (true, rest)
        } else if let Some(rest) = es.strip_prefix('+') {
            (false, rest)
        } else {
            (false, es)
        };
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let abs_exp: i32 = digits.parse().unwrap_or(i32::MAX / 2);
        if neg { -abs_exp } else { abs_exp }
    } else {
        0
    };

    // Strip leading zeros from integer part
    let int_d = int_raw.trim_start_matches('0');
    let total_digits = int_d.len() + frac_raw.len();

    // Helper to get digit at index in `int_d ++ frac_raw`
    let digit_at = |idx: usize| -> u8 {
        if idx < int_d.len() {
            int_d.as_bytes()[idx] - b'0'
        } else if idx - int_d.len() < frac_raw.len() {
            frac_raw.as_bytes()[idx - int_d.len()] - b'0'
        } else {
            0
        }
    };

    // Position of decimal point relative to start of `int_d ++ frac_raw`
    let dec_pos = int_d.len() as i64 + exp as i64;

    if dec_pos <= -4 {
        buf.push_str("<$0.0001");
        return Some(0);
    }

    // Case: all digits are in the integer part (no fractional digits)
    if dec_pos >= total_digits as i64 {
        let mut dollars: u64 = 0;
        let mut int_str_started = false;
        buf.push('$');
        for i in 0..(dec_pos as usize) {
            let d = if i < total_digits { digit_at(i) } else { 0 };
            if d != 0 || int_str_started {
                int_str_started = true;
                buf.push((b'0' + d) as char);
                dollars = dollars.saturating_mul(10).saturating_add(d as u64);
            }
        }
        if !int_str_started {
            buf.push('0');
        }
        buf.push_str(".00");
        return Some(dollars);
    }

    // Extract integer value (if dec_pos > 0) and first 5 fractional digits
    let mut dollars: u64 = 0;
    if dec_pos > 0 {
        for i in 0..(dec_pos as usize) {
            let d = digit_at(i);
            dollars = dollars.saturating_mul(10).saturating_add(d as u64);
        }
    }

    let frac_digit = |k: usize| -> u8 {
        let pos = dec_pos + k as i64;
        if pos < 0 {
            0
        } else {
            digit_at(pos as usize)
        }
    };

    let f0 = frac_digit(0);
    let f1 = frac_digit(1);
    let f2 = frac_digit(2);
    let f3 = frac_digit(3);
    let f4 = frac_digit(4);

    if dollars == 0 && f0 == 0 && f1 == 0 && f2 == 0 && f3 == 0 {
        buf.push_str("<$0.0001");
        return Some(0);
    }

    let frac5 = (f0 as u32) * 10000
        + (f1 as u32) * 1000
        + (f2 as u32) * 100
        + (f3 as u32) * 10
        + (f4 as u32);

    if dollars == 0 && frac5 < 995 {
        let u4 = (f0 as u32) * 1000
            + (f1 as u32) * 100
            + (f2 as u32) * 10
            + (f3 as u32)
            + if f4 >= 5 { 1 } else { 0 };
        let _ = write!(buf, "$0.{u4:04}");
        return Some(0);
    }

    let cents = (f0 as u64) * 10 + (f1 as u64) + if f2 >= 5 { 1 } else { 0 };
    let carry = cents / 100;
    let cents_rem = cents % 100;
    let final_dollars = dollars.saturating_add(carry);
    let _ = write!(buf, "${final_dollars}.{cents_rem:02}");
    Some(final_dollars)
}

pub fn format_usd(raw: &str) -> Option<String> {
    let mut buf = String::with_capacity(16);
    write_usd(&mut buf, raw)?;
    Some(buf)
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
            if i < bytes.len() && bytes[i] == b'[' {
                i += 1;
                while i < bytes.len() && !bytes[i].is_ascii_alphabetic() {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            } else {
                while i < bytes.len() && bytes[i] != b'm' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
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
