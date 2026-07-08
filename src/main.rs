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

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self { Parser { s: s.as_bytes(), i: 0 } }

    #[inline] fn ws(&mut self) { while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\t' | b'\n' | b'\r') { self.i += 1; } }
    #[inline] fn peek(&self) -> Option<u8> { self.s.get(self.i).copied() }

    fn skip_val(&mut self) {
        self.ws();
        match self.peek() {
            Some(b'"') => { self.i += 1; while self.i < self.s.len() && self.s[self.i] != b'"' { if self.s[self.i] == b'\\' { self.i += 1; } self.i += 1; } self.i += 1; }
            Some(b'{') => { self.i += 1; self.skip_obj(); }
            Some(b'[') => { self.i += 1; self.skip_arr(); }
            Some(b't') | Some(b'f') => { self.i += if self.s[self.i] == b't' { 4 } else { 5 }; }
            Some(b'n') => { self.i += 4; }
            Some(b'-' | b'0'..=b'9') => { while self.i < self.s.len() && matches!(self.s[self.i], b'-' | b'0'..=b'9' | b'.' | b'e' | b'E' | b'+') { self.i += 1; } }
            _ => {}
        }
    }
    fn skip_obj(&mut self) { loop { self.ws(); if self.peek() == Some(b'}') { self.i += 1; return; } self.skip_val(); self.ws(); if self.peek() == Some(b':') { self.i += 1; self.skip_val(); } self.ws(); if self.peek() == Some(b',') { self.i += 1; } } }
    fn skip_arr(&mut self) { loop { self.ws(); if self.peek() == Some(b']') { self.i += 1; return; } self.skip_val(); self.ws(); if self.peek() == Some(b',') { self.i += 1; } } }

    fn str_val(&mut self) -> &'a str {
        self.ws(); self.i += 1;
        let start = self.i;
        while self.i < self.s.len() && self.s[self.i] != b'"' { if self.s[self.i] == b'\\' { self.i += 1; } self.i += 1; }
        let end = self.i; self.i += 1;
        std::str::from_utf8(&self.s[start..end]).unwrap_or("")
    }
    fn num_f64(&mut self) -> f64 { self.ws(); let p = self.i; while self.i < self.s.len() && matches!(self.s[self.i], b'-' | b'0'..=b'9' | b'.' | b'e' | b'E' | b'+') { self.i += 1; } std::str::from_utf8(&self.s[p..self.i]).unwrap_or("0").parse().unwrap_or(0.0) }
    fn num_u64(&mut self) -> u64 { self.ws(); let p = self.i; while self.i < self.s.len() && matches!(self.s[self.i], b'0'..=b'9') { self.i += 1; } std::str::from_utf8(&self.s[p..self.i]).unwrap_or("0").parse().unwrap_or(0) }
    fn num_u32(&mut self) -> u32 { self.ws(); let p = self.i; while self.i < self.s.len() && matches!(self.s[self.i], b'0'..=b'9') { self.i += 1; } std::str::from_utf8(&self.s[p..self.i]).unwrap_or("0").parse().unwrap_or(0) }
    fn num_i64(&mut self) -> i64 { self.ws(); let p = self.i; while self.i < self.s.len() && matches!(self.s[self.i], b'-' | b'0'..=b'9') { self.i += 1; } std::str::from_utf8(&self.s[p..self.i]).unwrap_or("0").parse().unwrap_or(-1) }
    fn bool_val(&mut self) -> bool { self.ws(); let v = self.s[self.i] == b't'; self.i += if v { 4 } else { 5 }; v }
    fn arr_len(&mut self) -> u32 { self.ws(); self.i += 1; let mut n = 0u32; loop { self.ws(); if self.peek() == Some(b']') { self.i += 1; return n; } self.skip_val(); n += 1; self.ws(); if self.peek() == Some(b',') { self.i += 1; } } }
    fn is_null(&mut self) -> bool { self.ws(); if self.s.get(self.i..self.i+4) == Some(b"null") { self.i += 4; true } else { false } }
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
    if sec <= 0 { return String::new(); }
    let days = sec / 86400;
    let rem = sec % 86400;
    let hours = rem / 3600;
    let rem = rem % 3600;
    let mins = rem / 60;
    if days > 0 { if hours > 0 { format!("{}d {}h", days, hours) } else { format!("{}d", days) } }
    else if hours > 0 { if mins > 0 { format!("{}h {}m", hours, mins) } else { format!("{}h", hours) } }
    else if mins > 0 { format!("{}m", mins) }
    else { "<1m".to_string() }
}

