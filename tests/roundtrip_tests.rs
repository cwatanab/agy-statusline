use std::io::Write;
use std::process::{Command, Stdio};

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            while let Some(d) = chars.next() {
                if d == 'm' {
                    break;
                }
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
    let path = if cfg!(windows) {
        path.with_extension("exe")
    } else {
        path
    };
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
    assert!(
        stripped.contains("THINKING"),
        "Expected THINKING in: {}",
        stripped
    );
}

#[test]
fn working_shows_working() {
    let json = r#"{"agent_state":"working","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(
        stripped.contains("WORKING"),
        "Expected WORKING in: {}",
        stripped
    );
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
    assert!(
        stripped.contains("net-on"),
        "Expected 'net-on' in: {}",
        stripped
    );
}

#[test]
fn sandbox_on_no_net() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":true,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(
        stripped.contains("net-off"),
        "Expected 'net-off' in: {}",
        stripped
    );
}

#[test]
fn sandbox_off() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(
        stripped.contains("host"),
        "Expected 'host' in: {}",
        stripped
    );
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
    assert!(
        !stripped.contains("5H"),
        "Should not contain quota bar: {}",
        stripped
    );
}

#[test]
fn context_bar_shows_percentage() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":45.0,"total_input_tokens":15000,"total_output_tokens":3000,"context_window_size":200000},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("45.0%"), "Expected '45.0%' in: {}", stripped);
    assert!(stripped.contains("(90.0K/200.0K)"), "Expected '(90.0K/200.0K)' in: {}", stripped);
}

#[test]
fn classic_mode_uses_text_labels() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":5,"subagents":["a"],"task_count":3,"model":{"id":"","display_name":""},"terminal_width":200}"#;
    let out = run_statusline(json, &["--classic"]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(
        stripped.contains("artifacts 5"),
        "Classic should show 'artifacts 5': {}",
        stripped
    );
    assert!(
        stripped.contains("tasks 3"),
        "Classic should show 'tasks 3': {}",
        stripped
    );
    assert!(stripped.contains("ctx"), "Classic should show 'ctx': {}", stripped);
    assert!(
        stripped.contains("OFF"),
        "Classic should show 'OFF': {}",
        stripped
    );
}

#[test]
fn boxed_framed_layout() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"gpt","display_name":"GPT"},"terminal_width":60}"#;
    let out = run_statusline(json, &[]).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() >= 2, "Framed layout should have at least 2 lines, got: {:?}", lines);
    assert!(lines[0].contains("╭─"), "First line should have top-left border");
    assert!(strip_ansi(lines[0]).contains("READY"), "Line 1 should contain READY");
    assert!(strip_ansi(lines[0]).contains("GPT"), "Line 1 should contain model name");
}

#[test]
fn artifacts_subagents_tasks_counts() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":0,"total_input_tokens":0,"total_output_tokens":0,"context_window_size":0},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":5,"subagents":["a","b","c"],"task_count":3,"model":{"id":"","display_name":""},"terminal_width":200}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("5"), "Expected artifact count 5: {}", stripped);
    assert!(stripped.contains("3"), "Expected subagent/task count 3: {}", stripped);
}

#[test]
fn token_count_shown() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":45.0,"total_input_tokens":15000,"total_output_tokens":3000,"context_window_size":200000},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":120}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("total: 15.0K/3.0K"), "Expected token count: {}", stripped);
}

#[test]
fn turn_tokens_shown() {
    let json = r#"{"agent_state":"idle","context_window":{"used_percentage":45.0,"total_input_tokens":15000,"total_output_tokens":3000,"context_window_size":200000,"current_usage":{"input_tokens":500,"output_tokens":200}},"sandbox":{"enabled":false,"allow_network":false},"artifact_count":0,"subagents":[],"task_count":0,"model":{"id":"","display_name":""},"terminal_width":200}"#;
    let out = run_statusline(json, &[]).unwrap();
    let stripped = strip_ansi(&out);
    assert!(stripped.contains("turn: +500/200"), "Expected turn info: {}", stripped);
}

