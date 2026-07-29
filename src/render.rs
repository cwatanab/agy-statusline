use crate::bar::{build_bar, build_quota_bar, usage_color};
use crate::format::{format_pct_display, human_format};
use crate::icons::{
    select_icons, with_icon, Icons, ANSI_BRIGHT_CYAN, ANSI_BRIGHT_GREEN, ANSI_BRIGHT_MAGENTA,
    ANSI_BRIGHT_RED, ANSI_BRIGHT_YELLOW, ANSI_GRAY, ANSI_WHITE, BOLD, RESET,
};
use crate::parse::ParsedInput;
use crate::sys::git_info;

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
            (
                input.gemini_5h_pct,
                input.gemini_weekly_pct,
                input.gemini_5h_reset,
                input.gemini_weekly_reset,
            )
        } else if (input.third_party_5h_pct >= 0.0) || (input.third_party_weekly_pct >= 0.0) {
            (
                input.third_party_5h_pct,
                input.third_party_weekly_pct,
                input.third_party_5h_reset,
                input.third_party_weekly_reset,
            )
        } else {
            (-1.0, -1.0, -1, -1)
        };

    let state_str = match input.agent_state.as_str() {
        "idle" => format!(
            "{ANSI_BRIGHT_GREEN}{BOLD} {} READY{RESET}",
            icons.state_ready
        ),
        "thinking" => format!(
            "{ANSI_BRIGHT_YELLOW}{BOLD} {} THINKING{RESET}",
            icons.state_thinking
        ),
        "working" => format!(
            "{ANSI_BRIGHT_CYAN}{BOLD} {} WORKING{RESET}",
            icons.state_working
        ),
        "tool_use" => format!(
            "{ANSI_BRIGHT_MAGENTA}{BOLD} {} TOOL{RESET}",
            icons.state_tool
        ),
        other => format!(
            "{ANSI_WHITE}{BOLD} {} {}{RESET}",
            icons.state_unknown,
            other.to_uppercase()
        ),
    };

    let (vcs_branch, vcs_dirty) = git_info(&input.working_dir);
    let vcs_str = if vcs_branch.is_empty() {
        String::new()
    } else {
        let label = if vcs_dirty {
            format!("{vcs_branch}*")
        } else {
            vcs_branch
        };
        format!("{dot_l1}{}", with_icon(icons.vcs, &label, classic))
    };

    let model_str = if model_display.is_empty() {
        String::new()
    } else {
        format!(
            "{dot_l1}{}",
            with_icon(icons.model, model_display, classic)
        )
    };

    let sandbox_str = if input.sandbox_enabled {
        if input.sandbox_allow_network {
            if classic {
                format!("{ANSI_BRIGHT_YELLOW}ON (net){RESET}")
            } else {
                format!(
                    "{ANSI_BRIGHT_YELLOW}{} ON (net){RESET}",
                    icons.sandbox_net
                )
            }
        } else if classic {
            format!("{ANSI_BRIGHT_GREEN}ON (no-net){RESET}")
        } else {
            format!(
                "{ANSI_BRIGHT_GREEN}{} ON (no-net){RESET}",
                icons.sandbox_nonet
            )
        }
    } else if classic {
        format!("{ANSI_BRIGHT_RED}sandbox off{RESET}")
    } else {
        format!("{ANSI_BRIGHT_RED}{} OFF{RESET}", icons.sandbox_off)
    };

    let fill_color = usage_color(input.used_percentage);
    let num_fmt = format!(
        "{fill_color}{BOLD}{}%{RESET}",
        format_pct_display(input.used_percentage)
    );
    let bar = build_bar(input.used_percentage, fill_color, classic);
    let ctx_bar = if classic {
        format!("{ANSI_GRAY}ctx {fill_color}{bar} {num_fmt}")
    } else {
        format!("{fill_color}{}  {RESET}{bar} {num_fmt}", icons.context_bar)
    };

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

    let quota_str = if (active_5h >= 0.0) || (active_weekly >= 0.0) {
        let bar_5h = build_quota_bar(active_5h, "5H", active_5h_reset, classic, icons.reset);
        let bar_7d =
            build_quota_bar(active_weekly, "7D", active_weekly_reset, classic, icons.reset);
        format!("{bar_5h}{dot_l2}{bar_7d}")
    } else {
        String::new()
    };

    View {
        state_str,
        model_str,
        vcs_str,
        art_str,
        sub_str,
        task_str,
        sandbox_str,
        ctx_bar,
        ctx_size,
        tok_details,
        quota_str,
    }
}

/// Render a full status line from parsed input.
pub fn render_line(input: &ParsedInput, classic: bool) -> String {
    let icons = select_icons(classic);
    let view = build_view(input, &icons, classic);
    let dot_l1 = &icons.dot_l1;
    let dot_l2 = &icons.dot_l2;

    let line1 = format!("{}{}{}", view.state_str, view.model_str, view.vcs_str);
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
    if !ctx_combined.is_empty() {
        right_parts.push(ctx_combined.as_str());
    }
    if !view.quota_str.is_empty() {
        right_parts.push(view.quota_str.as_str());
    }
    if !view.tok_details.is_empty() {
        right_parts.push(view.tok_details.as_str());
    }

    let extra_str = if !right_parts.is_empty() {
        let joined = right_parts.join(dot_l2);
        format!("{dot_l1}{joined}")
    } else {
        String::new()
    };
    format!("{line1}{extra_str}")
}
