use std::env;
use std::io::{self, Read};
use std::process;

use statusline::parse;
use statusline::render;

const LEGEND_TEXT: &str = "\x1b[92m\x1b[1m🚀 Antigravity CLI Maximized Statusline Legend (v0.3.1)\x1b[0m
This statusline adapts dynamically to terminal width and displays high-density system & agent telemetry.

\x1b[1mLAYOUTS & AUTO-PACKING:\x1b[0m
  - \x1b[1mSmart Dynamic Line-Packing Engine:\x1b[0m Telemetry badges automatically pack into cleanly framed boxed rows (╭─, ├─, ╰─) without line wrapping.

\x1b[1mCOMPONENTS & ICONS:\x1b[0m
  \x1b[1mField                Nerd Font   Classic     Description\x1b[0m
  --------------------------------------------------------------------------------
  State: READY         \u{F192}          ●           Agent is idle, ready for user requests.
  State: THINKING      \u{F07F7}          ◆           Agent is processing/thinking.
  State: WORKING       \u{F423}          ⚙           Agent is executing background operations.
  State: TOOL          \u{F425}          🔧          Agent is running a tool.
  VCS Branch           \u{F418}          ╱           Current Git branch name (Red + * if dirty).
  Model                \u{F400}          (None)      Current active LLM model name/ID.
  Sandbox Network      \u{F0499}          ON (net)    Sandbox enabled with internet access.
  Sandbox Restricted   \u{F0D34}          ON (no-net) Sandbox enabled with network disabled.
  Sandbox Off          \u{F099C}          sandbox off Sandbox is disabled (runs on host).
  Context Bar          \u{F134F}          ctx         Context window usage bar (10 or 20 segments).
  Tokens Sum           \u{E26B}          (None)      Total input/output tokens & turn token delta.
  Sys resources        \u{F4BC}          sys         Host CPU load average & memory utilization.
  Artifacts            \u{F0F6}          artifacts   Number of active output artifacts.
  Subagents            \u{F167A}          subagents   Number of spawned active subagents.
  Background Tasks     \u{F0AE}          tasks       Number of background tasks running.
  Current Directory    \u{F0A83}          ╱           Current working directory path (shortened).
  Conversation ID      \u{F036A}          ╱           Short prefix of the current session ID.
  Quota Reset Time     ⌛️         ⌛          Remaining time until LLM quota resets.
  Power Mains (AC)     \u{F06A5}          AC          Host is connected to external AC power.
  Power Battery (UPS)  🔋          BAT         Host is running on battery (shows charge %).
";

fn print_legend() {
    print!("{LEGEND_TEXT}");
}

fn main() {
    let mut use_classic = false;
    let mut override_cols: Option<usize> = None;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--version" | "-v" => {
                println!("Antigravity CLI Statusline v0.3.1");
                process::exit(0);
            }
            "--legend" | "-l" | "legend" => {
                print_legend();
                process::exit(0);
            }
            "--compact" => {
                override_cols = Some(89);
            }
            "--medium" => {
                override_cols = Some(120);
            }
            "--medium-wide" => {
                override_cols = Some(150);
            }
            "--classic" | "--no-nerdfont" | "--compatibility" => {
                use_classic = true;
            }
            _ => {}
        }
    }

    let mut stdin = String::with_capacity(512);
    if io::stdin().read_to_string(&mut stdin).is_err() || stdin.trim().is_empty() {
        process::exit(0);
    }

    let input = parse::parse_input(&stdin);
    println!("{}", render::render_line(&input, use_classic, override_cols));
}
