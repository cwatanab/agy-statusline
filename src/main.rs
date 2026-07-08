mod parse;
mod sys;

use std::env;
use std::io::{self, Read};

use parse::ParsedInput;
use sys::{git_info, hostname, power_status, tailscale_ip};

// ─── ANSI Escape Codes ────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const ITALIC: &str = "\x1b[3m";

const ANSI_RED: &str = "\x1b[31m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_MAGENTA: &str = "\x1b[35m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_WHITE: &str = "\x1b[37m";

const ANSI_GRAY: &str = "\x1b[90m";
const ANSI_BRIGHT_RED: &str = "\x1b[91m";
const ANSI_BRIGHT_GREEN: &str = "\x1b[92m";
const ANSI_BRIGHT_YELLOW: &str = "\x1b[93m";
const ANSI_BRIGHT_BLUE: &str = "\x1b[94m";
const ANSI_BRIGHT_MAGENTA: &str = "\x1b[95m";
const ANSI_BRIGHT_CYAN: &str = "\x1b[96m";
const ANSI_BRIGHT_WHITE: &str = "\x1b[97m";

// ─── Nerd Font Icons ──────────────────────────────────────────────────────

struct Icons {
    dot_l1: String,
    dot_l2: String,
    vcs: &'static str,
    model: &'static str,
    sandbox_net: &'static str,
    sandbox_nonet: &'static str,
    sandbox_off: &'static str,
    context_bar: &'static str,
    artifacts: &'static str,
    subagents: &'static str,
    tasks: &'static str,
    dir: &'static str,
    conversation: &'static str,
    token_sum: &'static str,
    reset: &'static str,
    ac: &'static str,
    battery: &'static str,
    state_ready: &'static str,
    state_thinking: &'static str,
    state_working: &'static str,
    state_tool: &'static str,
    state_unknown: &'static str,
    user: &'static str,
    host: &'static str,
}

fn select_icons(classic: bool) -> Icons {
    if classic {
        Icons {
            dot_l1: preformat(ANSI_GRAY, " ╱ "),
            dot_l2: preformat(ANSI_GRAY, " · "),
            vcs: "", model: "",
            sandbox_net: "ON (net)", sandbox_nonet: "ON (no-net)", sandbox_off: "OFF",
            context_bar: "ctx", artifacts: "artifacts", subagents: "subagents", tasks: "tasks",
            dir: "", conversation: "", token_sum: "",
            reset: "\u{231B}", ac: "AC", battery: "BAT",
            state_ready: "●", state_thinking: "◆", state_working: "⚙", state_tool: "🔧", state_unknown: "\u{231B}",
            user: "", host: "",
        }
    } else {
        Icons {
            dot_l1: preformat(ANSI_GRAY, " | "),
            dot_l2: preformat(ANSI_GRAY, " | "),
            vcs: "\u{F418}", model: "\u{F400}",
            sandbox_net: "\u{F0499}", sandbox_nonet: "\u{F0D34}", sandbox_off: "\u{F099C}",
            context_bar: "\u{F134F}", artifacts: "\u{F0F6}", subagents: "\u{F167A}", tasks: "\u{F0AE}",
            dir: "\u{EA83}", conversation: "\u{F036A}", token_sum: "\u{E26B}",
            reset: "\u{231B}\u{FE0F}", ac: "\u{F06A5}", battery: "\u{1F50B}",
            state_ready: "\u{F192}", state_thinking: "\u{F07F7}", state_working: "\u{F423}", state_tool: "\u{F425}", state_unknown: "\u{F252}",
            user: "\u{F01EE}", host: "\u{F048B}",
        }
    }
}

fn preformat(color: &str, text: &str) -> String {
    format!("{color}{text}{RESET}")
}

// ─── Formatting Helpers ───────────────────────────────────────────────────

fn human_format(num: u64) -> String {
    if num >= 1_000_000 {
        format!("{}.{}M", num / 1_000_000, (num % 1_000_000) / 100_000)
    } else if num >= 1000 {
        format!("{}.{}K", num / 1000, (num % 1000) / 100)
    } else {
        num.to_string()
    }
}

fn format_reset_time(sec: i64) -> String {
    if sec <= 0 {
        return String::new();
    }
    let days = sec / 86400;
    let rem = sec % 86400;
    let hours = rem / 3600;
    let rem = rem % 3600;
    let minutes = rem / 60;

    if days > 0 {
        if hours > 0 { format!("{days}d {hours}h") } else { format!("{days}d") }
    } else if hours > 0 {
        if minutes > 0 { format!("{hours}h {minutes}m") } else { format!("{hours}h") }
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        "<1m".to_string()
    }
}

fn shorten_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let home = env::var("HOME").unwrap_or_default();
    let relative = if !home.is_empty() && path.starts_with(&home) {
        path.replacen(&home, "~", 1)
    } else {
        path.to_string()
    };
    if relative.len() > 25 {
        if let Some(name) = std::path::Path::new(&relative).file_name().and_then(|s| s.to_str()) {
            return format!("...{name}");
        }
    }
    relative
}

