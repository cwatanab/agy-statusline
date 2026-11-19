use std::env;
use std::io::{self, Read};
use std::process;

use statusline::parse;
use statusline::render;

const LEGEND_TEXT: &str = "\x1b[92m\x1b[1m🚀 Antigravity CLI Maximized Statusline Legend (v0.3.4)\x1b[0m
This statusline adapts dynamically to terminal width and displays high-density system & agent telemetry.

\x1b[1mLAYOUTS & AUTO-PACKING:\x1b[0m
  - \x1b[1mSmart Dynamic Line-Packing Engine:\x1b[0m Telemetry badges automatically pack into cleanly framed boxed rows (╭─, ├─, ╰─) without line wrapping.
  - \x1b[1mResponsive Bar Widths:\x1b[0m >= 235 cols use 20-segment context / 15-segment quota bars; narrower terminals use 10 / 8 segments.

\x1b[1mCOMPONENTS & ICONS:\x1b[0m
  \x1b[1mField                Nerd Font      Classic          Description\x1b[0m
  ------------------------------------------------------------------------------------
  State: READY         \u{F192}              ●                Agent is idle, ready for user requests.
  State: THINKING      \u{F07F7}              ◆                Agent is processing/thinking.
  State: WORKING       \u{F423}              ⚙                Agent is executing background operations.
  State: TOOL          \u{F425}              🔧               Agent is running a tool.
  State: UNKNOWN       \u{F252}              ⏳               Agent state is unknown or custom.
  Vim Editor Mode      (None)         [MODE]           Active Vim mode (MODE in styled, [MODE] in classic).
  VCS Branch           \u{F418}              ╱                Current Git branch name (Red + * if dirty).
  Model                \u{F400}              (None)           Current active LLM model name/ID.
  User Account         👤             (None)           Subscription plan tier & email (>= 130 cols).
  Host Diagnostics     \u{F048B}              (None)           Hostname & Tailscale IPv4 (>= 110 cols).
  CLI Version          vX.Y.Z         vX.Y.Z           Antigravity CLI version badge (>= 120 cols).
  Sandbox Network      \u{F0499} net-on       ON (net) net-on  Sandbox enabled with internet access.
  Sandbox Restricted   \u{F0D34} net-off      ON (no-net) net-off Sandbox enabled with network disabled.
  Sandbox Off          \u{F099C} host         OFF host         Sandbox is disabled (runs on host).
  Context Bar          \u{F134F}              ctx              Context window usage bar (10 or 20 segments).
  Tokens Sum           \u{E26B}              (None)           Total input/output tokens & turn token delta.
  Session Cost         \u{F0D6}              cost             Estimated session spend in USD (& subagent cost).
  Sys resources        \u{F4BC}              sys              Host CPU load average & memory utilization.
  Artifacts            \u{F0F6}              artifacts        Number of active output artifacts.
  Subagents            \u{F167A}              subagents        Number of spawned active subagents.
  Background Tasks     \u{F0AE}              tasks            Number of background tasks running.
  Current Directory    \u{F0A83}              ╱                Current working directory path (shortened).
  Conversation ID      \u{F036A}              ╱                Short prefix of the current session ID (>= 80 cols).
  Quota Reset Time     ⌛️             ⌛               Remaining time until LLM quota resets.
  Power Mains (AC)     \u{F06A5}              AC               Host is connected to external AC power.
  Power Battery (UPS)  🔋             BAT              Host is running on battery (shows charge %).
";

fn print_legend() {
    print!("{LEGEND_TEXT}");
}

fn main() {
    let mut use_classic = false;
    let mut override_cols: Option<usize> = None;
    let mut filter = render::FilterFlags::default();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--version" | "-v" => {
                println!("Antigravity CLI Statusline v0.3.4");
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
            "--no-state" | "-nostate" => {
                filter.show_state = false;
            }
            "--no-vim" | "--no-vim-mode" | "-novim" | "-novimmode" => {
                filter.show_vim = false;
            }
            "--no-branch" | "--no-git" | "-nobranch" | "-nogit" => {
                filter.show_branch = false;
            }
            "--no-model" | "-nomodel" => {
                filter.show_model = false;
            }
            "--no-dir" | "--no-cwd" | "-nodir" | "-nocwd" => {
                filter.show_dir = false;
            }
            "--no-conv" | "--no-conversation" | "-noconv" | "-noconversation" => {
                filter.show_conv = false;
            }
            "--no-account" | "--no-user" | "--no-plan" | "-noaccount" | "-nouser" | "-noplan" => {
                filter.show_account = false;
            }
            "--no-host" | "-nohost" => {
                filter.show_host = false;
            }
            "--no-version" | "-noversion" => {
                filter.show_version = false;
            }
            "--no-context-usage" | "--no-context" | "--no-ctx" | "-nocontextusage" | "-nocontext" => {
                filter.show_context_usage = false;
            }
            "--no-tokens-usage" | "--no-tokens" | "-notokensusage" | "-notokens" => {
                filter.show_tokens_usage = false;
            }
            "--no-cost" | "-nocost" => {
                filter.show_cost = false;
            }
            "--no-sys" | "--no-system" | "--no-resources" | "-nosys" | "-nosystem" | "-noresources" => {
                filter.show_sys = false;
            }
            "--no-artifacts" | "-noartifacts" => {
                filter.show_artifacts = false;
            }
            "--no-subagents" | "-nosubagents" => {
                filter.show_subagents = false;
            }
            "--no-tasks" | "-notasks" => {
                filter.show_tasks = false;
            }
            "--no-sandbox" | "-nosandbox" => {
                filter.show_sandbox = false;
            }
            "--no-quota" | "-noquota" => {
                filter.show_quota = false;
            }
            "--no-power" | "-nopower" => {
                filter.show_power = false;
            }
            _ => {}
        }
    }

    let mut stdin = String::with_capacity(512);
    if io::stdin().read_to_string(&mut stdin).is_err() {
        process::exit(0);
    }

    let stdin_str = if stdin.trim().is_empty() {
        "{}"
    } else {
        &stdin
    };

    let input = parse::parse_input(stdin_str);
    let rendered = render::render_line_with_filter(&input, use_classic, override_cols, filter);
    if !rendered.is_empty() {
        println!("{}", rendered);
    }
}
