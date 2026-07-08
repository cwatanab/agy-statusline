use std::env;
use std::io::{self, Read};
use std::process::Command;

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

// ─── Parsed Input ─────────────────────────────────────────────────────────

struct ParsedInput {
    agent_state: String,
    used_percentage: f64,
    sandbox_enabled: bool,
    sandbox_allow_network: bool,
    artifact_count: u32,
    subagent_count: u32,
    task_count: u32,
    model_id: String,
    model_display_name: String,
    terminal_width: u32,
    working_dir: String,
    conversation_id: String,
    version: String,
    plan_tier: String,
    email: String,
    total_input_tokens: u64,
    total_output_tokens: u64,
    context_window_size: u64,
    turn_input_tokens: u64,
    turn_output_tokens: u64,
    gemini_5h_pct: f64,
    gemini_weekly_pct: f64,
    third_party_5h_pct: f64,
    third_party_weekly_pct: f64,
    gemini_5h_reset: i64,
    gemini_weekly_reset: i64,
    third_party_5h_reset: i64,
    third_party_weekly_reset: i64,
}

// ─── JSON Parser ──────────────────────────────────────────────────────────

struct JsonParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        JsonParser { bytes: input.as_bytes(), pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn is_null(&mut self) -> bool {
        self.skip_whitespace();
        if self.bytes.get(self.pos..self.pos + 4) == Some(b"null") {
            self.pos += 4;
            true
        } else {
            false
        }
    }

    fn read_string(&mut self) -> &'a str {
        self.skip_whitespace();
        self.advance(); // skip opening quote
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'"' {
            if self.bytes[self.pos] == b'\\' {
                self.pos += 1;
            }
            self.pos += 1;
        }
        let end = self.pos;
        self.advance(); // skip closing quote
        std::str::from_utf8(&self.bytes[start..end]).unwrap_or("")
    }

    fn read_f64(&mut self) -> f64 {
        self.read_number_str().parse().unwrap_or(0.0)
    }

    fn read_u64(&mut self) -> u64 {
        self.read_number_str().parse().unwrap_or(0)
    }

    fn read_u32(&mut self) -> u32 {
        self.read_number_str().parse().unwrap_or(0)
    }

    fn read_i64(&mut self) -> i64 {
        self.read_number_str().parse().unwrap_or(-1)
    }

    fn read_number_str(&mut self) -> &'a str {
        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'-' | b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' => self.pos += 1,
                _ => break,
            }
        }
        std::str::from_utf8(&self.bytes[start..self.pos]).unwrap_or("0")
    }

    fn read_bool(&mut self) -> bool {
        self.skip_whitespace();
        if self.bytes[self.pos] == b't' {
            self.pos += 4;
            true
        } else {
            self.pos += 5;
            false
        }
    }

    fn read_array_len(&mut self) -> u32 {
        self.skip_whitespace();
        self.advance(); // skip '['
        let mut count = 0;
        loop {
            self.skip_whitespace();
            if self.peek() == Some(b']') {
                self.advance();
                return count;
            }
            self.skip_value();
            count += 1;
            self.skip_whitespace();
            if self.peek() == Some(b',') {
                self.advance();
            }
        }
    }

    fn skip_value(&mut self) {
        self.skip_whitespace();
        match self.peek() {
            Some(b'"') => {
                self.advance();
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'"' {
                    if self.bytes[self.pos] == b'\\' {
                        self.pos += 1;
                    }
                    self.pos += 1;
                }
                self.advance();
            }
            Some(b'{') => { self.advance(); self.skip_object(); }
            Some(b'[') => { self.advance(); self.skip_array(); }
            Some(b't') => { self.pos += 4; }
            Some(b'f') => { self.pos += 5; }
            Some(b'n') => { self.pos += 4; }
            Some(b'-' | b'0'..=b'9') => { self.read_number_str(); }
            _ => {}
        }
    }

    fn skip_object(&mut self) {
        loop {
            self.skip_whitespace();
            if self.peek() == Some(b'}') {
                self.advance();
                return;
            }
            self.skip_value(); // key
            self.skip_whitespace();
            if self.peek() == Some(b':') {
                self.advance();
                self.skip_value(); // value
            }
            self.skip_whitespace();
            if self.peek() == Some(b',') {
                self.advance();
            }
        }
    }

    fn skip_array(&mut self) {
        loop {
            self.skip_whitespace();
            if self.peek() == Some(b']') {
                self.advance();
                return;
            }
            self.skip_value();
            self.skip_whitespace();
            if self.peek() == Some(b',') {
                self.advance();
            }
        }
    }
}

