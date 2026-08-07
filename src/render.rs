// ─── Quota Helpers ─────────────────────────────────────────────────────────

use std::fmt::Write;
use crate::bar::{append_bar, append_badge, append_quota_bar, usage_color};
use crate::format::{write_pct_display, write_human_format, shorten_path, visible_len};
use crate::icons::{select_icons, BOLD, RESET};
use crate::parse::ParsedInput;
use crate::sys::{get_host_info, get_power_info, get_sys_info, git_info};

struct QuotaInfo {
    five_hour_pct: f64,
    weekly_pct: f64,
    five_hour_reset: i64,
    weekly_reset: i64,
}

#[inline]
fn is_3p_model(model_id: &str) -> bool {
    let model_lower = model_id.to_lowercase();
    model_lower.contains("claude")
        || model_lower.contains("gpt")
        || model_lower.contains("anthropic")
        || model_lower.contains("openai")
        || model_lower.contains("o1")
        || model_lower.contains("o3")
        || model_lower.contains("3p")
}

#[inline]
fn resolve_quota(input: &ParsedInput) -> QuotaInfo {
    let is_3p = is_3p_model(input.model_id);
    let (five_hour_pct, weekly_pct, five_hour_reset, weekly_reset) = if is_3p {
        if input.third_party_5h_pct >= 0.0 || input.third_party_weekly_pct >= 0.0 {
            (
                input.third_party_5h_pct,
                input.third_party_weekly_pct,
                input.third_party_5h_reset,
                input.third_party_weekly_reset,
            )
        } else {
            (
                input.gemini_5h_pct,
                input.gemini_weekly_pct,
                input.gemini_5h_reset,
                input.gemini_weekly_reset,
            )
        }
    } else if input.gemini_5h_pct >= 0.0 || input.gemini_weekly_pct >= 0.0 {
        (
            input.gemini_5h_pct,
            input.gemini_weekly_pct,
            input.gemini_5h_reset,
            input.gemini_weekly_reset,
        )
    } else {
        (
            input.third_party_5h_pct,
            input.third_party_weekly_pct,
            input.third_party_5h_reset,
            input.third_party_weekly_reset,
        )
    };

    QuotaInfo {
        five_hour_pct,
        weekly_pct,
        five_hour_reset,
        weekly_reset,
    }
}

// ─── Statusline Renderer ───────────────────────────────────────────────────

#[inline]
fn append_segment(
    buf: &mut String,
    bg_color: &str,
    fg_text: &str,
    text: &str,
    next_bg: Option<&str>,
    classic: bool,
) {
    if classic {
        let _ = write!(buf, "{bg_color}{text}{RESET} ");
        return;
    }

    let fg_sep = bg_color.replace("48;", "38;");
    if let Some(next) = next_bg {
        let _ = write!(buf, "{bg_color}{fg_text} {text} {next}{fg_sep}\u{E0B0}{RESET}");
    } else {
        let _ = write!(buf, "{bg_color}{fg_text} {text} \x1b[0m{fg_sep}\u{E0B0}{RESET}");
    }
}

