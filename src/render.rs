// ─── Quota Helpers ─────────────────────────────────────────────────────────

use std::fmt::Write;
use crate::bar::{append_bar, append_badge, append_quota_bar, usage_color};
use crate::format::{write_pct_display, write_human_format, shorten_path, visible_len};
use crate::icons::{
    select_icons, ANSI_BRIGHT_BLUE, ANSI_BRIGHT_CYAN, ANSI_BRIGHT_GREEN, ANSI_BRIGHT_MAGENTA,
    BOLD, RESET,
};
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
    let is_3p = is_3p_model(input.model_id.as_ref());
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

    let next_str = next_bg.unwrap_or("\x1b[0m");
    if let Some(code) = bg_color.strip_prefix("\x1b[48;5;") {
        let _ = write!(buf, "{bg_color}{fg_text} {text} {next_str}\x1b[38;5;{code}\u{E0B0}{RESET}");
    } else {
        let fg_sep = bg_color.replace("48;", "38;");
        let _ = write!(buf, "{bg_color}{fg_text} {text} {next_str}{fg_sep}\u{E0B0}{RESET}");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterFlags {
    pub show_state: bool,
    pub show_vim: bool,
    pub show_branch: bool,
    pub show_model: bool,
    pub show_dir: bool,
    pub show_conv: bool,
    pub show_account: bool,
    pub show_host: bool,
    pub show_version: bool,

    pub show_context_usage: bool,
    pub show_tokens_usage: bool,
    pub show_cost: bool,
    pub show_sys: bool,
    pub show_artifacts: bool,
    pub show_subagents: bool,
    pub show_tasks: bool,
    pub show_sandbox: bool,
    pub show_quota: bool,
    pub show_power: bool,
}

impl Default for FilterFlags {
    fn default() -> Self {
        Self {
            show_state: true,
            show_vim: true,
            show_branch: true,
            show_model: true,
            show_dir: true,
            show_conv: true,
            show_account: true,
            show_host: true,
            show_version: true,

            show_context_usage: true,
            show_tokens_usage: true,
            show_cost: true,
            show_sys: true,
            show_artifacts: true,
            show_subagents: true,
            show_tasks: true,
            show_sandbox: true,
            show_quota: true,
            show_power: true,
        }
    }
}

pub fn render_line(input: &ParsedInput, classic: bool, override_cols: Option<usize>) -> String {
    render_line_with_filter(input, classic, override_cols, FilterFlags::default())
}

pub fn render_line_with_filter(
    input: &ParsedInput,
    classic: bool,
    override_cols: Option<usize>,
    filter: FilterFlags,
) -> String {
    let icons = select_icons(classic);
    let cols = override_cols.unwrap_or(input.terminal_width).max(40);

    // Wide bars (20/15 segments) require >= 235 columns to fit alongside full telemetry without split
    let (bar_len, quota_bar_len) = if cols >= 235 {
        (20, 15)
    } else {
        (10, 8)
    };

    // Preallocate single buffer for entire output
    let mut out = String::with_capacity(1024);

    // ─── 1. Powerline LINE1 Assembly ─────────────────────────────────────────
    let mut seg_bufs: [String; 12] = [
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
    ];
    let mut seg_bgs: [&'static str; 12] = [""; 12];
    let mut seg_fgs: [&'static str; 12] = [""; 12];
    let mut seg_count = 0;

    // 1.1 State
    if filter.show_state {
        match input.agent_state.as_ref() {
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
    }

    // 1.2 Vim Editor Mode
    if filter.show_vim && !input.vim_mode.is_empty() {
        let mode_raw = input.vim_mode.as_ref();
        let is_normal = mode_raw.eq_ignore_ascii_case("NORMAL");
        let is_insert = mode_raw.eq_ignore_ascii_case("INSERT");
        let is_visual = mode_raw.len() >= 6 && mode_raw[..6].eq_ignore_ascii_case("VISUAL");

        if classic {
            let _ = write!(seg_bufs[seg_count], "[{}]", mode_raw);
            seg_bgs[seg_count] = if is_normal {
                ANSI_BRIGHT_BLUE
            } else if is_insert {
                ANSI_BRIGHT_GREEN
            } else if is_visual {
                ANSI_BRIGHT_MAGENTA
            } else {
                ANSI_BRIGHT_CYAN
            };
            seg_fgs[seg_count] = BOLD;
        } else {
            seg_bufs[seg_count].push_str(mode_raw);
            if is_normal {
                seg_bgs[seg_count] = "\x1b[48;5;33m";
                seg_fgs[seg_count] = "\x1b[38;5;255m\x1b[1m";
            } else if is_insert {
                seg_bgs[seg_count] = "\x1b[48;5;76m";
                seg_fgs[seg_count] = "\x1b[38;5;232m\x1b[1m";
            } else if is_visual {
                seg_bgs[seg_count] = "\x1b[48;5;135m";
                seg_fgs[seg_count] = "\x1b[38;5;255m\x1b[1m";
            } else {
                seg_bgs[seg_count] = "\x1b[48;5;37m";
                seg_fgs[seg_count] = "\x1b[38;5;232m\x1b[1m";
            }
        }
        seg_count += 1;
    }

    // 1.3 VCS Branch
    if filter.show_branch {
        let (vcs_branch, vcs_dirty) = git_info(
            input.working_dir.as_ref(),
            input.vcs_branch.as_ref(),
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
    }

    // 1.4 Model
    if filter.show_model {
        let model_disp = if !input.model_display_name.is_empty() {
            input.model_display_name.as_ref()
        } else {
            input.model_id.as_ref()
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
    }

    // 1.5 Directory
    if filter.show_dir && !input.working_dir.is_empty() {
        let cwd_short = shorten_path(input.working_dir.as_ref());
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
    }

    // 1.6 User Plan & Account
    if filter.show_account && (!input.plan_tier.is_empty() || !input.email.is_empty()) && cols >= 130 {
        if !input.plan_tier.is_empty() && !input.email.is_empty() {
            let _ = write!(seg_bufs[seg_count], "{} ({})", input.plan_tier, input.email);
        } else if !input.plan_tier.is_empty() {
            seg_bufs[seg_count].push_str(input.plan_tier.as_ref());
        } else {
            seg_bufs[seg_count].push_str(input.email.as_ref());
        }
        if !classic {
            seg_bufs[seg_count].insert_str(0, "👤 ");
        }
        seg_bgs[seg_count] = icons.theme.bg_meta;
        seg_fgs[seg_count] = icons.theme.fg_meta_text;
        seg_count += 1;
    }

    // 1.7 Conversation ID
    if filter.show_conv && !input.conversation_id.is_empty() && cols >= 80 {
        let conv_prefix = if input.conversation_id.len() > 8 {
            &input.conversation_id[..8]
        } else {
            input.conversation_id.as_ref()
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

    // 1.8 Host Info
    if filter.show_host && cols >= 110 {
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

    // 1.9 Version
    if filter.show_version && !input.version.is_empty() && cols >= 120 {
        let _ = write!(seg_bufs[seg_count], "v{}", input.version);
        seg_bgs[seg_count] = icons.theme.bg_meta;
        seg_fgs[seg_count] = icons.theme.fg_meta_text;
        seg_count += 1;
    }

    // Safeguard: drop trailing segments if LINE1 exceeds terminal width
    let calc_line1_len = |count: usize| -> usize {
        if count == 0 {
            return 0;
        }
        let mut total = if classic { 0 } else { 2 };
        for i in 0..count {
            let text_len = visible_len(&seg_bufs[i]);
            total += if classic { text_len + 1 } else { text_len + 3 };
        }
        total
    };

    while seg_count > 1 && calc_line1_len(seg_count) > cols {
        seg_count -= 1;
    }

    if seg_count > 0 {
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
    }

    // ─── 2. Telemetry Badges Stream Engine ─────────────────────────────────────
    let mut badge_bufs: [String; 12] = [
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
    ];
    let mut badge_count = 0;

    // 2.1 Context Usage Bar
    if filter.show_context_usage {
        let pct_int = input.used_percentage as usize;
        let fill_color = usage_color(input.used_percentage);

        let ctx_used = if input.used_percentage > 0.0 && input.context_window_size > 0 {
            ((input.used_percentage * input.context_window_size as f64) / 100.0).round() as u64
        } else if input.total_tokens > 0 {
            input.total_tokens
        } else if input.total_input_tokens > 0 {
            input.total_input_tokens
        } else {
            input.total_input_tokens + input.total_output_tokens
        };

        if classic {
            let _ = write!(badge_bufs[badge_count], "\x1b[90mctx {fill_color}");
            append_bar(&mut badge_bufs[badge_count], input.used_percentage, bar_len, "76", true);
            let _ = write!(badge_bufs[badge_count], " \x1b[97m{BOLD}");
            write_pct_display(&mut badge_bufs[badge_count], input.used_percentage);
            badge_bufs[badge_count].push_str("%\x1b[0m");
            if input.context_window_size > 0 {
                badge_bufs[badge_count].push_str(" \x1b[90m(");
                write_human_format(&mut badge_bufs[badge_count], ctx_used);
                badge_bufs[badge_count].push('/');
                write_human_format(&mut badge_bufs[badge_count], input.context_window_size);
                badge_bufs[badge_count].push_str(")\x1b[0m");
            } else if ctx_used > 0 {
                badge_bufs[badge_count].push_str(" \x1b[90m(");
                write_human_format(&mut badge_bufs[badge_count], ctx_used);
                badge_bufs[badge_count].push_str(")\x1b[0m");
            }
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
            badge_bufs[badge_count].push_str("%\x1b[22m");
            if input.context_window_size > 0 {
                badge_bufs[badge_count].push_str(" \x1b[38;5;250m(");
                write_human_format(&mut badge_bufs[badge_count], ctx_used);
                badge_bufs[badge_count].push('/');
                write_human_format(&mut badge_bufs[badge_count], input.context_window_size);
                badge_bufs[badge_count].push(')');
            } else if ctx_used > 0 {
                badge_bufs[badge_count].push_str(" \x1b[38;5;250m(");
                write_human_format(&mut badge_bufs[badge_count], ctx_used);
                badge_bufs[badge_count].push(')');
            }
            let _ = write!(badge_bufs[badge_count], "\x1b[0m\x1b[38;5;{label_bg}m\x1b[0m");
        }
        badge_count += 1;
    }

    // 2.2 Token Details
    if filter.show_tokens_usage {
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
    }

    // 2.3 System Resources
    if filter.show_sys {
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
    }

    // 2.4 Artifacts
    if filter.show_artifacts {
        let mut art_str = String::with_capacity(8);
        let _ = write!(art_str, "{}", input.artifact_count);
        append_badge(&mut badge_bufs[badge_count], icons.artifacts, &art_str, "75", classic);
        badge_count += 1;
    }

    // 2.5 Subagents
    if filter.show_subagents && input.subagent_count > 0 {
        let mut sub_str = String::with_capacity(8);
        let _ = write!(sub_str, "{}", input.subagent_count);
        append_badge(&mut badge_bufs[badge_count], icons.subagents, &sub_str, "37", classic);
        badge_count += 1;
    }

    // 2.6 Tasks
    if filter.show_tasks {
        let mut task_str = String::with_capacity(8);
        let _ = write!(task_str, "{}", input.task_count);
        append_badge(&mut badge_bufs[badge_count], icons.tasks, &task_str, "135", classic);
        badge_count += 1;
    }

    // 2.7 Sandbox
    if filter.show_sandbox {
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
    }

    // 2.8 Quotas
    if filter.show_quota {
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
    }

    // 2.9 Power Status
    if filter.show_power {
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
    }

    // ─── 3. Dynamic Line-Packing Engine & Framing ────────────────────────────
    // Classic mode lacks box-drawing borders (╭─, ├─, ╰─), so it uses full terminal width
    let max_vis = if classic {
        cols.saturating_sub(1).max(40)
    } else {
        cols.saturating_sub(4).max(40)
    };

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
            let prefix = if seg_count > 0 {
                if line_idx + 1 == num_lines {
                    "\x1b[90m╰─\x1b[0m"
                } else {
                    "\x1b[90m├─\x1b[0m"
                }
            } else {
                if line_idx == 0 {
                    "\x1b[90m╭─\x1b[0m"
                } else if line_idx + 1 == num_lines {
                    "\x1b[90m╰─\x1b[0m"
                } else {
                    "\x1b[90m├─\x1b[0m"
                }
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
    use std::borrow::Cow;

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
        input.model_id = Cow::Borrowed("claude-3-5-sonnet");
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
        input.model_id = Cow::Borrowed("gemini-1.5-pro");
        input.gemini_5h_pct = 85.0;
        input.gemini_weekly_pct = 40.0;

        let q = resolve_quota(&input);
        assert_eq!(q.five_hour_pct, 85.0);
        assert_eq!(q.weekly_pct, 40.0);
    }
}