fn visible_length(s: &str) -> usize {
    let mut count = 0;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            while let Some(d) = chars.next() {
                if d == 'm' {
                    break;
                }
            }
        } else {
            count += 1;
        }
    }
    count
}

// ─── Bar Drawing ──────────────────────────────────────────────────────────

fn build_quota_bar(
    remaining_pct: f64,
    label: &str,
    bar_color: &str,
    reset_sec: i64,
    use_classic: bool,
    reset_icon: &str,
) -> String {
    let separator = if use_classic {
        format!("{ANSI_GRAY} · {RESET}")
    } else {
        format!("{ANSI_GRAY}| {RESET}")
    };

    if remaining_pct < -0.5 {
        let bar_empty: String = (0..20)
            .map(|_| if use_classic { "·" } else { "░" })
            .collect();
        return format!(
            "{separator}{ANSI_BRIGHT_WHITE}{BOLD}{label}{RESET} {ANSI_GRAY}{bar_empty} N/A{RESET}"
        );
    }

    let pct_int = remaining_pct as u32;
    let text_color = match pct_int {
        0..=19 => ANSI_BRIGHT_RED,
        20..=49 => ANSI_BRIGHT_YELLOW,
        _ => ANSI_BRIGHT_GREEN,
    };

    let filled = pct_int * 20 / 100;
    let remainder = (pct_int * 20) % 100;

    let block_full = format!("{bar_color}█{RESET}");
    let block_75 = format!("{bar_color}▓{RESET}{ANSI_GRAY}");
    let block_50 = format!("{bar_color}▒{RESET}{ANSI_GRAY}");
    let block_25 = format!("{bar_color}░{RESET}{ANSI_GRAY}");
    let block_empty = format!("{ANSI_GRAY}░{RESET}");

    let mut bar = String::with_capacity(160);
    for i in 0..20u32 {
        if i < filled {
            if use_classic { bar.push('█'); } else { bar.push_str(&block_full); }
        } else if i == filled {
            if use_classic {
                bar.push_str(match remainder {
                    75.. => "▓", 50.. => "▒", 25.. => "░", _ => "·",
                });
            } else {
                bar.push_str(match remainder {
                    75.. => &block_75, 50.. => &block_50, 25.. => &block_25, _ => &block_empty,
                });
            }
        } else {
            if use_classic { bar.push('·'); } else { bar.push_str(&block_empty); }
        }
    }

    let reset_label = if reset_sec > 0 {
        format!(" {reset_icon} {}", format_reset_time(reset_sec))
    } else {
        String::new()
    };

    let pct_str = if remaining_pct % 1.0 == 0.0 {
        format!("{:.0}", remaining_pct)
    } else {
        format!("{:.1}", remaining_pct)
    };

    if use_classic {
        format!(
            "{separator}{ANSI_BRIGHT_WHITE}{BOLD}{label}{RESET} {bar_color}{bar}{RESET} {text_color}{pct_str}%{RESET}{reset_label}"
        )
    } else {
        format!(
            "{separator}{ANSI_BRIGHT_WHITE}{BOLD}{label}{RESET} {bar} {text_color}{pct_str}%{RESET}{reset_label}"
        )
    }
}