fn parse_input(json: &str) -> ParsedInput {
    let mut p = JsonParser::new(json);
    p.skip_whitespace();
    if p.peek() != Some(b'{') {
        return ParsedInput::default();
    }
    p.advance();

    let mut input = ParsedInput::default();

    loop {
        p.skip_whitespace();
        match p.peek() {
            None => break,
            Some(b'}') => break,
            Some(b'"') => {
                let key = p.read_string();
                p.skip_whitespace();
                if p.peek() == Some(b':') {
                    p.advance();
                } else {
                    continue;
                }
                parse_field(&mut p, &mut input, key);
                p.skip_whitespace();
                if p.peek() == Some(b',') {
                    p.advance();
                }
            }
            _ => {
                p.skip_value();
                p.skip_whitespace();
                if p.peek() == Some(b',') {
                    p.advance();
                }
            }
        }
    }

    input
}

fn parse_field(p: &mut JsonParser, input: &mut ParsedInput, key: &str) {
    match key {
        "agent_state" => {
            if !p.is_null() { input.agent_state = p.read_string().to_string(); }
        }
        "artifact_count" => input.artifact_count = p.read_u32(),
        "task_count" => input.task_count = p.read_u32(),
        "terminal_width" => input.terminal_width = p.read_u32(),
        "cwd" => {
            if !p.is_null() { input.working_dir = p.read_string().to_string(); }
        }
        "conversation_id" => {
            if !p.is_null() { input.conversation_id = p.read_string().to_string(); }
        }
        "version" => {
            if !p.is_null() { input.version = p.read_string().to_string(); }
        }
        "plan_tier" => {
            if !p.is_null() { input.plan_tier = p.read_string().to_string(); }
        }
        "email" => {
            if !p.is_null() { input.email = p.read_string().to_string(); }
        }
        "subagents" => {
            if p.is_null() {
                input.subagent_count = 0;
            } else {
                input.subagent_count = p.read_array_len();
            }
        }
        "context_window" => parse_context_window(p, input),
        "sandbox" => parse_sandbox(p, input),
        "model" => parse_model(p, input),
        "quota" => parse_quota(p, input),
        _ => { p.skip_value(); }
    }
}

fn parse_context_window(p: &mut JsonParser, input: &mut ParsedInput) {
    p.skip_whitespace();
    p.advance(); // skip '{'
    loop {
        p.skip_whitespace();
        if p.peek() == Some(b'}') { p.advance(); break; }
        let key = p.read_string();
        p.skip_whitespace();
        if p.peek() == Some(b':') { p.advance(); }
        match key {
            "used_percentage" => input.used_percentage = p.read_f64(),
            "total_input_tokens" => input.total_input_tokens = p.read_u64(),
            "total_output_tokens" => input.total_output_tokens = p.read_u64(),
            "context_window_size" => input.context_window_size = p.read_u64(),
            "current_usage" => {
                p.skip_whitespace();
                p.advance(); // skip '{'
                loop {
                    p.skip_whitespace();
                    if p.peek() == Some(b'}') { p.advance(); break; }
                    match p.read_string() {
                        "input_tokens" => { p.skip_whitespace(); p.advance(); input.turn_input_tokens = p.read_u64(); }
                        "output_tokens" => { p.skip_whitespace(); p.advance(); input.turn_output_tokens = p.read_u64(); }
                        _ => { p.skip_whitespace(); p.advance(); p.skip_value(); }
                    }
                    p.skip_whitespace();
                    if p.peek() == Some(b',') { p.advance(); }
                }
            }
            _ => { p.skip_value(); }
        }
        p.skip_whitespace();
        if p.peek() == Some(b',') { p.advance(); }
    }
}

fn parse_sandbox(p: &mut JsonParser, input: &mut ParsedInput) {
    p.skip_whitespace();
    p.advance(); // skip '{'
    loop {
        p.skip_whitespace();
        if p.peek() == Some(b'}') { p.advance(); break; }
        match p.read_string() {
            "enabled" => { p.skip_whitespace(); p.advance(); input.sandbox_enabled = p.read_bool(); }
            "allow_network" => { p.skip_whitespace(); p.advance(); input.sandbox_allow_network = p.read_bool(); }
            _ => { p.skip_whitespace(); p.advance(); p.skip_value(); }
        }
        p.skip_whitespace();
        if p.peek() == Some(b',') { p.advance(); }
    }
}