fn shorten_path(path: &str) -> String {
    if path.is_empty() { return String::new(); }
    let home = env::var("HOME").unwrap_or_default();
    let p = if !home.is_empty() && path.starts_with(&home) { path.replacen(&home, "~", 1) } else { path.to_string() };
    if p.len() > 25 { let base = std::path::Path::new(p.as_str()).file_name().and_then(|s| s.to_str()).unwrap_or(""); format!("...{}", base) }
    else { p }
}

fn visible_len(s: &str) -> usize {
    let mut n = 0;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' { while let Some(d) = chars.next() { if d == 'm' { break; } } }
        else { n += 1; }
    }
    n
}

fn hostname() -> String {
    Command::new("hostname").output().ok().and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| s.trim().to_string()).unwrap_or_default()
}
fn tailscale_ip() -> String {
    #[cfg(target_os = "linux")]
    {
        Command::new("ip").args(["-4", "addr", "show", "dev", "tailscale0"]).output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.lines().find_map(|l| {
                let t = l.trim(); if t.starts_with("inet ") { t.split_whitespace().nth(1).map(|x| x.split('/').next().unwrap_or("").to_string()) } else { None }
            })).unwrap_or_default()
    }
    #[cfg(not(target_os = "linux"))]
    { String::new() }
}
fn power_status() -> (bool, Option<u8>) {
    #[cfg(target_os = "linux")]
    {
        let ac = std::fs::read_to_string("/sys/class/power_supply/ACAD/online").ok().and_then(|s| s.trim().parse::<u8>().ok()).unwrap_or(1);
        let bat = std::fs::read_to_string("/sys/class/power_supply/BAT1/capacity").ok().and_then(|s| s.trim().parse::<u8>().ok());
        (ac == 0, bat)
    }
    #[cfg(not(target_os = "linux"))]
    (false, None)
}
fn git_info(cwd: &str) -> (String, String, bool) {
    let d = if cwd.is_empty() { "." } else { cwd };
    let br = Command::new("git").args(["-C", d, "rev-parse", "--abbrev-ref", "HEAD"]).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| s.trim().to_string()).unwrap_or_default();
    if br.is_empty() { return (String::new(), String::new(), false); }
    let dirty = Command::new("git").args(["-C", d, "status", "--porcelain"]).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| !s.trim().is_empty()).unwrap_or(false);
    ("git".to_string(), br, dirty)
}

fn make_quota_bar(val: f64, label: &str, bar_color: &str, reset_sec: i64, use_classic: bool, icon_reset: &str) -> String {
    let sep = if use_classic { format!("{FG_GRAY} · {R}") } else { format!("{FG_GRAY}| {R}") };
    if val < -0.5 {
        let bar: String = (0..20).map(|_| if use_classic { "·" } else { "░" }).collect();
        return format!("{sep}{FG_BRIGHT_WHITE}{B}{label}{R} {FG_GRAY}{bar} N/A{R}");
    }
    let vi = val as u32;
    let tc = if vi < 20 { FG_BRIGHT_RED } else if vi < 50 { FG_BRIGHT_YELLOW } else { FG_BRIGHT_GREEN };
    let filled = vi * 20 / 100;
    let rem = (vi * 20) % 100;
    let qb = format!("{bar_color}█{R}");
    let q75 = format!("{bar_color}▓{R}{FG_GRAY}");
    let q50 = format!("{bar_color}▒{R}{FG_GRAY}");
    let q25 = format!("{bar_color}░{R}{FG_GRAY}");
    let qe = format!("{FG_GRAY}░{R}");
    let mut bar = String::with_capacity(160);
    for i in 0..20u32 {
        if i < filled { if use_classic { bar.push('█'); } else { bar.push_str(&qb); } }
        else if i == filled {
            if use_classic { bar.push_str(match rem { 75.. => "▓", 50.. => "▒", 25.. => "░", _ => "·" }); }
            else { bar.push_str(match rem { 75.. => &q75, 50.. => &q50, 25.. => &q25, _ => &qe }); }
        } else { if use_classic { bar.push('·'); } else { bar.push_str(&qe); } }
    }
    let rst = if reset_sec > 0 { format!(" {icon_reset} {}", format_reset_time(reset_sec)) } else { String::new() };
    if use_classic { format!("{sep}{FG_BRIGHT_WHITE}{B}{label}{R} {bar_color}{bar}{R} {tc}{vi}%{R}{rst}") }
    else { format!("{sep}{FG_BRIGHT_WHITE}{B}{label}{R} {bar} {tc}{vi}%{R}{rst}") }
}