fn print_right_aligned(left: &str, right: &str, total_cols: u32) {
    let left_vis = visible_length(left);
    let right_vis = visible_length(right);
    let padding = if total_cols as usize > left_vis + right_vis {
        total_cols as usize - left_vis - right_vis
    } else {
        1
    };
    print!("{left}");
    for _ in 0..padding {
        print!(" ");
    }
    println!("{right}");
}

// ─── Output Construction ──────────────────────────────────────────────────

struct View {
    state_str: String,
    version_str: String,
    user_str: String,
    host_str: String,
    model_str: String,
    dir_str: String,
    vcs_str: String,
    conv_str: String,
    art_str: String,
    sub_str: String,
    task_str: String,
    sandbox_str: String,
    ctx_bar: String,
    tok_wide: String,
    tok_medium: String,
    quota_str: String,
    power_str: String,
    model_short_str: String,
}

fn build_view(input: &ParsedInput, icons: &Icons, classic: bool) -> View {
    let dot_l1 = &icons.dot_l1;
    let dot_l2 = &icons.dot_l2;

    let model_display = if !input.model_display_name.is_empty() {
        &input.model_display_name
    } else {
        &input.model_id
    };

    let context_used = input.total_input_tokens + input.total_output_tokens;

    let (active_5h, active_weekly, active_5h_reset, active_weekly_reset) =
        if (input.gemini_5h_pct >= 0.0) || (input.gemini_weekly_pct >= 0.0) {
            (input.gemini_5h_pct, input.gemini_weekly_pct, input.gemini_5h_reset, input.gemini_weekly_reset)
        } else if (input.third_party_5h_pct >= 0.0) || (input.third_party_weekly_pct >= 0.0) {
            (input.third_party_5h_pct, input.third_party_weekly_pct, input.third_party_5h_reset, input.third_party_weekly_reset)
        } else {
            (-1.0, -1.0, -1, -1)
        };

    // State indicator
    let state_str = match input.agent_state.as_str() {
        "idle" => format!("{ANSI_BRIGHT_GREEN}{BOLD} {} READY{RESET}", icons.state_ready),
        "thinking" => format!("{ANSI_BRIGHT_YELLOW}{BOLD} {} THINKING{RESET}", icons.state_thinking),
        "working" => format!("{ANSI_BRIGHT_CYAN}{BOLD} {} WORKING{RESET}", icons.state_working),
        "tool_use" => format!("{ANSI_BRIGHT_MAGENTA}{BOLD} {} TOOL{RESET}", icons.state_tool),
        other => format!("{ANSI_WHITE}{BOLD} {} {}{RESET}", icons.state_unknown, other.to_uppercase()),
    };

    // Version
    let version_str = if input.version.is_empty() {
        String::new()
    } else {
        format!("{dot_l1}{ANSI_GRAY}v{}{RESET}", input.version)
    };

    // User info
    let user_str = if !input.plan_tier.is_empty() || !input.email.is_empty() {
        let info = if !input.plan_tier.is_empty() && !input.email.is_empty() {
            format!("{} ({})", input.plan_tier, input.email)
        } else if !input.plan_tier.is_empty() {
            input.plan_tier.clone()
        } else {
            input.email.clone()
        };
        let truncated = if info.len() > 35 { format!("{}...", &info[..32]) } else { info };
        if classic {
            format!("{dot_l1}{ANSI_GRAY}{truncated}{RESET}")
        } else {
            format!("{dot_l1}{ANSI_GRAY}{} {truncated}{RESET}", icons.user)
        }
    } else {
        String::new()
    };

    // Host info
    let host_name = hostname();
    let ts_ip = tailscale_ip();
    let host_str = if !host_name.is_empty() {
        let details = if !ts_ip.is_empty() { format!("{host_name} ({ts_ip})") } else { host_name };
        if classic {
            format!("{dot_l1}{ANSI_BRIGHT_BLUE}{details}{RESET}")
        } else {
            format!("{dot_l1}{ANSI_BRIGHT_BLUE}{} {details}{RESET}", icons.host)
        }
    } else {
        String::new()
    };

    // Power status
    let power_str = {
        let (on_battery, capacity) = power_status();
        if on_battery {
            if let Some(cap) = capacity {
                if classic {
                    format!("{dot_l2}{ANSI_BRIGHT_YELLOW}{}:{}%{RESET}", icons.battery, cap)
                } else {
                    format!("{dot_l2}{ANSI_BRIGHT_YELLOW}{} {}%{RESET}", icons.battery, cap)
                }
            } else {
                format!("{dot_l2}{ANSI_BRIGHT_YELLOW}{}{RESET}", icons.battery)
            }
        } else if classic {
            format!("{dot_l2}{ANSI_GREEN}{}{RESET}", icons.ac)
        } else {
            format!("{dot_l2}{ANSI_GREEN}{} AC{RESET}", icons.ac)
        }
    };

    // Working directory
    let cwd_short = shorten_path(&input.working_dir);
    let dir_str = if !cwd_short.is_empty() {
        if classic {
            format!("{dot_l1}{ANSI_CYAN}{cwd_short}{RESET}")
        } else {
            format!("{dot_l1}{ANSI_CYAN}{} {cwd_short}{RESET}", icons.dir)
        }
    } else {
        String::new()
    };

    // Conversation ID
    let conv_str = if !input.conversation_id.is_empty() {
        let len = 8.min(input.conversation_id.len());
        if classic {
            format!("{dot_l1}{ANSI_GRAY}{}{RESET}", &input.conversation_id[..len])
        } else {
            format!("{dot_l1}{ANSI_GRAY}{} {}{RESET}", icons.conversation, &input.conversation_id[..len])
        }
    } else {
        String::new()
    };

    // VCS (from git directly)
    let (_, vcs_branch, vcs_dirty) = git_info(&input.working_dir);
    let vcs_str = if vcs_branch.is_empty() {
        String::new()
    } else if vcs_dirty {
        if classic {
            format!("{dot_l1}{ANSI_BRIGHT_RED}{vcs_branch}{ANSI_BRIGHT_YELLOW}*{RESET}")
        } else {
            format!("{dot_l1}{RESET}{ANSI_BRIGHT_RED}{} {vcs_branch}{ANSI_BRIGHT_YELLOW}*{RESET}", icons.vcs)
        }
    } else if classic {
        format!("{dot_l1}{ANSI_BRIGHT_BLUE}{vcs_branch}{RESET}")
    } else {
        format!("{dot_l1}{RESET}{ANSI_BRIGHT_BLUE}{} {vcs_branch}{RESET}", icons.vcs)
    };

    // Model
    let model_str = if !model_display.is_empty() {
        if classic {
            format!("{dot_l1}{ANSI_BRIGHT_MAGENTA}{ITALIC}{model_display}{RESET}")
        } else {
            format!("{dot_l1}{ANSI_BRIGHT_MAGENTA}{ITALIC}{} {model_display}{RESET}", icons.model)
        }
    } else {
        String::new()
    };

    // Sandbox badge
    let sandbox_str = if input.sandbox_enabled {
        if input.sandbox_allow_network {
            format!("{ANSI_GREEN}{} ON (net){RESET}", icons.sandbox_net)
        } else {
            format!("{ANSI_GREEN}{} ON (no-net){RESET}", icons.sandbox_nonet)
        }
    } else if classic {
        format!("{ANSI_GRAY}sandbox off{RESET}")
    } else {
        format!("{ANSI_RED}{} OFF{RESET}", icons.sandbox_off)
    };

    // Context window bar (20 segments)
    let pct_int = input.used_percentage as u32;
    let pct_x10 = (input.used_percentage * 10.0).round() as u32;
    let pct_display = format!("{}.{}", pct_x10 / 10, pct_x10 % 10);
    let num_fmt = format!("{ANSI_BRIGHT_WHITE}{BOLD}{pct_display}%{RESET}");

    let fill_color = if pct_int >= 90 { ANSI_BRIGHT_RED }
    else if pct_int >= 60 { ANSI_BRIGHT_YELLOW }
    else { ANSI_YELLOW };

    let filled_segments = pct_int * 20 / 100;
    let rem = (pct_int * 20) % 100;

    let block_full = format!("{fill_color}█{RESET}");
    let block_75 = format!("{fill_color}▓{RESET}{ANSI_GRAY}");
    let block_50 = format!("{fill_color}▒{RESET}{ANSI_GRAY}");
    let block_25 = format!("{fill_color}░{RESET}{ANSI_GRAY}");
    let block_empty = format!("{ANSI_GRAY}░{RESET}");

    let ctx_bar = if classic {
        let mut bar = String::with_capacity(40);
        for i in 0..20u32 {
            if i < filled_segments { bar.push('█'); }
            else if i == filled_segments { bar.push_str(match rem { 75.. => "▓", 50.. => "▒", 25.. => "░", _ => "·" }); }
            else { bar.push('·'); }
        }
        format!("{ANSI_GRAY}ctx {fill_color}{bar} {num_fmt}")
    } else {
        let mut bar = String::with_capacity(160);
        for i in 0..20u32 {
            if i < filled_segments { bar.push_str(&block_full); }
            else if i == filled_segments { bar.push_str(match rem { 75.. => &block_75, 50.. => &block_50, 25.. => &block_25, _ => &block_empty }); }
            else { bar.push_str(&block_empty); }
        }
        format!("{ANSI_YELLOW}{}  {RESET}{bar} {num_fmt}", icons.context_bar)
    };

    // Stats
    let art_str = if classic {
        format!("{ANSI_GRAY}artifacts {ANSI_BRIGHT_WHITE}{BOLD}{}{RESET}", input.artifact_count)
    } else {
        format!("{ANSI_BLUE}{} {ANSI_BRIGHT_WHITE}{BOLD}{}{RESET}", icons.artifacts, input.artifact_count)
    };
    let sub_str = if classic {
        format!("{ANSI_GRAY}subagents {ANSI_BRIGHT_WHITE}{BOLD}{}{RESET}", input.subagent_count)
    } else {
        format!("{ANSI_CYAN}{} {ANSI_BRIGHT_WHITE}{BOLD}{}{RESET}", icons.subagents, input.subagent_count)
    };
    let task_str = if classic {
        format!("{ANSI_GRAY}tasks {ANSI_BRIGHT_WHITE}{BOLD}{}{RESET}", input.task_count)
    } else {
        format!("{ANSI_MAGENTA}{} {ANSI_BRIGHT_WHITE}{BOLD}{}{RESET}", icons.tasks, input.task_count)
    };

    // Token details
    let itf = human_format(input.total_input_tokens);
    let otf = human_format(input.total_output_tokens);
    let clf = human_format(input.context_window_size);
    let cuf = human_format(context_used);
    let tif = human_format(input.turn_input_tokens);
    let tof = human_format(input.turn_output_tokens);

    let tok_wide = if context_used > 0 {
        let turn_info = if input.turn_input_tokens > 0 || input.turn_output_tokens > 0 {
            format!(" | turn: +{tif}/{tof}")
        } else {
            String::new()
        };
        if classic {
            format!(" ({cuf}/{clf}){dot_l2}(total: {itf}/{otf}{turn_info})")
        } else {
            format!(" ({cuf}/{clf}){dot_l2}{ANSI_YELLOW}{} {RESET} (total: {itf}/{otf}{turn_info})", icons.token_sum)
        }
    } else {
        String::new()
    };

    let tok_medium = if context_used > 0 {
        format!(" ({cuf}/{clf})")
    } else {
        String::new()
    };

    // Quota
    let quota_str = if (active_5h >= 0.0) || (active_weekly >= 0.0) {
        format!(
            "{} {}",
            build_quota_bar(active_5h, "5H", ANSI_BRIGHT_CYAN, active_5h_reset, classic, icons.reset),
            build_quota_bar(active_weekly, "7D", ANSI_BRIGHT_MAGENTA, active_weekly_reset, classic, icons.reset),
        )
    } else {
        String::new()
    };

    // Model short (for narrow layout)
    let model_short_str = if !model_display.is_empty() {
        let model_disp_short: String = model_display.chars().take(12).collect();
        if classic {
            format!("{ANSI_GRAY} ╱ {ANSI_BRIGHT_MAGENTA}{}{RESET}", model_disp_short)
        } else {
            format!("{ANSI_GRAY} ╱ {ANSI_BRIGHT_MAGENTA}{} {}{RESET}", icons.model, model_disp_short)
        }
    } else {
        String::new()
    };

    View {
        state_str, version_str, user_str, host_str, model_str, dir_str,
        vcs_str, conv_str, art_str, sub_str, task_str, sandbox_str,
        ctx_bar, tok_wide, tok_medium, quota_str, power_str, model_short_str,
    }
}

