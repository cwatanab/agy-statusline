// ─── ANSI Escape Codes ────────────────────────────────────────────────────

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

pub const ANSI_WHITE: &str = "\x1b[37m";
pub const ANSI_GRAY: &str = "\x1b[90m";
pub const ANSI_BRIGHT_RED: &str = "\x1b[91m";
pub const ANSI_BRIGHT_GREEN: &str = "\x1b[92m";
pub const ANSI_BRIGHT_YELLOW: &str = "\x1b[93m";
pub const ANSI_BRIGHT_MAGENTA: &str = "\x1b[95m";
pub const ANSI_BRIGHT_CYAN: &str = "\x1b[96m";

// ─── Nerd Font Icons ──────────────────────────────────────────────────────

pub struct Icons {
    pub dot_l1: String,
    pub dot_l2: String,
    pub vcs: &'static str,
    pub model: &'static str,
    pub sandbox_net: &'static str,
    pub sandbox_nonet: &'static str,
    pub sandbox_off: &'static str,
    pub context_bar: &'static str,
    pub artifacts: &'static str,
    pub subagents: &'static str,
    pub tasks: &'static str,
    pub token_sum: &'static str,
    pub reset: &'static str,
    pub state_ready: &'static str,
    pub state_thinking: &'static str,
    pub state_working: &'static str,
    pub state_tool: &'static str,
    pub state_unknown: &'static str,
}

fn preformat(color: &str, text: &str) -> String {
    format!("{color}{text}{RESET}")
}

pub fn select_icons(classic: bool) -> Icons {
    if classic {
        Icons {
            dot_l1: preformat(ANSI_GRAY, " ╱ "),
            dot_l2: preformat(ANSI_GRAY, " · "),
            vcs: "",
            model: "",
            sandbox_net: "ON (net)",
            sandbox_nonet: "ON (no-net)",
            sandbox_off: "OFF",
            context_bar: "ctx",
            artifacts: "artifacts",
            subagents: "subagents",
            tasks: "tasks",
            token_sum: "",
            reset: "\u{231B}",
            state_ready: "●",
            state_thinking: "◆",
            state_working: "⚙",
            state_tool: "🔧",
            state_unknown: "\u{231B}",
        }
    } else {
        Icons {
            dot_l1: preformat(ANSI_GRAY, " | "),
            dot_l2: preformat(ANSI_GRAY, " | "),
            vcs: "\u{F418}",
            model: "\u{F400}",
            sandbox_net: "\u{F0499}",
            sandbox_nonet: "\u{F0D34}",
            sandbox_off: "\u{F099C}",
            context_bar: "\u{F134F}",
            artifacts: "\u{F0F6}",
            subagents: "\u{F167A}",
            tasks: "\u{F0AE}",
            token_sum: "\u{E26B}",
            reset: "\u{231B}\u{FE0F}",
            state_ready: "\u{F192}",
            state_thinking: "\u{F07F7}",
            state_working: "\u{F423}",
            state_tool: "\u{F425}",
            state_unknown: "\u{F252}",
        }
    }
}

/// Prefix `text` with icon when not classic and icon is non-empty.
pub fn with_icon(icon: &str, text: &str, classic: bool) -> String {
    if classic || icon.is_empty() {
        text.to_string()
    } else {
        format!("{icon} {text}")
    }
}