#[test]
fn vim_mode_rendering() {
    for mode in ["NORMAL", "INSERT", "VISUAL", "VISUAL LINE", "CUSTOM_MODE"] {
        let json = format!(r#"{{"agent_state":"idle","vim":{{"mode":"{mode}"}},"terminal_width":100}}"#);
        let out_styled = run_statusline(&json, &[]).unwrap();
        let stripped_styled = strip_ansi(&out_styled);
        assert!(
            stripped_styled.contains(mode),
            "Styled mode should contain '{}': {}",
            mode,
            stripped_styled
        );

        let out_classic = run_statusline(&json, &["--classic"]).unwrap();
        let stripped_classic = strip_ansi(&out_classic);
        let expected_bracket = format!("[{}]", mode);
        assert!(
            stripped_classic.contains(&expected_bracket),
            "Classic mode should contain '{}': {}",
            expected_bracket,
            stripped_classic
        );
    }

    // Absent vim mode
    let json_absent = r#"{"agent_state":"idle","terminal_width":100}"#;
    let out_absent = run_statusline(json_absent, &["--classic"]).unwrap();
    let stripped = strip_ansi(&out_absent);
    assert!(!stripped.contains("NORMAL"), "Absent vim should not contain NORMAL: {}", stripped);
    assert!(!stripped.contains("INSERT"), "Absent vim should not contain INSERT: {}", stripped);
}

#[test]
fn subagent_zero_hidden() {
    let json_zero = r#"{"agent_state":"idle","subagents":[],"terminal_width":120}"#;
    let out_zero = run_statusline(json_zero, &["--classic"]).unwrap();
    let stripped_zero = strip_ansi(&out_zero);
    assert!(
        !stripped_zero.contains("subagents"),
        "Zero subagents should not display subagents badge: {}",
        stripped_zero
    );

    let json_some = r#"{"agent_state":"idle","subagents":["worker1"],"terminal_width":120}"#;
    let out_some = run_statusline(json_some, &["--classic"]).unwrap();
    let stripped_some = strip_ansi(&out_some);
    assert!(
        stripped_some.contains("subagents"),
        "Non-zero subagents should display subagents badge: {}",
        stripped_some
    );
}

#[test]
fn classic_ac_badge_not_duplicated() {
    use statusline::bar::make_badge;
    let badge = make_badge("AC", "AC", "76", true);
    let plain = strip_ansi(&badge);
    assert_eq!(plain, "AC", "Classic badge with icon == val should not duplicate: '{}'", plain);
}

#[test]
fn line1_responsive_width_safeguard() {
    // Narrow terminal with long values and vim mode
    let json = r#"{"agent_state":"working","vim":{"mode":"VISUAL LINE"},"vcs":{"branch":"feature/very-long-branch-name-overflowing-everything","dirty":true},"model":{"id":"gemini-2.0-flash-thinking-exp"},"terminal_width":60}"#;
    let out = run_statusline(json, &[]).unwrap();
    let first_line = out.lines().next().unwrap_or("");
    let plain = strip_ansi(first_line);
    assert!(
        plain.chars().count() <= 60,
        "LINE1 length ({}) exceeds terminal width (60): '{}'",
        plain.chars().count(),
        plain
    );
}

#[test]
fn empty_stdin_renders_idle() {
    let out = run_statusline("", &[]).expect("Expected statusline to succeed even with empty stdin");
    let stripped = strip_ansi(&out);
    assert!(
        stripped.contains("READY"),
        "Expected READY in output from empty stdin: {}",
        stripped
    );
}

#[test]
fn multi_turn_context_token_harmony() {
    let multi_turn_payload = r#"{"agent_state":"idle","terminal_width":120,"context_window":{"total_input_tokens":240123,"total_output_tokens":272777,"context_window_size":1048576,"used_percentage":22.9}}"#;
    let out = run_statusline(multi_turn_payload, &["--classic"]).unwrap();
    let plain = strip_ansi(&out);
    assert!(plain.contains("22.9%"), "Multi-turn context percentage ~22.9%: {}", plain);
    assert!(plain.contains("240.1K/1.0M"), "Multi-turn context used displays 240.1K/1.0M without cumulative output distortion: {}", plain);
    assert!(plain.contains("total: 240.1K/272.8K"), "Token details still display cumulative session totals: {}", plain);

    let out_nerd = run_statusline(multi_turn_payload, &[]).unwrap();
    let plain_nerd = strip_ansi(&out_nerd);
    assert!(plain_nerd.contains("22.9%"), "Nerd font context percentage: {}", plain_nerd);
    assert!(plain_nerd.contains("240.1K/1.0M"), "Nerd font context used displays 240.1K/1.0M: {}", plain_nerd);
}

#[test]
fn version_flag_reports_v0_3_3() {
    let out = run_statusline("{}", &["--version"]).unwrap();
    assert!(out.contains("0.3.3"), "Version output reports 0.3.3: {}", out);
    assert!(!out.contains("0.3.2"), "Version output does not contain stale 0.3.2");
    assert!(!out.contains("0.3.1"), "Version output does not contain stale 0.3.1");
    assert!(!out.contains("0.2.6"), "Version output does not contain stale 0.2.6");
}

#[test]
fn telemetry_suppression_flags() {
    let fixture = include_str!("fixtures/full_payload.json");

    // Header suppression
    let out = run_statusline(fixture, &["--no-state", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("WORKING"), "--no-state suppresses state");

    let out = run_statusline(fixture, &["--no-vim", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("NORMAL"), "--no-vim suppresses vim mode");
    let out = run_statusline(fixture, &["--no-vim-mode", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("NORMAL"), "--no-vim-mode suppresses vim mode");

    let out = run_statusline(fixture, &["--no-branch", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("qa/test-fixtures"), "--no-branch suppresses VCS branch");
    let out = run_statusline(fixture, &["--no-git", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("qa/test-fixtures"), "--no-git suppresses VCS branch");

    let out = run_statusline(fixture, &["--no-model", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("Gemini 2.0 Flash"), "--no-model suppresses model");

    let out = run_statusline(fixture, &["--no-dir", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("qa-fixtures"), "--no-dir suppresses working directory");
    let out = run_statusline(fixture, &["--no-cwd", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("qa-fixtures"), "--no-cwd suppresses working directory");

    let out = run_statusline(fixture, &["--no-conv", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("62e3d023"), "--no-conv suppresses conversation prefix");
    let out = run_statusline(fixture, &["--no-conversation", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("62e3d023"), "--no-conversation suppresses conversation prefix");

    let out = run_statusline(fixture, &["--no-account", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("rekvizitor"), "--no-account suppresses user account info");
    let out = run_statusline(fixture, &["--no-user", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("rekvizitor"), "--no-user suppresses user account info");
    let out = run_statusline(fixture, &["--no-plan", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("rekvizitor"), "--no-plan suppresses user account info");

    let out = run_statusline(fixture, &["--no-host", "--medium-wide"]).unwrap();
    assert!(!out.contains("\u{F048B}"), "--no-host suppresses host indicator");

    let out = run_statusline(fixture, &["--no-version", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("v0.2.4"), "--no-version suppresses version badge");

    // Pill badge suppression
    let out = run_statusline(fixture, &["--no-context-usage", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("14.2%"), "--no-context-usage suppresses context bar");
    let out = run_statusline(fixture, &["--no-context", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("14.2%"), "--no-context suppresses context bar");
    let out = run_statusline(fixture, &["--no-ctx", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("14.2%"), "--no-ctx suppresses context bar");

    let out = run_statusline(fixture, &["--no-tokens-usage", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("88.2K/61.1K"), "--no-tokens-usage suppresses token badge");
    let out = run_statusline(fixture, &["--no-tokens", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("88.2K/61.1K"), "--no-tokens suppresses token badge");

    let out = run_statusline(fixture, &["--no-cost", "--medium-wide"]).unwrap();
    assert!(!out.is_empty(), "--no-cost runs safely");

    let out = run_statusline(fixture, &["--no-sys", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("RAM:"), "--no-sys suppresses CPU/RAM badge");
    let out = run_statusline(fixture, &["--no-system", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("RAM:"), "--no-system suppresses CPU/RAM badge");
    let out = run_statusline(fixture, &["--no-resources", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("RAM:"), "--no-resources suppresses CPU/RAM badge");

    let out = run_statusline(fixture, &["--no-artifacts", "--classic"]).unwrap();
    assert!(!strip_ansi(&out).contains("artifacts"), "--no-artifacts suppresses artifacts badge");

    let out = run_statusline(fixture, &["--no-subagents", "--classic"]).unwrap();
    assert!(!strip_ansi(&out).contains("subagents"), "--no-subagents suppresses subagents badge");

    let out = run_statusline(fixture, &["--no-tasks", "--classic"]).unwrap();
    assert!(!strip_ansi(&out).contains("tasks"), "--no-tasks suppresses tasks badge");

    let out = run_statusline(fixture, &["--no-sandbox", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("net-on"), "--no-sandbox suppresses sandbox badge");

    let out = run_statusline(fixture, &["--no-quota", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("5H"), "--no-quota suppresses 5H quota bar");
    assert!(!strip_ansi(&out).contains("7D"), "--no-quota suppresses 7D quota bar");

    let out = run_statusline(fixture, &["--no-power", "--medium-wide"]).unwrap();
    assert!(!strip_ansi(&out).contains("AC") && !strip_ansi(&out).contains("BAT"), "--no-power suppresses power badge");
}

#[test]
fn header_collapse_and_full_suppression() {
    let fixture = include_str!("fixtures/full_payload.json");
    let all_l1_flags = &[
        "--no-state", "--no-vim", "--no-branch", "--no-model",
        "--no-dir", "--no-conv", "--no-account", "--no-host", "--no-version",
        "--medium-wide"
    ];

    // Header collapse: omits Line 1, starts first badge row with ╭─
    let hc_out = run_statusline(fixture, all_l1_flags).unwrap();
    let hc_first_line = hc_out.lines().next().unwrap_or("");
    assert!(!hc_out.contains("WORKING"), "Header collapse omits state");
    assert!(!hc_out.contains("NORMAL"), "Header collapse omits vim");
    assert!(!hc_out.contains("qa/test-fixtures"), "Header collapse omits branch");
    assert!(!hc_out.contains("Gemini 2.0 Flash"), "Header collapse omits model");
    assert!(hc_first_line.contains("╭─"), "Header collapse starts first badge row with ╭─: {}", hc_first_line);
    assert!(!hc_first_line.contains("├─"), "Header collapse first row does not start with ├─: {}", hc_first_line);
    assert!(hc_first_line.contains("ctx"), "Header collapse first row contains context badge: {}", hc_first_line);

    // Single badge row with header collapse
    let single_flags = &[
        "--no-state", "--no-vim", "--no-branch", "--no-model",
        "--no-dir", "--no-conv", "--no-account", "--no-host", "--no-version",
        "--no-tokens", "--no-sys", "--no-artifacts", "--no-subagents",
        "--no-tasks", "--no-sandbox", "--no-quota", "--no-power",
        "--medium-wide"
    ];
    let single_out = run_statusline(fixture, single_flags).unwrap();
    assert!(single_out.contains("╭─"), "Single row collapsed badge starts with ╭─: {}", single_out);
    let line_count = single_out.lines().filter(|l| !l.trim().is_empty()).count();
    assert_eq!(line_count, 1, "Single row collapsed badge renders exactly 1 row: {}", single_out);

    // Classic header collapse
    let classic_flags = &[
        "--classic",
        "--no-state", "--no-vim", "--no-branch", "--no-model",
        "--no-dir", "--no-conv", "--no-account", "--no-host", "--no-version",
        "--medium-wide"
    ];
    let classic_out = run_statusline(fixture, classic_flags).unwrap();
    assert!(!classic_out.contains("WORKING"), "Classic header collapse omits line 1");
    let classic_first = classic_out.lines().next().unwrap_or("");
    assert!(classic_first.contains("ctx"), "Classic header collapse begins directly with badges");

    // Full suppression produces clean empty output (0 bytes)
    let all_flags = &[
        "--no-state", "--no-vim", "--no-branch", "--no-model",
        "--no-dir", "--no-conv", "--no-account", "--no-host", "--no-version",
        "--no-context", "--no-tokens", "--no-cost", "--no-sys",
        "--no-artifacts", "--no-subagents", "--no-tasks", "--no-sandbox",
        "--no-quota", "--no-power",
        "--medium-wide"
    ];
    let full_supp_out = run_statusline(fixture, all_flags).unwrap();
    assert!(full_supp_out.trim().is_empty(), "Full suppression produces clean empty output: '{}'", full_supp_out);
}