pub fn render_line(input: &ParsedInput, classic: bool, override_cols: Option<usize>) -> String {
    let icons = select_icons(classic);
    let cols = override_cols.unwrap_or(input.terminal_width).max(40);

    let (bar_len, quota_bar_len) = if cols >= 180 {
        (20, 15)
    } else {
        (10, 8)
    };

    // Preallocate single buffer for entire output
    let mut out = String::with_capacity(1024);

    // ─── 1. Powerline LINE1 Assembly ─────────────────────────────────────────
    let mut seg_bufs: [String; 8] = [
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
    ];
    let mut seg_bgs: [&'static str; 8] = [""; 8];
    let mut seg_fgs: [&'static str; 8] = [""; 8];
    let mut seg_count = 0;

    // 1.1 State
    match input.agent_state {
        "idle" => {
            let _ = write!(seg_bufs[seg_count], "{} READY", icons.state_ready);
            seg_bgs[seg_count] = icons.theme.bg_ready;
            seg_fgs[seg_count] = icons.theme.fg_ready_text;
            seg_count += 1;
        }
        "thinking" => {
            let _ = write!(seg_bufs[seg_count], "{} THINKING", icons.state_thinking);
            seg_bgs[seg_count] = icons.theme.bg_thinking;
            seg_fgs[seg_count] = icons.theme.fg_thinking_text;
            seg_count += 1;
        }
        "working" => {
            let _ = write!(seg_bufs[seg_count], "{} WORKING", icons.state_working);
            seg_bgs[seg_count] = icons.theme.bg_working;
            seg_fgs[seg_count] = icons.theme.fg_working_text;
            seg_count += 1;
        }
        "tool_use" => {
            let _ = write!(seg_bufs[seg_count], "{} TOOL", icons.state_tool);
            seg_bgs[seg_count] = icons.theme.bg_tool;
            seg_fgs[seg_count] = icons.theme.fg_tool_text;
            seg_count += 1;
        }
        other => {
            let _ = write!(seg_bufs[seg_count], "{} {}", icons.state_unknown, other.to_uppercase());
            seg_bgs[seg_count] = icons.theme.bg_unknown;
            seg_fgs[seg_count] = icons.theme.fg_unknown_text;
            seg_count += 1;
        }
    }

    // 1.2 VCS Branch
    let (vcs_branch, vcs_dirty) = git_info(
        input.working_dir,
        input.vcs_branch,
        input.vcs_dirty,
    );
    if !vcs_branch.is_empty() {
        if vcs_dirty {
            let _ = write!(seg_bufs[seg_count], "{} {}*", icons.vcs, vcs_branch);
        } else {
            let _ = write!(seg_bufs[seg_count], "{} {}", icons.vcs, vcs_branch);
        }
        seg_bgs[seg_count] = if vcs_dirty { icons.theme.bg_git_dirty } else { icons.theme.bg_git_clean };
        seg_fgs[seg_count] = if vcs_dirty { icons.theme.fg_git_dirty_text } else { icons.theme.fg_git_clean_text };
        seg_count += 1;
    }

    // 1.3 Model
    let model_disp = if !input.model_display_name.is_empty() {
        input.model_display_name
    } else {
        input.model_id
    };
    if !model_disp.is_empty() {
        if classic || icons.model.is_empty() {
            seg_bufs[seg_count].push_str(model_disp);
        } else {
            let _ = write!(seg_bufs[seg_count], "{} {}", icons.model, model_disp);
        }
        seg_bgs[seg_count] = icons.theme.bg_model;
        seg_fgs[seg_count] = icons.theme.fg_model_text;
        seg_count += 1;
    }

    // 1.4 Directory
    let cwd_short = shorten_path(input.working_dir);
    if !cwd_short.is_empty() {
        if classic || icons.dir.is_empty() {
            seg_bufs[seg_count] = cwd_short;
        } else {
            let _ = write!(seg_bufs[seg_count], "{} {}", icons.dir, cwd_short);
        }
        seg_bgs[seg_count] = icons.theme.bg_dir;
        seg_fgs[seg_count] = icons.theme.fg_dir_text;
        seg_count += 1;
    }

    // 1.5 User Plan & Account
    if (!input.plan_tier.is_empty() || !input.email.is_empty()) && cols >= 130 {
        if !input.plan_tier.is_empty() && !input.email.is_empty() {
            let _ = write!(seg_bufs[seg_count], "{} ({})", input.plan_tier, input.email);
        } else if !input.plan_tier.is_empty() {
            seg_bufs[seg_count].push_str(input.plan_tier);
        } else {
            seg_bufs[seg_count].push_str(input.email);
        }
        if !classic {
            seg_bufs[seg_count].insert_str(0, "👤 ");
        }
        seg_bgs[seg_count] = icons.theme.bg_meta;
        seg_fgs[seg_count] = icons.theme.fg_meta_text;
        seg_count += 1;
    }

    // 1.6 Conversation ID
    if !input.conversation_id.is_empty() && cols >= 80 {
        let conv_prefix = if input.conversation_id.len() > 8 {
            &input.conversation_id[..8]
        } else {
            input.conversation_id
        };
        if classic || icons.conv.is_empty() {
            seg_bufs[seg_count].push_str(conv_prefix);
        } else {
            let _ = write!(seg_bufs[seg_count], "{} {}", icons.conv, conv_prefix);
        }
        seg_bgs[seg_count] = icons.theme.bg_meta;
        seg_fgs[seg_count] = icons.theme.fg_meta_text;
        seg_count += 1;
    }

    // 1.7 Host Info
    if cols >= 110 {
        if let Some(host_info) = get_host_info() {
            if classic {
                seg_bufs[seg_count] = host_info;
            } else {
                let _ = write!(seg_bufs[seg_count], "\u{F048B} {host_info}");
            }
            seg_bgs[seg_count] = icons.theme.bg_meta;
            seg_fgs[seg_count] = icons.theme.fg_meta_text;
            seg_count += 1;
        }
    }

    // 1.8 Version
    if !input.version.is_empty() && cols >= 120 {
        let _ = write!(seg_bufs[seg_count], "v{}", input.version);
        seg_bgs[seg_count] = icons.theme.bg_meta;
        seg_fgs[seg_count] = icons.theme.fg_meta_text;
        seg_count += 1;
    }

    // Write line 1 prefix if framed
    if !classic {
        out.push_str("\x1b[90m╭─\x1b[0m");
    }

    // Render LINE1 segments
    for i in 0..seg_count {
        let next_bg = if i + 1 < seg_count {
            Some(seg_bgs[i + 1])
        } else {
            None
        };
        append_segment(
            &mut out,
            seg_bgs[i],
            seg_fgs[i],
            &seg_bufs[i],
            next_bg,
            classic,
        );
    }
    out.push('\n');

    // ─── 2. Telemetry Badges Stream Engine ─────────────────────────────────────
    let mut badge_bufs: [String; 12] = [
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
    ];
    let mut badge_count = 0;

    // 2.1 Context Usage Bar
    let pct_int = input.used_percentage as usize;
    let fill_color = usage_color(input.used_percentage);

    if classic {
        let _ = write!(badge_bufs[badge_count], "\x1b[90mctx {fill_color}");
        append_bar(&mut badge_bufs[badge_count], input.used_percentage, bar_len, "76", true);
        let _ = write!(badge_bufs[badge_count], " \x1b[97m{BOLD}");
        write_pct_display(&mut badge_bufs[badge_count], input.used_percentage);
        badge_bufs[badge_count].push_str("%\x1b[0m");
    } else {
        let bar_c = if pct_int >= 90 { "197" } else { "214" };
        let label_bg = "236";
        let bar_bg = "235";
        let icon_cb = icons.context_bar;
        let _ = write!(
            badge_bufs[badge_count],
            "\x1b[38;5;{label_bg}m\x1b[48;5;{label_bg}m\x1b[38;5;220m{icon_cb} ctx\x1b[48;5;{bar_bg}m "
        );
        append_bar(&mut badge_bufs[badge_count], input.used_percentage, bar_len, bar_c, false);
        let _ = write!(badge_bufs[badge_count], "\x1b[48;5;{label_bg}m \x1b[38;5;220m\x1b[1m");
        write_pct_display(&mut badge_bufs[badge_count], input.used_percentage);
        let _ = write!(badge_bufs[badge_count], "%\x1b[0m\x1b[38;5;{label_bg}m\x1b[0m");
    }
    badge_count += 1;

    // 2.2 Token Details
    let context_used = input.total_input_tokens + input.total_output_tokens;
    if context_used > 0 {
        if classic {
            badge_bufs[badge_count].push_str("(total: ");
            write_human_format(&mut badge_bufs[badge_count], input.total_input_tokens);
            badge_bufs[badge_count].push('/');
            write_human_format(&mut badge_bufs[badge_count], input.total_output_tokens);
            if input.turn_input_tokens > 0 || input.turn_output_tokens > 0 {
                badge_bufs[badge_count].push_str(" | turn: +");
                write_human_format(&mut badge_bufs[badge_count], input.turn_input_tokens);
                badge_bufs[badge_count].push('/');
                write_human_format(&mut badge_bufs[badge_count], input.turn_output_tokens);
            }
            badge_bufs[badge_count].push(')');
        } else {
            let mut tok_val = String::with_capacity(32);
            tok_val.push_str("total: ");
            write_human_format(&mut tok_val, input.total_input_tokens);
            tok_val.push('/');
            write_human_format(&mut tok_val, input.total_output_tokens);
            if input.turn_input_tokens > 0 || input.turn_output_tokens > 0 {
                tok_val.push_str(" | turn: +");
                write_human_format(&mut tok_val, input.turn_input_tokens);
                tok_val.push('/');
                write_human_format(&mut tok_val, input.turn_output_tokens);
            }
            append_badge(&mut badge_bufs[badge_count], icons.token_sum, &tok_val, "220", false);
        }
        badge_count += 1;
    }

    // 2.3 System Resources
    let sys_info = get_sys_info();
    if let (Some(mem_pct), Some(load_1m)) = (sys_info.mem_pct, sys_info.load_1m) {
        let sys_color = if mem_pct >= 80 {
            "197"
        } else if mem_pct >= 65 {
            "214"
        } else {
            "76"
        };
        let mut val_str = String::with_capacity(24);
        let _ = write!(val_str, "RAM:{mem_pct}% | ld:{load_1m}");
        append_badge(&mut badge_bufs[badge_count], icons.sys, &val_str, sys_color, classic);
        badge_count += 1;
    }

    // 2.4 Artifacts
    let mut art_str = String::with_capacity(8);
    let _ = write!(art_str, "{}", input.artifact_count);
    append_badge(&mut badge_bufs[badge_count], icons.artifacts, &art_str, "75", classic);
    badge_count += 1;

    // 2.5 Subagents
    let mut sub_str = String::with_capacity(8);
    let _ = write!(sub_str, "{}", input.subagent_count);
    append_badge(&mut badge_bufs[badge_count], icons.subagents, &sub_str, "37", classic);
    badge_count += 1;

    // 2.6 Tasks
    let mut task_str = String::with_capacity(8);
    let _ = write!(task_str, "{}", input.task_count);
    append_badge(&mut badge_bufs[badge_count], icons.tasks, &task_str, "135", classic);
    badge_count += 1;

    // 2.7 Sandbox
    let (sb_label, sb_val, sb_color) = if input.sandbox_enabled {
        if input.sandbox_allow_network {
            (icons.sandbox_net, "net-on", "76")
        } else {
            (icons.sandbox_nonet, "net-off", "214")
        }
    } else {
        (icons.sandbox_off, "host", "244")
    };
    append_badge(&mut badge_bufs[badge_count], sb_label, sb_val, sb_color, classic);
    badge_count += 1;

    // 2.8 Quotas
    let quota = resolve_quota(input);
    if quota.five_hour_pct >= 0.0 || quota.weekly_pct >= 0.0 {
        append_quota_bar(
            &mut badge_bufs[badge_count],
            quota.five_hour_pct,
            "5H",
            quota_bar_len,
            "37",
            quota.five_hour_reset,
            classic,
            icons.reset,
        );
        badge_count += 1;

        append_quota_bar(
            &mut badge_bufs[badge_count],
            quota.weekly_pct,
            "7D",
            quota_bar_len,
            "135",
            quota.weekly_reset,
            classic,
            icons.reset,
        );
        badge_count += 1;
    }

    // 2.9 Power Status
    if let Some(power) = get_power_info() {
        if power.is_ac {
            append_badge(&mut badge_bufs[badge_count], icons.ac, "AC", "76", classic);
            badge_count += 1;
        } else {
            let mut bat_val = String::with_capacity(8);
            if let Some(p) = power.battery_pct {
                let _ = write!(bat_val, "{p}%");
            } else {
                bat_val.push_str("BAT");
            }
            append_badge(&mut badge_bufs[badge_count], icons.bat, &bat_val, "214", classic);
            badge_count += 1;
        }
    }

    // ─── 3. Dynamic Line-Packing Engine & Framing ────────────────────────────
    let max_vis = cols.saturating_sub(4).max(40);

    let mut line_starts: [usize; 12] = [0; 12];
    let mut line_counts: [usize; 12] = [0; 12];
    let mut num_lines = 0;

    let mut curr_start = 0;
    let mut curr_count = 0;
    let mut curr_vis = 0;

    for i in 0..badge_count {
        let b_vis = visible_len(&badge_bufs[i]);
        if b_vis == 0 {
            continue;
        }
        if curr_count == 0 {
            curr_start = i;
            curr_count = 1;
            curr_vis = b_vis;
        } else if curr_vis + 2 + b_vis <= max_vis {
            curr_count += 1;
            curr_vis += 2 + b_vis;
        } else {
            line_starts[num_lines] = curr_start;
            line_counts[num_lines] = curr_count;
            num_lines += 1;

            curr_start = i;
            curr_count = 1;
            curr_vis = b_vis;
        }
    }
    if curr_count > 0 {
        line_starts[num_lines] = curr_start;
        line_counts[num_lines] = curr_count;
        num_lines += 1;
    }

    for line_idx in 0..num_lines {
        if classic {
            let start = line_starts[line_idx];
            let count = line_counts[line_idx];
            for j in 0..count {
                if j > 0 {
                    out.push_str("  ");
                }
                out.push_str(&badge_bufs[start + j]);
            }
            out.push('\n');
        } else {
            let prefix = if line_idx + 1 == num_lines {
                "\x1b[90m╰─\x1b[0m"
            } else {
                "\x1b[90m├─\x1b[0m"
            };
            out.push_str(prefix);

            let start = line_starts[line_idx];
            let count = line_counts[line_idx];
            for j in 0..count {
                if j > 0 {
                    out.push_str("  ");
                }
                out.push_str(&badge_bufs[start + j]);
            }
            out.push('\n');
        }
    }

    if out.ends_with('\n') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_3p_model() {
        assert!(is_3p_model("claude-3-5-sonnet"));
        assert!(is_3p_model("gpt-4o"));
        assert!(is_3p_model("anthropic/claude-instant"));
        assert!(is_3p_model("openai/o1-mini"));
        assert!(is_3p_model("3p-custom-model"));
        assert!(!is_3p_model("gemini-1.5-pro"));
        assert!(!is_3p_model("gemini-2.0-flash"));
    }

    #[test]
    fn test_resolve_quota_3p_priority() {
        let mut input = ParsedInput::default();
        input.model_id = "claude-3-5-sonnet";
        input.third_party_5h_pct = 75.0;
        input.third_party_weekly_pct = 50.0;
        input.gemini_5h_pct = 90.0;

        let q = resolve_quota(&input);
        assert_eq!(q.five_hour_pct, 75.0);
        assert_eq!(q.weekly_pct, 50.0);
    }

    #[test]
    fn test_resolve_quota_gemini_fallback() {
        let mut input = ParsedInput::default();
        input.model_id = "gemini-1.5-pro";
        input.gemini_5h_pct = 85.0;
        input.gemini_weekly_pct = 40.0;

        let q = resolve_quota(&input);
        assert_eq!(q.five_hour_pct, 85.0);
        assert_eq!(q.weekly_pct, 40.0);
    }
}
