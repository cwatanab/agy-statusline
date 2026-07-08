use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::io::{self, Read};
use std::process::Command;

const R: &str = "\x1b[0m";
const B: &str = "\x1b[1m";
const I: &str = "\x1b[3m";

const FG_RED: &str = "\x1b[31m";
const FG_GREEN: &str = "\x1b[32m";
const FG_YELLOW: &str = "\x1b[33m";
const FG_BLUE: &str = "\x1b[34m";
const FG_MAGENTA: &str = "\x1b[35m";
const FG_CYAN: &str = "\x1b[36m";
const FG_WHITE: &str = "\x1b[37m";

const FG_GRAY: &str = "\x1b[90m";
const FG_BRIGHT_RED: &str = "\x1b[91m";
const FG_BRIGHT_GREEN: &str = "\x1b[92m";
const FG_BRIGHT_YELLOW: &str = "\x1b[93m";
const FG_BRIGHT_BLUE: &str = "\x1b[94m";
const FG_BRIGHT_MAGENTA: &str = "\x1b[95m";
const FG_BRIGHT_CYAN: &str = "\x1b[96m";
const FG_BRIGHT_WHITE: &str = "\x1b[97m";

const NUM_COLOR: &str = "\x1b[97m\x1b[1m";

#[derive(Deserialize, Default)]
#[allow(dead_code)]
struct Input {
    agent_state: Option<String>,
    context_window: Option<ContextWindow>,
    vcs: Option<Vcs>,
    sandbox: Option<Sandbox>,
    artifact_count: Option<u32>,
    subagents: Option<Vec<serde_json::Value>>,
    task_count: Option<u32>,
    model: Option<Model>,
    terminal_width: Option<u32>,
    cwd: Option<String>,
    conversation_id: Option<String>,
    product: Option<String>,
    version: Option<String>,
    plan_tier: Option<String>,
    email: Option<String>,
    quota: Option<HashMap<String, QuotaEntry>>,
}

#[derive(Deserialize, Default)]
#[allow(dead_code)]
struct ContextWindow {
    used_percentage: Option<f64>,
    remaining_percentage: Option<f64>,
    total_input_tokens: Option<u64>,
    total_output_tokens: Option<u64>,
    context_window_size: Option<u64>,
    current_usage: Option<CurrentUsage>,
}

#[derive(Deserialize, Default)]
struct CurrentUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

#[derive(Deserialize, Default)]
struct Vcs {
    #[allow(dead_code)]
    branch: Option<String>,
    #[allow(dead_code)]
    dirty: Option<bool>,
    #[allow(dead_code)]
    r#type: Option<String>,
    #[allow(dead_code)]
    client: Option<String>,
}

#[derive(Deserialize, Default)]
struct Sandbox {
    enabled: Option<bool>,
    allow_network: Option<bool>,
}

#[derive(Deserialize, Default)]
struct Model {
    id: Option<String>,
    display_name: Option<String>,
}

#[derive(Deserialize, Default)]
struct QuotaEntry {
    remaining_fraction: Option<f64>,
    reset_in_seconds: Option<i64>,
}

fn human_format(num: u64) -> String {
    if num >= 1_000_000 {
        format!("{}.{}M", num / 1_000_000, (num % 1_000_000) / 100_000)
    } else if num >= 1000 {
        format!("{}.{}K", num / 1000, (num % 1000) / 100)
    } else {
        format!("{}", num)
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
    let mins = rem / 60;

    if days > 0 {
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    } else if hours > 0 {
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    } else if mins > 0 {
        format!("{}m", mins)
    } else {
        "<1m".to_string()
    }
}

fn shorten_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let home = env::var("HOME").unwrap_or_default();
    let path = if !home.is_empty() && path.starts_with(&home) {
        path.replacen(&home, "~", 1)
    } else {
        path.to_string()
    };
    if path.len() > 25 {
        let basename = std::path::Path::new(path.as_str())
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        format!("...{}", basename)
    } else {
        path
    }
}