fn parse_model(p: &mut JsonParser, input: &mut ParsedInput) {
    p.skip_whitespace();
    p.advance(); // skip '{'
    loop {
        p.skip_whitespace();
        if p.peek() == Some(b'}') { p.advance(); break; }
        match p.read_string() {
            "id" => { p.skip_whitespace(); p.advance(); if !p.is_null() { input.model_id = p.read_string().to_string(); } }
            "display_name" => { p.skip_whitespace(); p.advance(); if !p.is_null() { input.model_display_name = p.read_string().to_string(); } }
            _ => { p.skip_whitespace(); p.advance(); p.skip_value(); }
        }
        p.skip_whitespace();
        if p.peek() == Some(b',') { p.advance(); }
    }
}

fn parse_quota(p: &mut JsonParser, input: &mut ParsedInput) {
    p.skip_whitespace();
    p.advance(); // skip '{'
    loop {
        p.skip_whitespace();
        if p.peek() == Some(b'}') { p.advance(); break; }
        let quota_key = p.read_string().to_string();
        p.skip_whitespace();
        p.advance(); // skip ':'
        p.skip_whitespace();
        p.advance(); // skip '{'
        let mut fraction = -1.0f64;
        let mut reset_sec = -1i64;
        loop {
            p.skip_whitespace();
            if p.peek() == Some(b'}') { p.advance(); break; }
            let entry_key = p.read_string();
            p.skip_whitespace();
            p.advance(); // skip ':'
            match entry_key {
                "remaining_fraction" => { if !p.is_null() { fraction = p.read_f64(); } }
                "reset_in_seconds" => { if !p.is_null() { reset_sec = p.read_i64(); } }
                _ => { p.skip_value(); }
            }
            p.skip_whitespace();
            if p.peek() == Some(b',') { p.advance(); }
        }
        let pct = if fraction >= 0.0 { (fraction * 1000.0).round() / 10.0 } else { -1.0 };
        match quota_key.as_str() {
            "gemini-5h" => { input.gemini_5h_pct = pct; input.gemini_5h_reset = reset_sec; }
            "gemini-weekly" => { input.gemini_weekly_pct = pct; input.gemini_weekly_reset = reset_sec; }
            "3p-5h" => { input.third_party_5h_pct = pct; input.third_party_5h_reset = reset_sec; }
            "3p-weekly" => { input.third_party_weekly_pct = pct; input.third_party_weekly_reset = reset_sec; }
            _ => {}
        }
        p.skip_whitespace();
        if p.peek() == Some(b',') { p.advance(); }
    }
}

