use statusline::parse::{parse_input, ParsedInput};

#[test]
fn empty_json() {
    let input = parse_input("{}");
    assert_eq!(input.agent_state, "idle");
    assert_eq!(input.terminal_width, 80);
    assert!(!input.sandbox_enabled);
}

#[test]
fn agent_state() {
    let input = parse_input(r#"{"agent_state": "thinking"}"#);
    assert_eq!(input.agent_state, "thinking");
}

#[test]
fn agent_state_null() {
    let input = parse_input(r#"{"agent_state": null}"#);
    assert_eq!(input.agent_state, "idle");
}

#[test]
fn simple_fields() {
    let input = parse_input(r#"{
        "artifact_count": 5,
        "task_count": 3,
        "terminal_width": 120,
        "cwd": "/home/user",
        "conversation_id": "abc12345",
        "version": "1.0.0",
        "plan_tier": "pro",
        "email": "user@example.com"
    }"#);
    assert_eq!(input.artifact_count, 5);
    assert_eq!(input.task_count, 3);
    assert_eq!(input.terminal_width, 120);
    assert_eq!(input.working_dir, "/home/user");
    assert_eq!(input.conversation_id, "abc12345");
    assert_eq!(input.version, "1.0.0");
    assert_eq!(input.plan_tier, "pro");
    assert_eq!(input.email, "user@example.com");
}

#[test]
fn nullable_strings() {
    let input = parse_input(r#"{"cwd": null, "version": null, "email": null}"#);
    assert_eq!(input.working_dir, "");
    assert_eq!(input.version, "");
    assert_eq!(input.email, "");
}

#[test]
fn subagents_count() {
    assert_eq!(parse_input(r#"{"subagents": ["a","b","c"]}"#).subagent_count, 3);
    assert_eq!(parse_input(r#"{"subagents": null}"#).subagent_count, 0);
    assert_eq!(parse_input(r#"{"subagents": []}"#).subagent_count, 0);
}

#[test]
fn sandbox_enabled_with_network() {
    let input = parse_input(r#"{"sandbox": {"enabled": true, "allow_network": true}}"#);
    assert!(input.sandbox_enabled);
    assert!(input.sandbox_allow_network);
}

#[test]
fn sandbox_disabled() {
    let input = parse_input(r#"{"sandbox": {"enabled": false, "allow_network": false}}"#);
    assert!(!input.sandbox_enabled);
    assert!(!input.sandbox_allow_network);
}

#[test]
fn model_full() {
    let input = parse_input(r#"{"model": {"id": "gpt-5", "display_name": "GPT-5"}}"#);
    assert_eq!(input.model_id, "gpt-5");
    assert_eq!(input.model_display_name, "GPT-5");
}

#[test]
fn model_null_fields() {
    let input = parse_input(r#"{"model": {"id": null, "display_name": null}}"#);
    assert_eq!(input.model_id, "");
    assert_eq!(input.model_display_name, "");
}

#[test]
fn model_display_only() {
    let input = parse_input(r#"{"model": {"display_name": "Gemini 2.5 Pro"}}"#);
    assert_eq!(input.model_display_name, "Gemini 2.5 Pro");
}

#[test]
fn context_window_full() {
    let input = parse_input(r#"{"context_window": {
        "used_percentage": 45.5,
        "total_input_tokens": 15000,
        "total_output_tokens": 3000,
        "context_window_size": 200000
    }}"#);
    assert!((input.used_percentage - 45.5).abs() < 0.01);
    assert_eq!(input.total_input_tokens, 15000);
    assert_eq!(input.total_output_tokens, 3000);
    assert_eq!(input.context_window_size, 200000);
}

#[test]
fn current_usage() {
    let input = parse_input(r#"{"context_window": {
        "current_usage": {"input_tokens": 500, "output_tokens": 200}
    }}"#);
    assert_eq!(input.turn_input_tokens, 500);
    assert_eq!(input.turn_output_tokens, 200);
}

#[test]
fn quota_gemini() {
    let input = parse_input(r#"{"quota": {
        "gemini-5h": {"remaining_fraction": 0.79, "reset_in_seconds": 3600},
        "gemini-weekly": {"remaining_fraction": 0.45, "reset_in_seconds": 86400}
    }}"#);
    assert!((input.gemini_5h_pct - 79.0).abs() < 0.1);
    assert!((input.gemini_weekly_pct - 45.0).abs() < 0.1);
    assert_eq!(input.gemini_5h_reset, 3600);
    assert_eq!(input.gemini_weekly_reset, 86400);
}

#[test]
fn quota_third_party() {
    let input = parse_input(r#"{"quota": {
        "3p-5h": {"remaining_fraction": 0.15, "reset_in_seconds": 1800},
        "3p-weekly": {"remaining_fraction": 0.05, "reset_in_seconds": 432000}
    }}"#);
    assert!((input.third_party_5h_pct - 15.0).abs() < 0.1);
    assert!((input.third_party_weekly_pct - 5.0).abs() < 0.1);
}

#[test]
fn quota_missing() {
    let input = parse_input(r#"{"quota": {"gemini-5h": {}, "gemini-weekly": {"remaining_fraction": null}}}"#);
    assert!((input.gemini_5h_pct + 1.0).abs() < 0.01);
    assert!((input.gemini_weekly_pct + 1.0).abs() < 0.01);
}

#[test]
fn full_input() {
    let json = r#"{
        "agent_state": "thinking",
        "context_window": {
            "used_percentage": 45.0,
            "total_input_tokens": 15000,
            "total_output_tokens": 3000,
            "context_window_size": 200000,
            "current_usage": {"input_tokens": 500, "output_tokens": 200}
        },
        "sandbox": {"enabled": true, "allow_network": false},
        "artifact_count": 5,
        "subagents": ["a", "b", "c"],
        "task_count": 3,
        "model": {"id": "claude-4", "display_name": "Claude 4 Sonnet"},
        "terminal_width": 120,
        "cwd": "/home/user/projects/myapp",
        "conversation_id": "abc12345def",
        "version": "1.17.15",
        "plan_tier": "pro",
        "email": "user@example.com",
        "quota": {
            "gemini-5h": {"remaining_fraction": 0.79, "reset_in_seconds": 3600},
            "gemini-weekly": {"remaining_fraction": 0.45, "reset_in_seconds": 86400}
        }
    }"#;
    let input = parse_input(json);
    assert_eq!(input.agent_state, "thinking");
    assert_eq!(input.total_input_tokens, 15000);
    assert_eq!(input.total_output_tokens, 3000);
    assert_eq!(input.context_window_size, 200000);
    assert_eq!(input.turn_input_tokens, 500);
    assert_eq!(input.turn_output_tokens, 200);
    assert!(input.sandbox_enabled);
    assert!(!input.sandbox_allow_network);
    assert_eq!(input.artifact_count, 5);
    assert_eq!(input.subagent_count, 3);
    assert_eq!(input.task_count, 3);
    assert_eq!(input.model_id, "claude-4");
    assert_eq!(input.model_display_name, "Claude 4 Sonnet");
    assert_eq!(input.terminal_width, 120);
    assert_eq!(input.working_dir, "/home/user/projects/myapp");
    assert_eq!(input.conversation_id, "abc12345def");
    assert_eq!(input.version, "1.17.15");
    assert_eq!(input.plan_tier, "pro");
    assert_eq!(input.email, "user@example.com");
    assert!((input.gemini_5h_pct - 79.0).abs() < 0.1);
    assert!((input.gemini_weekly_pct - 45.0).abs() < 0.1);
    assert_eq!(input.gemini_5h_reset, 3600);
    assert_eq!(input.gemini_weekly_reset, 86400);
}

#[test]
fn malformed_input() {
    let input = parse_input("not json");
    assert_eq!(input.agent_state, "idle");
}

#[test]
fn truncated_mid_key() {
    let input = parse_input(r#"{"agent_state":"#);
    assert_eq!(input.agent_state, "");
}

#[test]
fn truncated_mid_string() {
    let input = parse_input(r#"{"agent_state": "thin"#);
    assert_eq!(input.agent_state, "thin");
}

#[test]
fn null_objects() {
    let input = parse_input(r#"{"context_window": null, "sandbox": null, "model": null, "quota": null, "terminal_width": 120}"#);
    assert_eq!(input.terminal_width, 120);
    assert_eq!(input.model_id, "");
}