// ─── Entry Point ──────────────────────────────────────────────────────────

fn main() {
    let use_classic = env::args().any(|arg| {
        arg == "--classic" || arg == "--no-nerdfont" || arg == "--compatibility"
    });

    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).unwrap();

    let input = parse::parse_input(&stdin);
    let icons = select_icons(use_classic);
    let view = build_view(&input, &icons, use_classic);
    let cols = input.terminal_width;

    let dot_l2 = &icons.dot_l2;
    let quota_has_data = !view.quota_str.is_empty();

    if cols >= 180 {
        let line1 = format!(
            "{}{}{}{}{}{}{}{}",
            view.state_str, view.version_str, view.user_str, view.host_str,
            view.model_str, view.dir_str, view.vcs_str, view.conv_str,
        );
        let line2 = if quota_has_data {
            format!(
                "{}{dot_l2}{}{dot_l2}{}{dot_l2}{}{dot_l2}{}{}{}{}",
                view.art_str, view.sub_str, view.task_str, view.sandbox_str,
                view.ctx_bar, view.tok_wide, view.quota_str, view.power_str,
            )
        } else {
            format!(
                "{}{dot_l2}{}{dot_l2}{}{dot_l2}{}{dot_l2}{}{}{}",
                view.art_str, view.sub_str, view.task_str, view.sandbox_str,
                view.ctx_bar, view.tok_wide, view.power_str,
            )
        };
        print_right_aligned(&line1, &line2, cols);
    } else if cols >= 90 {
        let line1 = format!(
            "{}{}{}{}{}{}{}",
            view.state_str, view.version_str, view.user_str, view.host_str,
            view.model_str, view.dir_str, view.vcs_str,
        );
        let line2 = if quota_has_data {
            format!(
                " {}{}{dot_l2}{}{dot_l2}{}{dot_l2}{}{dot_l2}{}{}{}",
                view.ctx_bar, view.tok_medium, view.art_str, view.sub_str,
                view.task_str, view.sandbox_str, view.quota_str, view.power_str,
            )
        } else {
            format!(
                " {}{}{dot_l2}{}{dot_l2}{}{dot_l2}{}{dot_l2}{}{}",
                view.ctx_bar, view.tok_medium, view.art_str, view.sub_str,
                view.task_str, view.sandbox_str, view.power_str,
            )
        };
        println!("{ANSI_GRAY}╭─{RESET}{line1}");
        println!("{ANSI_GRAY}╰─{RESET}{line2}");
    } else {
        println!("{}{}", view.state_str, view.model_short_str);
        if quota_has_data {
            println!(
                "{}{dot_l2}{}{}{}",
                view.ctx_bar, view.task_str, view.quota_str, view.power_str,
            );
        } else {
            println!(
                "{}{dot_l2}{}{}",
                view.ctx_bar, view.task_str, view.power_str,
            );
        }
    }
}