impl Default for ParsedInput {
    fn default() -> Self {
        ParsedInput {
            agent_state: "idle".into(),
            used_percentage: 0.0,
            sandbox_enabled: false,
            sandbox_allow_network: false,
            artifact_count: 0,
            subagent_count: 0,
            task_count: 0,
            model_id: String::new(),
            model_display_name: String::new(),
            terminal_width: 80,
            working_dir: String::new(),
            conversation_id: String::new(),
            version: String::new(),
            plan_tier: String::new(),
            email: String::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            context_window_size: 0,
            turn_input_tokens: 0,
            turn_output_tokens: 0,
            gemini_5h_pct: -1.0,
            gemini_weekly_pct: -1.0,
            third_party_5h_pct: -1.0,
            third_party_weekly_pct: -1.0,
            gemini_5h_reset: -1,
            gemini_weekly_reset: -1,
            third_party_5h_reset: -1,
            third_party_weekly_reset: -1,
        }
    }
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

// ─── System Info Helpers ──────────────────────────────────────────────────

fn hostname() -> String {
    Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn tailscale_ip() -> String {
    #[cfg(target_os = "linux")]
    {
        Command::new("ip")
            .args(["-4", "addr", "show", "dev", "tailscale0"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|output| {
                output.lines().find_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.starts_with("inet ") {
                        trimmed
                            .split_whitespace()
                            .nth(1)
                            .map(|addr| addr.split('/').next().unwrap_or("").to_string())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_default()
    }
    #[cfg(not(target_os = "linux"))]
    {
        String::new()
    }
}

fn power_status() -> (bool, Option<u8>) {
    #[cfg(target_os = "linux")]
    {
        let ac = std::fs::read_to_string("/sys/class/power_supply/ACAD/online")
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())
            .unwrap_or(1);
        let battery = std::fs::read_to_string("/sys/class/power_supply/BAT1/capacity")
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok());
        (ac == 0, battery)
    }
    #[cfg(not(target_os = "linux"))]
    {
        (false, None)
    }
}

fn git_info(working_dir: &str) -> (String, String, bool) {
    let dir = if working_dir.is_empty() { "." } else { working_dir };

    let branch = Command::new("git")
        .args(["-C", dir, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if branch.is_empty() {
        return (String::new(), String::new(), false);
    }

    let dirty = Command::new("git")
        .args(["-C", dir, "status", "--porcelain"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    ("git".to_string(), branch, dirty)
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
            if use_classic {
                bar.push('█');
            } else {
                bar.push_str(&block_full);
            }
        } else if i == filled {
            if use_classic {
                bar.push_str(match remainder {
                    75.. => "▓",
                    50.. => "▒",
                    25.. => "░",
                    _ => "·",
                });
            } else {
                bar.push_str(match remainder {
                    75.. => &block_75,
                    50.. => &block_50,
                    25.. => &block_25,
                    _ => &block_empty,
                });
            }
        } else {
            if use_classic {
                bar.push('·');
            } else {
                bar.push_str(&block_empty);
            }
        }
    }

    let reset_label = if reset_sec > 0 {
        format!(" {reset_icon} {}", format_reset_time(reset_sec))
    } else {
        String::new()
    };

    if use_classic {
        format!(
            "{separator}{ANSI_BRIGHT_WHITE}{BOLD}{label}{RESET} {bar_color}{bar}{RESET} {text_color}{pct_int}%{RESET}{reset_label}"
        )
    } else {
        format!(
            "{separator}{ANSI_BRIGHT_WHITE}{BOLD}{label}{RESET} {bar} {text_color}{pct_int}%{RESET}{reset_label}"
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

    // Active quota selection
    let (active_5h, active_weekly, active_5h_reset, active_weekly_reset) =
        if (input.gemini_5h_pct >= 0.0) || (input.gemini_weekly_pct >= 0.0) {
            (
                input.gemini_5h_pct, input.gemini_weekly_pct,
                input.gemini_5h_reset, input.gemini_weekly_reset,
            )
        } else if (input.third_party_5h_pct >= 0.0) || (input.third_party_weekly_pct >= 0.0) {
            (
                input.third_party_5h_pct, input.third_party_weekly_pct,
                input.third_party_5h_reset, input.third_party_weekly_reset,
            )
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
        let truncated = if info.len() > 35 {
            format!("{}...", &info[..32])
        } else {
            info
        };
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
        let details = if !ts_ip.is_empty() {
            format!("{host_name} ({ts_ip})")
        } else {
            host_name
        };
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
        } else {
            if classic {
                format!("{dot_l2}{ANSI_GREEN}{}{RESET}", icons.ac)
            } else {
                format!("{dot_l2}{ANSI_GREEN}{} AC{RESET}", icons.ac)
            }
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

    let fill_color = if pct_int >= 90 {
        ANSI_BRIGHT_RED
    } else if pct_int >= 60 {
        ANSI_BRIGHT_YELLOW
    } else {
        ANSI_YELLOW
    };

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
            if i < filled_segments {
                bar.push('█');
            } else if i == filled_segments {
                bar.push_str(match rem {
                    75.. => "▓",
                    50.. => "▒",
                    25.. => "░",
                    _ => "·",
                });
            } else {
                bar.push('·');
            }
        }
        format!("{ANSI_GRAY}ctx {fill_color}{bar} {num_fmt}")
    } else {
        let mut bar = String::with_capacity(160);
        for i in 0..20u32 {
            if i < filled_segments {
                bar.push_str(&block_full);
            } else if i == filled_segments {
                bar.push_str(match rem {
                    75.. => &block_75,
                    50.. => &block_50,
                    25.. => &block_25,
                    _ => &block_empty,
                });
            } else {
                bar.push_str(&block_empty);
            }
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
        let max_len = 12.min(model_display.len());
        if classic {
            format!("{ANSI_GRAY} ╱ {ANSI_BRIGHT_MAGENTA}{}{RESET}", &model_display[..max_len])
        } else {
            format!("{ANSI_GRAY} ╱ {ANSI_BRIGHT_MAGENTA}{} {}{RESET}", icons.model, &model_display[..max_len])
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

    let input = parse_input(&stdin);
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
