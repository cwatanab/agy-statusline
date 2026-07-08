mod parse;
mod sys;

use std::env;
use std::io::{self, Read};

use parse::ParsedInput;
use sys::git_info;

// ─── ANSI Escape Codes ────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

const ANSI_WHITE: &str = "\x1b[37m";

const ANSI_GRAY: &str = "\x1b[90m";
const ANSI_BRIGHT_RED: &str = "\x1b[91m";
const ANSI_BRIGHT_GREEN: &str = "\x1b[92m";
const ANSI_BRIGHT_YELLOW: &str = "\x1b[93m";
const ANSI_BRIGHT_MAGENTA: &str = "\x1b[95m";
const ANSI_BRIGHT_CYAN: &str = "\x1b[96m";

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
    token_sum: &'static str,
    reset: &'static str,
    state_ready: &'static str,
    state_thinking: &'static str,
    state_working: &'static str,
    state_tool: &'static str,
    state_unknown: &'static str,
}

fn select_icons(classic: bool) -> Icons {
    if classic {
        Icons {
            dot_l1: preformat(ANSI_GRAY, " ╱ "),
            dot_l2: preformat(ANSI_GRAY, " · "),
            vcs: "", model: "",
            sandbox_net: "ON (net)", sandbox_nonet: "ON (no-net)", sandbox_off: "OFF",
            context_bar: "ctx", artifacts: "artifacts", subagents: "subagents", tasks: "tasks",
            token_sum: "",
            reset: "\u{231B}",
            state_ready: "●", state_thinking: "◆", state_working: "⚙", state_tool: "🔧", state_unknown: "\u{231B}",
        }
    } else {
        Icons {
            dot_l1: preformat(ANSI_GRAY, " | "),
            dot_l2: preformat(ANSI_GRAY, " | "),
            vcs: "\u{F418}", model: "\u{F400}",
            sandbox_net: "\u{F0499}", sandbox_nonet: "\u{F0D34}", sandbox_off: "\u{F099C}",
            context_bar: "\u{F134F}", artifacts: "\u{F0F6}", subagents: "\u{F167A}", tasks: "\u{F0AE}",
            token_sum: "\u{E26B}",
            reset: "\u{231B}\u{FE0F}",
            state_ready: "\u{F192}", state_thinking: "\u{F07F7}", state_working: "\u{F423}", state_tool: "\u{F425}", state_unknown: "\u{F252}",
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




// ─── Bar Drawing ──────────────────────────────────────────────────────────

fn build_quota_bar(
    remaining_pct: f64,
    label: &str,
    reset_sec: i64,
    use_classic: bool,
    reset_icon: &str,
) -> String {
    if remaining_pct < -0.5 {
        let bar_empty: String = (0..10)
            .map(|_| if use_classic { "·" } else { "░" })
            .collect();
        return format!(
            "{ANSI_GRAY}{BOLD}{label}{RESET} {ANSI_GRAY}{bar_empty} N/A{RESET}"
        );
    }

    let pct_int = remaining_pct as u32;
    let text_color = match pct_int {
        0..=19 => ANSI_BRIGHT_RED,
        20..=49 => ANSI_BRIGHT_YELLOW,
        _ => ANSI_BRIGHT_GREEN,
    };

    let filled_grades = ((remaining_pct * 80.0) / 100.0).round() as i32;
    let filled_grades = filled_grades.clamp(0, 80) as u32;
    let filled_chars = filled_grades / 8;
    let rem_grades = filled_grades % 8;

    let block_full = format!("{text_color}█{RESET}");
    let block_empty = format!("{ANSI_GRAY}·{RESET}");
    const BLOCK_CHARS: [&str; 8] = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];

    let mut bar = String::with_capacity(80);
    for i in 0..10u32 {
        if i < filled_chars {
            if use_classic { bar.push('█'); } else { bar.push_str(&block_full); }
        } else if i == filled_chars {
            let block_char = BLOCK_CHARS[rem_grades as usize];
            if use_classic {
                if block_char.is_empty() {
                    bar.push('·');
                } else {
                    bar.push_str(block_char);
                }
            } else {
                if block_char.is_empty() {
                    bar.push_str(&block_empty);
                } else {
                    bar.push_str(&format!("{text_color}{block_char}{RESET}"));
                }
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
            "{text_color}{BOLD}{label}{RESET} {text_color}{bar}{RESET} {text_color}{pct_str}%{RESET}{reset_label}"
        )
    } else {
        format!(
            "{text_color}{BOLD}{label}{RESET} {bar} {text_color}{pct_str}%{RESET}{reset_label}"
        )
    }
}



// ─── Output Construction ──────────────────────────────────────────────────

struct View {
    state_str: String,
    model_str: String,
    vcs_str: String,
    art_str: String,
    sub_str: String,
    task_str: String,
    sandbox_str: String,
    ctx_bar: String,
    ctx_size: String,
    tok_details: String,
    quota_str: String,
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



    // VCS (from git directly)
    let (_, vcs_branch, vcs_dirty) = git_info(&input.working_dir);
    let vcs_str = if vcs_branch.is_empty() {
        String::new()
    } else if vcs_dirty {
        if classic {
            format!("{dot_l1}{vcs_branch}*")
        } else {
            format!("{dot_l1}{} {vcs_branch}*", icons.vcs)
        }
    } else if classic {
        format!("{dot_l1}{vcs_branch}")
    } else {
        format!("{dot_l1}{} {vcs_branch}", icons.vcs)
    };

    // Model
    let model_str = if !model_display.is_empty() {
        if classic {
            format!("{dot_l1}{model_display}")
        } else {
            format!("{dot_l1}{} {model_display}", icons.model)
        }
    } else {
        String::new()
    };

    // Sandbox badge
    let sandbox_str = if input.sandbox_enabled {
        if input.sandbox_allow_network {
            if classic {
                format!("{ANSI_BRIGHT_YELLOW}ON (net){RESET}")
            } else {
                format!("{ANSI_BRIGHT_YELLOW}{} ON (net){RESET}", icons.sandbox_net)
            }
        } else if classic {
            format!("{ANSI_BRIGHT_GREEN}ON (no-net){RESET}")
        } else {
            format!("{ANSI_BRIGHT_GREEN}{} ON (no-net){RESET}", icons.sandbox_nonet)
        }
    } else if classic {
        format!("{ANSI_BRIGHT_RED}sandbox off{RESET}")
    } else {
        format!("{ANSI_BRIGHT_RED}{} OFF{RESET}", icons.sandbox_off)
    };

    // Context window bar (10 segments)
    let pct_int = input.used_percentage as u32;
    let pct_x10 = (input.used_percentage * 10.0).round() as u32;
    let pct_display = format!("{}.{}", pct_x10 / 10, pct_x10 % 10);
    
    let fill_color = if pct_int >= 80 { ANSI_BRIGHT_RED }
    else if pct_int >= 50 { ANSI_BRIGHT_YELLOW }
    else { ANSI_BRIGHT_GREEN };
    let num_fmt = format!("{fill_color}{BOLD}{pct_display}%{RESET}");

    let filled_grades = ((input.used_percentage * 80.0) / 100.0).round() as i32;
    let filled_grades = filled_grades.clamp(0, 80) as u32;
    let filled_chars = filled_grades / 8;
    let rem_grades = filled_grades % 8;

    let block_full = format!("{fill_color}█{RESET}");
    let block_empty = format!("{ANSI_GRAY}·{RESET}");
    const BLOCK_CHARS: [&str; 8] = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];

    let ctx_bar = if classic {
        let mut bar = String::with_capacity(20);
        for i in 0..10u32 {
            if i < filled_chars {
                bar.push('█');
            } else if i == filled_chars {
                let block_char = BLOCK_CHARS[rem_grades as usize];
                if block_char.is_empty() {
                    bar.push('·');
                } else {
                    bar.push_str(block_char);
                }
            } else {
                bar.push('·');
            }
        }
        format!("{ANSI_GRAY}ctx {fill_color}{bar} {num_fmt}")
    } else {
        let mut bar = String::with_capacity(80);
        for i in 0..10u32 {
            if i < filled_chars {
                bar.push_str(&block_full);
            } else if i == filled_chars {
                let block_char = BLOCK_CHARS[rem_grades as usize];
                if block_char.is_empty() {
                    bar.push_str(&block_empty);
                } else {
                    bar.push_str(&format!("{fill_color}{block_char}{RESET}"));
                }
            } else {
                bar.push_str(&block_empty);
            }
        }
        format!("{fill_color}{}  {RESET}{bar} {num_fmt}", icons.context_bar)
    };

    // Stats
    let art_str = if classic {
        format!("artifacts {BOLD}{}{RESET}", input.artifact_count)
    } else {
        format!("{} {BOLD}{}{RESET}", icons.artifacts, input.artifact_count)
    };
    let sub_str = if classic {
        format!("subagents {BOLD}{}{RESET}", input.subagent_count)
    } else {
        format!("{} {BOLD}{}{RESET}", icons.subagents, input.subagent_count)
    };
    let task_str = if classic {
        format!("tasks {BOLD}{}{RESET}", input.task_count)
    } else {
        format!("{} {BOLD}{}{RESET}", icons.tasks, input.task_count)
    };

    // Token details and context size
    let itf = human_format(input.total_input_tokens);
    let otf = human_format(input.total_output_tokens);
    let clf = human_format(input.context_window_size);
    let cuf = human_format(context_used);
    let tif = human_format(input.turn_input_tokens);
    let tof = human_format(input.turn_output_tokens);

    let (ctx_size, tok_details) = if context_used > 0 {
        let turn_info = if input.turn_input_tokens > 0 || input.turn_output_tokens > 0 {
            format!(" | turn: +{tif}/{tof}")
        } else {
            String::new()
        };
        let size_str = format!(" ({cuf}/{clf})");
        let details_str = if classic {
            format!("(total: {itf}/{otf}{turn_info})")
        } else {
            format!("{} (total: {itf}/{otf}{turn_info})", icons.token_sum)
        };
        (size_str, details_str)
    } else {
        (String::new(), String::new())
    };


    // Quota
    let quota_str = if (active_5h >= 0.0) || (active_weekly >= 0.0) {
        let bar_5h = build_quota_bar(active_5h, "5H", active_5h_reset, classic, icons.reset);
        let bar_7d = build_quota_bar(active_weekly, "7D", active_weekly_reset, classic, icons.reset);
        format!("{bar_5h}{dot_l2}{bar_7d}")
    } else {
        String::new()
    };

    View {
        state_str, model_str,
        vcs_str, art_str, sub_str, task_str, sandbox_str,
        ctx_bar, ctx_size, tok_details, quota_str,
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

    let dot_l1 = &icons.dot_l1;
    let dot_l2 = &icons.dot_l2;

    let line1 = format!(
        "{}{}{}",
        view.state_str,
        view.model_str, view.vcs_str,
    );
    let ctx_combined = if !view.ctx_bar.is_empty() {
        format!("{}{}", view.ctx_bar, view.ctx_size)
    } else {
        view.ctx_size.clone()
    };

    let mut right_parts = Vec::new();
    right_parts.push(view.art_str.as_str());
    right_parts.push(view.sub_str.as_str());
    right_parts.push(view.task_str.as_str());
    right_parts.push(view.sandbox_str.as_str());
    if !ctx_combined.is_empty() { right_parts.push(ctx_combined.as_str()); }
    if !view.quota_str.is_empty() { right_parts.push(view.quota_str.as_str()); }
    if !view.tok_details.is_empty() { right_parts.push(view.tok_details.as_str()); }

    let extra_str = if !right_parts.is_empty() {
        let joined = right_parts.join(dot_l2);
        format!("{dot_l1}{joined}")
    } else {
        String::new()
    };
    println!("{}{}", line1, extra_str);
}