fn visible_len(s: &str) -> usize {
    let mut count = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some(&'[') = chars.peek() {
                chars.next();
                while let Some(&d) = chars.peek() {
                    chars.next();
                    if d == 'm' {
                        break;
                    }
                }
            }
        } else {
            count += 1;
        }
    }
    count
}

fn hostname() -> String {
    Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn tailscale_ip() -> String {
    Command::new("ip")
        .args(["-4", "addr", "show", "dev", "tailscale0"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.lines()
                .find_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.starts_with("inet ") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if parts.len() >= 2 {
                            Some(parts[1].split('/').next().unwrap_or("").to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
        })
        .unwrap_or_default()
}

fn power_status() -> (bool, Option<u8>) {
    let ac_on = std::fs::read_to_string("/sys/class/power_supply/ACAD/online")
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(1);
    let bat_cap = std::fs::read_to_string("/sys/class/power_supply/BAT1/capacity")
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok());

    if ac_on == 0 {
        (true, bat_cap)
    } else {
        (false, None)
    }
}

fn git_info(cwd: &str) -> (String, String, bool) {
    let git_dir = if cwd.is_empty() { "." } else { cwd };

    let branch = Command::new("git")
        .args(["-C", git_dir, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if branch.is_empty() {
        return (String::new(), String::new(), false);
    }

    let dirty = Command::new("git")
        .args(["-C", git_dir, "status", "--porcelain"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    ("git".to_string(), branch, dirty)
}

fn make_quota_bar(
    val: f64,
    label: &str,
    bar_color: &str,
    reset_sec: i64,
    use_classic: bool,
    icon_reset: &str,
) -> String {
    let separator = if use_classic {
        format!("{FG_GRAY} · {R}")
    } else {
        format!("{FG_GRAY}| {R}")
    };

    if val < -0.5 {
        let bar: String = (0..20)
            .map(|_| if use_classic { "·" } else { "░" })
            .collect();
        return format!("{separator}{FG_BRIGHT_WHITE}{B}{label}{R} {FG_GRAY}{bar} N/A{R}");
    }

    let val_int = val as u32;
    let text_color = if val_int < 20 {
        FG_BRIGHT_RED
    } else if val_int < 50 {
        FG_BRIGHT_YELLOW
    } else {
        FG_BRIGHT_GREEN
    };

    let bar_len: u32 = 20;
    let filled = val_int * bar_len / 100;
    let remainder = (val_int * bar_len) % 100;

    let mut bar = String::with_capacity(bar_len as usize * 20);
    for i in 0..bar_len {
        if i < filled {
            if use_classic {
                bar.push('█');
            } else {
                bar.push_str(&format!("{bar_color}█{R}"));
            }
        } else if i == filled {
            if use_classic {
                bar.push_str(if remainder >= 75 {
                    "▓"
                } else if remainder >= 50 {
                    "▒"
                } else if remainder >= 25 {
                    "░"
                } else {
                    "·"
                });
            } else if remainder >= 75 {
                bar.push_str(&format!("{bar_color}▓{R}{FG_GRAY}"));
            } else if remainder >= 50 {
                bar.push_str(&format!("{bar_color}▒{R}{FG_GRAY}"));
            } else if remainder >= 25 {
                bar.push_str(&format!("{bar_color}░{R}{FG_GRAY}"));
            } else {
                bar.push_str(&format!("{FG_GRAY}░{R}"));
            }
        } else if use_classic {
            bar.push('·');
        } else {
            bar.push_str(&format!("{FG_GRAY}░{R}"));
        }
    }

    let mut reset_str = String::new();
    if reset_sec > 0 {
        reset_str = format!(" {icon_reset} {}", format_reset_time(reset_sec));
    }

    if use_classic {
        format!(
            "{separator}{FG_BRIGHT_WHITE}{B}{label}{R} {bar_color}{bar}{R} {text_color}{val_int}%{R}{reset_str}"
        )
    } else {
        format!(
            "{separator}{FG_BRIGHT_WHITE}{B}{label}{R} {bar} {text_color}{val_int}%{R}{reset_str}"
        )
    }
}

fn print_right_aligned(left: &str, right: &str, total_cols: u32) {
    let left_vis = visible_len(left);
    let right_vis = visible_len(right);
    let pad = if total_cols as usize > left_vis + right_vis {
        total_cols as usize - left_vis - right_vis
    } else {
        1
    };
    print!("{}", left);
    for _ in 0..pad {
        print!(" ");
    }
    println!("{}", right);
}

fn main() {
    let use_classic = env::args().any(|a| {
        a == "--classic" || a == "--no-nerdfont" || a == "--compatibility"
    });

    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).unwrap();

    let input: Input = serde_json::from_str(&stdin).unwrap_or_default();

    let state = input.agent_state.unwrap_or_else(|| "idle".into());
    let used_pct = input
        .context_window
        .as_ref()
        .and_then(|c| c.used_percentage)
        .unwrap_or(0.0);

    let sandbox_on = input
        .sandbox
        .as_ref()
        .and_then(|s| s.enabled)
        .unwrap_or(false);
    let sandbox_net = input
        .sandbox
        .as_ref()
        .and_then(|s| s.allow_network)
        .unwrap_or(false);

    let artifacts = input.artifact_count.unwrap_or(0);
    let subagents = input
        .subagents
        .as_ref()
        .map(|a| a.len() as u32)
        .unwrap_or(0);
    let bg_tasks = input.task_count.unwrap_or(0);

    let model_id = input
        .model
        .as_ref()
        .and_then(|m| m.id.clone())
        .unwrap_or_default();
    let model_name = input
        .model
        .as_ref()
        .and_then(|m| m.display_name.clone())
        .unwrap_or_default();
    let model_disp = if !model_name.is_empty() {
        model_name
    } else {
        model_id
    };

    let cols = input.terminal_width.unwrap_or(80);
    let cwd = input.cwd.unwrap_or_default();
    let conv_id = input.conversation_id.unwrap_or_default();
    let cli_version = input.version.unwrap_or_default();
    let plan_tier = input.plan_tier.unwrap_or_default();
    let user_email = input.email.unwrap_or_default();

    let input_tokens = input
        .context_window
        .as_ref()
        .and_then(|c| c.total_input_tokens)
        .unwrap_or(0);
    let output_tokens = input
        .context_window
        .as_ref()
        .and_then(|c| c.total_output_tokens)
        .unwrap_or(0);
    let ctx_limit = input
        .context_window
        .as_ref()
        .and_then(|c| c.context_window_size)
        .unwrap_or(0);
    let ctx_used = input_tokens + output_tokens;

    let turn_input = input
        .context_window
        .as_ref()
        .and_then(|c| c.current_usage.as_ref())
        .and_then(|u| u.input_tokens)
        .unwrap_or(0);
    let turn_output = input
        .context_window
        .as_ref()
        .and_then(|c| c.current_usage.as_ref())
        .and_then(|u| u.output_tokens)
        .unwrap_or(0);

    let quota_entries = input.quota.unwrap_or_default();
    let gemini_5h = quota_entries
        .get("gemini-5h")
        .and_then(|q| q.remaining_fraction)
        .map(|v| (v * 1000.0).round() / 10.0)
        .unwrap_or(-1.0);
    let gemini_wk = quota_entries
        .get("gemini-weekly")
        .and_then(|q| q.remaining_fraction)
        .map(|v| (v * 1000.0).round() / 10.0)
        .unwrap_or(-1.0);
    let tp_5h = quota_entries
        .get("3p-5h")
        .and_then(|q| q.remaining_fraction)
        .map(|v| (v * 1000.0).round() / 10.0)
        .unwrap_or(-1.0);
    let tp_wk = quota_entries
        .get("3p-weekly")
        .and_then(|q| q.remaining_fraction)
        .map(|v| (v * 1000.0).round() / 10.0)
        .unwrap_or(-1.0);

    let gemini_5h_r = quota_entries
        .get("gemini-5h")
        .and_then(|q| q.reset_in_seconds)
        .unwrap_or(-1);
    let gemini_wk_r = quota_entries
        .get("gemini-weekly")
        .and_then(|q| q.reset_in_seconds)
        .unwrap_or(-1);
    let tp_5h_r = quota_entries
        .get("3p-5h")
        .and_then(|q| q.reset_in_seconds)
        .unwrap_or(-1);
    let tp_wk_r = quota_entries
        .get("3p-weekly")
        .and_then(|q| q.reset_in_seconds)
        .unwrap_or(-1);

    let (q_5h, q_wk, q_5h_r, q_wk_r) =
        if (gemini_5h >= 0.0) || (gemini_wk >= 0.0) {
            (gemini_5h, gemini_wk, gemini_5h_r, gemini_wk_r)
        } else if (tp_5h >= 0.0) || (tp_wk >= 0.0) {
            (tp_5h, tp_wk, tp_5h_r, tp_wk_r)
        } else {
            (-1.0, -1.0, -1, -1)
        };

    // ─── Icons ────────────────────────────────────────────────────────────────
    let (
        dot_l1, dot_l2, icon_vcs, icon_model, icon_sandbox_net, icon_sandbox_nonet, icon_sandbox_off,
        icon_ctx_bar, icon_artifacts, icon_subagents, icon_tasks, icon_dir, icon_conv, icon_tok_sum,
        icon_reset, icon_ac, icon_bat,
    ) = if use_classic {
        (
            format!("{FG_GRAY} ╱ {R}"),
            format!("{FG_GRAY} · {R}"),
            "",
            "",
            "ON (net)",
            "ON (no-net)",
            "OFF",
            "ctx",
            "artifacts",
            "subagents",
            "tasks",
            "",
            "",
            "",
            "\u{231B}",
            "AC",
            "BAT",
        )
    } else {
        (
            format!("{FG_GRAY} | {R}"),
            format!("{FG_GRAY} | {R}"),
            "\u{F418}",
            "\u{F400}",
            "\u{F0499}",
            "\u{F0D34}",
            "\u{F099C}",
            "\u{F134F}",
            "\u{F0F6}",
            "\u{F167A}",
            "\u{F0AE}",
            "\u{EA83}",
            "\u{F036A}",
            "\u{E26B}",
            "\u{231B}\u{FE0F}",
            "\u{F06A5}",
            "\u{1F50B}",
        )
    };

    // ─── Human formatted tokens ───────────────────────────────────────────────
    let input_tok_fmt = human_format(input_tokens);
    let output_tok_fmt = human_format(output_tokens);
    let ctx_limit_fmt = human_format(ctx_limit);
    let ctx_used_fmt = human_format(ctx_used);
    let turn_input_fmt = human_format(turn_input);
    let turn_output_fmt = human_format(turn_output);

    // ─── Computed values ──────────────────────────────────────────────────────
    let pct_int = used_pct as u32;
    let pct_fmt = format!("{:.1}", used_pct);

    // ─── CLI Version ─────────────────────────────────────────────────────────
    let cli_ver_fmt = if cli_version.is_empty() {
        String::new()
    } else {
        format!("{dot_l1}{FG_GRAY}v{cli_version}{R}")
    };

    // ─── User info ────────────────────────────────────────────────────────────
    let user_fmt = if !plan_tier.is_empty() || !user_email.is_empty() {
        let user_info = if !plan_tier.is_empty() && !user_email.is_empty() {
            format!("{} ({})", plan_tier, user_email)
        } else if !plan_tier.is_empty() {
            plan_tier.clone()
        } else {
            user_email.clone()
        };
        let truncated = if user_info.len() > 35 {
            format!("{}...", &user_info[..32])
        } else {
            user_info
        };
        if use_classic {
            format!("{dot_l1}{FG_GRAY}{truncated}{R}")
        } else {
            format!("{dot_l1}{FG_GRAY}\u{F01EE} {truncated}{R}")
        }
    } else {
        String::new()
    };

    // ─── Host info ────────────────────────────────────────────────────────────
    let host_name = hostname();
    let ts_ip = tailscale_ip();
    let host_fmt = if !host_name.is_empty() {
        let host_details = if !ts_ip.is_empty() {
            format!("{} ({})", host_name, ts_ip)
        } else {
            host_name
        };
        if use_classic {
            format!("{dot_l1}{FG_BRIGHT_BLUE}{host_details}{R}")
        } else {
            format!("{dot_l1}{FG_BRIGHT_BLUE}\u{F048B} {host_details}{R}")
        }
    } else {
        String::new()
    };

    // ─── Power status ─────────────────────────────────────────────────────────
    let power_fmt = {
        let (on_battery, bat_cap) = power_status();
        if on_battery {
            if let Some(cap) = bat_cap {
                if use_classic {
                    format!("{dot_l2}{FG_BRIGHT_YELLOW}{icon_bat}:{}%{R}", cap)
                } else {
                    format!("{dot_l2}{FG_BRIGHT_YELLOW}{icon_bat} {}%{R}", cap)
                }
            } else {
                format!("{dot_l2}{FG_BRIGHT_YELLOW}{icon_bat}{R}")
            }
        } else {
            if use_classic {
                format!("{dot_l2}{FG_GREEN}{icon_ac}{R}")
            } else {
                format!("{dot_l2}{FG_GREEN}{icon_ac} AC{R}")
            }
        }
    };

    // ─── CWD ──────────────────────────────────────────────────────────────────
    let cwd_short = shorten_path(&cwd);
    let dir_fmt = if !cwd_short.is_empty() {
        if use_classic {
            format!("{dot_l1}{FG_CYAN}{cwd_short}{R}")
        } else {
            format!("{dot_l1}{FG_CYAN}{icon_dir} {cwd_short}{R}")
        }
    } else {
        String::new()
    };

    // ─── Conversation ID ──────────────────────────────────────────────────────
    let conv_fmt = if !conv_id.is_empty() {
        let len = 8.min(conv_id.len());
        if use_classic {
            format!("{dot_l1}{FG_GRAY}{}{R}", &conv_id[..len])
        } else {
            format!("{dot_l1}{FG_GRAY}{icon_conv} {}{R}", &conv_id[..len])
        }
    } else {
        String::new()
    };

    // ─── State Indicator ──────────────────────────────────────────────────────
    let (icon_ready, icon_thinking, icon_working, icon_tool, icon_unknown) = if use_classic {
        ("●", "◆", "⚙", "🔧", "\u{231B}")
    } else {
        ("\u{F192}", "\u{F07F7}", "\u{F423}", "\u{F425}", "\u{F252}")
    };

    let state_str = match state.as_str() {
        "idle" => format!("{FG_BRIGHT_GREEN}{B} {icon_ready} READY{R}"),
        "thinking" => format!("{FG_BRIGHT_YELLOW}{B} {icon_thinking} THINKING{R}"),
        "working" => format!("{FG_BRIGHT_CYAN}{B} {icon_working} WORKING{R}"),
        "tool_use" => format!("{FG_BRIGHT_MAGENTA}{B} {icon_tool} TOOL{R}"),
        other => format!(
            "{FG_WHITE}{B} {icon_unknown} {}{R}",
            other.to_uppercase()
        ),
    };

    // ─── VCS (from git directly) ──────────────────────────────────────────────
    let (vcs_type, vcs_branch, vcs_dirty) = git_info(&cwd);
    let _vcs_type = vcs_type;
    let vcs = if vcs_branch.is_empty() {
        String::new()
    } else if vcs_dirty {
        if use_classic {
            format!("{dot_l1}{FG_BRIGHT_RED}{vcs_branch}{FG_BRIGHT_YELLOW}*{R}")
        } else {
            format!(
                "{dot_l1}{R}{FG_BRIGHT_RED}{icon_vcs} {vcs_branch}{FG_BRIGHT_YELLOW}*{R}"
            )
        }
    } else if use_classic {
        format!("{dot_l1}{FG_BRIGHT_BLUE}{vcs_branch}{R}")
    } else {
        format!("{dot_l1}{R}{FG_BRIGHT_BLUE}{icon_vcs} {vcs_branch}{R}")
    };

    // ─── Model ────────────────────────────────────────────────────────────────
    let model_fmt = if !model_disp.is_empty() {
        if use_classic {
            format!("{dot_l1}{FG_BRIGHT_MAGENTA}{I}{model_disp}{R}")
        } else {
            format!("{dot_l1}{FG_BRIGHT_MAGENTA}{I}{icon_model} {model_disp}{R}")
        }
    } else {
        String::new()
    };

    // ─── Sandbox ──────────────────────────────────────────────────────────────
    let sandbox = if sandbox_on {
        if sandbox_net {
            format!("{FG_GREEN}{icon_sandbox_net} ON (net){R}")
        } else {
            format!("{FG_GREEN}{icon_sandbox_nonet} ON (no-net){R}")
        }
    } else if use_classic {
        format!("{FG_GRAY}sandbox off{R}")
    } else {
        format!("{FG_RED}{icon_sandbox_off} OFF{R}")
    };

    // ─── Context Bar (20 segments) ────────────────────────────────────────────
    let bar_len: u32 = 20;
    let filled = pct_int * bar_len / 100;
    let remainder = (pct_int * bar_len) % 100;

    let fill_color = if pct_int >= 90 {
        FG_BRIGHT_RED
    } else if pct_int >= 60 {
        FG_BRIGHT_YELLOW
    } else {
        FG_YELLOW
    };

    let ctx_bar = if use_classic {
        let mut bar = String::with_capacity(bar_len as usize * 4);
        for i in 0..bar_len {
            if i < filled {
                bar.push('█');
            } else if i == filled {
                bar.push_str(if remainder >= 75 {
                    "▓"
                } else if remainder >= 50 {
                    "▒"
                } else if remainder >= 25 {
                    "░"
                } else {
                    "·"
                });
            } else {
                bar.push('·');
            }
        }
        format!("{FG_GRAY}ctx {fill_color}{bar} {NUM_COLOR}{pct_fmt}%{R}")
    } else {
        let mut bar = String::with_capacity(bar_len as usize * 20);
        for i in 0..bar_len {
            if i < filled {
                bar.push_str(&format!("{fill_color}█{R}"));
            } else if i == filled {
                if remainder >= 75 {
                    bar.push_str(&format!("{fill_color}▓{R}{FG_GRAY}"));
                } else if remainder >= 50 {
                    bar.push_str(&format!("{fill_color}▒{R}{FG_GRAY}"));
                } else {
                    bar.push_str(&format!("{fill_color}░{R}{FG_GRAY}"));
                }
            } else {
                bar.push_str(&format!("{FG_GRAY}░{R}"));
            }
        }
        format!("{FG_YELLOW}{icon_ctx_bar}  {R}{bar} {NUM_COLOR}{pct_fmt}%{R}")
    };

    // ─── Stats ────────────────────────────────────────────────────────────────
    let art_fmt = if use_classic {
        format!("{FG_GRAY}artifacts {NUM_COLOR}{artifacts}{R}")
    } else {
        format!("{FG_BLUE}{icon_artifacts} {NUM_COLOR}{artifacts}{R}")
    };
    let sub_fmt = if use_classic {
        format!("{FG_GRAY}subagents {NUM_COLOR}{subagents}{R}")
    } else {
        format!("{FG_CYAN}{icon_subagents} {NUM_COLOR}{subagents}{R}")
    };
    let bg_fmt = if use_classic {
        format!("{FG_GRAY}tasks {NUM_COLOR}{bg_tasks}{R}")
    } else {
        format!("{FG_MAGENTA}{icon_tasks} {NUM_COLOR}{bg_tasks}{R}")
    };

    // ─── Token Details ────────────────────────────────────────────────────────
    let tok_details_wide = if ctx_used > 0 {
        let turn_str = if turn_input > 0 || turn_output > 0 {
            format!(" | turn: +{}/{}", turn_input_fmt, turn_output_fmt)
        } else {
            String::new()
        };
        if use_classic {
            format!(
                " ({}/{}){dot_l2}(total: {}/{}{})",
                ctx_used_fmt, ctx_limit_fmt, input_tok_fmt, output_tok_fmt, turn_str
            )
        } else {
            format!(
                " ({}/{}){dot_l2}{FG_YELLOW}{icon_tok_sum} {R} (total: {}/{}{})",
                ctx_used_fmt, ctx_limit_fmt, input_tok_fmt, output_tok_fmt, turn_str
            )
        }
    } else {
        String::new()
    };

    let tok_details_med = if ctx_used > 0 {
        format!(" ({}/{})", ctx_used_fmt, ctx_limit_fmt)
    } else {
        String::new()
    };

    // ─── Quota formatting ─────────────────────────────────────────────────────
    let quota_fmt = if (q_5h >= 0.0) || (q_wk >= 0.0) {
        format!(
            "{} {}",
            make_quota_bar(q_5h, "5H", FG_BRIGHT_CYAN, q_5h_r, use_classic, icon_reset),
            make_quota_bar(q_wk, "7D", FG_BRIGHT_MAGENTA, q_wk_r, use_classic, icon_reset)
        )
    } else {
        String::new()
    };

    // ─── Output Assembly ──────────────────────────────────────────────────────
    if cols >= 180 {
        let line1 = format!(
            "{state_str}{cli_ver_fmt}{user_fmt}{host_fmt}{model_fmt}{dir_fmt}{vcs}{conv_fmt}"
        );
        let line2 = if !quota_fmt.is_empty() {
            format!(
                "{art_fmt}{dot_l2}{sub_fmt}{dot_l2}{bg_fmt}{dot_l2}{sandbox}{dot_l2}{ctx_bar}{tok_details_wide}{quota_fmt}{power_fmt}"
            )
        } else {
            format!(
                "{art_fmt}{dot_l2}{sub_fmt}{dot_l2}{bg_fmt}{dot_l2}{sandbox}{dot_l2}{ctx_bar}{tok_details_wide}{power_fmt}"
            )
        };
        print_right_aligned(&line1, &line2, cols);
    } else if cols >= 90 {
        let line1 = format!(
            "{state_str}{cli_ver_fmt}{user_fmt}{host_fmt}{model_fmt}{dir_fmt}{vcs}"
        );
        let line2 = if !quota_fmt.is_empty() {
            format!(
                " {ctx_bar}{tok_details_med}{dot_l2}{art_fmt}{dot_l2}{sub_fmt}{dot_l2}{bg_fmt}{dot_l2}{sandbox}{quota_fmt}{power_fmt}"
            )
        } else {
            format!(
                " {ctx_bar}{tok_details_med}{dot_l2}{art_fmt}{dot_l2}{sub_fmt}{dot_l2}{bg_fmt}{dot_l2}{sandbox}{power_fmt}"
            )
        };
        println!("{FG_GRAY}╭─{R}{line1}");
        println!("{FG_GRAY}╰─{R}{line2}");
    } else {
        let model_short = if !model_disp.is_empty() {
            let max_len = 12.min(model_disp.len());
            if use_classic {
                format!(
                    "{FG_GRAY} ╱ {FG_BRIGHT_MAGENTA}{}{R}",
                    &model_disp[..max_len]
                )
            } else {
                format!(
                    "{FG_GRAY} ╱ {FG_BRIGHT_MAGENTA}{icon_model} {}{R}",
                    &model_disp[..max_len]
                )
            }
        } else {
            String::new()
        };
        println!("{state_str}{model_short}");
        if !quota_fmt.is_empty() {
            println!("{ctx_bar}{dot_l2}{bg_fmt}{quota_fmt}{power_fmt}");
        } else {
            println!("{ctx_bar}{dot_l2}{bg_fmt}{power_fmt}");
        }
    }
}
