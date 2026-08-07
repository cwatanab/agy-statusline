// ─── ANSI Escape Codes ────────────────────────────────────────────────────

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

pub const ANSI_WHITE: &str = "\x1b[37m";
pub const ANSI_GRAY: &str = "\x1b[90m";
pub const ANSI_BRIGHT_RED: &str = "\x1b[91m";
pub const ANSI_BRIGHT_GREEN: &str = "\x1b[92m";
pub const ANSI_BRIGHT_YELLOW: &str = "\x1b[93m";
pub const ANSI_BRIGHT_CYAN: &str = "\x1b[96m";
pub const ANSI_BRIGHT_MAGENTA: &str = "\x1b[95m";
pub const ANSI_BRIGHT_BLUE: &str = "\x1b[94m";

// ─── Color Palette Structs ─────────────────────────────────────────────────

pub struct SegmentTheme {
    pub bg_ready: &'static str,
    pub fg_ready_text: &'static str,
    pub bg_thinking: &'static str,
    pub fg_thinking_text: &'static str,
    pub bg_working: &'static str,
    pub fg_working_text: &'static str,
    pub bg_tool: &'static str,
    pub fg_tool_text: &'static str,
    pub bg_unknown: &'static str,
    pub fg_unknown_text: &'static str,
    pub bg_git_clean: &'static str,
    pub fg_git_clean_text: &'static str,
    pub bg_git_dirty: &'static str,
    pub fg_git_dirty_text: &'static str,
    pub bg_model: &'static str,
    pub fg_model_text: &'static str,
    pub bg_dir: &'static str,
    pub fg_dir_text: &'static str,
    pub bg_meta: &'static str,
    pub fg_meta_text: &'static str,
}

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
    pub ac: &'static str,
    pub bat: &'static str,
    pub sys: &'static str,
    pub dir: &'static str,
    pub conv: &'static str,
    pub theme: SegmentTheme,
}

pub fn select_icons(classic: bool) -> Icons {
    if classic {
        Icons {
            dot_l1: format!("{ANSI_GRAY} ╱ {RESET}"),
            dot_l2: format!("{ANSI_GRAY} · {RESET}"),
            vcs: "╱",
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
            ac: "AC",
            bat: "BAT",
            sys: "sys",
            dir: "╱",
            conv: "╱",
            theme: SegmentTheme {
                bg_ready: ANSI_BRIGHT_GREEN,
                fg_ready_text: BOLD,
                bg_thinking: ANSI_BRIGHT_YELLOW,
                fg_thinking_text: BOLD,
                bg_working: ANSI_BRIGHT_CYAN,
                fg_working_text: BOLD,
                bg_tool: ANSI_BRIGHT_MAGENTA,
                fg_tool_text: BOLD,
                bg_unknown: ANSI_WHITE,
                fg_unknown_text: BOLD,
                bg_git_clean: ANSI_BRIGHT_BLUE,
                fg_git_clean_text: BOLD,
                bg_git_dirty: ANSI_BRIGHT_RED,
                fg_git_dirty_text: BOLD,
                bg_model: ANSI_BRIGHT_MAGENTA,
                fg_model_text: "",
                bg_dir: ANSI_BRIGHT_CYAN,
                fg_dir_text: "",
                bg_meta: ANSI_GRAY,
                fg_meta_text: "",
            },
        }
    } else {
        Icons {
            dot_l1: format!("{ANSI_GRAY} | {RESET}"),
            dot_l2: format!("{ANSI_GRAY} | {RESET}"),
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
            ac: "\u{F06A5}",
            bat: "🔋",
            sys: "\u{F4BC}",
            dir: "\u{F0A83}",
            conv: "\u{F036A}",
            theme: SegmentTheme {
                bg_ready: "\x1b[48;5;76m",
                fg_ready_text: "\x1b[38;5;232m\x1b[1m",
                bg_thinking: "\x1b[48;5;214m",
                fg_thinking_text: "\x1b[38;5;232m\x1b[1m",
                bg_working: "\x1b[48;5;37m",
                fg_working_text: "\x1b[38;5;232m\x1b[1m",
                bg_tool: "\x1b[48;5;135m",
                fg_tool_text: "\x1b[38;5;255m\x1b[1m",
                bg_unknown: "\x1b[48;5;244m",
                fg_unknown_text: "\x1b[38;5;255m\x1b[1m",
                bg_git_clean: "\x1b[48;5;33m",
                fg_git_clean_text: "\x1b[38;5;255m\x1b[1m",
                bg_git_dirty: "\x1b[48;5;197m",
                fg_git_dirty_text: "\x1b[38;5;255m\x1b[1m",
                bg_model: "\x1b[48;5;63m",
                fg_model_text: "\x1b[38;5;255m\x1b[1m",
                bg_dir: "\x1b[48;5;38m",
                fg_dir_text: "\x1b[38;5;232m\x1b[1m",
                bg_meta: "\x1b[48;5;236m",
                fg_meta_text: "\x1b[38;5;250m",
            },
        }
    }
}

pub fn with_icon(icon: &str, text: &str, classic: bool) -> String {
    if classic || icon.is_empty() {
        text.to_string()
    } else {
        format!("{icon} {text}")
    }
}

pub fn to_ansi_color(code: &str) -> &'static str {
    match code {
        "75" => ANSI_BRIGHT_BLUE,
        "37" => ANSI_BRIGHT_CYAN,
        "135" => ANSI_BRIGHT_MAGENTA,
        "76" => ANSI_BRIGHT_GREEN,
        "197" => ANSI_BRIGHT_RED,
        "214" => ANSI_BRIGHT_YELLOW,
        "244" => ANSI_GRAY,
        _ => "",
    }
}
