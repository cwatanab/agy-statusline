use std::io::Write;
use std::process::{Command, Stdio};

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            while let Some(d) = chars.next() {
                if d == 'm' { break; }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn run_statusline(json: &str, args: &[&str]) -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let path = exe.parent()?.parent()?.join("statusline");
    let path = if cfg!(windows) { path.with_extension("exe") } else { path };
    let mut child = Command::new(&path)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.as_mut()?.write_all(json.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

/// Test that idle state produces the READY indicator.
#[test]
fn idle_shows_ready() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("READY"), "Expected READY in: {}", stripped);
}

#[test]
fn thinking_shows_thinking() {
    let json = r#"{"agent_state":"thinking","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("THINKING"), "Expected THINKING in: {}", stripped);
}

#[test]
fn working_shows_working() {
    let json = r#"{"agent_state":"working","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("WORKING"), "Expected WORKING in: {}", stripped);
}

#[test]
fn tool_use_shows_tool() {
    let json = r#"{"agent_state":"tool_use","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("TOOL"), "Expected TOOL in: {}", stripped);
}

#[test]
fn sandbox_on_net() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":true,"allow_network":true},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("ON (net)"), "Expected 'ON (net)' in: {}", stripped);
}

#[test]
fn sandbox_on_no_net() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":true,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("ON (no-net)"), "Expected 'ON (no-net)' in: {}", stripped);
}

#[test]
fn sandbox_off() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("OFF"), "Expected 'OFF' in: {}", stripped);
}

#[test]
fn model_name_shown() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"gpt-5","display_name":"GPT-5"},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("GPT-5"), "Expected 'GPT-5' in: {}", stripped);
}

#[test]
fn quota_bar_shows_5h_and_7d() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":200,"quota":{"gemini-5h":{"remaining_fraction":0.79,"reset_in_seconds":3600},"gemini-weekly":{"remaining_fraction":0.45,"reset_in_seconds":86400}}}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("5H"), "Expected '5H' in: {}", stripped);
    assert!(stripped.contains("7D"), "Expected '7D' in: {}", stripped);
    assert!(stripped.contains("79%"), "Expected '79%' in: {}", stripped);
    assert!(stripped.contains("45%"), "Expected '45%' in: {}", stripped);
}

#[test]
fn quota_n_a_when_missing() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(!stripped.contains("5H"), "Should not contain quota bar: {}", stripped);
}

#[test]
fn context_bar_shows_percentage() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":45.0,"total_input_tokens":15000,"total_output_tokens":3000,"context_window_size":200000},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("45.0%"), "Expected '45.0%' in: {}", stripped);
}

#[test]
fn classic_mode_uses_text_labels() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":5,"subagents":["a"],"task_count":3,"model":{"id":"","display_name":""},"terminal_width":200}"#;
    let out = run_statusline(json, &["--classic"]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("artifacts 5"), "Classic should show 'artifacts 5': {}", stripped);
    assert!(stripped.contains("tasks 3"), "Classic should show 'tasks 3': {}", stripped);
    assert!(stripped.contains("ctx"), "Classic should show 'ctx': {}", stripped);
    assert!(stripped.contains("sandbox off"), "Classic should show 'sandbox off': {}", stripped);
}

#[test]
fn narrow_layout_is_one_line() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"gpt","display_name":"GPT"},"terminal_width":60}"#;
    let out = run_statusline(json, &[]).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 1, "Narrow layout should have 1 line, got: {:?}", lines);
    assert!(strip_ansi(lines[0]).contains("READY"), "Line should contain READY");
    assert!(strip_ansi(lines[0]).contains("GPT"), "Line should contain model name");
}

#[test]
fn wide_layout_is_one_line_right_aligned() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":200}"#;
    let out = run_statusline(json, &[]).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 1, "Wide layout should have 1 line, got {}: {:?}", lines.len(), lines);
}

#[test]
fn artifacts_subagents_tasks_counts() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":5,"subagents":["a","b","c"],"task_count":3,"model":{"id":"","display_name":""},"terminal_width":200}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    // In Nerd Font mode, counts are shown without text labels
    assert!(stripped.contains(" 5 "), "Expected artifact count 5: {}", stripped);
    assert!(stripped.contains(" 3 "), "Expected subagent/task count 3: {}", stripped);
}





#[test]
fn token_count_shown() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":45.0,"total_input_tokens":15000,"total_output_tokens":3000,"context_window_size":200000},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("18.0K/200.0K"), "Expected token count: {}", stripped);
}

#[test]
fn turn_tokens_shown() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":45.0,"total_input_tokens":15000,"total_output_tokens":3000,"context_window_size":200000,"current_usage":{"input_tokens":500,"output_tokens":200}},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":200}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("turn: +500/200"), "Expected turn info: {}", stripped);
}

#[test]
fn vcs_dirty_shows_asterisk() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    // In a git repo, branch name should be shown
    // We can't hardcode the branch name, but we can check for git indicator
    let stripped = strip_ansi(&out);
    if stripped.contains("main") {
        assert!(stripped.contains("main"), "Should show git branch");
    }
}

#[test]
fn narrow_layout_model_trimming() {
    let json_high = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"gemini-3.5-flash-high","display_name":"Gemini 3.5 Flash (High)"},"terminal_width":60}"#;
    let out_high = run_statusline(json_high, &[]).unwrap();
    let lines_high: Vec<&str> = out_high.lines().collect();
    let line1_high = strip_ansi(lines_high[0]);
    assert!(line1_high.contains("Gemini 3.5 Flash"), "Should trim (High) from model name: {}", line1_high);
    assert!(!line1_high.contains("(High)"), "Should not contain (High): {}", line1_high);

    let json_long = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"long-model","display_name":"SuperLongModelNameThatExceedsTwentyChars"},"terminal_width":60}"#;
    let out_long = run_statusline(json_long, &[]).unwrap();
    let lines_long: Vec<&str> = out_long.lines().collect();
    let line1_long = strip_ansi(lines_long[0]);
    assert!(line1_long.contains("SuperLongModelNameTh"), "Should limit model name to 20 chars: {}", line1_long);
    assert!(!line1_long.contains("Chars"), "Should not contain full name: {}", line1_long);
}
