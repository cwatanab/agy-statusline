use std::time::Instant;
use statusline::{parse, render};

#[test]
fn bench_parse_and_render() {
    let json = r#"{
        "agent_state": "working",
        "context_window": {
            "used_percentage": 42.5,
            "total_input_tokens": 125000,
            "total_output_tokens": 15000,
            "context_window_size": 200000,
            "current_usage": {
                "input_tokens": 1200,
                "output_tokens": 350
            }
        },
        "sandbox": {
            "enabled": true,
            "allow_network": true
        },
        "artifact_count": 3,
        "subagents": ["sub1", "sub2"],
        "task_count": 2,
        "model": {
            "id": "claude-3-5-sonnet",
            "display_name": "Claude 3.5 Sonnet"
        },
        "cwd": "D:/Develop/agy-statusline",
        "terminal_width": 120,
        "version": "0.2.1",
        "plan_tier": "Pro",
        "email": "user@example.com",
        "conversation_id": "abc123456789",
        "vcs": {
            "branch": "main",
            "dirty": true
        },
        "quota": {
            "3p-5h": {
                "remaining_fraction": 0.85,
                "reset_in_seconds": 7200
            },
            "3p-weekly": {
                "remaining_fraction": 0.60,
                "reset_in_seconds": 86400
            }
        }
    }"#;

    // Warmup
    for _ in 0..100 {
        let input = parse::parse_input(json);
        let _ = render::render_line(&input, false, None);
    }

    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let input = parse::parse_input(json);
        let _ = render::render_line(&input, false, None);
    }
    let duration = start.elapsed();
    let per_op = duration.as_nanos() / iterations as u128;
    println!("\n=== BENCHMARK RESULT ===");
    println!("Total time for {} iterations: {:?}", iterations, duration);
    println!("Time per parse+render: {} ns ({} us)\n", per_op, per_op as f64 / 1000.0);
}