fn print_right_aligned(left: &str, right: &str, total_cols: u32) {
    let lv = visible_len(left); let rv = visible_len(right);
    let pad = if total_cols as usize > lv + rv { total_cols as usize - lv - rv } else { 1 };
    print!("{left}"); for _ in 0..pad { print!(" "); } println!("{right}");
}

fn main() {
    let use_classic = env::args().any(|a| a == "--classic" || a == "--no-nerdfont" || a == "--compatibility");
    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).unwrap();

    let mut p = Parser::new(&stdin);

    // Parse top-level object
    p.ws();
    if p.peek() != Some(b'{') { return; }
    p.i += 1;

    // Default values
    let mut state = "idle";
    let mut used_pct = 0.0f64;
    let mut sandbox_on = false;
    let mut sandbox_net = false;
    let mut artifacts = 0u32;
    let mut subagents = 0u32;
    let mut bg_tasks = 0u32;
    let mut model_id = "";
    let mut model_name = "";
    let mut cols = 80u32;
    let mut cwd = "";
    let mut conv_id = "";
    let mut cli_version = "";
    let mut plan_tier = "";
    let mut user_email = "";
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;
    let mut ctx_limit = 0u64;
    let mut turn_input = 0u64;
    let mut turn_output = 0u64;
    let mut gemini_5h = -1.0f64;
    let mut gemini_wk = -1.0f64;
    let mut tp_5h = -1.0f64;
    let mut tp_wk = -1.0f64;
    let mut gemini_5h_r = -1i64;
    let mut gemini_wk_r = -1i64;
    let mut tp_5h_r = -1i64;
    let mut tp_wk_r = -1i64;

    loop {
        p.ws();
        match p.peek() {
            None => break,
            Some(b'}') => break,
            Some(b'"') => {
                let key = p.str_val();
                p.ws();
                if p.peek() == Some(b':') { p.i += 1; } else { continue; }
                match key {
                    "agent_state" => if !p.is_null() { state = p.str_val(); } else { state = "idle"; },
                    "artifact_count" => artifacts = p.num_u32(),
                    "task_count" => bg_tasks = p.num_u32(),
                    "terminal_width" => cols = p.num_u32(),
                    "cwd" => if !p.is_null() { cwd = p.str_val(); },
                    "conversation_id" => if !p.is_null() { conv_id = p.str_val(); },
                    "version" => if !p.is_null() { cli_version = p.str_val(); },
                    "plan_tier" => if !p.is_null() { plan_tier = p.str_val(); },
                    "email" => if !p.is_null() { user_email = p.str_val(); },
                    "subagents" => if p.is_null() { subagents = 0; } else { subagents = p.arr_len(); },
                    "context_window" => {
                        p.ws(); p.i += 1; // {
                        loop { p.ws(); if p.peek() == Some(b'}') { p.i += 1; break; }
                            let ck = p.str_val(); p.ws(); p.i += 1; // :
                            match ck {
                                "used_percentage" => used_pct = p.num_f64(),
                                "total_input_tokens" => input_tokens = p.num_u64(),
                                "total_output_tokens" => output_tokens = p.num_u64(),
                                "context_window_size" => ctx_limit = p.num_u64(),
                                "current_usage" => {
                                    p.ws(); p.i += 1; // {
                                    loop { p.ws(); if p.peek() == Some(b'}') { p.i += 1; break; }
                                        let uk = p.str_val(); p.ws(); p.i += 1; // :
                                        match uk { "input_tokens" => turn_input = p.num_u64(), "output_tokens" => turn_output = p.num_u64(), _ => { p.skip_val(); } }
                                        p.ws(); if p.peek() == Some(b',') { p.i += 1; }
                                    }
                                }
                                _ => { p.skip_val(); }
                            }
                            p.ws(); if p.peek() == Some(b',') { p.i += 1; }
                        }
                    },
                    "sandbox" => {
                        p.ws(); p.i += 1; // {
                        loop { p.ws(); if p.peek() == Some(b'}') { p.i += 1; break; }
                            let sk = p.str_val(); p.ws(); p.i += 1; // :
                            match sk { "enabled" => sandbox_on = p.bool_val(), "allow_network" => sandbox_net = p.bool_val(), _ => { p.skip_val(); } }
                            p.ws(); if p.peek() == Some(b',') { p.i += 1; }
                        }
                    },
                    "model" => {
                        p.ws(); p.i += 1; // {
                        loop { p.ws(); if p.peek() == Some(b'}') { p.i += 1; break; }
                            let mk = p.str_val(); p.ws(); p.i += 1; // :
                            match mk { "id" => if !p.is_null() { model_id = p.str_val(); }, "display_name" => if !p.is_null() { model_name = p.str_val(); }, _ => { p.skip_val(); } }
                            p.ws(); if p.peek() == Some(b',') { p.i += 1; }
                        }
                    },
                    "quota" => {
                        p.ws(); p.i += 1; // {
                        loop { p.ws(); if p.peek() == Some(b'}') { p.i += 1; break; }
                            let qk = p.str_val(); p.ws(); p.i += 1; // :
                            // Parse quota entry
                            p.ws(); p.i += 1; // {
                            let mut rf = -1.0f64; let mut ri = -1i64;
                            loop { p.ws(); if p.peek() == Some(b'}') { p.i += 1; break; }
                                let qek = p.str_val(); p.ws(); p.i += 1; // :
                                match qek { "remaining_fraction" => { if !p.is_null() { rf = p.num_f64(); } }, "reset_in_seconds" => { if !p.is_null() { ri = p.num_i64(); } }, _ => { p.skip_val(); } }
                                p.ws(); if p.peek() == Some(b',') { p.i += 1; }
                            }
                            match qk {
                                "gemini-5h" => { gemini_5h = (rf * 1000.0).round() / 10.0; gemini_5h_r = ri; }
                                "gemini-weekly" => { gemini_wk = (rf * 1000.0).round() / 10.0; gemini_wk_r = ri; }
                                "3p-5h" => { tp_5h = (rf * 1000.0).round() / 10.0; tp_5h_r = ri; }
                                "3p-weekly" => { tp_wk = (rf * 1000.0).round() / 10.0; tp_wk_r = ri; }
                                _ => {}
                            }
                            p.ws(); if p.peek() == Some(b',') { p.i += 1; }
                        }
                    },
                    _ => { p.skip_val(); }
                }
                p.ws(); if p.peek() == Some(b',') { p.i += 1; }
            }
            _ => { p.skip_val(); p.ws(); if p.peek() == Some(b',') { p.i += 1; } }
        }
    }

    let model_disp = if !model_name.is_empty() { model_name } else { model_id };
    let ctx_used = input_tokens + output_tokens;

    let (q_5h, q_wk, q_5h_r, q_wk_r) =
        if (gemini_5h >= 0.0) || (gemini_wk >= 0.0) { (gemini_5h, gemini_wk, gemini_5h_r, gemini_wk_r) }
        else if (tp_5h >= 0.0) || (tp_wk >= 0.0) { (tp_5h, tp_wk, tp_5h_r, tp_wk_r) }
        else { (-1.0, -1.0, -1, -1) };

    // ─── Icons ────────────────────────────────────────────────────────────────
    let dot_l1 = if use_classic { format!("{FG_GRAY} ╱ {R}") } else { format!("{FG_GRAY} | {R}") };
    let dot_l2 = if use_classic { format!("{FG_GRAY} · {R}") } else { format!("{FG_GRAY} | {R}") };
    let (icon_vcs, icon_model, icon_sn, icon_snn, icon_so, icon_cb, icon_art, icon_sub, icon_tsk, icon_dir, icon_conv, icon_ts, icon_rst, icon_ac, icon_bat) = if use_classic {
        ("", "", "ON (net)", "ON (no-net)", "OFF", "ctx", "artifacts", "subagents", "tasks", "", "", "", "\u{231B}", "AC", "BAT")
    } else {
        ("\u{F418}", "\u{F400}", "\u{F0499}", "\u{F0D34}", "\u{F099C}", "\u{F134F}", "\u{F0F6}", "\u{F167A}", "\u{F0AE}", "\u{EA83}", "\u{F036A}", "\u{E26B}", "\u{231B}\u{FE0F}", "\u{F06A5}", "\u{1F50B}")
    };
    let (icon_rdy, icon_thk, icon_wrk, icon_ttl, icon_unk) = if use_classic { ("●", "◆", "⚙", "🔧", "\u{231B}") } else { ("\u{F192}", "\u{F07F7}", "\u{F423}", "\u{F425}", "\u{F252}") };

    // ─── Computed ─────────────────────────────────────────────────────────────
    let pct_int = used_pct as u32;
    let pct_x10 = (used_pct * 10.0).round() as u32;
    let pct_fmt = format!("{}.{}", pct_x10 / 10, pct_x10 % 10);
    let itf = human_format(input_tokens); let otf = human_format(output_tokens);
    let clf = human_format(ctx_limit); let cuf = human_format(ctx_used);
    let tif = human_format(turn_input); let tof = human_format(turn_output);

    // ─── State ────────────────────────────────────────────────────────────────
    let state_str = match state {
        "idle" => format!("{FG_BRIGHT_GREEN}{B} {icon_rdy} READY{R}"),
        "thinking" => format!("{FG_BRIGHT_YELLOW}{B} {icon_thk} THINKING{R}"),
        "working" => format!("{FG_BRIGHT_CYAN}{B} {icon_wrk} WORKING{R}"),
        "tool_use" => format!("{FG_BRIGHT_MAGENTA}{B} {icon_ttl} TOOL{R}"),
        other => format!("{FG_WHITE}{B} {icon_unk} {}{R}", other.to_uppercase()),
    };

    // ─── VCS ──────────────────────────────────────────────────────────────────
    let (_, vcs_branch, vcs_dirty) = git_info(cwd);
    let vcs = if vcs_branch.is_empty() { String::new() }
    else if vcs_dirty {
        if use_classic { format!("{dot_l1}{FG_BRIGHT_RED}{vcs_branch}{FG_BRIGHT_YELLOW}*{R}") }
        else { format!("{dot_l1}{R}{FG_BRIGHT_RED}{icon_vcs} {vcs_branch}{FG_BRIGHT_YELLOW}*{R}") }
    } else if use_classic { format!("{dot_l1}{FG_BRIGHT_BLUE}{vcs_branch}{R}") }
    else { format!("{dot_l1}{R}{FG_BRIGHT_BLUE}{icon_vcs} {vcs_branch}{R}") };

    // ─── Model ────────────────────────────────────────────────────────────────
    let model_fmt = if !model_disp.is_empty() {
        if use_classic { format!("{dot_l1}{FG_BRIGHT_MAGENTA}{I}{model_disp}{R}") }
        else { format!("{dot_l1}{FG_BRIGHT_MAGENTA}{I}{icon_model} {model_disp}{R}") }
    } else { String::new() };

    // ─── Sandbox ──────────────────────────────────────────────────────────────
    let sandbox = if sandbox_on {
        if sandbox_net { format!("{FG_GREEN}{icon_sn} ON (net){R}") }
        else { format!("{FG_GREEN}{icon_snn} ON (no-net){R}") }
    } else if use_classic { format!("{FG_GRAY}sandbox off{R}") }
    else { format!("{FG_RED}{icon_so} OFF{R}") };

    // ─── Context Bar ──────────────────────────────────────────────────────────
    let fill_color = if pct_int >= 90 { FG_BRIGHT_RED } else if pct_int >= 60 { FG_BRIGHT_YELLOW } else { FG_YELLOW };
    let filled = pct_int * 20 / 100;
    let rem = (pct_int * 20) % 100;
    let fb = format!("{fill_color}█{R}");
    let f75 = format!("{fill_color}▓{R}{FG_GRAY}");
    let f50 = format!("{fill_color}▒{R}{FG_GRAY}");
    let f25 = format!("{fill_color}░{R}{FG_GRAY}");
    let fe = format!("{FG_GRAY}░{R}");
    let ctx_bar = if use_classic {
        let mut bar = String::with_capacity(40);
        for i in 0..20u32 {
            if i < filled { bar.push('█'); }
            else if i == filled { bar.push_str(match rem { 75.. => "▓", 50.. => "▒", 25.. => "░", _ => "·" }); }
            else { bar.push('·'); }
        }
        format!("{FG_GRAY}ctx {fill_color}{bar} {FG_BRIGHT_WHITE}{B}{pct_fmt}%{R}")
    } else {
        let mut bar = String::with_capacity(160);
        for i in 0..20u32 {
            if i < filled { bar.push_str(&fb); }
            else if i == filled { bar.push_str(match rem { 75.. => &f75, 50.. => &f50, 25.. => &f25, _ => &fe }); }
            else { bar.push_str(&fe); }
        }
        format!("{FG_YELLOW}{icon_cb}  {R}{bar} {FG_BRIGHT_WHITE}{B}{pct_fmt}%{R}")
    };

    // ─── Stats ────────────────────────────────────────────────────────────────
    let art_fmt = if use_classic { format!("{FG_GRAY}artifacts {FG_BRIGHT_WHITE}{B}{artifacts}{R}") }
    else { format!("{FG_BLUE}{icon_art} {FG_BRIGHT_WHITE}{B}{artifacts}{R}") };
    let sub_fmt = if use_classic { format!("{FG_GRAY}subagents {FG_BRIGHT_WHITE}{B}{subagents}{R}") }
    else { format!("{FG_CYAN}{icon_sub} {FG_BRIGHT_WHITE}{B}{subagents}{R}") };
    let bg_fmt = if use_classic { format!("{FG_GRAY}tasks {FG_BRIGHT_WHITE}{B}{bg_tasks}{R}") }
    else { format!("{FG_MAGENTA}{icon_tsk} {FG_BRIGHT_WHITE}{B}{bg_tasks}{R}") };

    // ─── Token Details ────────────────────────────────────────────────────────
    let tok_wide = if ctx_used > 0 {
        let ts = if turn_input > 0 || turn_output > 0 { format!(" | turn: +{}/{}", tif, tof) } else { String::new() };
        if use_classic { format!(" ({}/{}){dot_l2}(total: {}/{}{})", cuf, clf, itf, otf, ts) }
        else { format!(" ({}/{}){dot_l2}{FG_YELLOW}{icon_ts} {R} (total: {}/{}{})", cuf, clf, itf, otf, ts) }
    } else { String::new() };
    let tok_med = if ctx_used > 0 { format!(" ({}/{})", cuf, clf) } else { String::new() };

    // ─── Quota ────────────────────────────────────────────────────────────────
    let quota_fmt = if (q_5h >= 0.0) || (q_wk >= 0.0) {
        format!("{} {}", make_quota_bar(q_5h, "5H", FG_BRIGHT_CYAN, q_5h_r, use_classic, icon_rst), make_quota_bar(q_wk, "7D", FG_BRIGHT_MAGENTA, q_wk_r, use_classic, icon_rst))
    } else { String::new() };

    // ─── Optional segments ────────────────────────────────────────────────────
    let cli_ver_fmt = if cli_version.is_empty() { String::new() } else { format!("{dot_l1}{FG_GRAY}v{cli_version}{R}") };
    let user_fmt = if !plan_tier.is_empty() || !user_email.is_empty() {
        let ui = if !plan_tier.is_empty() && !user_email.is_empty() { format!("{} ({})", plan_tier, user_email) }
        else if !plan_tier.is_empty() { plan_tier.to_string() } else { user_email.to_string() };
        let t = if ui.len() > 35 { format!("{}...", &ui[..32]) } else { ui };
        if use_classic { format!("{dot_l1}{FG_GRAY}{t}{R}") } else { format!("{dot_l1}{FG_GRAY}\u{F01EE} {t}{R}") }
    } else { String::new() };
    let host_name = hostname();
    let ts_ip = tailscale_ip();
    let host_fmt = if !host_name.is_empty() {
        let hd = if !ts_ip.is_empty() { format!("{} ({})", host_name, ts_ip) } else { host_name };
        if use_classic { format!("{dot_l1}{FG_BRIGHT_BLUE}{hd}{R}") } else { format!("{dot_l1}{FG_BRIGHT_BLUE}\u{F048B} {hd}{R}") }
    } else { String::new() };
    let power_fmt = {
        let (on_bat, cap) = power_status();
        if on_bat {
            if let Some(c) = cap { if use_classic { format!("{dot_l2}{FG_BRIGHT_YELLOW}{icon_bat}:{}%{R}", c) } else { format!("{dot_l2}{FG_BRIGHT_YELLOW}{icon_bat} {}%{R}", c) } }
            else { format!("{dot_l2}{FG_BRIGHT_YELLOW}{icon_bat}{R}") }
        } else { if use_classic { format!("{dot_l2}{FG_GREEN}{icon_ac}{R}") } else { format!("{dot_l2}{FG_GREEN}{icon_ac} AC{R}") } }
    };
    let cwd_short = shorten_path(cwd);
    let dir_fmt = if !cwd_short.is_empty() {
        if use_classic { format!("{dot_l1}{FG_CYAN}{cwd_short}{R}") } else { format!("{dot_l1}{FG_CYAN}{icon_dir} {cwd_short}{R}") }
    } else { String::new() };
    let conv_fmt = if !conv_id.is_empty() {
        let len = 8.min(conv_id.len());
        if use_classic { format!("{dot_l1}{FG_GRAY}{}{R}", &conv_id[..len]) } else { format!("{dot_l1}{FG_GRAY}{icon_conv} {}{R}", &conv_id[..len]) }
    } else { String::new() };

    // ─── Output Assembly ──────────────────────────────────────────────────────
    if cols >= 180 {
        let line1 = format!("{state_str}{cli_ver_fmt}{user_fmt}{host_fmt}{model_fmt}{dir_fmt}{vcs}{conv_fmt}");
        let line2 = if !quota_fmt.is_empty() {
            format!("{art_fmt}{dot_l2}{sub_fmt}{dot_l2}{bg_fmt}{dot_l2}{sandbox}{dot_l2}{ctx_bar}{tok_wide}{quota_fmt}{power_fmt}")
        } else { format!("{art_fmt}{dot_l2}{sub_fmt}{dot_l2}{bg_fmt}{dot_l2}{sandbox}{dot_l2}{ctx_bar}{tok_wide}{power_fmt}") };
        print_right_aligned(&line1, &line2, cols);
    } else if cols >= 90 {
        let line1 = format!("{state_str}{cli_ver_fmt}{user_fmt}{host_fmt}{model_fmt}{dir_fmt}{vcs}");
        let line2 = if !quota_fmt.is_empty() {
            format!(" {ctx_bar}{tok_med}{dot_l2}{art_fmt}{dot_l2}{sub_fmt}{dot_l2}{bg_fmt}{dot_l2}{sandbox}{quota_fmt}{power_fmt}")
        } else { format!(" {ctx_bar}{tok_med}{dot_l2}{art_fmt}{dot_l2}{sub_fmt}{dot_l2}{bg_fmt}{dot_l2}{sandbox}{power_fmt}") };
        println!("{FG_GRAY}╭─{R}{line1}");
        println!("{FG_GRAY}╰─{R}{line2}");
    } else {
        let model_short = if !model_disp.is_empty() {
            let ml = 12.min(model_disp.len());
            if use_classic { format!("{FG_GRAY} ╱ {FG_BRIGHT_MAGENTA}{}{R}", &model_disp[..ml]) }
            else { format!("{FG_GRAY} ╱ {FG_BRIGHT_MAGENTA}{icon_model} {}{R}", &model_disp[..ml]) }
        } else { String::new() };
        println!("{state_str}{model_short}");
        if !quota_fmt.is_empty() { println!("{ctx_bar}{dot_l2}{bg_fmt}{quota_fmt}{power_fmt}"); }
        else { println!("{ctx_bar}{dot_l2}{bg_fmt}{power_fmt}"); }
    }
}
